# Cache Formats

This project uses multiple cache formats optimized for different types of data, providing optimal performance and file size.

## Available Formats

### BSON Format (`bson_ext.rs`)
- **Extension**: `.bson.zst`
- **Magic Bytes**: `ZBSN`
- **Use Case**: General data caching (artist info, song metadata, etc.)
- **Pros**:
  - More compact than JSON
  - Faster serialization/deserialization
  - Native binary data support
  - Better performance for large datasets
- **Cons**:
  - Not human-readable
  - Less widely supported for debugging

### Image Format (`img_ext.rs`)
- **Extension**: `.avif`
- **Magic Bytes**: `ZIMG`
- **Use Case**: Image caching (artist photos, album art, user avatars)
- **Pros**:
  - Converts images to AVIF format for maximum compression
  - Supports multiple input formats (JPEG, PNG, GIF, BMP, TIFF, WebP, etc.)
  - Configurable quality and encoding speed
  - Excellent compression ratios
- **Cons**:
  - AVIF format not supported by all browsers (fallback provided)
  - Encoding can be CPU intensive

## Usage

Both formats implement the `CacheFormat` trait and are used throughout the application:

```rust
use crate::util::cache::{CacheManager, CacheConfig, BsonFormat, ImageFormat};

// Using BSON format for data
let cache_manager = CacheManager::new(CacheConfig::default());
cache_manager.store::<_, BsonFormat>(&entry, "key", "subdir").await?;
let loaded = cache_manager.load::<_, BsonFormat>("key", "subdir").await?;

// Using Image format for images
let image_format = ImageFormat {
    quality: 80,  // AVIF quality (0-100)
    speed: 6,     // Encoding speed (0-10, higher = faster but larger)
};
cache_manager.store::<_, ImageFormat>(&image_entry, "image_key", "images").await?;
let image = cache_manager.load::<_, ImageFormat>("image_key", "images").await?;
```

## Format Selection

- **Use BSON Format** for:
  - Artist data, song metadata, playlist information
  - Any structured data that needs to be serialized
  - General application data caching

- **Use Image Format** for:
  - Artist profile pictures
  - Album artwork
  - User avatars
  - Any image data that needs to be cached

## Benefits

- **Performance**: BSON is faster to serialize/deserialize than JSON
- **Size**: Both formats provide excellent compression
- **Specialized**: Each format is optimized for its specific use case
- **Flexibility**: Configurable quality settings for images
- **Compatibility**: Supports multiple input image formats 