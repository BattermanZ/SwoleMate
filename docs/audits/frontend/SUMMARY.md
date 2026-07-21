# SwoleMate Frontend Audit — Launch-Readiness Rollup

Adversarially-verified data-integrity + client-security audit of the SvelteKit (Svelte 5 runes, offline-first PWA) frontend, 8 subsystems. Each finding was severity-gated and verified by a diverse-lens panel (code-truth / reachability / already-mitigated). **Low findings are unverified** (panel did not vote) — treat as leads. The real authorization gate is the backend; client-guard issues here are data-leak/UX, not enforcement.

**Tally:** 1 critical · 4 high · 9 medium · 8 low (unverified). Dominant risk: **data loss in the offline workout-session engine** — nearly every serious finding ends in *silently losing or duplicating a user's logged sets*. XSS (cat 06) came back clean.

## Confirmed findings by severity

| Severity | Type | Category | Title | Location |
|---|---|---|---|---|
| CRITICAL | data-integrity | 05 auth-guard | A single 401 mid-workout wipes ALL unsynced logged sets (cookie expiry / redeploy triggers it) | `auth/index.ts:129-134,26-66` |
| HIGH | data-integrity | 01 session-engine | Online reducers capture a stale session snapshot across an `await` and `.set()` the whole store, discarding concurrent mutations | `today/controller/actions/*` |
| HIGH | data-integrity | 02 offline-persist | Online end-session that fails on the network loses mood/notes and resurrects the workout (un-ended forever) | `today/controller` / `offline` |
| HIGH | data-integrity | 03 sync-reconnect | No idempotency key on createWorkout/createExercise — a lost HTTP response duplicates the workout + its sets on replay | `today/controller/sync.ts` |
| HIGH | data-integrity | 05 auth-guard | User-switch / clearClientSensitiveData destroys OTHER users' unsynced offline sessions on a shared device | `auth/index.ts` |
| MEDIUM | data-integrity | 01 session-engine | Network failure on final endWorkout drops entered mood/notes, leaves workout un-ended | `today/controller` |
| MEDIUM | data-integrity | 02 offline-persist | One unparseable/old-shape record aborts syncing of ALL pending sessions | `offline/todaySessions.ts` |
| MEDIUM | data-integrity | 02 offline-persist | Read-modify-write race in persistInProgressSession drops merge-only fields (deletions / id map) | `offline/storage.ts` |
| MEDIUM | data-integrity | 02 offline-persist | No quota/IndexedDB error handling — a failed persist silently discards the just-logged set | `offline/storage.ts` |
| MEDIUM | data-integrity | 03 sync-reconnect | refreshFromBackend deletes an offline record with unsynced edits when the server session was completed elsewhere | `today/controller/sync.ts` |
| MEDIUM | data-integrity | 03 sync-reconnect | submitEndSession online path: mid-flight failure in endExercise Promise.all loses mood/feedback, leaves un-ended | `today/controller` |
| MEDIUM | data-integrity | 04 api-seam | No request timeout/abort in api.ts wedges the reconnect sync loop and stalls writes | `api.ts` |
| MEDIUM | security | 05 auth-guard | Protected routes render cached data during 'unknown' auth status (flash-of-content data leak) | `routes/+layout.svelte` |
| MEDIUM | data-integrity | 08 sw-pwa | skipWaiting + activate cache-purge forces a mid-session full reload / broken nav after a deploy | `svelte.config.js` / SW |

### Low (unverified — leads only)
- 01 split-weight toggle rewrites logged weight to max(left,right), changing recorded set/volume
- 02 scopedKey falls back to an unscoped key when user id unknown, orphaning offline sessions
- 04 handleResponse treats any 2xx non-JSON body as a successful void write; remote logger drops in-flight batch + disables logging on 401/403
- 05 must_change_password redirect overridden by the layout's authenticated→'/' effect
- 07 stale `auth.lastUser` briefly leaks previous user's identity + cached workouts on shared device; startup log captures full URL + userAgent
- 08 SW cache-exclusion only covers `/api/`; same-origin GET to `/oauth`, `/mcp`, `/.well-known` falls through to the shared cache

## Top launch blockers

1. **[CRITICAL · 05] 401 mid-workout silently wipes every unsynced set.** The global unauthorized handler calls `clearClientSensitiveData()`, which unconditionally deletes all in-progress offline session records. A *routine, non-adversarial* 401 (backend session TTL lapse, redeploy, transient error) during a live workout therefore destroys everything logged-but-not-yet-synced, then bounces to `/login`. **Fix:** on 401, preserve `pending_sync`/in-progress records — only clear read-only cache + session metadata; gate the full wipe behind explicit logout.
2. **[HIGH · 05] User-switch destroys other users' offline sessions** on a shared device (same wipe, wrong scope).
3. **[HIGH · 03] No idempotency key on reconnect replay** → a dropped HTTP response duplicates the whole workout and its sets. Needs a client-generated idempotency key honored by the backend.
4. **[HIGH · 01] Stale-snapshot `.set()` overwrite** discards mutations that landed during an in-flight await — the session engine needs to merge/re-read rather than blind-overwrite.
5. **[HIGH · 02 + MEDIUM 01/03] End-session network failure loses mood/notes and resurrects the workout** — a recurring pattern across three categories; the end-session flow needs atomic, retryable completion that preserves entered data.

**Cross-cutting fix:** items 1, 2, 4, 5 all stem from the offline engine treating in-memory/persisted session state as disposable on any auth or network hiccup. The durable fix is to make unsynced session records sacrosanct — never deleted by auth transitions, never overwritten by a stale snapshot, always replayable idempotently.
