# Technical Overview

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

## Development Features
- **Debug Logging**: Comprehensive logging for development and debugging
- **Error Handling**: Graceful error handling with user-friendly messages
- **Rate Limiting**: Built-in protection against abuse 