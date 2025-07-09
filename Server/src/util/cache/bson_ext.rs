// BSON format implementation for cache serialization
// This is the primary cache format, providing compact binary serialization
// with zstd compression for optimal performance and file size
use crate::util::cache::{CacheConfig, CacheEntry, CacheError, CacheFormat, CacheResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use zstd;

pub struct BsonFormat;

impl Default for BsonFormat {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl CacheFormat for BsonFormat {
    const EXTENSION: &'static str = "bson.zst";
    const MAGIC_BYTES: &'static [u8] = b"ZBSN"; // "Zstd BSON" magic bytes
    const VERSION: u8 = 1;

    async fn serialize<T>(&self, value: &CacheEntry<T>, _config: &CacheConfig) -> CacheResult<Vec<u8>>
    where
        T: Serialize + for<'de> Deserialize<'de> + Send + Sync + bincode::Encode,
    {
        let mut result = Vec::new();
        result.extend_from_slice(Self::MAGIC_BYTES);
        result.push(Self::VERSION);

        // Serialize to BSON first
        let bson_data = bson::to_vec(value)?;

        // Compress with zstd
        // Use default compression level of 3
        let compression_level = 3;
        let compressed_data = zstd::encode_all(bson_data.as_slice(), compression_level)?;

        // Store the original size for potential future use
        result.extend_from_slice(&(bson_data.len() as u32).to_le_bytes());
        result.extend_from_slice(&compressed_data);

        Ok(result)
    }

    async fn deserialize<T>(&self, bytes: &[u8], _config: &CacheConfig) -> CacheResult<CacheEntry<T>>
    where
        T: Serialize + for<'de> Deserialize<'de> + Send + Sync,
    {
        self.validate_format(bytes)?;

        let header_size = Self::MAGIC_BYTES.len() + 1 + 4; // magic + version + original_size
        if bytes.len() < header_size {
            return Err(CacheError::Unsupported("Invalid file format: too short".to_string()));
        }

        let data_start = Self::MAGIC_BYTES.len() + 1;

        // Read the original size (for validation if needed)
        let original_size_bytes = &bytes[data_start..data_start + 4];
        let _original_size = u32::from_le_bytes([
            original_size_bytes[0],
            original_size_bytes[1],
            original_size_bytes[2],
            original_size_bytes[3]
        ]);

        // Decompress the data
        let compressed_data = &bytes[data_start + 4..];
        let decompressed_data = zstd::decode_all(compressed_data)?;

        // Deserialize from BSON
        Ok(bson::from_slice(&decompressed_data)?)
    }

    fn should_auto_expire(&self) -> bool {
        true // BSON files auto-expire after 7 days
    }
} 