import { DOM } from '/assets/js/dom.js';

export function bindNav() {
    DOM.pageLinks.forEach(el => el.addEventListener('click', e => {
        e.preventDefault();
        setPage(el.dataset.page);
    }));
}

export function setPage(page) {
    fetchAndInsert(page, () => {}).catch(console.error);
}

export async function setPageQueryArtist(artist) {
    await fetchAndInsert(`list?artist_name=${encodeURIComponent(artist)}`, () => {});
}

export async function setPageQueryPlaylist(playlist, username = null, is_public = false) {
    if (is_public) {
        const usernameParam = username ? `?username=${encodeURIComponent(username)}` : '';
        await fetchAndInsert(`playlist/public/${encodeURIComponent(playlist)}${usernameParam}`, () => {
        });
    } else {
        await fetchAndInsert(`playlist/private/${encodeURIComponent(playlist)}`, () => {
        });
    }
}

async function fetchAndInsert(url, callback) {
    try {
        const res = await fetch(url);
        if (res.status === 401) {
            window.location.href = "/login";
            return;
        }
        if (!res.ok) throw new Error(res.statusText);

        DOM.content.innerHTML = await res.text();
        await new Promise(resolve => requestAnimationFrame(resolve));
        callback?.();
    } catch (e) {
        console.error(`Failed to fetch ${url}:`, e);
        DOM.content.innerHTML = '<p>Error loading content</p>';
    }
}
