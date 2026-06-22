# Offline Layer Audit — 2026-06-19

Deep dive into SwoleMate's offline/sync layer (client). Findings ordered by
severity. Each item cites the relevant file and line so it can be acted on
directly.

## Scope & architecture map

The offline layer is made of five collaborating pieces:

- **Network detection** — `client/src/lib/stores/network.ts` (an `online`
  Svelte store) plus `client/src/lib/today/controller/utils.ts:isNetworkFailure`
  (try/catch on `fetch` combined with `navigator.onLine`).
- **Persistence** — `client/src/lib/offline/storage.ts` (IndexedDB `kv` store
  with a localStorage fallback) wrapped by
  `client/src/lib/offline/todaySessions.ts` and
  `client/src/lib/today/controller/actions/plannedTemplate.ts`. Keys are
  user-scoped via `client/src/lib/auth/scope.ts`.
- **Mutation routing** — every action under
  `client/src/lib/today/controller/actions/*` has an "online path" and a mirror
  "offline path", gated on `offlineMode || session.id < 0 || exerciseId < 0`.
- **Reconciliation** — `client/src/lib/today/controller/actions/sync.ts`
  (triggers) → `client/src/lib/today/controller/offline.ts:syncOne` (replays a
  locally-recorded session against the server API).
- **Service worker** — `client/static/service-worker.js`: network-first for
  navigations, cache-first for immutable assets, never caches `/api/`.

The shape is sound. The holes are in the details.

---

## 🔴 Critical

### 1. `makeLocalNumericId()` overflows `MAX_SAFE_INTEGER` → ID collisions

`client/src/lib/today/controller/utils.ts:13-16`

```ts
export function makeLocalNumericId(): number {
	const rand = Math.floor(Math.random() * 1_000_000);
	return -(Date.now() * 1_000_000 + rand);
}
```

`Date.now() * 1_000_000` ≈ `1.78e18`, far beyond `Number.MAX_SAFE_INTEGER`
(`9.0e15`). The `rand` component is **entirely lost to floating-point rounding**
(generated IDs end in `00`). Measured behaviour:

```
collisions in 200k rapid calls: 91686 (≈46%)
Number.isSafeInteger(id): false
```

**Reachable today:** offline demo-mode seeding loops `addExercise`/`addSet`
within microseconds of each other (`actions/session.ts:146-168`,
`actions/exercise/core.ts:164-175`), so colliding IDs are likely. Because
`updateSet`/`removeSet` operate **by id** (`actions/exercise/sets.ts:238-246`),
editing or deleting one set silently affects its colliding twin.

A collision-safe generator already exists — `client/src/lib/utils/id.ts`
(`createId`, string-based) — but the offline numeric path doesn't use it.

**Fix sketch:** keep IDs as safe negative integers (e.g. a module-level
decrementing counter seeded from `Date.now()`), or move local IDs to strings via
`createId` and adjust the `id < 0` sentinel checks accordingly.

### 2. `syncOne` is not idempotent across partial failures → duplicate workouts

`client/src/lib/today/controller/offline.ts:157-266`

When a local (negative-id) session syncs, `createWorkout` returns a new id and
the exercise map is mutated **in memory**, but the record is only persisted at
the very end (line 259). If any `createExercise` / `replaceSets` /
`endExercise` throws mid-loop — likely, since we *just* regained connectivity —
the function exits and `server_workout_id` + `server_exercise_ids_by_local` are
never saved. The next sync re-enters with `workoutId` undefined →
**`createWorkout` runs again** → duplicate workout on the server, plus duplicate
exercises for everything created before the failure point. There is no
server-side dedup key.

**Fix sketch:** persist the record immediately after `createWorkout` succeeds,
and again after each exercise is created/mapped, so a retry resumes instead of
restarting. Alternatively, give offline-created entities a client-generated
idempotency key the server can dedupe on.

### 3. Offline edits to a *server-started* session are silently dropped on refresh

`client/src/lib/today/controller/offline.ts:51-55`,
`client/src/lib/today/controller/actions/backend.ts:29`,
`client/src/lib/today/controller/actions/sync.ts:77-79`

Flow: start a session online (positive id) → go offline → log sets (saved to
IndexedDB with `status: in_progress` and `server_workout_id` set).

On reconnect, `refreshPendingSyncCount` does **not** count this record (it
requires `id < 0`), so the `online` handler does nothing. Meanwhile
`refreshFromBackend` overwrites `currentSession` with the **server's** copy,
which lacks the offline edits. Because SvelteKit unmounts/remounts the page on
route change, simply switching to another tab and back re-runs
`onMount → start → refreshFromBackend` (`src/routes/+page.svelte:62-65`). The
offline sets vanish from the UI and are never pushed to the server.

**Fix sketch:** count in-progress records with a `server_workout_id` as pending
and sync them on reconnect, OR merge offline edits into the server copy in
`refreshFromBackend` instead of overwriting.

---

## 🟠 Significant

### 4. No sync mutex → concurrent `syncPendingSessions` can double-submit

`client/src/lib/today/controller/actions/sync.ts:34-70`

`syncPendingSessions` has no in-flight guard. Flaky connectivity fires
`online`/`offline`/`online` in quick succession; each `online` invokes it
(line 78). Two concurrent runs both call `listOfflineSessions()` and both
`createWorkout` the same negative-id record → duplicate. The "Sync now" button
is gated by `loading`, but the automatic trigger races against itself.

**Fix sketch:** a module-level `isSyncing` boolean (or a shared promise) that
short-circuits re-entrant calls.

### 5. `isNetworkFailure` swallows real bugs as "offline"

`client/src/lib/today/controller/utils.ts:6-11`

```ts
if (e instanceof TypeError) return true;
```

**Any** `TypeError` is treated as a network failure. A genuine bug (an undefined
access during response parsing, a `TypeError` thrown by app code) is
misclassified → the app flips into offline mode and persists locally instead of
surfacing the error. This hides real failures and can strand data in a
"pending" state the server never actually rejected.

**Fix sketch:** narrow the match to fetch-specific failures (message regex +
`navigator.onLine === false`), and let other `TypeError`s propagate as errors.

### 6. `today.plannedTemplate` is never cleared on logout → cross-user residue

`client/src/lib/auth/index.ts:27-84`,
`client/src/lib/today/controller/actions/plannedTemplate.ts`

`clearClientSensitiveData` filters keys by `offline.today.session.`,
`offline.today.recentSessions`, `currentWorkoutId`, and
`swolemate:currentWorkoutState`. The planned-template key
(`today.plannedTemplate`, holding planned exercise names + notes) matches
**none** of these, so it survives logout. On a shared device, the next user sees
the previous user's planned-template residue. User-id scoping mitigates *read*
collisions only while `activeUserId` is correct; the data is never purged.

**Fix sketch:** add `today.plannedTemplate` to the logout key filter (both the
localStorage sweep and the IndexedDB `kvListKeys` sweep).

---

## 🟡 Minor / cleanup

### 7. Dead code masquerading as the offline layer

- `client/src/lib/offlineCache.ts` (47 lines) — **zero references** anywhere in
  `src` (verified by grep). Anyone auditing "the offline cache" lands here first
  and reads the wrong system.
- `client/src/lib/workoutState.ts` (~140 lines) — fully dead except
  `clearWorkoutState()`, which is called on logout to clear a key **nothing
  writes anymore**. The `replaceWorkoutId/ExerciseId/SetId` machinery (the *old*
  offline reconciliation) is orphaned.

**Fix sketch:** delete both modules and the now-pointless `clearWorkoutState()`
call.

### 8. Confused logout-clearing of recent sessions

`client/src/lib/auth/index.ts:33`

```ts
localStorage.removeItem('offline.today.recentSessions');
```

This removes an **unscoped localStorage** key, but recent sessions are written
via `kvSet(scopedKey(...))` into **IndexedDB**. The line is a no-op; the real
cleanup happens later in the IndexedDB sweep (lines 59-67). Harmless but
misleading.

### 9. `precacheAppShell` silently no-ops if `/` isn't the real shell

`client/static/service-worker.js:16-26`

The install step regex-scrapes `/_app/immutable/` URLs from `fetch('/')`. If
that fetch is intercepted, errors, or returns the offline page during install,
no app-shell assets are precached and the PWA won't boot offline — with only a
`console.warn`. Worth a fallback or an explicit failure signal.

---

## 🟠 Long-term offline availability ("opens weeks later")

Separate concern from sync correctness: ensuring the app still cold-launches
offline after long periods of disuse. Two independent risks.

### 10. Cache/IndexedDB can be evicted → blank app weeks later

By default, Cache Storage and IndexedDB are **best-effort** storage and may be
evicted under pressure. Critically, **iOS Safari** caps script-writable storage
at **7 days of no use** for sites that are *not* installed to the Home Screen
(ITP). Open the app in a tab, ignore it for a week, and the cache is wiped — the
most likely "it was blank when I came back" scenario. Installed PWAs are exempt
from the 7-day rule on iOS 16.4+, but can still be evicted under disk pressure on
any platform.

The app currently never requests persistent storage
(`src/routes/+layout.svelte:44-56` registers the SW but calls no
`navigator.storage.persist()`), so it is always in the best-effort tier.

**Fix sketch:** request persistent storage once after SW registration, and nudge
users to install to the Home Screen on iOS (required for any of this to survive a
week).

```ts
if (navigator.storage?.persist && !(await navigator.storage.persisted())) {
	const granted = await navigator.storage.persist();
	logger.debug('pwa', 'persistent storage', { granted });
}
```

### 11. Precache holds only what was visited → routes blank offline

`client/static/service-worker.js:16-26` (`precacheAppShell`) regex-scrapes `/`'s
HTML for `/_app/immutable/...` URLs at install. Two gaps:

1. If install runs while connectivity is flaky (or `/` returns an unexpected
   body), it silently caches nothing and the PWA cannot boot offline (same root
   cause as #9).
2. The cache-first handler for `/_app/immutable/` only stores chunks **actually
   requested during a session** (`service-worker.js:68-79`). Lazy route chunks
   for pages the user never opened (e.g. Progress, when they only used Today) are
   never cached, so those routes are blank offline weeks later.

**Fix sketch:** precache the full build manifest deterministically instead of
scraping HTML. SvelteKit's `$service-worker` virtual module exposes the complete
list — but that requires moving the worker from `static/service-worker.js` to
`src/service-worker.ts` so Vite processes it:

```ts
import { build, files, version } from '$service-worker';
const CACHE = `swolemate-${version}`;
const PRECACHE = [...build, ...files]; // every chunk + static asset, all routes
self.addEventListener('install', (e) => {
	e.waitUntil(
		caches.open(CACHE).then((c) => c.addAll(PRECACHE)).then(() => self.skipWaiting())
	);
});
```

`build` covers every hashed JS/CSS chunk for **all** routes, `files` covers
`static/`, and `version` cache-busts on deploy automatically. The existing fetch
strategy (network-first navigations, cache-first immutable, skip `/api/`) can be
kept as-is.

**Verification:** DevTools → Application → Storage should report **Persistent**
(not best-effort), and Cache Storage should contain every `_app/immutable/*`
chunk *before* visiting those routes. Hard test: install to Home Screen →
airplane mode → cold-launch; then repeat after a week.

---

## Suggested priority

The data-integrity trio (**#1, #2, #3**) should come first — they cause silent
data loss or server-side duplication, the two worst outcomes for a workout
logger.

- **#1** is essentially a one-line fix (collision-safe ID generation).
- **#2** needs `server_workout_id` / exercise-map persistence immediately after
  each successful create, so retries resume rather than restart.
- **#3** needs either counting in-progress server sessions as pending or merging
  (rather than overwriting) on refresh.

**#4–#6** are correctness/privacy follow-ups. **#7–#9** are cleanup that will
make the next audit of this layer much faster. **#10–#11** govern long-term
offline availability — **#10** (`navigator.storage.persist()`) is the single
biggest lever for surviving weeks of disuse and is ~10 lines; **#11** is the
proper fix for #9 and guarantees every route is cached, not just visited ones.

### Test-coverage gaps observed

Existing tests (`src/test/today-controller-sync-actions.test.ts`,
`today-offline.test.ts`, `offline-storage.test.ts`) cover the happy path. None
exercise: partial sync failure + retry (#2), concurrent sync (#4), offline edits
to a server-started session followed by refresh (#3), or rapid-fire local ID
generation (#1). Each fix above should land with a regression test for its
scenario.
