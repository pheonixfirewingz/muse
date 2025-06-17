# muse

A lightweight music streaming server built with Rust that allows you to host and stream your MP3 music collection through a web interface.

## Features
- MP3 streaming support
- Web-based user interface
- Artist and song browsing
- Real-time audio playback controls
- Responsive design for mobile and desktop
- Built with Rust for optimal performance
- Automatic music library scanning
- MusicBrainz integration for artist information

## Technical Stack
- Backend: Rust with Axum web framework
- Frontend: HTML, JavaScript, and CSS
- Database: SQLite for music library management
- Template Engine: MiniJinja

## Installation

### Prerequisites
- Rust (latest stable version)
- Cargo package manager

## Usage

1. Create a `runtime/music` directory and place your MP3 files there
2. Run the server: `cargo run --release`
3. Access the web interface at `http://127.0.0.1:8000`
## Development

The server includes:
- Automatic template reloading in debug mode
- Static file serving
- RESTful API endpoints
- Music library scanning and indexing

## Contributing
Contributions are welcome! Please feel free to submit a Pull Request.

## Project Status
🚧 Under Development

## License
GNU LESSER GENERAL PUBLIC LICENSE Version 3, 29 June 2007 (LGPL 3)
