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
    try {
        await DOM.audioPlayer.play();
        DOM.playPauseBtn.textContent = '⏸';
    } catch (_) {}
}

// Toggles sidebar visibility (e.g., for navigation menu)
function toggleSidebar() {
    DOM.sidebar.classList.toggle('active');
}

// Helper function to determine if imageSetup should be called
function shouldCallImageSetup(url) {
    // Call imageSetup for pages that have cards with images
    return url.includes('pages/artists.html') ||
        url.includes('pages/songs.html') ||
        url.startsWith('list?artist=');
}

// Enhanced fetch content from a URL and insert it into the main content area, then run callback
async function fetchAndInsert(url, callback) {
    try {
        const res = await fetch(url);
        if (res.status === 401) {
            window.location.href = "/login";
        }
        if (!res.ok) throw new Error(res.statusText);

        DOM.content.innerHTML = await res.text();

        // Wait for next frame to ensure DOM is fully updated
        await new Promise(resolve => requestAnimationFrame(resolve));

        // Determine if we need to call imageSetup based on URL pattern
        if (shouldCallImageSetup(url)) {
            imageSetup(url.match('pages/artists.html'));
        }

        // Run the optional callback
        callback?.();

    } catch (e) {
        console.error(`Failed to fetch ${url}:`, e);
        DOM.content.innerHTML = '<p>Error loading content</p>';
    }
}

// Load and display a page by name, update URL and load images if relevant
function setPage(page) {
    fetchAndInsert(`pages/${page}.html`, () => {
        window.history.pushState({ page }, '', `#${page}`);
    }).catch(e => console.error(e));
}

// Bind navigation links to load pages without a full refresh
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

// Load content filtered by artist and update the page accordingly
async function setPageQueryArtist(artist) {
    const url = `list?artist=${encodeURIComponent(artist)}`;

    await fetchAndInsert(url, () => {
        window.history.pushState({ page: artist }, "", url);
    });
}

function imageSetup(artist) {
    console.log("setting up images")
    const cards = document.querySelectorAll('.card');
    if (cards.length === 0) return;
    if (!artist) {
        for (const card of cards) {
            const img = card.querySelector('img');
            const spinner = card.querySelector('.loading-spinner');
            const fallback = card.querySelector('.fallback-icon');
            let _ = loadImage(card.querySelector('p').textContent, card.querySelector('h3').textContent, img, spinner, fallback);
        }
    } else {
        for (const card of cards) {
            const img = card.querySelector('img');
            const spinner = card.querySelector('.loading-spinner');
            const fallback = card.querySelector('.fallback-icon');
            let _ = loadImage(card.querySelector('h3').textContent, null, img, spinner, fallback);
        }
    }
}

async function loadImage(artist, song = null, imgElement, spinnerElement = null, fallbackElement = null) {
    const url = song
        ? `/cache?artist_name=${encodeURIComponent(artist)}&song_name=${encodeURIComponent(song)}&info_type=image`
        : `/cache?artist_name=${encodeURIComponent(artist)}&info_type=image`;

    try {
        const res = await fetch(url);
        if (!res.ok) throw new Error('Network response was not ok');

        // Check content type to determine how to parse the response
        const contentType = res.headers.get('content-type');

        if (contentType && contentType.includes('application/json')) {
            // Parse as JSON if the response is JSON
            const data = await res.json();

            if (data && data.image_url) {
                imgElement.src = data.image_url;
            }
        }
        imgElement.onload = () => {
            if (fallbackElement) fallbackElement.style.display = 'none';
            imgElement.style.display = 'block';
        };
    } catch (e) {
        if (fallbackElement) fallbackElement.style.display = 'block';
        imgElement.style.display = 'none';
    }
    if (spinnerElement) spinnerElement.style.display = 'none';
}