/// <reference types="@sveltejs/kit" />
/// <reference no-default-lib="true"/>
/// <reference lib="esnext" />
/// <reference lib="webworker" />

import { build, files, prerendered, version } from '$service-worker';

const sw = self as unknown as ServiceWorkerGlobalScope;

// Keep the `swolemate-cache-` prefix so the logout cache sweep in
// $lib/auth (CACHE_STORAGE_PREFIX) still matches. `version` busts the cache on
// every deploy automatically.
const CACHE = `swolemate-cache-${version}`;

// Deterministic precache of the *entire* build: every hashed JS/CSS chunk for
// all routes (`build`), everything in static/ (`files`, includes offline.html,
// icons, manifest, sounds), and any prerendered pages. This replaces the old
// approach of regex-scraping `/` for chunk URLs, which silently cached nothing
// if that fetch was intercepted and never cached lazy chunks for unvisited
// routes — leaving those routes blank offline.
const PRECACHE = [...build, ...files, ...prerendered];

sw.addEventListener('install', (event) => {
	event.waitUntil(
		(async () => {
			const cache = await caches.open(CACHE);
			await cache.addAll(PRECACHE);
			// Best-effort grab of the SPA shell so the app can cold-boot offline
			// before any successful online navigation has cached it. A single known
			// URL (not scraped), so a failure here can't take down the rest of the
			// precache.
			await cache.add('/').catch(() => undefined);
			await sw.skipWaiting();
		})()
	);
});

sw.addEventListener('activate', (event) => {
	event.waitUntil(
		(async () => {
			for (const key of await caches.keys()) {
				if (key !== CACHE) await caches.delete(key);
			}
			await sw.clients.claim();
		})()
	);
});

async function cacheFirst(request: Request): Promise<Response> {
	const cache = await caches.open(CACHE);
	const cached = await cache.match(request);
	if (cached) return cached;
	const response = await fetch(request);
	if (response.ok && response.type === 'basic') cache.put(request, response.clone());
	return response;
}

sw.addEventListener('fetch', (event) => {
	const url = new URL(event.request.url);

	// Don't interfere with cross-origin requests (e.g. an API on another origin).
	if (url.origin !== sw.location.origin) return;
	if (event.request.method !== 'GET') return;

	// Cache-first for immutable hashed build assets.
	if (url.pathname.startsWith('/_app/immutable/')) {
		event.respondWith(cacheFirst(event.request));
		return;
	}

	// Network-first for navigations so new deployments aren't stuck on a stale
	// shell; cache the shell on success and fall back to it (or offline.html)
	// when offline.
	if (event.request.mode === 'navigate') {
		event.respondWith(
			(async () => {
				try {
					const response = await fetch(event.request);
					const cache = await caches.open(CACHE);
					cache.put('/', response.clone());
					return response;
				} catch {
					const cache = await caches.open(CACHE);
					return (
						(await cache.match(event.request)) ??
						(await cache.match('/')) ??
						(await cache.match('/offline.html')) ??
						Response.error()
					);
				}
			})()
		);
		return;
	}

	// Never cache API responses (avoids leaking authenticated data across users).
	if (url.pathname.startsWith('/api/')) return;

	// Cache-first for everything else same-origin.
	event.respondWith(cacheFirst(event.request));
});
