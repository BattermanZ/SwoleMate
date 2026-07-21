# Service worker / PWA cache lifecycle

2 confirmed (1 medium, 1 low unverified), 0 refuted.

## Confirmed findings

### MEDIUM [data-integrity]: skipWaiting + activate cache-purge forces a mid-session full reload / broken route navigation after a deploy

- **Trigger:** User on `/` (Today) with an in-progress logged session; a new version deploys while the tab is open. New SW installs, calls `skipWaiting()`, claims all clients, then deletes the old cache. User navigates to a route requiring a not-yet-loaded chunk (e.g., taps History) → old-hash chunk URL is requested → 404 → SvelteKit forces full-page reload mid-session.

- **Location:** `client/src/service-worker.ts:33, 38–47, 66–69`
  - Line 33: `sw.skipWaiting()` called unconditionally in install handler
  - Lines 41–42: Activate handler deletes every cache whose key is not the new `swolemate-cache-<version>`
  - Line 44: `clients.claim()` called without update-prompt logic
  - Lines 66–69: Fetch handler's cacheFirst for `/_app/immutable/` only checks current-version cache

- **What happens:** The moment a new build finishes precaching, the new SW activates and purges the OLD version cache holding all old hashed `/_app/immutable/` chunks. A client still running the OLD build (e.g., mid-workout on the Today live logger) is now controlled by the new SW. Any subsequent lazy chunk import requests an OLD hashed URL; the new SW's cacheFirst misses (new cache only has new hashes) and falls through to network; on adapter-static the old-hash file has been overwritten by the deploy, so it 404s, the dynamic import rejects, and SvelteKit falls back to a full-page reload. There is no 'update available' prompt and no controllerchange gating — the reload happens mid-session.

- **Why:** Committed sets are persisted to IndexedDB after each action (`persistInProgressSession` in `client/src/lib/today/controller/offline.ts`) and rehydrated on boot, so logged sets survive — hence medium not critical. But any not-yet-submitted input in the live logger (a typed-but-unsaved set, an in-progress note) is lost, and the view reloads mid-set. This is exactly the 'update flow that forces a reload mid-session' hazard, the classic reason SPAs avoid unconditional `skipWaiting()`.

- **Fix sketch:** Do not call `skipWaiting()`/`clients.claim()` unconditionally. Let the new SW wait in registration.waiting, detect this from the page, and surface a user-triggered 'Update ready — reload' affordance (suppressed while a live session is in progress). At minimum, keep the previous version's immutable chunks reachable so a stale client can finish its session without a hard-404 reload.

### LOW [security] (unverified): SW cache-exclusion only covers /api/; same-origin GET to /oauth, /mcp, /.well-known falls through to the shared cache

- **Trigger:** A GET to `/oauth/...` or `/.well-known/...` returns a 200 basic response; cacheFirst stores it in the shared `swolemate-cache-<version>` bucket. On the same device the next user's identical GET is served the cached copy instead of a fresh per-identity response.

- **Location:** `client/src/service-worker.ts:96–100`
  - Line 96: Comment states 'Never cache API responses (avoids leaking authenticated data across users)'
  - Line 97: Guard only matches `url.pathname.startsWith('/api/')`
  - Line 100: Catch-all cacheFirst for all non-excluded same-origin paths

- **What happens:** The proxy seam exposes `/oauth`, `/mcp`, and `/.well-known` as same-origin paths (vite.config.ts proxy; settings/+page.svelte builds `${origin}/mcp`). Any same-origin GET to one of those paths returning a 200 basic response falls through to the catch-all cacheFirst and is stored in the single shared `swolemate-cache-<version>` bucket keyed only by URL with no per-user scoping; cacheFirst then serves it before the network on a later request (including a different user on a shared device). Navigations to `/oauth*` are additionally captured by the navigate branch and written under the `/` key (line 80), which could poison the offline shell.

- **Why:** In practice the SPA drives all data through `/api/` (`client/src/lib/api.ts`), the `/mcp` and `/oauth` surfaces target external MCP clients, and logout sweeps `swolemate-cache-*` (`client/src/lib/auth/index.ts`), so real exposure is narrow — hence low. But the exclusion list not matching its stated intent is a defense-in-depth gap on exactly the paths the comment is meant to protect.

- **Fix sketch:** Broaden the non-cache guard to all backend-proxied prefixes (`/api/`, `/mcp`, `/oauth`, `/.well-known`) and exclude those paths from the navigate-branch cache.put('/') as well, so no identity-scoped response can enter the shared cache.

## Refuted (not real / already handled)

None.
