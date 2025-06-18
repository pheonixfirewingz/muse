use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use tokio::fs;

static CACHE_DIR: Lazy<String> = Lazy::new(|| {
    env::var("CACHE_DIR").expect("CACHE_DIR must be set")
});

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + for<'d> Deserialize<'d>")]
pub enum Cached<T> {
    Found(T),
    NotFound,
}

#[derive(Debug)]
pub enum CacheError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Other(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::Io(e) => write!(f, "IO error: {}", e),
            CacheError::Json(e) => write!(f, "JSON error: {}", e),
            CacheError::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl std::error::Error for CacheError {}

impl From<std::io::Error> for CacheError {
    fn from(err: std::io::Error) -> Self {
        CacheError::Io(err)
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(err: serde_json::Error) -> Self {
        CacheError::Json(err)
    }
}

pub async fn load_cache<T>(name: &str,sub_dir:&str) -> Result<Option<Cached<T>>, CacheError>
where T: for<'de> Deserialize<'de> + Serialize,
{
    let file_name = format!("{}.json", name.replace(' ', "_").to_lowercase());
    let path: PathBuf = [(CACHE_DIR.clone() + "/" + sub_dir).as_str(), &file_name].iter().collect();
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path).await?;
    let parsed = serde_json::from_str::<Cached<T>>(&content)?;
    Ok(Some(parsed))
}

pub async fn store_cache<T>(data: &Cached<T>, name: &str,sub_dir:&str) -> Result<(), CacheError>
where T: for<'de> Deserialize<'de> + Serialize,
{
    let dir_path = CACHE_DIR.clone() + "/" + sub_dir;
    let cache_path = Path::new(&dir_path);
    if !cache_path.exists() {
        fs::create_dir_all(cache_path).await.expect("Failed to create cache directory");
    }
    let file_name = format!("{}.json", name.replace(' ', "_").to_lowercase());
    let path: PathBuf = [&dir_path, &file_name].iter().collect();
    let json = serde_json::to_string_pretty(data)?;
    fs::write(path, json).await?;
    Ok(())
}

pub async fn init_cache() {
    let cache_path = Path::new(CACHE_DIR.as_str());
    if !cache_path.exists() {
        fs::create_dir_all(cache_path).await.expect("Failed to create cache directory");
    }
}