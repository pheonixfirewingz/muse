async function getCoverArt(songName) {
    const apiUrl = `http://127.0.0.1:8000/cover-art?song=${encodeURIComponent(songName)}`;

    try {
        const response = await fetch(apiUrl);
        const result = await response.json();

        return result.cover_url || null;
    } catch (error) {
        console.error("Error fetching cover art:", error);
        return null;
    }
}

document.addEventListener("DOMContentLoaded", async () => {
    const images = document.querySelectorAll(".song-cover");

    for (const img of images) {
        const songName = img.dataset.song;
        if (!songName) continue;

        const coverUrl = await getCoverArt(songName);
        if (coverUrl) {
            img.src = coverUrl;
        }
    }
});