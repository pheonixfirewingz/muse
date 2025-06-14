const DOM = {
    audioPlayer: document.getElementById("audio-player"),
    playPauseBtn: document.getElementById("play-pause-btn"),
    seekBar: document.getElementById("seek-bar"),
    totalDuration: document.getElementById("total-duration"),
    currentTime: document.getElementById("current-time"),
    playerThumbnail: document.getElementById("player-thumbnail-id"),
    sidebar: document.querySelector(".sidebar"),
    content: document.getElementById("content"),
    pageLinks: document.querySelectorAll("a[data-page]")
};

const utils = {
    formatTime: t => `${Math.floor(t/60)}:${String(Math.floor(t%60)).padStart(2,'0')}`,
    titleCase: s => s.toLowerCase().split(' ').map(w=>w[0].toUpperCase()+w.slice(1)).join(' '),
    normalize: s => s.toLowerCase().replace(/\s+/g,'_').replace(/song_by_/g,'')
};

let isMobile = /android|iphone|ipad/i.test(navigator.userAgent);

function toggleSidebar() {
    DOM.sidebar.classList.toggle('active');
}

function togglePlay() {
    if (!DOM.audioPlayer.src) return;
    const method = DOM.audioPlayer.paused ? 'play' : 'pause';
    DOM.audioPlayer[method]();
    DOM.playPauseBtn.textContent = method === 'play' ? '⏸' : '▶';
}

function updateSeek() {
    if (!DOM.audioPlayer.duration) return;
    DOM.seekBar.value = (DOM.audioPlayer.currentTime / DOM.audioPlayer.duration) * 100;
    DOM.currentTime.textContent = utils.formatTime(DOM.audioPlayer.currentTime);
}

function resetPlayer() {
    DOM.currentTime.textContent = '0:00';
    DOM.seekBar.value = 0;
    DOM.playPauseBtn.textContent = '▶';
}

function initPlayer() {
    DOM.playPauseBtn.addEventListener('click', togglePlay);
    DOM.audioPlayer.addEventListener('timeupdate', updateSeek);
    DOM.audioPlayer.addEventListener('loadedmetadata', () => {
        DOM.totalDuration.textContent = utils.formatTime(DOM.audioPlayer.duration);
    });
    DOM.audioPlayer.addEventListener('ended', resetPlayer);
    DOM.seekBar.addEventListener('input', () => {
        if (DOM.audioPlayer.duration) DOM.audioPlayer.currentTime = (DOM.seekBar.value/100)*DOM.audioPlayer.duration;
    });
}

async function fetchAndInsert(url, callback) {
    try {
        const res = await fetch(url);
        if (!res.ok) throw new Error(res.statusText);
        DOM.content.innerHTML = await res.text();
        callback?.();
    } catch (e) {
        console.error(`Failed to fetch ${url}:`, e);
        DOM.content.innerHTML = '<p>Error loading content</p>';
    }
}

function setPage(page) {
    let _;
    fetchAndInsert(`pages/${page}.html`, () => {
        window.history.pushState({page}, '', `#${page}`);
        if (page === 'artists') _ = loadImages('/artist', '.card');
        if (page === 'songs') _ = loadImages('/album', '.card', true);
    }).then( r => _).catch(e => console.error(e));
}

function bindNav() {
    DOM.pageLinks.forEach(el => el.addEventListener('click', e => {
        e.preventDefault();
        setPage(el.dataset.page);
    }));
}

window.addEventListener('popstate', e => e.state?.page && setPage(e.state.page));

document.addEventListener('DOMContentLoaded', () => {
    initPlayer();
    bindNav();
    setPage(window.location.hash.slice(1)||'home');
});

async function playSong(song, artist) {
    const sn = utils.normalize(song), an = utils.normalize(artist);
    const p = window.parent;
    ['audio-player','play-pause-btn','player_title','player_artist'].forEach(id=>p.document.getElementById(id));
    p.document.getElementById('player_title').innerText = utils.titleCase(song);
    p.document.getElementById('player_artist').innerText = utils.titleCase(artist);
    await loadImageByArtistSong({
        artist,
        song,
        basePath: '',
        imgElement: DOM.playerThumbnail,
        spinnerElement: null,
        fallbackElement: null});
    DOM.audioPlayer.src = `stream?song=${sn}&artist=${an}&is_mobile=${isMobile}`;
    DOM.audioPlayer.type = isMobile? 'audio/aac':'audio/mpeg';
    DOM.audioPlayer.load();
    togglePlay();
}

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
            basePath: '',
            imgElement: img,
            spinnerElement: spinner,
            fallbackElement: fallback
        });
    }
}


async function loadImageByArtistSong({
                                         artist,
                                         song = null,
                                         basePath = '',
                                         imgElement,
                                         spinnerElement = null,
                                         fallbackElement = null
                                     }) {
    const cleanArtist = artist.replace('Song By ', '').trim()
    const url = song
        ? `${basePath}/album/${encodeURIComponent(cleanArtist)}/${encodeURIComponent(song.trim())}`
        : `${basePath}/artist/${encodeURIComponent(cleanArtist)}`;

    try {
        const res = await fetch(url);
        const json = await res.json();

        if (!json.success) {
            imgElement.src = 'assets/images/place_holder.webp';
        }
        else {
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

async function setPageQueryArtist(artist) {
    const url = `list?artist=${encodeURIComponent(artist)}`;
    try {
        const res = await fetch(url);
        if (!res.ok) throw new Error(res.statusText);
        DOM.content.innerHTML = await res.text();
        window.history.pushState({ page: artist }, "", `#${artist}`);
        loadImages('/artist', '.card'); // assumes artist listing
    } catch (e) {
        console.error("Failed to load artist page:", e);
        DOM.content.innerHTML = "<p>Error loading content</p>";
    }
}