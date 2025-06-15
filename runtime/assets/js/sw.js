
const CACHE_NAME = 'muse-cache-v1';
const URLS_TO_CACHE = [
    '/',
    '/app',
    '/assets/css/style.css',
    '/assets/js/script.js',
    '/pages/home.html',
    '/pages/songs.html',
    '/pages/artists.html',
];

// Helper function for logging
const log = {
    info: (message, ...args) => {
        console.log(`[ServiceWorker Info] ${message}`, ...args);
    },
    error: (message, ...args) => {
        console.error(`[ServiceWorker Error] ${message}`, ...args);
    },
    warn: (message, ...args) => {
        console.warn(`[ServiceWorker Warning] ${message}`, ...args);
    },
    debug: (message, ...args) => {
        console.debug(`[ServiceWorker Debug] ${message}`, ...args);
    }
};

// Error handling wrapper
const handleError = async (operation, errorHandler) => {
    try {
        return await operation();
    } catch (error) {
        log.error(`${errorHandler.name} failed:`, error);
        return errorHandler(error);
    }
};

// Install event handler
self.addEventListener('install', (event) => {
    log.info('Installing Service Worker...');

    const cacheResources = async () => {
        try {
            const cache = await caches.open(CACHE_NAME);
            log.info('Cache opened successfully');

            const results = await Promise.allSettled(
                URLS_TO_CACHE.map(async (url) => {
                    try {
                        await cache.add(url);
                        log.debug(`Successfully cached: ${url}`);
                        return url;
                    } catch (error) {
                        log.error(`Failed to cache: ${url}`, error);
                        throw error;
                    }
                })
            );

            const failed = results
                .filter(r => r.status === 'rejected')
                .map((r, i) => URLS_TO_CACHE[i]);

            if (failed.length > 0) {
                log.warn('Some resources failed to cache:', failed);
            } else {
                log.info('All resources cached successfully');
            }
        } catch (error) {
            log.error('Cache initialization failed:', error);
            throw error;
        }
    };

    event.waitUntil(
        handleError(cacheResources, (error) => {
            log.error('Service Worker installation failed:', error);
            return Promise.reject(error);
        })
    );
});

// Fetch event handler
self.addEventListener('fetch', (event) => {
    const handleFetch = async () => {
        try {
            // Try to get from cache first
            const cachedResponse = await caches.match(event.request);
            if (cachedResponse) {
                log.debug(`Serving from cache: ${event.request.url}`);
                return cachedResponse;
            }

            // Clone the request for future caching
            const fetchRequest = event.request.clone();

            try {
                const response = await fetch(fetchRequest);

                // Validate response
                if (!response || response.status !== 200 || response.type !== 'basic') {
                    log.warn(`Invalid response for ${event.request.url}:`, {
                        exists: !!response,
                        status: response?.status,
                        type: response?.type
                    });
                    return response;
                }

                // Clone the response for caching
                const responseToCache = response.clone();

                // Cache the response
                try {
                    const cache = await caches.open(CACHE_NAME);
                    await cache.put(event.request, responseToCache);
                    log.debug(`Cached new response for: ${event.request.url}`);
                } catch (cacheError) {
                    log.error(`Failed to cache response for ${event.request.url}:`, cacheError);
                }

                return response;
            } catch (fetchError) {
                log.error(`Network request failed for ${event.request.url}:`, fetchError);
                throw fetchError;
            }
        } catch (error) {
            log.error(`Fetch handler failed for ${event.request.url}:`, error);
            throw error;
        }
    };

    event.respondWith(
        handleError(handleFetch, (error) => {
            log.error('Fetch operation failed:', error);
            return new Response('Network error', {
                status: 503,
                statusText: 'Service Unavailable'
            });
        })
    );
});

// Activate event handler
self.addEventListener('activate', (event) => {
    log.info('Activating Service Worker...');

    const cleanupCaches = async () => {
        try {
            const cacheNames = await caches.keys();
            log.debug('Found caches:', cacheNames);

            const deletionResults = await Promise.allSettled(
                cacheNames.map(async (cacheName) => {
                    if (cacheName !== CACHE_NAME) {
                        try {
                            await caches.delete(cacheName);
                            log.info(`Deleted old cache: ${cacheName}`);
                            return cacheName;
                        } catch (error) {
                            log.error(`Failed to delete cache ${cacheName}:`, error);
                            throw error;
                        }
                    }
                })
            );

            const failed = deletionResults
                .filter(r => r.status === 'rejected')
                .map((r, i) => cacheNames[i]);

            if (failed.length > 0) {
                log.warn('Some caches failed to delete:', failed);
            } else {
                log.info('Cache cleanup completed successfully');
            }
        } catch (error) {
            log.error('Cache cleanup failed:', error);
            throw error;
        }
    };

    event.waitUntil(
        handleError(cleanupCaches, (error) => {
            log.error('Service Worker activation failed:', error);
            return Promise.reject(error);
        })
    );
});

// Handle service worker updates
self.addEventListener('message', (event) => {
    log.debug('Received message:', event.data);

    if (event.data && event.data.type === 'SKIP_WAITING') {
        log.info('Skip waiting requested');
        self.skipWaiting();
    }
});

// Error event handler
self.addEventListener('error', (event) => {
    log.error('Service Worker global error:', event.error);
});

// Unhandled rejection handler
self.addEventListener('unhandledrejection', (event) => {
    log.error('Unhandled promise rejection:', event.reason);
});

// Periodic cache validation
const validateCache = async () => {
    try {
        log.debug('Starting cache validation');
        const cache = await caches.open(CACHE_NAME);
        const requests = await cache.keys();

        const validationResults = await Promise.allSettled(
            requests.map(async (request) => {
                try {
                    const response = await cache.match(request);
                    if (!response || !response.ok) {
                        await cache.delete(request);
                        log.warn(`Removed invalid cache entry: ${request.url}`);
                    }
                    return request.url;
                } catch (error) {
                    log.error(`Cache validation failed for ${request.url}:`, error);
                    throw error;
                }
            })
        );

        const failed = validationResults
            .filter(r => r.status === 'rejected')
            .length;

        if (failed > 0) {
            log.warn(`Cache validation completed with ${failed} failures`);
        } else {
            log.info('Cache validation completed successfully');
        }
    } catch (error) {
        log.error('Cache validation failed:', error);
    }
};

// Run cache validation periodically
setInterval(validateCache, 24 * 60 * 60 * 1000); // Once per day