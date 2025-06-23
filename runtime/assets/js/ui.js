export function toggleDropdown(event, button) {
    event.stopPropagation();
    const dropdown = button.closest(".dropdown").querySelector(".dropdown-content");
    const { songName, artistName } = dropdown.dataset;

    renderPlaylistDropdown(dropdown, songName, artistName);
    dropdown.classList.toggle("show");
}

function renderPlaylistDropdown(container, songName, artistName) {
    container.innerHTML = ''; // Clear existing

    const header = document.createElement("div");
    header.className = "dropdown-header";
    header.textContent = "Add to Playlist";
    container.appendChild(header);

    window.playlists.forEach(playlist => {
        const item = document.createElement("div");
        item.className = "playlist-item";
        item.innerHTML = `<i class="fas fa-list"></i> ${playlist.name}`;
        item.onclick = () => addToPlaylist(songName, artistName, playlist.uuid, playlist.name);
        container.appendChild(item);
    });

    const createItem = document.createElement("div");
    createItem.className = "playlist-item create-playlist-item";
    createItem.innerHTML = `<i class="fas fa-plus-circle"></i> Create New Playlist`;
    createItem.onclick = () => openCreatePlaylistModal(songName, artistName);
    container.appendChild(createItem);
}

document.addEventListener('click', () => {
    document.querySelectorAll('.dropdown-content').forEach(d => d.classList.remove('show'));
});

function onNewPlaylistCreated(newPlaylist) {
    window.playlists.push(newPlaylist);
}