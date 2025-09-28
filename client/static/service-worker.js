// Cache name
const STATIC_CACHE = 'swolemate-static-v2';
const API_CACHE = 'swolemate-api-v1';

// Assets to cache
const ASSETS_TO_CACHE = [
    './',
    './manifest.json',
    './favicon.png',
    './pwa-192.png',
    './pwa-512.png',
    './_app/immutable/',  // SvelteKit assets
    './app.css'
];

// Install event
self.addEventListener('install', (event) => {
    event.waitUntil(
        caches.open(STATIC_CACHE)
            .then((cache) => {
                console.log('Opened cache');
                return cache.addAll(ASSETS_TO_CACHE);
            })
            .then(() => {
                return self.skipWaiting();
            })
    );
});

// Activate event
self.addEventListener('activate', (event) => {
    event.waitUntil(
        Promise.all([
            // Clean up old caches
            caches.keys().then((cacheNames) => {
                return Promise.all(
                    cacheNames.map((cacheName) => {
                        if (![STATIC_CACHE, API_CACHE].includes(cacheName)) {
                            return caches.delete(cacheName);
                        }
                    })
                );
            }),
            // Take control of all pages immediately
            self.clients.claim()
        ])
    );
});

// Fetch event with network-first strategy for API calls and cache-first for assets
self.addEventListener('fetch', (event) => {
    if (event.request.method !== 'GET') {
        return;
    }

    const url = new URL(event.request.url);
    
    // Network-first strategy for API calls
    if (url.pathname.startsWith('/api/')) {
        event.respondWith(networkFirst(event.request));
        return;
    }

    // Cache-first strategy for assets and other requests
    event.respondWith(
        caches.match(event.request).then((response) => {
            if (response) {
                return response;
            }
            return fetch(event.request).then((networkResponse) => {
                if (!networkResponse || networkResponse.status !== 200 || networkResponse.type !== 'basic') {
                    return networkResponse;
                }

                const cachedResponse = networkResponse.clone();

                caches.open(STATIC_CACHE).then((cache) => {
                    cache.put(event.request, cachedResponse);
                });

                return networkResponse;
            }).catch(async () => {
                const cache = await caches.open(STATIC_CACHE);
                const fallback = await cache.match('./');
                return fallback || new Response('Offline', { status: 503, statusText: 'Offline' });
            });
        })
    );
});

async function networkFirst(request) {
    const cache = await caches.open(API_CACHE);
    try {
        const networkResponse = await fetch(request);
        if (networkResponse && networkResponse.status === 200) {
            cache.put(request, networkResponse.clone());
        }
        return networkResponse;
    } catch (error) {
        const cachedResponse = await cache.match(request);
        if (cachedResponse) {
            return cachedResponse;
        }
        return new Response('Offline', { status: 503, statusText: 'Offline' });
    }
}
