#!/usr/bin/env bash
# --- Bash Script: download_audio.sh ---
# Exit immediately if a command fails
set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Path to the text file containing URLs (one per line)
INPUT_FILE="$SCRIPT_DIR/urls.txt"

# Path to the yt-audio CLI executable (built from your Rust project)
CLI_PATH="$SCRIPT_DIR/YtDownloader"

# Directory to save all downloaded files
MUSIC_DIR="$SCRIPT_DIR/music"

# Max number of concurrent downloads (handled by CLI)
MAX_CONCURRENT=8

# Make sure the music directory exists
if [ ! -d "$MUSIC_DIR" ]; then
    mkdir -p "$MUSIC_DIR"
    echo "📁 Created output directory: $MUSIC_DIR"
fi

# Read URLs from file, ignoring empty lines
mapfile -t URLS < <(grep -v '^[[:space:]]*$' "$INPUT_FILE")

if [ "${#URLS[@]}" -eq 0 ]; then
    echo "❌ No URLs found in $INPUT_FILE"
    exit 1
fi

# Build CLI arguments
# Example: yt-audio download --output "./music" --max-concurrent 8 <url1> <url2> ...
ARGS=("download" "--output" "$MUSIC_DIR" "--max-concurrent" "$MAX_CONCURRENT" "${URLS[@]}")

echo "▶ Starting yt-audio with ${#URLS[@]} URLs (max $MAX_CONCURRENT concurrent)..."
echo ""

# Run the yt-audio CLI
"$CLI_PATH" "${ARGS[@]}"

echo ""
echo "✅ All downloads completed! Saved in: $MUSIC_DIR"
