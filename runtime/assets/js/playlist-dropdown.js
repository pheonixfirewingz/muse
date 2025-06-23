export async function addToPlaylist(songName, artistName) {
    const res = await fetch(`/api/playlists`);
    if (!res.ok) {
        Swal.fire({
            icon: 'error',
            title: 'Error',
            text: 'Failed to load playlists',
        });
        return;
    }
    
    const result = await res.json();
    if (!result.success) {
        Swal.fire({
            icon: 'error',
            title: 'Error',
            text: result.message || 'Failed to load playlists',
        });
        return;
    }
    
    const playlists = result.data;
    if (!Array.isArray(playlists)) {
        Swal.fire({
            icon: 'error',
            title: 'Error',
            text: 'Invalid playlist data received',
        });
        return;
    }
    
    const playlistOptions = playlists.map(playlist =>
        `<option value="${playlist.name}">${playlist.name}</option>`
    ).join('');

    Swal.fire({
        title: 'Add to Playlist',
        html: `
                    <div style="text-align: left; padding: 1rem;">
                        <p style="margin-bottom: 1rem; color: var(--primary-color);">
                            <strong>${songName}</strong> by ${artistName}
                        </p>
                        <label for="playlist-select" style="display: block; margin-bottom: 0.5rem; font-weight: 500;">
                            Choose Playlist:
                        </label>
                        <select id="playlist-select" style="width: 100%; padding: 0.75rem; border: 2px solid #e9ecef; border-radius: 8px; font-size: 1rem;">
                            <option value="">Select a playlist...</option>
                            ${playlistOptions}
                            <option value="new">+ Create New Playlist</option>
                        </select>
                    </div>
                `,
        showCancelButton: true,
        confirmButtonText: 'Add Song',
        cancelButtonText: 'Cancel',
        preConfirm: () => {
            const selectedPlaylist = document.getElementById('playlist-select').value;
            if (!selectedPlaylist) {
                Swal.showValidationMessage('Please select a playlist');
                return false;
            }
            return selectedPlaylist;
        }
    }).then(async (result) => {
        if (result.isConfirmed) {
            if (result.value === 'new') {
                let _ = await createNewPlaylist(songName, artistName);
            } else {
                addSongToExistingPlaylist(songName, artistName, result.value);
            }
        }
    });
}

async function addSongToExistingPlaylist(songName, artistName, playlistName) {
    try {
        const formData = new FormData();
        formData.append('playlist_name', playlistName);
        formData.append('song_name', songName);
        formData.append('artist_name', artistName);

        const response = await fetch('/api/playlists/songs', {
            method: 'POST',
            body: new URLSearchParams(formData)
        });

        const result = await response.json();

        if (response.ok && result.success) {
            Swal.fire({
                icon: 'success',
                title: 'Added to Playlist!',
                text: result.message,
                timer: 2000,
                showConfirmButton: false
            });
        } else {
            Swal.fire({
                icon: 'error',
                title: 'Error',
                text: result.message || 'Failed to add song to playlist',
            });
        }
    } catch (error) {
        console.error('Error adding song to playlist:', error);
        Swal.fire({
            icon: 'error',
            title: 'Error',
            text: 'Failed to add song to playlist. Please try again.',
        });
    }
}

export async function createNewPlaylist(songName = null, artistName = null, createOnly = false) {
    Swal.fire({
        title: 'Create New Playlist',
        html: `
            <div style="text-align: left; padding: 1rem;">
                ${songName && artistName ? `<p style="margin-bottom: 1rem; color: var(--primary-color);">
                    <strong>${songName}</strong> by ${artistName}
                </p>` : ''}
                <label for="playlist-name" style="display: block; margin-bottom: 0.5rem; font-weight: 500;">
                    Playlist Name:
                </label>
                <input type="text" id="playlist-name" placeholder="Enter playlist name..." 
                       style="width: 100%; padding: 0.75rem; border: 2px solid #e9ecef; border-radius: 8px; font-size: 1rem;"
                       maxlength="50">
                <div style="margin-top: 1rem;">
                    <label style="display: flex; align-items: center; gap: 0.5rem;">
                        <input type="checkbox" id="playlist-public" style="margin: 0;">
                        <span>Make playlist public</span>
                    </label>
                </div>
            </div>
        `,
        showCancelButton: true,
        confirmButtonText: createOnly ? 'Create & Add Song' : 'Create Playlist',
        cancelButtonText: 'Cancel',
        preConfirm: () => {
            const playlistName = document.getElementById('playlist-name').value.trim();
            if (!playlistName) {
                Swal.showValidationMessage('Please enter a playlist name');
                return false;
            }
            if (playlistName.length < 1 || playlistName.length > 50) {
                Swal.showValidationMessage('Playlist name must be between 1 and 50 characters');
                return false;
            }
            return {
                name: playlistName,
                public: document.getElementById('playlist-public').checked
            };
        }
    }).then((result) => {
        if (result.isConfirmed) {
            if (createOnly) {
                createPlaylistOnly(result.value.name, result.value.public);
            } else {
                createPlaylistAndAddSong(songName, artistName, result.value.name, result.value.public);
            }
        }
    });
}

async function createPlaylistOnly(playlistName, isPublic) {
    try {
        const formData = new FormData();
        formData.append('playlist_name', playlistName);
        formData.append('public', isPublic.toString());

        const response = await fetch('/api/playlists/create', {
            method: 'POST',
            body: new URLSearchParams(formData)
        });

        const result = await response.json();

        if (response.ok && result.success) {
            Swal.fire({
                icon: 'success',
                title: 'Playlist Created!',
                text: result.message,
                timer: 2000,
                showConfirmButton: false
            });
            if (typeof onNewPlaylistCreated === 'function') {
                onNewPlaylistCreated({
                    name: playlistName,
                    uuid: result.playlist_uuid
                });
            }
        } else {
            Swal.fire({
                icon: 'error',
                title: 'Error',
                text: result.message || 'Failed to create playlist',
            });
        }
    } catch (error) {
        console.error('Error creating playlist:', error);
        Swal.fire({
            icon: 'error',
            title: 'Error',
            text: 'Failed to create playlist. Please try again.',
        });
    }
}

async function createPlaylistAndAddSong(songName, artistName, playlistName, isPublic) {
    try {
        const formData = new FormData();
        formData.append('playlist_name', playlistName);
        formData.append('song_name', songName);
        formData.append('artist_name', artistName);
        formData.append('public', isPublic.toString());

        const response = await fetch('/api/playlists/create_and_add', {
            method: 'POST',
            body: new URLSearchParams(formData)
        });

        const result = await response.json();

        if (response.ok && result.success) {
            Swal.fire({
                icon: 'success',
                title: 'Playlist Created!',
                text: result.message,
                timer: 2000,
                showConfirmButton: false
            });
            
            // Optionally refresh the playlist list if there's a global function to do so
            if (typeof onNewPlaylistCreated === 'function') {
                onNewPlaylistCreated({
                    name: playlistName,
                    uuid: result.playlist_uuid
                });
            }
        } else {
            Swal.fire({
                icon: 'error',
                title: 'Error',
                text: result.message || 'Failed to create playlist',
            });
        }
    } catch (error) {
        console.error('Error creating playlist:', error);
        Swal.fire({
            icon: 'error',
            title: 'Error',
            text: 'Failed to create playlist. Please try again.',
        });
    }
}

export async function deletePlaylist(playlistName, btnElem) {
    const confirmed = await Swal.fire({
        title: 'Delete Playlist?',
        text: `Are you sure you want to delete the playlist "${playlistName}"? This cannot be undone!`,
        icon: 'warning',
        showCancelButton: true,
        confirmButtonText: 'Delete',
        cancelButtonText: 'Cancel',
        confirmButtonColor: '#d33',
        cancelButtonColor: '#3085d6',
    });
    if (!confirmed.isConfirmed) return;

    try {
        const formData = new FormData();
        formData.append('playlist_name', playlistName);
        const response = await fetch('/api/playlists/delete', {
            method: 'POST',
            body: new URLSearchParams(formData)
        });
        if (response.ok) {
            Swal.fire({
                icon: 'success',
                title: 'Deleted!',
                text: `Playlist "${playlistName}" has been deleted.`,
                timer: 1500,
                showConfirmButton: false
            });
            // Remove the card from the DOM
            if (btnElem && btnElem.closest('.card')) {
                btnElem.closest('.card').remove();
            }
        } else {
            const result = await response.json();
            Swal.fire({
                icon: 'error',
                title: 'Error',
                text: result.message || 'Failed to delete playlist.'
            });
        }
    } catch (error) {
        Swal.fire({
            icon: 'error',
            title: 'Error',
            text: 'Failed to delete playlist. Please try again.'
        });
    }
}

export async function showPlaylistSongsModal({ name, type, username }) {
    ensureModalElements();
    let url = '';
    let title = name;
    if (type === 'public') {
        url = `/api/playlists/public_songs?name=${encodeURIComponent(name)}&username=${encodeURIComponent(username)}`;
        title += ` (by ${username})`;
    } else {
        url = `/api/playlists/private_songs?name=${encodeURIComponent(name)}`;
    }
    modalTitle.textContent = title;
    modalSongs.innerHTML = '<div class="loading-spinner"></div>';
    playlistSongsModal.style.display = 'block';
    try {
        const resp = await fetch(url);
        const data = await resp.json();
        if (!data.success) throw new Error(data.message);
        const songs = data.data.songs;
        if (!songs || songs.length === 0) {
            modalSongs.innerHTML = '<p>No songs in this playlist.</p>';
        } else {
            modalSongs.innerHTML = songs.map(song =>
                `<div class=\"song-row\"><strong>${song.name}</strong> <span style=\"color:#888;\">by ${song.artist_name}</span></div>`
            ).join('');
        }
    } catch (err) {
        modalSongs.innerHTML = `<p style=\"color:red;\">Failed to load songs: ${err.message}</p>`;
    }
}