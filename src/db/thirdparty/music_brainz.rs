use std::env;
use std::process::exit;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::{RwLock, Semaphore};
use tokio::time::{Duration, Instant, sleep};
use tracing::{debug, error};
use crate::db::thirdparty::{ArtistData, SongData};
/// Rate limiter for MusicBrainz API calls
/// MusicBrainz allows 1 request per second for anonymous users
pub struct RateLimiter {
    last_request: RwLock<Instant>,
    semaphore: Semaphore,
    min_interval: Duration,
}

impl RateLimiter {
    pub fn new(requests_per_second: u32) -> Self {
        let min_interval = Duration::from_millis(1000 / requests_per_second as u64);
        Self {
            last_request: RwLock::new(Instant::now() - min_interval),
            semaphore: Semaphore::new(1), // Only allow one request at a time
            min_interval,
        }
    }

    /// Wait until it's safe to make another request
    pub async fn acquire(&self) {
        let _permit = self.semaphore.acquire().await.unwrap();

        let mut last_request = self.last_request.write().await;
        let now = Instant::now();
        let elapsed = now.duration_since(*last_request);

        if elapsed < self.min_interval {
            let wait_time = self.min_interval - elapsed;
            debug!("Rate limiting: waiting {:?}", wait_time);
            sleep(wait_time).await;
        }

        *last_request = Instant::now();
    }
}

// Rate limiter for MusicBrainz (1 request per second)
static MUSICBRAINZ_RATE_LIMITER: Lazy<RateLimiter> = Lazy::new(|| RateLimiter::new(1));

/// MusicBrainz artist search result
#[derive(Deserialize)]
pub struct MusicBrainzArtist {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub tags: Vec<MusicBrainzTag>,
    #[serde(rename = "life-span")]
    pub _life_span: Option<MusicBrainzLifeSpan>,
    #[serde(default)]
    #[serde(rename = "aliases")]
    pub _aliases: Vec<MusicBrainzAlias>,
}

#[derive(Deserialize)]
pub struct MusicBrainzTag {
    pub name: String,
    #[serde(rename = "count")]
    pub _count: Option<u32>,
}

#[derive(Deserialize)]
pub struct MusicBrainzLifeSpan {
    #[serde(rename = "begin")]
    pub _begin: Option<String>,
    #[serde(rename = "end")]
    pub _end: Option<String>,
    #[serde(rename = "ended")]
    pub _ended: Option<bool>,
}

#[derive(Deserialize)]
pub struct MusicBrainzAlias {
    #[serde(rename = "name")]
    pub _name: String,
    #[serde(rename = "sort-name")]
    pub _sort_name: String,
}

#[derive(Deserialize)]
struct ArtistSearchResponse {
    artists: Vec<MusicBrainzArtist>,
    #[serde(rename = "count")]
    _count: u32,
    #[serde(rename = "offset")]
    _offset: u32,
}

/// MusicBrainz recording (song) search result
#[derive(Deserialize)]
pub struct MusicBrainzRecording {
    pub id: String,
    pub title: String,
    #[serde(rename = "length")]
    pub _length: Option<u32>, // Duration in milliseconds
    #[serde(rename = "artist-credit")]
    pub artist_credit: Vec<MusicBrainzArtistCredit>,
    pub releases: Option<Vec<MusicBrainzReleaseInfo>>,
    #[serde(default)]
    #[serde(rename = "tags")]
    pub _tags: Vec<MusicBrainzTag>,
}

#[derive(Deserialize)]
pub struct MusicBrainzArtistCredit {
    pub artist: MusicBrainzArtistSimple,
    #[serde(rename = "name")]
    pub _name: String,
}

#[derive(Deserialize)]
pub struct MusicBrainzArtistSimple {
    #[serde(rename = "id")]
    pub _id: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct MusicBrainzReleaseInfo {
    pub id: String,
    pub title: String,
    #[serde(rename = "release-group")]
    pub release_group: Option<MusicBrainzReleaseGroup>,
    #[serde(rename = "date")]
    pub _date: Option<String>,
    #[serde(rename = "track-count")]
    pub _track_count: Option<u32>,
}

#[derive(Deserialize)]
pub struct MusicBrainzReleaseGroup {
    #[serde(rename = "id")]
    pub _id: String,
    #[serde(rename = "title")]
    pub _title: String,
    #[serde(rename = "primary-type")]
    pub primary_type: Option<String>,
}

#[derive(Deserialize)]
struct RecordingSearchResponse {
    recordings: Vec<MusicBrainzRecording>,
    #[serde(rename = "count")]
    _count: u32,
    #[serde(rename = "offset")]
    _offset: u32,
}

/// Make a rate-limited API call to MusicBrainz
pub async fn api_call<T: for<'de> serde::Deserialize<'de>>(url: &str) -> Result<T, String> {
    // Wait for rate limiter
    MUSICBRAINZ_RATE_LIMITER.acquire().await;

    let client = Client::new();
    let user_agent = format!(
        "{}/{} ( {} )",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env::var("CONTACT_EMAIL").unwrap_or_else(|_| {
            error!("the host needs to provide an email as it's required by music brainz and thus must be set for muse to run as it's a none optional");
            exit(0)
        }));

    debug!(%url, "Making MusicBrainz API request");

    match client
        .get(url)
        .header("User-Agent", user_agent)
        .header("Accept", "application/json")
        .send()
        .await
    {
        Ok(res) => {
            if !res.status().is_success() {
                return Err(format!("MusicBrainz API returned status: {}", res.status()));
            }

            match res.json::<T>().await {
                Ok(data) => Ok(data),
                Err(err) => Err(format!("Failed to parse MusicBrainz response: {}", err)),
            }
        }
        Err(err) => Err(format!("Failed to request MusicBrainz API: {}", err)),
    }
}

/// Fetch artist data from MusicBrainz by artist name
///
/// # Arguments
///
/// * `artist_name` - The display name of the artist to look up
///
/// # Returns
///
/// An `Ok(Some(ArtistData))` if the artist is found, or `Ok(None)` if not found.
/// Errors are returned as `Err(...)`.
pub async fn get_artist_data(artist_name: &str) -> Result<Option<ArtistData>, String> {
    let encoded_name = urlencoding::encode(artist_name);
    let url = format!(
        "https://musicbrainz.org/ws/2/artist/?query=artist:\"{}\"&fmt=json&limit=10",
        encoded_name
    );

    debug!(%url, artist_name, "Sending MusicBrainz artist search request");

    let res = api_call::<ArtistSearchResponse>(&url).await?;

    debug!(
        artist_name,
        count = res.artists.len(),
        "Received artist search results from MusicBrainz"
    );

    for artist in &res.artists {
        let matched = artist.name.eq_ignore_ascii_case(artist_name);
        debug!(
            artist_name,
            candidate_name = %artist.name,
            matched,
            "Evaluating artist match"
        );
    }

    let artist = match res
        .artists
        .into_iter()
        .find(|artist| artist.name.eq_ignore_ascii_case(artist_name))
    {
        Some(a) => {
            debug!(artist_name, artist_id = %a.id, "Matched artist ID");
            a
        }
        None => {
            debug!(artist_name, "No exact match found for artist name");
            return Ok(None);
        }
    };

    // MusicBrainz doesn't provide images directly, so we'll leave picture_url empty
    // You might want to integrate with Cover Art Archive API separately
    let picture_url = String::new();

    // Extract genres from tags
    let genres: Vec<String> = artist
        .tags
        .into_iter()
        .map(|tag| tag.name)
        .collect();

    Ok(Some(ArtistData {
        name: artist.name,
        picture_url,
        genres,
    }))
}

/// Fetch song data from MusicBrainz by searching for a recording
///
/// # Arguments
///
/// * `song_name` - The name of the song to search for
/// * `artist_name` - Optional artist name to narrow the search
///
/// # Returns
///
/// An `Ok(Some(SongData))` if the song is found, or `Ok(None)` if not found.
/// Errors are returned as `Err(...)`.
async fn get_song_data_(song_name: &str, artist_name: Option<&str>) -> Result<Option<SongData>, String> {
    let encoded_song = urlencoding::encode(song_name);

    let query = if let Some(artist) = artist_name {
        let encoded_artist = urlencoding::encode(artist);
        format!("recording:\"{}\" AND artist:\"{}\"", encoded_song, encoded_artist)
    } else {
        format!("recording:\"{}\"", encoded_song)
    };

    let url = format!(
        "https://musicbrainz.org/ws/2/recording/?query={}&fmt=json&limit=10&inc=releases+artist-credits+tags",
        urlencoding::encode(&query)
    );

    debug!(%url, song_name, ?artist_name, "Sending MusicBrainz recording search request");

    let res = api_call::<RecordingSearchResponse>(&url).await?;

    debug!(
        song_name,
        ?artist_name,
        count = res.recordings.len(),
        "Received recording search results from MusicBrainz"
    );

    for recording in &res.recordings {
        let song_matched = recording.title.eq_ignore_ascii_case(song_name);
        let artist_matched = artist_name.map_or(true, |name| {
            recording.artist_credit.iter().any(|credit| credit.artist.name.eq_ignore_ascii_case(name))
        });

        debug!(
            song_name,
            candidate_song = %recording.title,
            candidate_artists = ?recording.artist_credit.iter().map(|c| &c.artist.name).collect::<Vec<_>>(),
            song_matched,
            artist_matched,
            "Evaluating recording match"
        );
    }

    let recording = match res.recordings.into_iter().find(|recording| {
        let song_matched = recording.title.eq_ignore_ascii_case(song_name);
        let artist_matched = artist_name.map_or(true, |name| {
            recording.artist_credit.iter().any(|credit| credit.artist.name.eq_ignore_ascii_case(name))
        });
        song_matched && artist_matched
    }) {
        Some(r) => {
            debug!(song_name, ?artist_name, recording_id = %r.id, "Matched recording ID");
            r
        }
        None => {
            debug!(song_name, ?artist_name, "No exact match found for recording");
            return Ok(None);
        }
    };

    // Get album information from the first release
    let (album_name, album_type) = recording
        .releases
        .as_ref()
        .and_then(|releases| releases.first())
        .map(|release| {
            let album_type = release
                .release_group
                .as_ref()
                .and_then(|rg| rg.primary_type.clone())
                .unwrap_or_else(|| "Album".to_string());
            (release.title.clone(), album_type)
        })
        .unwrap_or_else(|| ("Unknown Album".to_string(), "Album".to_string()));

    // Collect artist names
    let artists: Vec<String> = recording
        .artist_credit
        .iter()
        .map(|credit| credit.artist.name.clone())
        .collect();
    
    let album_art_url = String::new();

    Ok(Some(SongData {
        name: recording.title,
        artists,
        album_name,
        album_art_url,
        album_type,
    }))
}

/// Get Cover Art Archive URL for a release
/// This is a separate API that provides album artwork
pub fn get_cover_art_url(release_id: &str) -> String {
    format!("https://coverartarchive.org/release/{}/front", release_id)
}

/// Enhanced song data fetching that includes cover art from Cover Art Archive
pub async fn get_song_data(song_name: &str, artist_name: Option<&str>) -> Result<Option<SongData>, String> {
    let mut song_data = match get_song_data_(song_name, artist_name).await? {
        Some(data) => data,
        None => return Ok(None),
    };

    // If we found the song, try to get the release ID and fetch cover art
    let encoded_song = urlencoding::encode(song_name);
    let query = if let Some(artist) = artist_name {
        let encoded_artist = urlencoding::encode(artist);
        format!("recording:\"{}\" AND artist:\"{}\"", encoded_song, encoded_artist)
    } else {
        format!("recording:\"{}\"", encoded_song)
    };

    let url = format!(
        "https://musicbrainz.org/ws/2/recording/?query={}&fmt=json&limit=1&inc=releases",
        urlencoding::encode(&query)
    );

    if let Ok(res) = api_call::<RecordingSearchResponse>(&url).await {
        if let Some(recording) = res.recordings.first() {
            if let Some(releases) = &recording.releases {
                if let Some(release) = releases.first() {
                    // Try to get cover art (this might fail if no cover art exists)
                    let cover_art_url = get_cover_art_url(&release.id);

                    // We could make a HEAD request to check if the cover art exists, 
                    // but for now, we'll just set the URL and let the client handle 404 s
                    song_data.album_art_url = cover_art_url;
                }
            }
        }
    }

    Ok(Some(song_data))
}