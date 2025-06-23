import { DOM } from '/assets/js/dom.js';
import { formatTime } from '/assets/js/utils.js';

let isUserSeeking = false;
let audioReady = false;
let targetSeekTime = null;

async function fetchImageAsBlob(url) {
    try {
        const response = await fetch(url);
        if (!response.ok) {
            return null;
        }
        const blob = await response.blob();
        return URL.createObjectURL(blob);
    } catch (error) {
        return null;
    }
}

export function bindPlayerControls() {
    DOM.playPauseBtn.addEventListener('click', togglePlayPause);

    DOM.audioPlayer.addEventListener('timeupdate', () => {
        if (!isUserSeeking) {
            const current = DOM.audioPlayer.currentTime;
            const duration = DOM.audioPlayer.duration;
            if (!isNaN(duration) && duration > 0) {
                DOM.seekBar.value = (current / duration) * 100;
                DOM.currentTimeEl.textContent = formatTime(current);
            }
        }
    });

    DOM.seekBar.addEventListener('mousedown', () => isUserSeeking = true);
    DOM.seekBar.addEventListener('mouseup', () => setTimeout(() => isUserSeeking = false, 100));

    DOM.seekBar.addEventListener('input', () => {
        if (isUserSeeking && audioReady) {
            const duration = DOM.audioPlayer.duration;
            if (!isNaN(duration) && duration > 0) {
                targetSeekTime = (duration / 100) * DOM.seekBar.value;

                const forceSeek = () => {
                    DOM.audioPlayer.currentTime = targetSeekTime;
                    setTimeout(() => {
                        if (Math.abs(DOM.audioPlayer.currentTime - targetSeekTime) > 1) {
                            DOM.audioPlayer.currentTime = targetSeekTime;
                        }
                    }, 50);
                };
                forceSeek();
                setTimeout(forceSeek, 100);
            }
        }
    });

    DOM.audioPlayer.addEventListener('loadedmetadata', () => {
        audioReady = true;
        DOM.durationEl.textContent = formatTime(DOM.audioPlayer.duration);
    });

    DOM.audioPlayer.addEventListener('loadstart', () => {
        audioReady = false;
        isUserSeeking = false;
    });

    document.addEventListener('keydown', keyboardShortcuts);
}

function togglePlayPause() {
    const audio = DOM.audioPlayer;
    if (audio.paused) {
        audio.play();
        DOM.playPauseBtn.textContent = '⏸';
    } else {
        audio.pause();
        DOM.playPauseBtn.textContent = '▶';
    }
}

function keyboardShortcuts(e) {
    const active = document.activeElement;
    if (active && (active.tagName === 'INPUT' || active.tagName === 'TEXTAREA')) return;

    const key = e.key.toLowerCase();
    const audio = DOM.audioPlayer;

    switch (key) {
        case ' ':
        case 'k':
            e.preventDefault();
            togglePlayPause();
            break;
        case 'arrowleft':
        case 'j':
            e.preventDefault();
            audio.currentTime = Math.max(0, audio.currentTime - (e.shiftKey ? 10 : 5));
            break;
        case 'arrowright':
        case 'l':
            e.preventDefault();
            audio.currentTime = Math.min(audio.duration, audio.currentTime + (e.shiftKey ? 10 : 5));
            break;
        case 'arrowup':
            e.preventDefault();
            audio.volume = Math.min(1, +(audio.volume + 0.1).toFixed(2));
            break;
        case 'arrowdown':
            e.preventDefault();
            audio.volume = Math.max(0, +(audio.volume - 0.1).toFixed(2));
            break;
        case 'm':
            e.preventDefault();
            audio.muted = !audio.muted;
            break;
    }
}

export async function playSong(song, artist) {
    DOM.playerTitle.textContent = song;
    DOM.playerArtist.textContent = artist;

    // Detect platform for best format
    let userAgent = navigator.userAgent || navigator.vendor || window.opera;
    let format = 'mp3';
    if (/iPhone|iPad|Macintosh/.test(userAgent)) {
        format = 'm4a';
    }

    DOM.audioPlayer.src = `/api/stream?artist_name=${encodeURIComponent(artist)}&song_name=${encodeURIComponent(song)}&format=${format}`;
    
    // Fetch image using the new approach
    const imageUrl = `/api/images/song?artist_name=${encodeURIComponent(artist)}&song_name=${encodeURIComponent(song)}`;
    const blobUrl = await fetchImageAsBlob(imageUrl);
    DOM.playerThumbnail.src = blobUrl || '/assets/images/place_holder.webp';

    try {
        await DOM.audioPlayer.play();
        DOM.playPauseBtn.textContent = '⏸';
    } catch (error) {
        console.error("Failed to play audio:", error);
    }
}
