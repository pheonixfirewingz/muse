// meta-cache.service.ts
// Caches song and artist data in IndexedDB for 24 hours, only deletes expired data if online

const META_DB_NAME = 'meta-cache-db';
const META_DB_VERSION = 1;
const SONGS_STORE = 'songs';
const ARTISTS_STORE = 'artists';
const CACHE_DURATION = 7 * 24 * 60 * 60 * 1000; // 1 week in milliseconds

function openMetaDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(META_DB_NAME, META_DB_VERSION);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(SONGS_STORE)) {
        db.createObjectStore(SONGS_STORE, { keyPath: 'key' });
      }
      if (!db.objectStoreNames.contains(ARTISTS_STORE)) {
        db.createObjectStore(ARTISTS_STORE, { keyPath: 'key' });
      }
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

function getCachedMeta(db: IDBDatabase, store: string, key: string): Promise<{data: any, timestamp: number} | null> {
  return new Promise((resolve, reject) => {
    const tx = db.transaction(store, 'readonly');
    const s = tx.objectStore(store);
    const req = s.get(key);
    req.onsuccess = () => resolve(req.result || null);
    req.onerror = () => reject(req.error);
  });
}

function putCachedMeta(db: IDBDatabase, store: string, key: string, data: any, timestamp: number) {
  return new Promise((resolve, reject) => {
    const tx = db.transaction(store, 'readwrite');
    const s = tx.objectStore(store);
    s.put({ key, data, timestamp });
    tx.oncomplete = resolve;
    tx.onerror = () => reject(tx.error);
  });
}

function deleteCachedMeta(db: IDBDatabase, store: string, key: string) {
  return new Promise((resolve, reject) => {
    const tx = db.transaction(store, 'readwrite');
    const s = tx.objectStore(store);
    s.delete(key);
    tx.oncomplete = resolve;
    tx.onerror = () => reject(tx.error);
  });
}

export class MetaCacheService {
  static async getSongs(key: string = 'all'): Promise<any | null> {
    const db = await openMetaDB();
    const cached = await getCachedMeta(db, SONGS_STORE, key);
    const now = Date.now();
    if (cached) {
      if ((now - cached.timestamp) < CACHE_DURATION) {
        return cached.data;
      } else if (!navigator.onLine) {
        // If offline, serve expired data
        return cached.data;
      } else {
        // If online and expired, delete
        await deleteCachedMeta(db, SONGS_STORE, key);
        return null;
      }
    }
    return null;
  }

  static async setSongs(data: any, key: string = 'all') {
    const db = await openMetaDB();
    await putCachedMeta(db, SONGS_STORE, key, data, Date.now());
  }

  static async getArtists(key: string = 'all'): Promise<any | null> {
    const db = await openMetaDB();
    const cached = await getCachedMeta(db, ARTISTS_STORE, key);
    const now = Date.now();
    if (cached) {
      if ((now - cached.timestamp) < CACHE_DURATION) {
        return cached.data;
      } else if (!navigator.onLine) {
        // If offline, serve expired data
        return cached.data;
      } else {
        // If online and expired, delete
        await deleteCachedMeta(db, ARTISTS_STORE, key);
        return null;
      }
    }
    return null;
  }

  static async setArtists(data: any, key: string = 'all') {
    const db = await openMetaDB();
    await putCachedMeta(db, ARTISTS_STORE, key, data, Date.now());
  }

  static async getTotal(entity: 'songs' | 'artists'): Promise<number | null> {
    const db = await openMetaDB();
    const store = entity === 'songs' ? SONGS_STORE : ARTISTS_STORE;
    const cached = await getCachedMeta(db, store, entity + '_total');
    const now = Date.now();
    if (cached) {
      if ((now - cached.timestamp) < CACHE_DURATION) {
        return cached.data;
      } else if (!navigator.onLine) {
        return cached.data;
      } else {
        await deleteCachedMeta(db, store, entity + '_total');
        return null;
      }
    }
    return null;
  }

  static async setTotal(entity: 'songs' | 'artists', total: number) {
    const db = await openMetaDB();
    const store = entity === 'songs' ? SONGS_STORE : ARTISTS_STORE;
    await putCachedMeta(db, store, entity + '_total', total, Date.now());
  }
}
