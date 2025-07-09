# Configuration Guide

## Environment Variables

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

## Spotify Integration Setup

1. Create a Spotify Developer account at https://developer.spotify.com
2. Create a new application to get your Client ID and Client Secret
3. Set `HAS_SPOTIFY=true` in your `.env` file
4. Add your Spotify credentials to the environment variables

## Music Library Configuration

### Automatic Scanning
- The server automatically scans for new MP3 files on startup
- Uses ID3 tags for song and artist information
- Supports nested directory structures (up to 3 levels deep)

### File Organization
- Place your MP3 files in the `runtime/music` directory
- The application will automatically index and organize your music
- Supports standard ID3 tag metadata 