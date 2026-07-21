# API seam (api.ts backend contract)

**3 confirmed (1 medium, 2 low—unverified), 0 refuted.**

## Confirmed findings

### MEDIUM [data-integrity]: No request timeout/abort in api.ts wedges the reconnect sync loop and stalls writes

- **Trigger:** Online event fires after connectivity loss (subway exit, elevator). Browser detects connection but socket is half-open/stalled for tens of seconds. `syncPendingSessions` starts and the first sync fetch stalls indefinitely.
- **Location:** `client/src/lib/api.ts:97-107` (and all request sites)
- **What happens:** Every request in api.ts is a raw `fetch()` with no `AbortController` or timeout. The `withCredentials` helper (lines 53–55) only sets credentials; no signal is attached. While a hung fetch blocks, `state.loading` stays true (spinner stuck), and the module-level `isSyncing` guard (sync.ts:31, 41–42) prevents any subsequent `online` event from re-attempting sync. In the pathological case, if the socket never errors, `isSyncing` is never reset in the finally block (sync.ts:77) and auto-sync silently dies for the page's lifetime, stranding all logged data as 'pending'.
- **Why:** Silently strands a user's logged workout as unsynced and blocks all further auto-sync with no error surfaced, defeating the offline-first guarantee that changes "will sync later."
- **Fix sketch:** Attach an `AbortController` with a sane timeout (e.g., 15–30s) to each fetch via a wrapper in api.ts, and translate abort into a network-failure-shaped error so `isNetworkFailure()` classifies it as offline. This lets `syncOne` fail fast, reset `isSyncing`, and re-arm on the next reconnect.

### LOW [data-integrity]: handleResponse treats any 2xx with a non-JSON body as a successful void write (unverified)

- **Trigger:** Reconnect/sync path, online event: a same-origin proxy in front of the Rust backend returns HTTP 200 with an HTML body for `POST /api/workouts/{id}/end` during `syncOne`.
- **Location:** `client/src/lib/api.ts:81-90`
- **What happens:** On a 2xx response with non-`application/json` content-type, `handleResponse` returns `undefined` and treats the call as a successful no-payload write (lines 82–85), never throwing. For void endpoints in the reconnect replay, `syncOne` calls `api.endWorkout` (client/src/lib/today/controller/offline.ts:270) and unconditionally calls `deleteOfflineSession(record.key)` at line 275. If a same-origin proxy answers with 2xx text/html or text/plain instead of backend JSON, the workout finalization (end_time/mood/feedback) is never persisted yet the offline record is deleted, so it can never be retried.
- **Why:** A response that never reached the backend is accepted as a completed write, and the local copy that would allow retry is deleted, causing recoverable-but-real loss of workout finalization data.
- **Fix sketch:** For endpoints that expect a body, require `application/json` on success and throw an `ApiError` on unexpected content-type instead of returning `undefined`. Reserve the `undefined` path for 204 responses only, or have `syncOne` verify the write before deleting the offline record.

### LOW [data-integrity]: Remote logger drops the in-flight batch and permanently disables logging on 401/403 (unverified)

- **Trigger:** Online event: a warn/error is queued, the session cookie momentarily lapses so `POST /api/logs` returns 401. The batch is dropped and remote logging disables for the session.
- **Location:** `client/src/lib/logger.ts:111-119`
- **What happens:** `processLogQueue` drains the queue into a local `logs` array (lines 98–99) before POSTing. On a 401/403 it sets `remoteEnabled=false` and returns (lines 112–115) without unshifting `logs` back onto the queue, so that batch of warn/error diagnostics is silently discarded. Remote logging stays off until `setRemoteEnabled(true)` is called again (typically only on next auth bootstrap). A transient 401 during token refresh therefore both loses the current diagnostics and blinds remote logging for the rest of the session.
- **Why:** Silent loss of diagnostic logs plus a sticky disable makes post-incident debugging of auth/network failures unreliable. (Low impact: no user-facing workout data affected, diagnostics only.)
- **Fix sketch:** On 401/403, unshift the drained `logs` back onto the queue (as the generic-failure branch already does) before disabling, and/or re-enable remote logging automatically once auth is re-established.

## Refuted (not real / already handled)

(None)
