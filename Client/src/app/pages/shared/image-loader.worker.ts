// image-loader.worker.ts

const CACHE_DURATION = 24 * 60 * 60 * 1000; // 24 hours in ms
const DB_NAME = 'image-cache-db';
const STORE_NAME = 'images';
const DB_VERSION = 1;

function openDB() {
  return new Promise<IDBDatabase>((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME, { keyPath: 'url' });
      }
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

function getCachedImage(db: IDBDatabase, url: string): Promise<{blob?: Blob, timestamp: number, notFound?: boolean} | null> {
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, 'readonly');
    const store = tx.objectStore(STORE_NAME);
    const req = store.get(url);
    req.onsuccess = () => resolve(req.result || null);
    req.onerror = () => reject(req.error);
  });
}

function putCachedImage(db: IDBDatabase, url: string, blob: Blob | null, timestamp: number, notFound?: boolean) {
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, 'readwrite');
    const store = tx.objectStore(STORE_NAME);
    const value = blob ? { url, blob, timestamp } : { url, notFound: true, timestamp };
    store.put(value);
    tx.oncomplete = resolve;
    tx.onerror = () => reject(tx.error);
  });
}

function deleteCachedImage(db: IDBDatabase, url: string) {
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, 'readwrite');
    const store = tx.objectStore(STORE_NAME);
    store.delete(url);
    tx.oncomplete = resolve;
    tx.onerror = () => reject(tx.error);
  });
}

self.onmessage = async function (event) {
  const { url, headers } = event.data;
  const now = Date.now();
  let db;
  try {
    db = await openDB();
    const cached = await getCachedImage(db, url);
    if (cached && (now - cached.timestamp) < CACHE_DURATION) {
      if (cached.notFound) {
        self.postMessage({ notFound: true, url, cached: true });
        return;
      } else if (cached.blob) {
        const objectUrl = URL.createObjectURL(cached.blob);
        self.postMessage({ objectUrl, url, cached: true });
        return;
      }
    } else if (cached) {
      // Clean up expired
      await deleteCachedImage(db, url);
    }
  } catch (e) {
    db = null;
  }
  try {
    const response = await fetch(url, { headers });
    if (!response.ok) {
      if (db) await putCachedImage(db, url, null, now, true);
      self.postMessage({ notFound: true, url, cached: false });
      return;
    }
    const arrayBuffer = await response.arrayBuffer();
    if (!arrayBuffer || arrayBuffer.byteLength === 0) {
      if (db) await putCachedImage(db, url, null, now, true);
      self.postMessage({ notFound: true, url, cached: false });
      return;
    }
    const contentType = response.headers.get('content-type') || 'image/avif';
    const blob = new Blob([arrayBuffer], { type: contentType });
    if (db) {
      await putCachedImage(db, url, blob, now);
    }
    const objectUrl = URL.createObjectURL(blob);
    self.postMessage({ objectUrl, url, cached: false });
  } catch (error) {
    if (db) await putCachedImage(db, url, null, now, true);
    self.postMessage({ notFound: true, url, cached: false });
  }
};
