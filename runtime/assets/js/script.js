import { bindPlayerControls, playSong } from '/assets/js/audioPlayer.js';
import { bindNav, setPage ,setPageQueryArtist, setPageQueryPlaylist } from '/assets/js/navigation.js';
import { addToPlaylist, createNewPlaylist, deletePlaylist } from '/assets/js/playlist-dropdown.js'
import '/assets/js/imageObserver.js';

document.addEventListener('DOMContentLoaded', () => {
    bindNav();
    bindPlayerControls();
    setPage(window.location.hash.slice(1) || 'home');
});

//add to globals
window.addToPlaylist = addToPlaylist;
window.playSong = playSong;
window.setPageQueryArtist = setPageQueryArtist;
window.setPageQueryPlaylist = setPageQueryPlaylist;
window.createNewPlaylist = createNewPlaylist;
window.deletePlaylist = deletePlaylist;

// Optional setup
bindPlayerControls();