# Client auth / scope / route guards

**Summary:** 4 confirmed (1 critical, 1 high, 1 medium, 1 unverified low). 0 refuted.

## Confirmed findings

### CRITICAL [data-integrity]: A single 401 during an in-progress workout silently wipes all unsynced logged sets

- **Trigger:** Online, mid-workout. User logs several sets over a long session; backend session TTL lapses or backend restarts. Next automatic API call the controller makes returns 401. In-progress workout record is deleted from IndexedDB/localStorage and user is bounced to /login. Sets are gone.
- **Location:** `client/src/lib/auth/index.ts:129-134, 26-66`
- **What happens:** The global unauthorized handler calls clearClientSensitiveData(), which unconditionally deletes every offline in-progress session record from both localStorage and IndexedDB (kvListKeys('') filtered by substrings 'offline.today.session.', 'currentWorkoutId', 'swolemate:currentWorkoutState' at lines 45-47 and 55-62). This handler runs for ANY 401 via api.ts:73 (unauthorizedHandler?.() in handleResponse). During a live workout the today controller makes ordinary authenticated API calls (loadEstimated1RmBaseline / loadLastTimeForExercise fire per active exercise via +page.svelte:67-74, plus reconnect syncs). If the backend session cookie expires mid-session, the server restarts, or returns a transient/spurious 401, that call resolves to 401 → handleUnauthorized → clearClientSensitiveData deletes the persisted in-progress session, and the layout effect (+layout.svelte:26-33) navigates to /login, destroying the in-memory currentSession store too. Every set logged in the current session not yet persisted server-side is lost silently and unrecoverably.
- **Why:** Rubric's critical case: silent loss of a user's logged workout. 401 mid-session is a routine non-adversarial event (cookie expiry / redeploy), and the wipe is total and unrecoverable.
- **Fix sketch:** On 401, do NOT delete in-progress / pending_sync offline session records; only clear cached read-only data and session metadata, preserving unsynced workout records for post-login replay. Or gate the wipe behind explicit logout and on 401 just flip auth status + redirect while keeping the offline session.

### HIGH [data-integrity]: clearClientSensitiveData / user-switch destroys OTHER users' unsynced offline sessions on a shared device

- **Trigger:** Shared device. User A finishes a workout offline (record status pending_sync, never synced). Without A syncing, user B signs in; login() sees previousUserId=A ≠ B and calls clearClientSensitiveData, which substring-matches and deletes A's u{A}:offline.today.session.* record. A's workout is lost.
- **Location:** `client/src/lib/auth/index.ts:53-63, 166-169`
- **What happens:** Offline session records use per-user scoped keys u{id}:offline.today.session.{n} (scope.ts:30-34, todaySessions.ts:22-24), but clearClientSensitiveData deletes by unscoped substring: it lists ALL keys with kvListKeys('') and removes any whose name .includes('offline.today.session.') etc (lines 55-62), same for localStorage (37-47). Since the u{id}: prefix still contains those substrings, the clear wipes every user's records, not just the active user's. It runs on logout, on 401, and on user switch (login()/refresh() call it when previousUserId differs, lines 142-144 and 166-169) BEFORE any sync attempt. So if user A ended a workout while offline (pending_sync record never uploaded) and user B then logs in on the same browser, A's completed-but-unsynced workout is deleted permanently.
- **Why:** The app deliberately scopes offline data per user for shared devices, so cross-user destruction on switch/logout is a real supported scenario. Deleting unsynced completed workouts with no upload attempt is silent data loss.
- **Fix sketch:** Scope the deletion to the active user: derive the filter from scopedKey(prefix)/getActiveUserId() so only u{activeId}: keys are removed, or match exact scoped prefixes rather than a bare substring. Also attempt to sync pending_sync records before clearing on user switch.

### MEDIUM [security]: Protected routes render cached data during 'unknown' auth status (flash-of-content data leak)

- **Trigger:** Any cold load of a protected route (/workouts, /progress) with a stale auth.lastUser in localStorage: cached content flashes before validation. Sharper on a shared/borrowed device where the server session is invalid but localStorage still holds the prior user's cached data.
- **Location:** `client/src/routes/+layout.svelte:26-33, 147-149`
- **What happens:** The auth store initializes status='unknown' with the user restored from localStorage (auth/index.ts:120-127). The layout only redirects to /login when status==='unauthenticated' (line 27); for status==='unknown' it renders protected children immediately (else branch, lines 147-149). auth.refresh() validating the cookie fires in onMount and resolves asynchronously, so on every cold load the last user's cached workout history/progress paints before authMe confirms the session. If the cookie is expired or belongs to another account, the prior user's logged data is visible until the 401 clears it. On a network failure the offline path keeps status at 'unknown' (never 'unauthenticated', auth/index.ts:149-154), so a revoked-but-offline user keeps seeing cached data with no redirect.
- **Why:** Backend is the real authz gate, so this is data-leak/UX, not enforcement (medium). But it surfaces one user's stored/cached workout data on-screen before authorization is confirmed.
- **Fix sketch:** Render a neutral loading/splash while status==='unknown' instead of protected children, or gate the else-branch on status==='authenticated'. Reveal cached content only after refresh() confirms auth (or explicit offline-with-known-user).

### LOW [security]: must_change_password redirect is overridden by the layout's authenticated->'/' effect (unverified)

- **Trigger:** Admin creates a user with must_change_password=true. User logs in; instead of routing to /settings to change the temporary password, they land on home.
- **Location:** `client/src/routes/login/+page.svelte:17-20`
- **What happens:** On successful login, submit() sets auth state authenticated then navigates: goto(mustChange ? '/settings' : '/') (lines 17-20). But the layout has a reactive effect that fires when status becomes authenticated while on /login and calls goto('/') (+layout.svelte:30-32). Since isLogin is $derived from pathname (updates only after navigation resolves), the effect flush right after login() sees isLogin still true and issues goto('/'), the later of the competing navigations, which wins. A user flagged must_change_password is dropped on '/' instead of /settings where they'd change the password.
- **Why:** Low: backend still enforces the password-change requirement on sensitive actions, so this is a UX/routing defect (weakened forced-password-change prompt), not an enforcement bypass.
- **Fix sketch:** Have the layout effect respect must_change_password (redirect to /settings when set), or remove the client's authenticated->'/' auto-redirect on /login and let the login page own post-login routing.

## Refuted (not real / already handled)

None.
