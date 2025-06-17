use std::path::Path;
use std::fs;
use reqwest;
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

const MUSICBRAINZ_API_BASE: &str = "https://musicbrainz.org/ws/2";
const COVER_ART_ARCHIVE_BASE: &str = "https://coverartarchive.org";
const CACHE_DIR: &str = "runtime/cache/images";
const USER_AGENT: &str = "Muse/1.0 (https://github.com/digitech/muse)";

#[derive(Debug, Deserialize)]
struct MusicBrainzArtistResponse {
    artists: Vec<MusicBrainzArtist>,
}

#[derive(Debug, Deserialize)]
struct MusicBrainzArtist {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MusicBrainzReleaseResponse {
    releases: Vec<MusicBrainzRelease>,
}

#[derive(Debug, Deserialize)]
struct MusicBrainzRelease {
    id: String,
    title: String,
}

#[derive(Debug, Deserialize)]
struct CoverArtResponse {
    images: Vec<CoverArtImage>,
}

#[derive(Debug, Deserialize)]
struct CoverArtImage {
    image: String,
    front: bool,
}

#[derive(Debug, Serialize)]
pub struct ImageResult {
    pub url: String,
    pub data: String, // base64 encoded image data
}

fn check_notfound_cache(key: &str, image_type: &str) -> bool {
    let file_path = Path::new(CACHE_DIR)
        .join(image_type)
        .join(format!("{}.notfound", key));
    
    file_path.exists()
}

fn create_notfound_cache(key: &str, image_type: &str) {
    let cache_path = Path::new(CACHE_DIR).join(image_type);
    fs::create_dir_all(&cache_path).ok();
    
    let file_path = cache_path.join(format!("{}.notfound", key));
    fs::write(file_path, "").ok();
}

pub async fn get_artist_image(artist_name: &str) -> Option<ImageResult> {
    if check_notfound_cache(artist_name, "artist") {
        debug!("Not found cache hit for artist: {}", artist_name);
        return None;
    }

    if let Some(cached) = get_cached_image(artist_name, "artist") {
        debug!("Cached image hit for artist: {}", artist_name);
        return Some(cached);
    }

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let response = client.get(format!("{}/artist", MUSICBRAINZ_API_BASE))
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .query(&[
            ("query", artist_name),
            ("fmt", &String::from("json")),
            ("limit", &String::from("5")) // get more results for fuzzy matching
        ])
        .send()
        .await;

    let response = match response {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error querying MusicBrainz API: {}", e);
            create_notfound_cache(artist_name, "artist");
            return None;
        }
    };

    let response_text = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Error getting response text: {}", e);
            create_notfound_cache(artist_name, "artist");
            return None;
        }
    };

    let mb_response: MusicBrainzArtistResponse = match serde_json::from_str::<MusicBrainzArtistResponse>(&response_text) {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error parsing MusicBrainz response: {}", e);
            eprintln!("Response text was: {}", response_text);
            create_notfound_cache(artist_name, "artist");
            return None;
        }
    };

    use strsim::levenshtein;

    let artist = mb_response.artists
        .iter()
        .min_by_key(|a| levenshtein(&a.name.to_lowercase(), &artist_name.to_lowercase()))
        .filter(|a| levenshtein(&a.name.to_lowercase(), &artist_name.to_lowercase()) < 5); // tune threshold as needed

    let artist = match artist {
        Some(a) => {
            debug!("Selected fuzzy match artist: {} (ID: {})", a.name, a.id);
            a
        }
        None => {
            debug!("No suitable fuzzy match found for artist: {}", artist_name);
            create_notfound_cache(artist_name, "artist");
            return None;
        }
    };

    debug!("Querying MusicBrainz for artist's releases");

    let releases_response = client.get(format!("{}/release", MUSICBRAINZ_API_BASE))
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .query(&[
            ("artist", &artist.id),
            ("fmt", &String::from("json")),
            ("limit", &String::from("1"))
        ])
        .send()
        .await;

    let releases_response = match releases_response {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Failed to get MusicBrainz releases response: {}", e);
            create_notfound_cache(artist_name, "artist");
            return None;
        }
    };

    let response_text = match releases_response.text().await {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Error getting releases response text: {}", e);
            create_notfound_cache(artist_name, "artist");
            return None;
        }
    };

    let releases_data: MusicBrainzReleaseResponse = match serde_json::from_str(&response_text) {
        Ok(data) => data,
        Err(e) => {
            debug!("Failed to parse MusicBrainz releases response: {}", e);
            create_notfound_cache(artist_name, "artist");
            return None;
        }
    };

    let release = match releases_data.releases.first() {
        Some(r) => r, 
        None => {
            debug!("No releases found for artist");
            create_notfound_cache(artist_name, "artist");
            return None;
        }
    };

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    debug!("Querying Cover Art Archive for release ID: {}", release.id);
    let cover_art_response = client.get(format!("{}/release/{}", COVER_ART_ARCHIVE_BASE, release.id))
        .header("Accept", "application/json")
        .send()
        .await;

    let cover_art_response = match cover_art_response {
        Ok(resp) => resp,
        Err(_) => {
            debug!("Failed to get Cover Art Archive response");
            create_notfound_cache(artist_name, "artist");
            return None;
        }
    };
    
    if cover_art_response.status() == 404 {
        debug!("No cover art found for this release");
        create_notfound_cache(artist_name, "artist");
        return None;
    }

    let response_text = match cover_art_response.text().await {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Error getting Cover Art Archive response text: {}", e);
            create_notfound_cache(artist_name, "artist");
            return None;
        }
    };

    let cover_art_data: CoverArtResponse = match serde_json::from_str(&response_text) {
        Ok(data) => data,
        Err(_) => {
            debug!("Failed to parse Cover Art Archive response");
            create_notfound_cache(artist_name, "artist");
            return None;
        }
    };

    if let Some(image) = cover_art_data.images.iter().find(|img| img.front) {
        debug!("Found front image URL: {}", image.image);
        if let Some(image_data) = download_and_cache_image(&image.image, artist_name, "artist").await {
            return Some(image_data);
        }
    } else {
        debug!("No front image found in Cover Art Archive response");
        create_notfound_cache(artist_name, "artist");
    }
    None
}


use strsim::jaro_winkler;
use tracing::debug;

pub async fn get_album_image(artist_name: &str, album_title: &str) -> Option<ImageResult> {
    let cache_key = format!("{} - {}", artist_name, album_title);

    if check_notfound_cache(&cache_key, "album") {
        debug!("Not found cache hit for album: {}", cache_key);
        return None;
    }

    if let Some(cached) = get_cached_image(&cache_key, "album") {
        debug!("Cached image hit for album: {}", cache_key);
        return Some(cached);
    }

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/release", MUSICBRAINZ_API_BASE))
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .query(&[
            ("query", &format!("artist:\"{}\" AND release:\"{}\"", artist_name, album_title)),
            ("fmt", &String::from("json")),
            ("limit", &String::from("10")),
        ])
        .send()
        .await;

    let response = match response {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error querying MusicBrainz API: {}", e);
            create_notfound_cache(&cache_key, "album");
            return None;
        }
    };

    let response_text = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Error reading MusicBrainz response: {}", e);
            create_notfound_cache(&cache_key, "album");
            return None;
        }
    };

    let mb_response = match serde_json::from_str::<MusicBrainzReleaseResponse>(&response_text) {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error parsing MusicBrainz release response: {}", e);
            create_notfound_cache(&cache_key, "album");
            return None;
        }
    };

    let mut best_match = None;
    let mut best_score = 0.0;

    for release in mb_response.releases.iter() {
        let score = jaro_winkler(&release.title.to_lowercase(), &album_title.to_lowercase());
        if score > best_score {
            best_score = score;
            best_match = Some(release);
        }
    }

    if let Some(release) = best_match {
        if best_score < 0.55 {
            debug!("Fuzzy match score too low ({:.2}) for album '{}'", best_score, release.title);
            create_notfound_cache(&cache_key, "album");
            return None;
        }

        debug!("Fuzzy matched release: {} (ID: {}, score: {:.2})", release.title, release.id, best_score);

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let cover_art_response = client
            .get(format!("{}/release/{}", COVER_ART_ARCHIVE_BASE, release.id))
            .header("Accept", "application/json")
            .send()
            .await;

        if let Ok(cover_art_response) = cover_art_response {
            if cover_art_response.status() == 404 {
                debug!("No cover art found for album");
                create_notfound_cache(&cache_key, "album");
                return None;
            }

            let cover_text = match cover_art_response.text().await {
                Ok(text) => text,
                Err(e) => {
                    eprintln!("Error reading Cover Art Archive response: {}", e);
                    create_notfound_cache(&cache_key, "album");
                    return None;
                }
            };

            if let Ok(cover_data) = serde_json::from_str::<CoverArtResponse>(&cover_text) {
                if let Some(image) = cover_data.images.iter().find(|img| img.front) {
                    debug!("Found album front image: {}", image.image);
                    if let Some(image_data) = download_and_cache_image(&image.image, &cache_key, "album").await {
                        return Some(image_data);
                    }
                } else {
                    debug!("No front image found in cover art response");
                    create_notfound_cache(&cache_key, "album");
                }
            } else {
                debug!("Failed to parse Cover Art Archive JSON");
                create_notfound_cache(&cache_key, "album");
            }
        } else {
            debug!("Failed to fetch Cover Art Archive");
            create_notfound_cache(&cache_key, "album");
        }
    } else {
        debug!(
            "{} No matching album found for artist {} by this name registered to MusicBrainz",
            album_title, artist_name
        );
        create_notfound_cache(&cache_key, "album");
    }

    None
}

async fn download_and_cache_image(url: &str, key: &str, image_type: &str) -> Option<ImageResult> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await.ok()?;
    let image_data = response.bytes().await.ok()?;
    
    // Create cache directory if it doesn't exist
    let cache_path = Path::new(CACHE_DIR).join(image_type);
    fs::create_dir_all(&cache_path).ok()?;
    
    // Save to cache
    let file_path = cache_path.join(format!("{}.jpg", key));
    fs::write(&file_path, &image_data).ok()?;
    
    // Convert to base64
    let base64_data = BASE64.encode(&image_data);
    
    Some(ImageResult {
        url: url.to_string(),
        data: base64_data,
    })
}

fn get_cached_image(key: &str, image_type: &str) -> Option<ImageResult> {
    let file_path = Path::new(CACHE_DIR)
        .join(image_type)
        .join(format!("{}.jpg", key));
    
    if file_path.exists() {
        if let Ok(image_data) = fs::read(file_path) {
            let base64_data = BASE64.encode(&image_data);
            return Some(ImageResult {
                url: format!("cache://{}/{}", image_type, key),
                data: base64_data,
            });
        }
    }
    
    None
}

// Initialize cache directory
pub fn init_cache() {
    let cache_path = Path::new(CACHE_DIR);
    if !cache_path.exists() {
        fs::create_dir_all(cache_path).expect("Failed to create cache directory");
    }
} 