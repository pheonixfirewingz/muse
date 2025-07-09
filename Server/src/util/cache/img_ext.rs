use crate::util::cache::{CacheConfig, CacheEntry, CacheError, CacheFormat, CacheResult};
use async_trait::async_trait;
use ravif::{Encoder, Img};
use rgb::RGBA;
use serde::{Deserialize, Serialize};
use tokio::task;
use tracing::{info, warn, error};

pub struct ImageFormat {
    pub quality: u8,
    pub speed: u8,
}

impl Default for ImageFormat {
    fn default() -> Self {
        Self {
            quality: 80,
            speed: 6,
        }
    }
}

impl ImageFormat {
    async fn convert_to_avif(&self, image_bytes: &[u8]) -> CacheResult<Vec<u8>> {
        if self.is_avif(image_bytes) {
            info!("Input is already AVIF format, skipping conversion");
            return Ok(image_bytes.to_vec());
        }
        
        let sanitized = self.find_image_start(image_bytes)
            .ok_or_else(|| {
                error!("Failed to identify supported image format in byte stream");
                CacheError::InvalidFormat("Unknown or unsupported image format".to_string())
            })?;
        
        let img = image::load_from_memory(sanitized)
            .map_err(|e| {
                error!("Image decoding failed: {}", e);
                CacheError::InvalidFormat(format!("Failed to decode image: {}", e))
            })?;

        let (width, height) = (img.width(), img.height());
        info!("Successfully decoded image: {}x{} pixels", width, height);

        let rgba_img = img.to_rgba8();
        
        let rgba_pixels: Vec<RGBA<u8>> = rgba_img
            .pixels()
            .map(|p| RGBA::new(p[0], p[1], p[2], p[3]))
            .collect();

        let quality = self.quality;
        let speed = self.speed;
        // Move encoding to a blocking task
        let avif_result = task::spawn_blocking(move || {
            // Convert moved Vec into a slice inside the closure
            let img_buffer = Img::new(rgba_pixels.as_slice(), width as usize, height as usize);
            let encoder = Encoder::new()
                .with_quality(quality as f32)
                .with_alpha_quality(quality as f32)
                .with_speed(speed);
            let start_time = std::time::Instant::now();
            let result = encoder.encode_rgba(img_buffer);
            let encoding_duration = start_time.elapsed();

            match &result {
                Ok(_) => {
                    info!("AVIF encoding successful in {:?}", encoding_duration);
                }
                Err(e) => {
                    error!("AVIF encoding failed after {:?}: {}", encoding_duration, e);
                }
            }

            result
        }).await.map_err(|e| {
            error!("Blocking task join error: {}", e);
            CacheError::Compression(format!("Task join error: {}", e))
        })?
            .map_err(|e| {
                error!("AVIF encoding error: {}", e);
                CacheError::Compression(format!("AVIF encoding failed: {}", e))
            })?;

        let output_size = avif_result.avif_file.len();
        let compression_ratio = (output_size as f32 / image_bytes.len() as f32) * 100.0;
        info!("AVIF conversion complete: {} -> {} bytes ({:.1}% of original)", 
              image_bytes.len(), output_size, compression_ratio);

        Ok(avif_result.avif_file)
    }
    fn is_avif(&self, bytes: &[u8]) -> bool {

        if bytes.len() < 12 {
            return false;
        }
        &bytes[4..8] == b"ftyp" && (&bytes[8..12] == b"avif" || &bytes[8..12] == b"avis")
    }
    
    fn find_image_start<'a>(&self, data: &'a [u8]) -> Option<&'a [u8]> {
        const HEADERS: &[(&[u8], &str)] = &[
            (&[0xFF, 0xD8], "JPEG"),
            (&[0x89, b'P', b'N', b'G'], "PNG"),
            (&[b'G', b'I', b'F', b'8'], "GIF"),
            (&[b'B', b'M'], "BMP"),
            (&[b'I', b'I', 0x2A, 0x00], "TIFF (LE)"),
            (&[b'M', b'M', 0x00, 0x2A], "TIFF (BE)"),
            (&[b'R', b'I', b'F', b'F'], "WebP"),
            (&[0x00, 0x00, 0x01, 0x00], "ICO"),
            (&[b'P', b'1'], "PNM P1"), (&[b'P', b'2'], "PNM P2"), (&[b'P', b'3'], "PNM P3"),
            (&[b'P', b'4'], "PNM P4"), (&[b'P', b'5'], "PNM P5"), (&[b'P', b'6'], "PNM P6"),
            (&[b'#', b'?', b'R', b'A'], "HDR (Radiance)"),
        ];

        for (header, format_name) in HEADERS {
            if let Some(pos) = data.windows(header.len()).position(|w| w == *header) {
                warn!("Found {} format at position {}", format_name, pos);
                return Some(&data[pos..]);
            }
        }

        warn!("No supported image format found in byte stream");
        None
    }
}

#[async_trait]
impl CacheFormat for ImageFormat {
    const EXTENSION: &'static str = "avif";
    const MAGIC_BYTES: &'static [u8] = b"ZIMG";
    const VERSION: u8 = 1;

    async fn serialize<T>(
        &self,
        value: &CacheEntry<T>,
        _config: &CacheConfig,
    ) -> CacheResult<Vec<u8>>
    where
        T: Serialize + for<'de> Deserialize<'de> + Send + Sync + bincode::Encode,
    {
        let (data, created_at, _original_size_bytes) = match value {
            CacheEntry::Hit { data, created_at, size_bytes } => {
                (data, *created_at, *size_bytes)
            }
            CacheEntry::Miss => {
                return Err(CacheError::Unsupported("Cannot serialize cache miss".into()));
            }
        };
        let image_bytes = bincode::encode_to_vec(data, bincode::config::standard())
            .map_err(|e| {
                error!("Bincode serialization failed: {}", e);
                CacheError::InvalidFormat(format!("Failed to serialize data: {}", e))
            })?;
        
        let avif_data = self.convert_to_avif(&image_bytes).await?;
        
        let mut result = Vec::new();
        result.extend_from_slice(Self::MAGIC_BYTES);
        result.push(Self::VERSION);
        
        result.extend_from_slice(&created_at.to_le_bytes());
        // Store the actual AVIF data size as size_bytes
        result.extend_from_slice(&(avif_data.len() as u64).to_le_bytes());
        
        // Also store the AVIF data length (same value)
        result.extend_from_slice(&(avif_data.len() as u64).to_le_bytes());
        result.extend_from_slice(&avif_data);
        
        Ok(result)
    }

    async fn deserialize<T>(
        &self,
        bytes: &[u8],
        _config: &CacheConfig,
    ) -> CacheResult<CacheEntry<T>>
    where
        T: Serialize + for<'de> Deserialize<'de> + Send + Sync + bincode::Decode<()>,
    {
        self.validate_format(bytes)?;

        let mut pos = Self::MAGIC_BYTES.len() + 1; // Skip magic + version

        // Ensure we have enough bytes for metadata
        if bytes.len() < pos + 8 + 8 + 8 {
            error!("File too short for metadata: need {} bytes, got {}", pos + 8 + 8 + 8, bytes.len());
            return Err(CacheError::InvalidFormat("File too short for metadata".into()));
        }

        // Read cache metadata
        let created_at = u64::from_le_bytes(
            bytes[pos..pos + 8].try_into().unwrap()
        );
        pos += 8;

        let size_bytes = u64::from_le_bytes(
            bytes[pos..pos + 8].try_into().unwrap()
        );
        pos += 8;

        // Read AVIF data
        let avif_len = u64::from_le_bytes(
            bytes[pos..pos + 8].try_into().unwrap()
        ) as usize;
        pos += 8;

        if bytes.len() < pos + avif_len {
            error!("File too short for AVIF data: need {} bytes, got {}",
               pos + avif_len, bytes.len());
            return Err(CacheError::InvalidFormat("File too short for AVIF data".into()));
        }

        let avif_bytes = &bytes[pos..pos + avif_len];

        // T is Vec<u8> containing AVIF image data - just return it directly
        let avif_vec = avif_bytes.to_vec();
        let data = unsafe {
            std::ptr::read(&avif_vec as *const Vec<u8> as *const T)
        };
        std::mem::forget(avif_vec); // Prevent double-free

        // Reconstruct the CacheEntry
        let cache_entry = CacheEntry::Hit {
            data,
            created_at,
            size_bytes,
        };

        Ok(cache_entry)
    }

    fn should_auto_expire(&self) -> bool {
        false // Image files do not auto-expire
    }
}