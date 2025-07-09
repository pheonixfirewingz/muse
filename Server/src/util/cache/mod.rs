use std::{
    env,
    path::{Path, PathBuf},
    process::exit,
    time::{Duration, SystemTime, UNIX_EPOCH},
    hash::{Hash, Hasher},
    collections::hash_map::DefaultHasher,
};
use tokio::fs;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use tracing::{error, debug};
use thiserror::Error;
use async_trait::async_trait;
use bincode::{Decode, Encode};

pub mod bson_ext;
pub mod img_ext;

static CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    env::var("CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            error!("CACHE_DIR environment variable must be set for muse to run");
            exit(1);
        })
});

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(bound = "T: Serialize + for<'d> Deserialize<'d>")]
pub enum CacheEntry<T> {
    Hit {
        data: T,
        created_at: u64,
        size_bytes: u64,
    },
    Miss,
}

impl<T> CacheEntry<T> {
    pub fn new_hit(data: T, size_bytes: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self::Hit {
            data,
            created_at: now,
            size_bytes,
        }
    }

    pub fn get_data(&self) -> Option<&T> {
        match self {
            Self::Hit { data, .. } => Some(data),
            Self::Miss => None,
        }
    }

    pub fn is_older_than(&self, max_age: Duration) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        match self {
            Self::Hit { created_at, .. } => {
                now.saturating_sub(*created_at) > max_age.as_secs()
            }
            Self::Miss => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_age: Duration,
    pub max_size_mb: Option<u64>,
    pub auto_cleanup: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_age: Duration::from_secs(7 * 24 * 3600), // 7 days
            max_size_mb: Some(1024),
            auto_cleanup: true,
        }
    }
}

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("BSON serialization error: {0}")]
    Bson(#[from] bson::ser::Error),
    #[error("BSON deserialization error: {0}")]
    BsonDeserialize(#[from] bson::de::Error),
    #[error("Compression error: {0}")]
    Compression(String),
    #[error("Invalid cache format: {0}")]
    InvalidFormat(String),
    #[error("Version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u8, found: u8 },
    #[error("Entry too large: {size} bytes (max: {max})")]
    EntryTooLarge { size: u64, max: u64 },
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
}

pub type CacheResult<T> = Result<T, CacheError>;

#[async_trait]
pub trait CacheFormat: Send + Sync + 'static {
    const EXTENSION: &'static str;
    const MAGIC_BYTES: &'static [u8];
    const VERSION: u8;

    async fn serialize<T>(&self, value: &CacheEntry<T>, config: &CacheConfig) -> CacheResult<Vec<u8>>
    where
        T: Serialize + for<'de> Deserialize<'de> + Send + Sync + bincode::Encode;

    async fn deserialize<T>(&self, bytes: &[u8], config: &CacheConfig) -> CacheResult<CacheEntry<T>>
    where
        T: Serialize + for<'de> Deserialize<'de> + Send + Sync + bincode::Decode<()>;

    fn validate_format(&self, bytes: &[u8]) -> CacheResult<()> {
        if !bytes.starts_with(Self::MAGIC_BYTES) {
            return Err(CacheError::InvalidFormat("Invalid magic bytes".into()));
        }
        if bytes.len() < Self::MAGIC_BYTES.len() + 1 {
            return Err(CacheError::InvalidFormat("File too short".into()));
        }
        let version = bytes[Self::MAGIC_BYTES.len()];
        if version != Self::VERSION {
            return Err(CacheError::VersionMismatch {
                expected: Self::VERSION,
                found: version,
            });
        }
        Ok(())
    }
    
    /// Whether this format should auto-expire entries
    fn should_auto_expire(&self) -> bool {
        true // Default to auto-expiring
    }
}

/// Generate a safe cache key from input
pub fn cache_key(name: &str) -> String {
    // Create a hash-based key for very long names
    if name.len() > 100 {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        format!("hash_{:x}", hasher.finish())
    } else {
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect::<String>()
            .to_lowercase()
    }
}

/// Simplified cache manager
pub struct CacheManager {
    config: CacheConfig,
}

impl CacheManager {
    pub fn new(config: CacheConfig) -> Self {
        Self { config }
    }

    /// Load a cached object
    pub async fn load<T, F>(&self, name: &str, sub_dir: &str) -> CacheResult<CacheEntry<T>>
    where
        T: for<'de> Deserialize<'de> + Serialize + Send + Sync + bincode::Encode + bincode::Decode<()>,
        F: CacheFormat + Default,
    {
        let format = F::default();
        let file_name = format!("{}.{}", cache_key(name), F::EXTENSION);
        let path = self.build_path(sub_dir, &file_name)?;

        if !path.exists() {
            debug!("Cache miss: file does not exist");
            return Ok(CacheEntry::Miss);
        }

        let content = fs::read(&path).await?;

        // Validate format first
        format.validate_format(&content)?;

        let entry = format.deserialize(&content, &self.config).await?;

        // Check if expired and delete if so (only if format supports auto-expiration)
        if format.should_auto_expire() && entry.is_older_than(self.config.max_age) {
            debug!("Cache entry expired, deleting");
            let _ = fs::remove_file(&path).await; // Don't fail on deletion errors
            return Ok(CacheEntry::Miss);
        }

        Ok(entry)
    }

    /// Store a cached object
    pub async fn store<T, F>(&self, data: &CacheEntry<T>, name: &str, sub_dir: &str) -> CacheResult<()>
    where
        T: for<'de> Deserialize<'de> + Serialize + Send + Sync + bincode::Encode,
        F: CacheFormat + Default,
    {
        let format = F::default();
        let dir_path = self.build_dir_path(sub_dir)?;
        self.ensure_dir_exists(&dir_path).await?;

        // Auto-cleanup old entries if enabled
        if self.config.auto_cleanup {
            self.cleanup_old_entries::<F>(&dir_path).await?;
        }

        let file_name = format!("{}.{}", cache_key(name), F::EXTENSION);
        let path = dir_path.join(file_name);
        let content = format.serialize(data, &self.config).await?;

        // Check size limits
        if let Some(max_size_mb) = self.config.max_size_mb {
            let max_bytes = max_size_mb * 1024 * 1024;
            if content.len() as u64 > max_bytes {
                return Err(CacheError::EntryTooLarge {
                    size: content.len() as u64,
                    max: max_bytes,
                });
            }
        }

        fs::write(&path, &content).await?;
        debug!("Successfully stored cache entry");
        Ok(())
    }
    
    /// Clean up old cache entries
    async fn cleanup_old_entries<F>(&self, dir_path: &Path) -> CacheResult<()>
    where
        F: CacheFormat + Default,
    {
        let format = F::default();
        let extension = F::EXTENSION;
        
        // Only clean up if this format supports auto-expiration
        if !format.should_auto_expire() {
            return Ok(());
        }
        
        if let Ok(mut entries) = fs::read_dir(dir_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == extension {
                        // Try to read and check if expired
                        if let Ok(content) = fs::read(&path).await {
                            // Use a simple struct that implements the required traits
                            #[derive(Serialize, Deserialize, Encode, Decode)]
                            struct TempData(String);
                            
                            if let Ok(entry) = format.deserialize::<TempData>(&content, &self.config).await {
                                if entry.is_older_than(self.config.max_age) {
                                    let _ = fs::remove_file(&path).await;
                                    debug!("Cleaned up expired cache entry: {:?}", path);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    // Private helper methods
    fn build_path(&self, sub_dir: &str, file_name: &str) -> CacheResult<PathBuf> {
        Ok(self.build_dir_path(sub_dir)?.join(file_name))
    }

    fn build_dir_path(&self, sub_dir: &str) -> CacheResult<PathBuf> {
        Ok(CACHE_DIR.join(sub_dir))
    }

    async fn ensure_dir_exists(&self, path: &Path) -> CacheResult<()> {
        if !path.exists() {
            fs::create_dir_all(path).await?;
        }
        Ok(())
    }
}

/// Global cache manager instance
static CACHE_MANAGER: Lazy<CacheManager> = Lazy::new(|| {
    CacheManager::new(CacheConfig::default())
});

/// Convenience functions for backward compatibility
pub async fn load_cache<T, F>(name: &str, sub_dir: &str) -> CacheResult<Option<CacheEntry<T>>>
where
    T: for<'de> Deserialize<'de> + Serialize + Send + Sync + bincode::Encode + bincode::Decode<()>,
    F: CacheFormat + Default,
{
    match CACHE_MANAGER.load::<T, F>(name, sub_dir).await? {
        CacheEntry::Miss => Ok(None),
        entry => Ok(Some(entry)),
    }
}

pub async fn store_cache<T, F>(data: &CacheEntry<T>, name: &str, sub_dir: &str) -> CacheResult<()>
where
    T: for<'de> Deserialize<'de> + Serialize + Send + Sync + bincode::Encode,
    F: CacheFormat + Default,
{
    CACHE_MANAGER.store::<T, F>(data, name, sub_dir).await
}

/// Initialize the cache system
pub async fn init_cache() -> CacheResult<()> {
    let cache_path = CACHE_DIR.as_path();

    if !cache_path.exists() {
        fs::create_dir_all(cache_path).await?;
    }

    Ok(())
}