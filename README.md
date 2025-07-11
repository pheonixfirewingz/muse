# Muse

A lightweight music streaming server built with Rust that allows you to host and stream your MP3 music collection through a web interface with user authentication and third-party music metadata integration.

## Quick Start

1. **Clone and setup**:
   ```bash
   git clone <repository-url>
   cd muse
   cp .env_example .env
   mkdir -p runtime/music runtime/cache
   ```

2. **Configure environment** in `.env`:
   ```env
   SERVER_BIND="127.0.0.1:8000"
   CACHE_DIR="runtime/cache"
   CONTACT_EMAIL="contact@example.com"
   ```

3. **Add MP3 files** to `runtime/music` and run:
   ```bash
   cargo run --release
   ```

4. **Access** at `http://127.0.0.1:8000`

## Features

- **MP3 Streaming**: Stream your local MP3 files with HTTP range requests
- **Web Interface**: Modern, responsive web interface for desktop and mobile
- **User Authentication**: Secure login and registration system
- **Playlist Management**: Create and manage personal playlists
- **Music Metadata**: Integration with Spotify and MusicBrainz for metadata
- **Intelligent Caching**: Optimized caching system for performance

## Documentation

📚 **Complete documentation is available in the [`docs/`](docs/) directory:**

- **[Installation Guide](docs/installation.md)** - Detailed setup instructions
- **[Configuration Guide](docs/configuration.md)** - Environment variables and settings
- **[API Reference](docs/api-reference.md)** - Complete API documentation
- **[Technical Overview](docs/technical-overview.md)** - Architecture and features
- **[Usage Guide](docs/usage.md)** - How to use the application
- **[Cache Formats](docs/CACHE_FORMATS.md)** - Cache system documentation

## Technical Stack

- **Backend**: Rust with Axum web framework
- **Database**: SQLite with SQLx
- **Frontend**: HTML/CSS/JavaScript with responsive design
- **Authentication**: bcrypt with cookie-based sessions
- **Caching**: BSON format with zstd compression

## License

This project is licensed under the GNU Lesser General Public License v3.0 - see the [LICENSE](LICENSE) file for details.
