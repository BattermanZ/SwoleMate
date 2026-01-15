// Cache name (bump to invalidate old cached bundles)
const CACHE_NAME = 'swolemate-cache-v3';

// Assets to cache
const ASSETS_TO_CACHE = [
	'/',
	'/offline.html',
	'/manifest.json',
	'/favicon.png',
	'/pwa-192.png',
	'/pwa-512.png'
];

async function precacheAppShell(cache) {
	try {
		const response = await fetch('/', { cache: 'no-cache' });
		const html = await response.text();
		const matches = Array.from(html.matchAll(/(?:href|src)=["'](\/_app\/immutable\/[^"']+)["']/g));
		const urls = Array.from(new Set(matches.map((m) => m[1])));
		if (urls.length) await cache.addAll(urls);
	} catch (err) {
		console.warn('App shell pre-cache failed', err);
	}
}

// Install event
self.addEventListener('install', (event) => {
	event.waitUntil(
		(async () => {
			const cache = await caches.open(CACHE_NAME);
			await cache.addAll(ASSETS_TO_CACHE);
			await precacheAppShell(cache);
			await self.skipWaiting();
		})()
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

	// Don't interfere with cross-origin requests (e.g. API hosted on a different domain/port).
	if (url.origin !== self.location.origin) return;

	// Cache-first for built assets (critical for iOS offline PWA reliability).
	if (url.pathname.startsWith('/_app/immutable/')) {
		event.respondWith(
			caches.open(CACHE_NAME).then(async (cache) => {
				const cached = await cache.match(event.request);
				if (cached) return cached;
				const response = await fetch(event.request);
				if (response.ok) cache.put(event.request, response.clone());
				return response;
			})
		);
		return;
	}

	// Always go to the network first for navigations so new deployments aren't stuck on a stale cache.
	if (event.request.mode === 'navigate') {
		event.respondWith(
			fetch(event.request).catch(async () => {
				const cache = await caches.open(CACHE_NAME);
				return (
					(await cache.match(event.request)) ??
					(await cache.match('/')) ??
					(await cache.match('/offline.html'))
				);
			})
		);
		return;
	}

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
		caches.open(CACHE_NAME).then(async (cache) => {
			const response = await cache.match(event.request);
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

				cache.put(event.request, responseToCache);

				return response;
			});
		})
	);
});
