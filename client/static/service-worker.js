// Cache name
const CACHE_NAME = 'swolemate-cache-v1';

// Assets to cache
const ASSETS_TO_CACHE = ['/', '/manifest.json', '/favicon.png', '/pwa-192.png', '/pwa-512.png'];

// Install event
self.addEventListener('install', (event) => {
	event.waitUntil(
		caches
			.open(CACHE_NAME)
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
						if (cacheName !== CACHE_NAME) {
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
	const url = new URL(event.request.url);

	// Network-first strategy for API calls
	if (url.pathname.startsWith('/api/')) {
		if (event.request.method !== 'GET') return;
		event.respondWith(
			fetch(event.request)
				.then((response) => {
					const responseToCache = response.clone();
					caches.open(CACHE_NAME).then((cache) => cache.put(event.request, responseToCache));
					return response;
				})
				.catch(() => {
					return caches.match(event.request);
				})
		);
		return;
	}

	// Cache-first strategy for assets and other requests
	event.respondWith(
		caches.match(event.request).then((response) => {
			if (response) {
				return response;
			}
			return fetch(event.request).then((response) => {
				// Check if we received a valid response
				if (!response || response.status !== 200 || response.type !== 'basic') {
					return response;
				}

				// Clone the response as it can only be consumed once
				const responseToCache = response.clone();

				caches.open(CACHE_NAME).then((cache) => {
					cache.put(event.request, responseToCache);
				});

				return response;
			});
		})
	);
});
