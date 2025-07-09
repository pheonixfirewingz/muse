# Installation Guide

## Prerequisites
- Rust (latest stable version)
- Cargo package manager

## Setup

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd muse
   ```

2. **Create environment configuration**:
   ```bash
   cp .env_example .env
   ```

3. **Configure environment variables** in `.env`:
   ```env
   SERVER_BIND="127.0.0.1:8000"           # Required: Server bind address
   CACHE_DIR="runtime/cache"              # Required: Cache directory
   CONTACT_EMAIL="contact@example.com"    # Required: Contact email for API requests
   WEBSITE_URL="127.0.0.1:8000"          # Optional: Website URL (defaults to SERVER_BIND)
   LOG_LEVEL="info"                       # Optional: Log level (defaults to info)
   
   # Spotify Integration (Optional)
   HAS_SPOTIFY="false"                     # Enable/disable Spotify integration
   SPOTIFY_CLIENT_ID="your_client_id"     # Spotify API client ID
   SPOTIFY_CLIENT_SECRET="your_secret"    # Spotify API client secret
   ```

4. **Create required directories**:
   ```bash
   mkdir -p runtime/music
   mkdir -p runtime/cache
   ```

5. **Add your MP3 files** to the `runtime/music` directory

6. **Build and run**:
   ```bash
   cargo run --release
   ```

7. **Access the application** at `http://127.0.0.1:8000`

## First Time Setup
1. The application will automatically scan your `runtime/music` directory for MP3 files
2. Register a new account at `http://127.0.0.1:8000/register`
3. Log in with your credentials
4. Start browsing and playing your music!

## Building for Production
```bash
cargo build --release
```

The release binary will be optimized for performance and can be deployed to production environments. 