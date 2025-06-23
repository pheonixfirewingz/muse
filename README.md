# Muse

A lightweight music streaming server built with Rust that allows you to host and stream your MP3 music collection through a web interface with user authentication and third-party music metadata integration.

## Features

### Core Functionality
- **MP3 Streaming Support**: Stream your local MP3 files with HTTP range requests
- **Web-based User Interface**: Modern, responsive web interface for desktop and mobile
- **Artist and Song Browsing**: Browse your music library by artists and songs
- **Real-time Audio Playback**: Full audio player controls with play/pause, seek, and volume
- **Automatic Music Library Scanning**: Automatically scans and indexes your MP3 collection using ID3 tags

### User Management
- **User Authentication**: Secure login and registration system with bcrypt password hashing
- **Session Management**: Cookie-based session management with configurable TTL
- **User Registration**: Email validation, username requirements, and password strength validation
- **Profanity Filtering**: Username validation to prevent inappropriate content

### Music Metadata Integration
- **Spotify Integration**: Optional Spotify API integration for artist images, album art, and metadata
- **MusicBrainz Integration**: Fallback to MusicBrainz API for music metadata when Spotify is unavailable
- **Intelligent Caching**: Caches metadata responses to reduce API calls and improve performance
- **Rate Limiting**: Respects API rate limits for third-party services

### Playlist Management
- **User Playlists**: Create and manage personal playlists
- **Add to Playlist**: Add songs to existing playlists or create new ones
- **Playlist Browsing**: View and manage your playlists

### Technical Features
- **Built with Rust**: High-performance backend using Axum web framework
- **SQLite Database**: Lightweight database for music library and user management
- **Template Engine**: MiniJinja templating for dynamic web pages
- **Static File Serving**: Efficient serving of CSS, JavaScript, and images
- **HTTP Compression**: Automatic response compression for better performance
- **Rate Limiting**: Built-in rate limiting for API endpoints
- **Logging**: Comprehensive logging with configurable log levels

## Technical Stack

### Backend
- **Framework**: Rust with Axum web framework
- **Database**: SQLite with SQLx for async database operations
- **Authentication**: bcrypt for password hashing, cookie-based sessions
- **Templating**: MiniJinja template engine
- **HTTP**: Tower HTTP with compression and cookie management

### Frontend
- **HTML/CSS/JavaScript**: Modern web interface with responsive design
- **Audio API**: HTML5 Audio API for music playback
- **Font Awesome**: Icons for the user interface
- **SweetAlert2**: Enhanced user interactions and modals

### Third-party Integrations
- **Spotify API**: For music metadata and album artwork (optional)
- **MusicBrainz API**: Fallback music metadata service
- **Cover Art Archive**: Album artwork via MusicBrainz integration

## Installation

### Prerequisites
- Rust (latest stable version)
- Cargo package manager

### Setup

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

## Usage

### First Time Setup
1. The application will automatically scan your `runtime/music` directory for MP3 files
2. Register a new account at `http://127.0.0.1:8000/register`
3. Log in with your credentials
4. Start browsing and playing your music!

### Music Library
- **Automatic Scanning**: The server automatically scans for new MP3 files on startup
- **ID3 Tag Support**: Uses ID3 tags for song and artist information
- **File Organization**: Supports nested directory structures (up to 3 levels deep)

### User Interface
- **Responsive Design**: Works on desktop, tablet, and mobile devices
- **Dark Theme**: Modern dark theme with accent colors
- **Accessibility**: ARIA labels and keyboard navigation support
- **Real-time Updates**: Dynamic content loading without page refreshes

### Playlists
- **Create Playlists**: Add songs to new or existing playlists
- **Manage Playlists**: View and manage your personal playlists
- **Quick Add**: Right-click or use the plus button to add songs to playlists

## Development
### Development Features
- **Debug Logging**: Comprehensive logging for development and debugging
- **Error Handling**: Graceful error handling with user-friendly messages
- **Rate Limiting**: Built-in protection against abuse

### Building for Production
```bash
cargo build --release
```

The release binary will be optimized for performance and can be deployed to production environments.

## Configuration

### Environment Variables

| Variable | Required | Description | Default |
|----------|----------|-------------|---------|
| `SERVER_BIND` | Yes | Server bind address and port | - |
| `CACHE_DIR` | Yes | Directory for caching metadata | - |
| `CONTACT_EMAIL` | Yes | Email for API rate limiting | - |
| `WEBSITE_URL` | No | Public website URL | `SERVER_BIND` |
| `LOG_LEVEL` | No | Logging level | `info` |
| `HAS_SPOTIFY` | No | Enable Spotify integration | `false` |
| `SPOTIFY_CLIENT_ID` | No* | Spotify API client ID | - |
| `SPOTIFY_CLIENT_SECRET` | No* | Spotify API client secret | - |

*Required if `HAS_SPOTIFY=true`

### Spotify Integration Setup
1. Create a Spotify Developer account at https://developer.spotify.com
2. Create a new application to get your Client ID and Client Secret
3. Set `HAS_SPOTIFY=true` in your `.env` file
4. Add your Spotify credentials to the environment variables

## API Endpoints

> **All `/api/*` endpoints require authentication via a valid session cookie.**
> If the session is missing or invalid, the API will return a 401 Unauthorized error. This includes all playlist, image, and streaming endpoints.

### Authentication
- `GET /login` — Login page
- `POST /login/submit` — Login form submission
- `GET /register` — Registration page
- `POST /register/submit` — Registration form submission

### Music Streaming & Metadata
- `GET /api/stream?artist_name=X&song_name=Y[&format=mp3|m4a|aac]` — Stream a song (with optional format selection)
- `GET /api/images/artist?artist_name=X` — Get artist image (returns `{ image_url }` JSON)
- `GET /api/images/song?artist_name=X&song_name=Y` — Get song/album image (returns `{ image_url }` JSON)

### Playlists
- `GET /api/playlists` — Get current user's playlists (requires session cookie)
- `POST /api/playlists` — Create a new playlist (form: `name`, `public?`)
- `POST /api/playlists/songs` — Add a song to a playlist (form: `playlist_name`, `song_name`, `artist_name`)
- `POST /api/playlists/create_and_add` — Create a playlist and add a song (form: `playlist_name`, `song_name`, `artist_name`, `public?`)
- `POST /api/playlists/delete` — Delete a playlist (form: `playlist_name`)

### Web Interface
- `GET /app` — Main application interface
- `GET /artists` — Artists list page
- `GET /songs` — Songs list page
- `GET /home` — Home page
- `GET /list?artist_name=X` — List songs by artist (HTML)
- `GET /assets/*` — Static assets (CSS, JS, images, etc.)

### Other
- `GET /manifest.json`, `/robots.txt`, `/sitemap.xml` — Standard web manifest and robots files

**Notes:**
- Playlist endpoints expect form data for POST requests.
- The `/api/stream` endpoint supports HTTP range requests for direct streaming and on-the-fly conversion for unsupported formats.
- The `/api/images/*` endpoints return a JSON object with an `image_url` field.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Guidelines
- Follow Rust coding conventions
- Add appropriate error handling
- Include logging for debugging
- Test your changes thoroughly
- Update documentation as needed
- everyone is welcome but to keep politics out of it

## Project Status
🚧 Under Active Development

## License
GNU LESSER GENERAL PUBLIC LICENSE Version 3, 29 June 2007 (LGPL 3)
