// Cache important DOM elements used across the app
const DOM = {
    playerTitle: document.getElementById("player-title-id"),
    playerArtist: document.getElementById("player-artist-id"),
    playerThumbnail: document.getElementById("player-thumbnail-id"),
    audioPlayer: document.getElementById('audio-player'),
    seekBar: document.getElementById('seek-bar'),
    currentTimeEl: document.getElementById('current-time'),
    durationEl: document.getElementById('total-duration'),
    playPauseBtn: document.getElementById('play-pause-btn'),
    sidebar: document.querySelector(".sidebar"),
    content: document.getElementById("content"),
    pageLinks: document.querySelectorAll("a[data-page]"),
};

// Formats seconds into "minutes:seconds" string for display
function formatTime(seconds) {
    if (isNaN(seconds) || seconds < 0) return '0:00';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
}

// Toggle play/pause state of the audio player on button click
DOM.playPauseBtn.addEventListener('click', () => {
    if (DOM.audioPlayer.paused) {
        DOM.audioPlayer.play();
        DOM.playPauseBtn.textContent = '⏸';
    } else {
        DOM.audioPlayer.pause();
        DOM.playPauseBtn.textContent = '▶';
    }
});

// Update seek bar position and current time display as audio plays
DOM.audioPlayer.addEventListener('timeupdate', () => {
    const current = DOM.audioPlayer.currentTime;
    const duration = DOM.audioPlayer.duration;

    if (!isUserSeeking) {
        if (!isNaN(duration) && duration > 0) {
            DOM.seekBar.value = (current / duration) * 100;
            DOM.currentTimeEl.textContent = formatTime(current);
        }
    }
});

let isUserSeeking = false;

// Detect when user starts dragging the seek bar
DOM.seekBar.addEventListener('mousedown', () => {
    isUserSeeking = true;
});

let audioReady = false;
let targetSeekTime = null;

// Handle seek bar input to seek audio playback accordingly
DOM.seekBar.addEventListener('input', () => {
    if (isUserSeeking && audioReady) {
        const duration = DOM.audioPlayer.duration;
        if (!isNaN(duration) && duration > 0) {
            targetSeekTime = (duration / 100) * DOM.seekBar.value;

            // Force seek multiple times to ensure the player jumps to desired time
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

// When audio metadata is loaded, mark audio as ready and update total duration display
DOM.audioPlayer.addEventListener('loadedmetadata', () => {
    audioReady = true;
    DOM.durationEl.textContent = formatTime(DOM.audioPlayer.duration);
});

// When new audio starts loading, reset ready and seeking flags
DOM.audioPlayer.addEventListener('loadstart', () => {
    audioReady = false;
    isUserSeeking = false;
});

// Reset user seeking flag shortly after user releases mouse button on seek bar
DOM.seekBar.addEventListener('mouseup', () => {
    setTimeout(() => {
        isUserSeeking = false;
    }, 100);
});

document.addEventListener('keydown', (e) => {
    const audio = DOM.audioPlayer;

    // Ignore key events if focus is on input or textarea to avoid interfering with typing
    const active = document.activeElement;
    if (active && (active.tagName === 'INPUT' || active.tagName === 'TEXTAREA')) return;

    // Normalize key
    const key = e.key.toLowerCase();

    switch (key) {
        case ' ':
        case 'k':
            e.preventDefault();
            if (audio.paused) {
                audio.play();
                DOM.playPauseBtn.textContent = '⏸'; // pause icon
            } else {
                audio.pause();
                DOM.playPauseBtn.textContent = '▶'; // play icon
            }
            break;
        case 'arrowleft':
        case 'j':
            e.preventDefault();
            if (!isNaN(audio.duration)) {
                // Shift+Left for bigger seek backward
                const seekAmount = e.shiftKey ? 10 : 5;
                audio.currentTime = Math.max(0, audio.currentTime - seekAmount);
            }
            break;
        case 'arrowright':
        case 'l':
            e.preventDefault();
            if (!isNaN(audio.duration)) {
                // Shift+Right for bigger seek forward
                const seekAmount = e.shiftKey ? 10 : 5;
                audio.currentTime = Math.min(audio.duration, audio.currentTime + seekAmount);
            }
            break;
        case 'arrowup':
            e.preventDefault();
            // Increase volume by 0.1, clamp between 0 and 1
            audio.volume = Math.min(1, +(audio.volume + 0.1).toFixed(2));
            break;
        case 'arrowdown':
            e.preventDefault();
            // Decrease volume by 0.1, clamp between 0 and 1
            audio.volume = Math.max(0, +(audio.volume - 0.1).toFixed(2));
            break;
        case 'm':
            e.preventDefault();
            audio.muted = !audio.muted;
            break;
    }
});

// Load and play a song, updating UI elements accordingly
async function playSong(song, artist) {
    DOM.playerTitle.textContent = song;
    DOM.playerArtist.textContent = artist;
    DOM.playerThumbnail.src = 'assets/images/place_holder.webp';
    DOM.audioPlayer.src = `stream?song=${encodeURIComponent(song)}&artist=${encodeURIComponent(artist)}`;
    const _ = loadImageByArtistSong({
        artist,
        song,
        imgElement: DOM.playerThumbnail
    });
    try {
        await DOM.audioPlayer.play();
        DOM.playPauseBtn.textContent = '⏸';
    } catch (_) {}
}

// Toggles sidebar visibility (e.g., for navigation menu)
function toggleSidebar() {
    DOM.sidebar.classList.toggle('active');
}

// Fetch content from a URL and insert it into the main content area, then run callback
async function fetchAndInsert(url, callback) {
    try {
        const res = await fetch(url);
        if (res.status === 401) {
            window.location.href = "/login";
        }
        if (!res.ok) throw new Error(res.statusText);
        DOM.content.innerHTML = await res.text();
        callback?.();
    } catch (e) {
        console.error(`Failed to fetch ${url}:`, e);
        DOM.content.innerHTML = '<p>Error loading content</p>';
    }
}

// Load and display a page by name, update URL and load images if relevant
function setPage(page) {
    let _;
    fetchAndInsert(`pages/${page}.html`, () => {
        window.history.pushState({ page }, '', `#${page}`);
        if (page === 'artists') _ = loadImages('/artist', '.card');
        if (page === 'songs') _ = loadImages('/album', '.card', true);
    }).then(_).catch(e => console.error(e));
}

// Bind navigation links to load pages without full refresh
function bindNav() {
    DOM.pageLinks.forEach(el => el.addEventListener('click', e => {
        e.preventDefault();
        setPage(el.dataset.page);
    }));
}

// Handle browser back/forward navigation
window.addEventListener('popstate', e => {
    if (e.state?.page) setPage(e.state.page);
});

// Initialize the app once DOM is ready: bind nav and load initial page
document.addEventListener('DOMContentLoaded', () => {
    bindNav();
    setPage(window.location.hash.slice(1) || 'home');
});

// Load images for cards on the page, either artists or songs
async function loadImages(basePath, selector, useTitleDesc = false) {
    const cards = document.querySelectorAll(selector);

    for (const card of cards) {
        let artist, song = null;

        if (useTitleDesc) {
            artist = card.querySelector('p').textContent.replace('Song By ', '').trim();
            song = card.querySelector('h3').textContent.trim();
        } else {
            artist = card.ariaLabel;
        }

        const img = card.querySelector('img');
        const spinner = card.querySelector('.loading-spinner');
        const fallback = card.querySelector('.fallback-icon');

        await loadImageByArtistSong({
            artist,
            song,
            imgElement: img,
            spinnerElement: spinner,
            fallbackElement: fallback
        });
    }
}

// Helper to load an image by artist and optionally song, with fallback and spinner UI
async function loadImageByArtistSong({
                                         artist,
                                         song = null,
                                         imgElement,
                                         spinnerElement = null,
                                         fallbackElement = null
                                     }) {
    imgElement.src = 'assets/images/place_holder.webp';
    return;

    const cleanArtist = artist.replace('Song By ', '').trim();
    const url = song
        ? `/album/${encodeURIComponent(cleanArtist)}/${encodeURIComponent(song.trim())}`
        : `/artist/${encodeURIComponent(cleanArtist)}`;

    try {
        const res = await fetch(url);
        const json = await res.json();

        if (!json.success) {
            imgElement.src = 'assets/images/place_holder.webp';
        } else {
            imgElement.src = `data:image/jpeg;base64,${json.data}`;
        }

        imgElement.onload = () => {
            if (spinnerElement) spinnerElement.style.display = 'none';
            if (fallbackElement) fallbackElement.style.display = 'none';
            imgElement.style.display = 'block';
        };
    } catch {
        if (spinnerElement) spinnerElement.style.display = 'none';
        if (fallbackElement) fallbackElement.style.display = 'block';
        imgElement.style.display = 'none';
    }
}

// Load content filtered by artist and update the page accordingly
async function setPageQueryArtist(artist) {
    const url = `list?artist=${encodeURIComponent(artist)}`;
    try {
        const res = await fetch(url);
        if (res.status === 401) {
            window.location.href = "/login";
        }
        if (!res.ok) throw new Error(res.statusText);
        DOM.content.innerHTML = await res.text();
        window.history.pushState({ page: artist }, "", url);
        await loadImages('/artist', '.card');
    } catch (e) {
        console.error("Failed to load artist page:", e);
        DOM.content.innerHTML = "<p>Error loading content</p>";
    }
}
