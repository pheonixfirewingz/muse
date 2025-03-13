const audioPlayer = document.getElementById("audio-player");
const playPauseBtn = document.getElementById("play-pause-btn");
const seekBar = document.getElementById("seek-bar");
const totalDuration = document.getElementById("total-duration");
const sidebar = document.querySelector(".sidebar");

// Sidebar toggle
function toggleSidebar() {
    sidebar.classList.toggle("active");
}


// Play/Pause functionality
playPauseBtn.addEventListener("click", () => {
    try {
        if (audioPlayer.paused) {
            audioPlayer.play();
            playPauseBtn.textContent = "⏸";
        } else {
            audioPlayer.pause();
            playPauseBtn.textContent = "▶";
        }
    } catch (error) {
        console.error(error);
        playPauseBtn.textContent = "▶";
    }
});


audioPlayer.addEventListener("timeupdate", () => {
    const progress = (audioPlayer.currentTime / audioPlayer.duration) * 100;
    seekBar.value = progress;
    const minutes = Math.floor(audioPlayer.currentTime / 60);
    const seconds = Math.floor(audioPlayer.currentTime % 60);
    document.getElementById("current-time").textContent = `${minutes}:${seconds.toString().padStart(2, "0")}`;

});

// Update total duration
audioPlayer.addEventListener("loadedmetadata", () => {
    const minutes = Math.floor(audioPlayer.duration / 60);
    const seconds = Math.floor(audioPlayer.duration % 60);
    totalDuration.textContent = `${minutes}:${seconds.toString().padStart(2, "0")}`;
});

// Seek functionality
seekBar.addEventListener("input", () => {
    audioPlayer.currentTime = (seekBar.value / 100) * audioPlayer.duration; 
});

audioPlayer.addEventListener("ended", () => {
    document.getElementById("current-time").textContent = "0:00";
    seekBar.value = 0;
    playPauseBtn.textContent = "▶";
});

function setIFrameTo(page) {
    const iframe = document.getElementById("content");
    iframe.src = "pages/" + page + ".html";
}

function playSong(song, artist) {
    song = song.toLowerCase().replace(/\s+/g, "_");
    artist = artist.toLowerCase().replace(/\s+/g, "_").replace(/song_by_/g, "");

    // this is a hack and sketchy but idk how else to do it
    const audioPlayer = window.parent.document.getElementById("audio-player");
    const playPauseBtn = window.parent.document.getElementById("play-pause-btn");
    if (navigator.userAgent.match(/(android|iphone|ipad)/i)) {
        audioPlayer.src = "stream?song=" + song + "&artist=" + artist + "&is_mobile=true";
        audioPlayer.type = "audio/aac";
    } else {
        audioPlayer.src = "stream?song=" + song + "&artist=" + artist + "&is_mobile=false";
        audioPlayer.type = "audio/mpeg";
    }
    audioPlayer.load();
    try {
        if (audioPlayer.paused) {
            audioPlayer.play();
            playPauseBtn.textContent = "⏸";
        } else {
            audioPlayer.pause();
            playPauseBtn.textContent = "▶";
        }
    } catch (error) {
        console.error(error);
        playPauseBtn.textContent = "▶";
    }
}