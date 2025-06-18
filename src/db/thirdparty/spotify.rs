use std::env;
use base64::Engine;
use base64::engine::general_purpose;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::RwLock;
use std::time::Instant;
use tower_cookies::cookie::time::Duration;
use tracing::debug;
use crate::db::thirdparty::{ArtistData, SongData};


static SPOTIFY_ENABLED: Lazy<bool> = Lazy::new(|| {
    if env::var("HAS_SPOTIFY").unwrap_or_else(|_| "false".to_string()) != "true" {
        return false;
    }

    let client_id = env::var("SPOTIFY_CLIENT_ID").unwrap_or_default();
    let client_secret = env::var("SPOTIFY_CLIENT_SECRET").unwrap_or_default();

    !client_id.is_empty() && !client_secret.is_empty()
});

/// Represents the response structure from Spotify's access token endpoint.
#[derive(Debug, Deserialize)]
pub struct SpotifyTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u32,
}

/// A cache for Spotify access tokens that refreshes automatically when expired.
pub struct SpotifyTokenCache {
    token: Option<String>,
    expires_at: Instant,
}

impl SpotifyTokenCache {
    /// Creates a new Spotify token cache with no token stored.
    pub fn new() -> Self {
        Self {
            token: None,
            expires_at: Instant::now(),
        }
    }

    /// Retrieves a valid Spotify access token.
    ///
    /// If the cached token is still valid, it returns that token.
    /// Otherwise, it fetches a new token from Spotify using the provided credentials,
    /// updates the cache, and returns the new token.
    ///
    /// # Arguments
    ///
    /// * `client_id` - Your Spotify application's client ID.
    /// * `client_secret` - Your Spotify application's client secret.
    ///
    /// # Returns
    ///
    /// A `Result` containing the valid access token as a `String`, or an error if the request fails.
    pub async fn get(
        &mut self,
    ) -> Result<String,String> {
        let now = Instant::now();

        if let Some(ref token) = self.token {
            if now < self.expires_at {
                return Ok(token.clone());
            }
        }

        // Inline fetch logic
        let auth = general_purpose::STANDARD.encode(format!("{}:{}",
                                                            env::var("SPOTIFY_CLIENT_ID").unwrap_or_default(),
                                                            env::var("SPOTIFY_CLIENT_SECRET").unwrap_or_default()));
        let client = Client::new();

        let res = match client.post("https://accounts.spotify.com/api/token")
            .header("Authorization", format!("Basic {}", auth))
            .form(&[("grant_type", "client_credentials")]).send().await {
            Ok(res) => {
                match res.json::<SpotifyTokenResponse>().await {
                    Ok(data) => Ok(data),
                    Err(err) => Err(format!("failed to parse response: {}", err)),
                }
            }
            Err(err) => Err(format!("failed to request spotify api: {}", err)),
        }?;

        self.token = Some(res.access_token.clone());
        self.expires_at = now + Duration::seconds((res.expires_in as u64 - 60) as i64); // buffer

        Ok(res.access_token)
    }
}

static SPOTIFY_TOKEN_CACHE: Lazy<RwLock<SpotifyTokenCache>> = Lazy::new(|| RwLock::new(SpotifyTokenCache::new()));
async fn get_spotify_token() -> Result<String, String> {
    SPOTIFY_TOKEN_CACHE.write().await.get().await
}

/// Represents a Spotify image object.
#[derive(Debug, Deserialize)]
pub struct SpotifyImage {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Basic Spotify artist detail from search or detail endpoints.
#[derive(Debug, Deserialize)]
pub struct SpotifyArtist {
    pub id: String,
    pub name: String,
    pub genres: Vec<String>,
    pub images: Option<Vec<SpotifyImage>>, // optional in search
}

/// Search response wrapping artist items.
#[derive(Debug, Deserialize)]
struct ArtistSearchResponse {
    artists: ArtistList,
}

#[derive(Debug, Deserialize)]
struct ArtistList {
    items: Vec<SpotifyArtist>,
}

/// Represents a Spotify track/song from search or detail endpoints.
#[derive(Debug, Deserialize)]
pub struct SpotifyTrack {
    pub id: String,
    pub name: String,
    pub artists: Vec<SpotifyArtistSimple>,
    pub album: SpotifyAlbumSimple,
    pub duration_ms: u32,
    pub popularity: u8,
    pub explicit: bool,
    pub preview_url: Option<String>,
    pub external_urls: SpotifyExternalUrls,
}

/// Simplified artist info used in track responses
#[derive(Debug, Deserialize)]
pub struct SpotifyArtistSimple {
    pub id: String,
    pub name: String,
}

/// Simplified album info used in track responses
#[derive(Debug, Deserialize)]
pub struct SpotifyAlbumSimple {
    pub id: String,
    pub name: String,
    pub images: Vec<SpotifyImage>,
    pub release_date: String,
    pub album_type: String,
}

/// External URLs from Spotify
#[derive(Debug, Deserialize)]
pub struct SpotifyExternalUrls {
    pub spotify: String,
}

/// Search response wrapping track items.
#[derive(Debug, Deserialize)]
struct TrackSearchResponse {
    tracks: TrackList,
}

#[derive(Debug, Deserialize)]
struct TrackList {
    items: Vec<SpotifyTrack>,
}

pub async fn api_call<T: for<'de> serde::Deserialize<'de>>(url: &str) -> Result<T, String> {
    let client = Client::new();
    match client.get(&*url).bearer_auth(get_spotify_token().await?.as_str())
        .send().await {
        Ok(res) => {
            match res.json::<T>().await {
                Ok(data) => Ok(data),
                Err(err) => Err(format!("failed to parse response: {}", err)),
            }
        }
        Err(err) => Err(format!("failed to request spotify api: {}", err)),
    }

}

/// Fetch the artist's full profile, including images.
/// Searches Spotify by artist name and returns enriched artist data with image and genres.
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
        "https://api.spotify.com/v1/search?q=artist%3A%22{}%22&type=artist&limit=10",
        encoded_name
    );

    debug!(%url, artist_name, "Sending Spotify artist search request");

    let res = api_call::<ArtistSearchResponse>(&url).await?;

    debug!(
        artist_name,
        count = res.artists.items.len(),
        "Received artist search results"
    );

    for artist in &res.artists.items {
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
        .items
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

    let images: Vec<SpotifyImage> = artist.images.unwrap_or_default();
    let (picture_url, _picture_width, _picture_height) = match images.into_iter().next() {
        Some(img) => (img.url, img.width, img.height),
        None => ("".to_string(), None, None),
    };

    Ok(Some(ArtistData {
        name: artist.name,
        picture_url,
        genres: artist.genres,
    }))
}

/// Fetch song data by searching for a track name and optional artist name.
/// Returns enriched song data with album art, artist info, and metadata.
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
pub async fn get_song_data(song_name: &str, artist_name: Option<&str>) -> Result<Option<SongData>, String> {
    let encoded_song = urlencoding::encode(song_name);

    let query = if let Some(artist) = artist_name {
        let encoded_artist = urlencoding::encode(artist);
        format!("track%3A%22{}%22%20artist%3A%22{}%22", encoded_song, encoded_artist)
    } else {
        format!("track%3A%22{}%22", encoded_song)
    };

    let url = format!(
        "https://api.spotify.com/v1/search?q={}&type=track&limit=10",
        query
    );

    debug!(%url, song_name, ?artist_name, "Sending Spotify track search request");

    let res = api_call::<TrackSearchResponse>(&url).await?;

    debug!(
        song_name,
        ?artist_name,
        count = res.tracks.items.len(),
        "Received track search results"
    );

    for track in &res.tracks.items {
        let song_matched = track.name.eq_ignore_ascii_case(song_name);
        let artist_matched = artist_name.map_or(true, |name| {
            track.artists.iter().any(|artist| artist.name.eq_ignore_ascii_case(name))
        });

        debug!(
            song_name,
            candidate_song = %track.name,
            candidate_artists = ?track.artists.iter().map(|a| &a.name).collect::<Vec<_>>(),
            song_matched,
            artist_matched,
            "Evaluating track match"
        );
    }

    let track = match res.tracks.items.into_iter().find(|track| {
        let song_matched = track.name.eq_ignore_ascii_case(song_name);
        let artist_matched = artist_name.map_or(true, |name| {
            track.artists.iter().any(|artist| artist.name.eq_ignore_ascii_case(name))
        });
        song_matched && artist_matched
    }) {
        Some(t) => {
            debug!(song_name, ?artist_name, track_id = %t.id, "Matched track ID");
            t
        }
        None => {
            debug!(song_name, ?artist_name, "No exact match found for track");
            return Ok(None);
        }
    };

    // Get the largest album image
    let album_art_url = track.album.images
        .into_iter()
        .max_by_key(|img| img.width.unwrap_or(0))
        .map(|img| img.url)
        .unwrap_or_default();

    // Collect artist names
    let artists: Vec<String> = track.artists.iter().map(|a| a.name.clone()).collect();

    Ok(Some(SongData {
        name: track.name,
        artists,
        album_name: track.album.name,
        album_art_url,
        album_type: track.album.album_type,
    }))
}
pub fn is_spotify_enabled() -> bool {
    *SPOTIFY_ENABLED
}