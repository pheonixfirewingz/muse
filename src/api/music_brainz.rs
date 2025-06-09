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
    // Check .notfound cache first
    if check_notfound_cache(artist_name, "artist") {
        println!("Not found cache hit for artist: {}", artist_name);
        return None;
    }

    // Check regular cache
    if let Some(cached) = get_cached_image(artist_name, "artist") {
        println!("Cached image hit for artist: {}", artist_name);
        return Some(cached);
    }

    //we need to rate limit this so add a sleep
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Query MusicBrainz API for artist ID
    let client = reqwest::Client::new();
    let response = client.get(format!("{}/artist", MUSICBRAINZ_API_BASE))
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .query(&[
            ("query", artist_name),
            ("fmt", &String::from("json")),
            ("limit", &String::from("1"))
        ])
        .send()
        .await;
    
    if let Err(e) = response {
        eprintln!("Error querying MusicBrainz API: {}", e);
        create_notfound_cache(artist_name, "artist");
        return None;
    }
    let response = response.unwrap();
    println!("MusicBrainz API Response status: {}", response.status());

    let response_text = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            println!("Error getting response text: {}", e);
            create_notfound_cache(artist_name, "artist");
            return None;
        }
    };
    println!("Raw MusicBrainz API Response: {}", response_text);

    let mb_response = match serde_json::from_str::<MusicBrainzArtistResponse>(&response_text) {
        Ok(resp) => {
            println!("MusicBrainz API Response parsed successfully. Found {} artists", resp.artists.len());
            resp
        },
        Err(e) => {
            println!("Error parsing MusicBrainz response: {}", e);
            println!("Response text was: {}", response_text);
            create_notfound_cache(artist_name, "artist");
            return None;
        }
    };
    
    if let Some(artist) = mb_response.artists.first() {
        println!("Found artist: {} (ID: {})", artist.name, artist.id);
        println!("Querying MusicBrainz for artist's releases");
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

        if let Ok(releases_response) = releases_response {
            println!("MusicBrainz Releases Response status: {}", releases_response.status());
            let response_text = match releases_response.text().await {
                Ok(text) => text,
                Err(e) => {
                    println!("Error getting releases response text: {}", e);
                    create_notfound_cache(artist_name, "artist");
                    return None;
                }
            };
            println!("Raw MusicBrainz Releases Response: {}", response_text);

            if let Ok(releases_data) = serde_json::from_str::<MusicBrainzReleaseResponse>(&response_text) {
                if let Some(release) = releases_data.releases.first() {
                    println!("Found release: {} (ID: {})", release.title, release.id);
                    
                    // Now query Cover Art Archive for the release's image
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    
                    println!("Querying Cover Art Archive for release ID: {}", release.id);
                    let cover_art_response = client.get(format!("{}/release/{}", COVER_ART_ARCHIVE_BASE, release.id))
                        .header("Accept", "application/json")
                        .send()
                        .await;

                    if let Ok(cover_art_response) = cover_art_response {
                        println!("Cover Art Archive Response status: {}", cover_art_response.status());
                        
                        // Handle 404 specifically - no cover art found
                        if cover_art_response.status() == 404 {
                            println!("No cover art found for this release");
                            create_notfound_cache(artist_name, "artist");
                            return None;
                        }

                        let response_text = match cover_art_response.text().await {
                            Ok(text) => text,
                            Err(e) => {
                                println!("Error getting Cover Art Archive response text: {}", e);
                                create_notfound_cache(artist_name, "artist");
                                return None;
                            }
                        };
                        println!("Raw Cover Art Archive Response: {}", response_text);

                        if let Ok(cover_art_data) = serde_json::from_str::<CoverArtResponse>(&response_text) {
                            println!("Found {} images in Cover Art Archive response", cover_art_data.images.len());
                            if let Some(image) = cover_art_data.images.iter().find(|img| img.front) {
                                println!("Found front image URL: {}", image.image);
                                // Download and cache the image
                                if let Some(image_data) = download_and_cache_image(&image.image, artist_name, "artist").await {
                                    return Some(image_data);
                                }
                            } else {
                                println!("No front image found in Cover Art Archive response");
                                create_notfound_cache(artist_name, "artist");
                            }
                        } else {
                            println!("Failed to parse Cover Art Archive response");
                            create_notfound_cache(artist_name, "artist");
                        }
                    } else {
                        println!("Failed to get Cover Art Archive response");
                        create_notfound_cache(artist_name, "artist");
                    }
                } else {
                    println!("No releases found for artist");
                    create_notfound_cache(artist_name, "artist");
                }
            } else {
                println!("Failed to parse MusicBrainz releases response");
                create_notfound_cache(artist_name, "artist");
            }
        } else {
            println!("Failed to get MusicBrainz releases response");
            create_notfound_cache(artist_name, "artist");
        }
    } else {
        println!("No artist found in MusicBrainz response");
        create_notfound_cache(artist_name, "artist");
    }
    
    None
}

pub async fn get_album_image(artist_name: &str, album_name: &str) -> Option<ImageResult> {
    let cache_key = format!("{}_{}", artist_name, album_name);
    
    // Check .notfound cache first
    if check_notfound_cache(&cache_key, "album") {
        println!("Not found cache hit for album: {} by {}", album_name, artist_name);
        return None;
    }
    
    // Check regular cache
    if let Some(cached) = get_cached_image(&cache_key, "album") {
        println!("Cached image hit for album: {} by {}", album_name, artist_name);
        return Some(cached);
    }
    
    //we need to rate limit this so add a sleep
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Query MusicBrainz API for release ID
    let client = reqwest::Client::new();
    let response = client.get(format!("{}/release", MUSICBRAINZ_API_BASE))
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .query(&[
            ("query", &format!("artist:\"{}\" AND release:\"{}\"", artist_name, album_name)),
            ("fmt", &String::from("json")),
            ("limit", &String::from("1"))
        ])
        .send()
        .await;

    if let Err(e) = response {
        println!("Error querying MusicBrainz API: {}", e);
        create_notfound_cache(&cache_key, "album");
        return None;
    }
    let response = response.unwrap();
    println!("MusicBrainz API Response status: {}", response.status());

    let response_text = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            println!("Error getting response text: {}", e);
            create_notfound_cache(&cache_key, "album");
            return None;
        }
    };
    println!("Raw MusicBrainz API Response: {}", response_text);

    let mb_response: MusicBrainzReleaseResponse = match serde_json::from_str(&response_text) {
        Ok(resp) => resp,
        Err(e) => {
            println!("Error parsing MusicBrainz response: {}", e);
            println!("Response text was: {}", response_text);
            create_notfound_cache(&cache_key, "album");
            return None;
        }
    };
    
    if let Some(release) = mb_response.releases.first() {
        println!("Found release: {} (ID: {})", release.title, release.id);
        
        // Now query Cover Art Archive for the release's image
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        println!("Querying Cover Art Archive for release ID: {}", release.id);
        let cover_art_response = client.get(format!("{}/release/{}", COVER_ART_ARCHIVE_BASE, release.id))
            .header("Accept", "application/json")
            .send()
            .await;

        if let Ok(cover_art_response) = cover_art_response {
            println!("Cover Art Archive Response status: {}", cover_art_response.status());
            
            // Handle 404 specifically - no cover art found
            if cover_art_response.status() == 404 {
                println!("No cover art found for this release");
                create_notfound_cache(&cache_key, "album");
                return None;
            }

            let response_text = match cover_art_response.text().await {
                Ok(text) => text,
                Err(e) => {
                    println!("Error getting Cover Art Archive response text: {}", e);
                    create_notfound_cache(&cache_key, "album");
                    return None;
                }
            };
            println!("Raw Cover Art Archive Response: {}", response_text);

            if let Ok(cover_art_data) = serde_json::from_str::<CoverArtResponse>(&response_text) {
                println!("Found {} images in Cover Art Archive response", cover_art_data.images.len());
                if let Some(image) = cover_art_data.images.iter().find(|img| img.front) {
                    println!("Found front image URL: {}", image.image);
                    // Download and cache the image
                    if let Some(image_data) = download_and_cache_image(&image.image, &cache_key, "album").await {
                        return Some(image_data);
                    }
                } else {
                    println!("No front image found in Cover Art Archive response");
                    create_notfound_cache(&cache_key, "album");
                }
            } else {
                println!("Failed to parse Cover Art Archive response");
                create_notfound_cache(&cache_key, "album");
            }
        } else {
            println!("Failed to get Cover Art Archive response");
            create_notfound_cache(&cache_key, "album");
        }
    } else {
        println!("No release found in MusicBrainz response");
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