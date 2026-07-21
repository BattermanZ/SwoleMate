# Audit Remediation Log

Source of truth for resuming remediation work if a session is interrupted.
Statuses: ⬜ todo · 🟡 in-progress · ✅ done · ⏭️ skipped (with reason)

Update this file **after every implemented fix** (before committing) so on-disk
state always reflects reality. Each done item records the commit that shipped it.

---

## Batch 1 — Frontend data-loss blockers

| St | ID | Title | Location | Commit |
|----|----|-------|----------|--------|
| ✅ | F-CRIT-1 | 401 mid-workout wipes ALL unsynced sets | `auth/index.ts:129-134,26-66` | pending |
| ✅ | F-HIGH-4 | User-switch destroys other users' offline sessions | `auth/index.ts` | pending |
| ✅ | F-HIGH-1 | Stale session snapshot across `await`, blind `.set()` | `today/controller/actions/*` | pending |
| ✅ | F-HIGH-2 | Online end-session network failure loses mood/notes, resurrects workout | `today/controller` / `offline` | pending |
| ✅ | F-MED-1 | Network failure on final endWorkout drops mood/notes | `today/controller` | pending |
| ✅ | F-MED-6 | submitEndSession online mid-flight failure loses mood/feedback | `today/controller` | pending |

## Batch 2 — Idempotent sync

| St | ID | Title | Location | Commit |
|----|----|-------|----------|--------|
| ✅ | F-HIGH-3 | No idempotency key on createWorkout/createExercise | `today/controller/sync.ts` + backend | pending |
| ✅ | F-MED-5 | refreshFromBackend deletes offline record with unsynced edits | `today/controller/sync.ts` | pending |
| ✅ | F-MED-2 | One unparseable record aborts all pending sync | `offline/todaySessions.ts` | pending |
| ✅ | F-MED-3 | Read-modify-write race in persistInProgressSession | `offline/storage.ts` | pending |
| ✅ | F-MED-4 | No quota/IndexedDB error handling | `offline/storage.ts` | pending |
| ✅ | F-MED-7 | No request timeout/abort in api.ts | `api.ts` | pending |

## Batch 3 — Backend auth / DoS blockers + hardening

| St | ID | Title | Location | Commit |
|----|----|-------|----------|--------|
| ✅ | B-HIGH-1 | Permanent account lockout DoS | `routes/auth.rs:58-64,69`; `db/auth.rs:245-260` | pending |
| ✅ | B-MED-7 | disable_user lacks last-admin guard; no re-enable route | `routes/admin.rs:68-80` | pending |
| ✅ | B-MED-4 | Password change doesn't revoke MCP/OAuth tokens | `routes/auth.rs:150-152` | pending |
| ✅ | B-MED-5 | MCP rate limiter keyed per-token not per-user | `mcp/routes.rs:825` | pending |
| ✅ | B-MED-3 | MCP auth middleware unrated DB reads/writes | `middleware/mcp_auth.rs:99-133` | pending |
| ✅ | B-MED-6 | Panic on non-char-boundary byte slice of metadata | `routes.rs:313` | pending |
| ✅ | B-MED-1 | Username enumeration via divergent login responses | `routes/auth.rs:44-74` | pending |
| ✅ | B-MED-2 | Refresh-token rotation no reuse-detection / family revocation | `oauth/routes.rs:778-830` | pending |

## Batch 4 — Backend correctness / availability

| St | ID | Title | Location | Commit |
|----|----|-------|----------|--------|
| ✅ | B-MED-8 | Failed backup restore leaves DB pool permanently closed | `routes.rs:438-449,526-530` | pending |
| ✅ | B-MED-9 | Unbounded N+1 query fan-out in progress/exercise reads | `db/progress.rs:376-399` | pending |
| ✅ | B-MED-10 | Progress overview loads entire history into memory | `db/progress.rs:968-1022` | pending |
| ✅ | B-MED-11 | get_workout_stats counts never-ended workouts as zero-duration | `db/progress.rs:411-522` | pending |

## Batch 5 — Frontend medium + client hardening

| St | ID | Title | Location | Commit |
|----|----|-------|----------|--------|
| ✅ | F-MED-8 | Protected routes render cached data during 'unknown' auth | `routes/+layout.svelte` | pending |
| ✅ | F-MED-9 | skipWaiting cache-purge forces mid-session reload | `svelte.config.js` / SW | pending |

## Batch 6 — Low-severity hardening (unverified leads — confirm then fix)

| St | ID | Title | Location | Commit |
|----|----|-------|----------|--------|
| ✅ | B-LOW-1 | Login endpoint exempt from CSRF origin check | `middleware/session_auth.rs:86-97` | pending |
| ✅ | B-LOW-2 | Auth-code replay doesn't revoke already-issued tokens | `oauth/routes.rs:643-683` | pending |
| ✅ | B-LOW-3 | Username enumeration in authorize_post | `oauth/routes.rs:462-494` | pending |
| ✅ | B-LOW-4 | request_ip trusts X-Real-IP unconditionally | `auth/rate_limit.rs:26` | pending |
| ✅ | B-LOW-5 | Log injection via unsanitized timestamp field | `routes.rs:302` | pending |
| ✅ | B-LOW-6 | Internal error exposure toggle by mere env-var presence | `errors.rs:6` | pending |
| ✅ | B-LOW-7 | Unbounded set array in replace_sets | `routes.rs:229` | pending |
| ✅ | B-LOW-8 | Auth response builder blindly appends `?code=` | `oauth/routes.rs:549-555` | pending |
| ✅ | B-LOW-9 | Token rotation copies original absolute expires_at | `db/mcp_tokens.rs:172-206` | pending |
| ✅ | B-LOW-10 | Login limiter check-then-record non-atomic | `routes/auth.rs:37` | pending |
| ✅ | B-LOW-11 | Backups written non-atomically | `backup.rs:94` | pending |
| ✅ | B-LOW-12 | Restore extraction would follow symlink/hardlink entries | `backup.rs:250-265` | pending |
| ✅ | B-LOW-13 | TOCTOU on single-active-session check | `db/templates.rs:396-447` | pending |
| ✅ | B-LOW-14 | PR detection case-sensitive vs case-insensitive volume | `db/progress.rs:95-99` | pending |
| ✅ | F-LOW-1 | split-weight toggle rewrites logged weight to max(l,r) | frontend | pending |
| ✅ | F-LOW-2 | scopedKey falls back to unscoped key when user id unknown | `offline` | pending |
| ✅ | F-LOW-3 | handleResponse treats 2xx non-JSON as success; logger drops batch on 401/403 | `api.ts` / logger | pending |
| ✅ | F-LOW-4 | must_change_password redirect overridden by layout effect | `+layout.svelte` | pending |
| ✅ | F-LOW-5 | stale auth.lastUser leaks previous user; startup log captures URL+UA | `auth` | pending |
| ✅ | F-LOW-6 | SW cache-exclusion only covers /api/ | SW | pending |

---

## Change log (append one line per shipped fix)
```
F-CRIT-1 + F-HIGH-4: clearClientSensitiveData now preserves unsynced offline
  sessions on 401 (preserveWorkoutSessions) and scopes deletion to the departing
  user on account switch (userId). Full wipe only on explicit logout. Tests:
  auth-preserve-offline-session.test.ts.
F-HIGH-1: online reducers (addExercise/removeExercise/addSet/markExerciseDone)
  now merge into the live store via currentSession.update() instead of .set()-ing
  a pre-await snapshot, so concurrent mid-await edits survive. Test:
  today-controller-concurrent-edit.test.ts.
F-HIGH-2 + F-MED-1 + F-MED-6: submitEndSession's network-failure catch now
  persists a pending_sync end record (mood/notes/endedAt via shared persistPendingEnd)
  and clears the UI, so a drop mid-end no longer loses mood/notes or leaves the
  workout un-ended — reconnect's syncOne endWorkout branch fires. Test added to
  today-controller-session-actions.test.ts.
F-MED-2: syncPendingSessions wraps each syncOne in its own try/catch — a corrupt
  record is skipped (surfaced via error count) instead of blocking the whole queue;
  network failures still stop the run and flip offline.
F-MED-5: refreshFromBackend no longer deletes an offline in_progress record whose
  server workout was completed on another device — it keeps the record (still
  pending) and shows a conflict notice, so unsynced edits replay on reconnect.
  Tests in today-controller-sync-actions.test.ts.
F-MED-3: persistInProgressSession now runs through a single-flight promise chain
  and accepts a functional `extra` so merge-only fields (deleted ids) are derived
  inside the critical section — overlapping persists can't clobber a deletion.
F-MED-4: kvSet wraps IDB/localStorage writes and throws a typed StorageWriteError
  (quota-aware); withStore also rejects on tx abort so a quota abort can't hang.
  Tests in today-persist-race.test.ts.
F-MED-7: withCredentials now attaches AbortSignal.timeout(30s) to every request
  (unless the caller supplies a signal), so a dead socket can't hang the reconnect
  sync loop. isNetworkFailure (today + auth) now classifies TimeoutError/AbortError
  as offline so timed-out writes are queued and retried. Test in today-network-failure.
F-HIGH-3: end-to-end idempotency for offline-sync create replay. Backend: schema
  v14 idempotency_keys table, db/idempotency.rs (lookup/record with race cleanup),
  create_workout/create_exercise services dedup on an Idempotency-Key header.
  Client: syncOne sends stable keys (w:<sessionId> / e:<exerciseId>) so a lost
  response no longer duplicates the workout/exercise on reconnect. Tests: backend
  api.rs (2) + client today-offline-syncone.test.ts.
B-HIGH-1 + B-MED-1: login no longer hard-blocks on account lock state. The handler
  always runs an argon2 verify + fixed 200ms delay on failure and returns a uniform
  401 for unknown/disabled/wrong-password/previously-locked accounts, and a correct
  password for an enabled account always succeeds — so bad-password floods can't
  freeze an account (DoS) and the response no longer leaks account existence. Brute
  force stays bounded by the per-IP limiter. Tests: repeated_failed_logins_do_not_
  lock_the_owner_out, login_does_not_leak_account_existence_via_status.
B-MED-7: disable_user now rejects disabling the last active admin (409, mirroring
  delete_user) and a new POST /api/admin/users/{id}/enable route (db.enable_user
  clears disabled_at + lockout) makes the state recoverable via the API. Test:
  admin_cannot_disable_last_admin_and_can_re_enable_users.
B-MED-4: change_password (and admin reset_user_password) now revoke the user's live
  MCP tokens and OAuth access/refresh tokens (new db.revoke_all_mcp_tokens_for_user
  + revoke_all_oauth_tokens_for_user) so a leaked bearer token can't outlive the
  credential rotation. Test: changing_password_revokes_mcp_tokens.
B-MED-5: MCP per-minute limiter now keys on user:{user_id} alone, so a user can no
  longer multiply their budget by minting extra personal tokens / OAuth clients.
B-MED-3: McpBearerAuth applies a per-IP pre-auth throttle (MCP_AUTH_RATE_LIMIT_PER_
  MINUTE, default 120) BEFORE any token DB lookup, and touch_mcp_token_last_used is
  debounced to at most once/5min per token — so a bogus- or valid-token flood can't
  amplify into unbounded SQLite reads/writes. Added reset_rate_limit_state() test
  hook. Tests: mcp_auth_is_rate_limited_per_ip_before_db_lookup (+ full mcp suite).
B-MED-6 + B-LOW-5: write_logs now floors the oversized-metadata truncation to a UTF-8
  char boundary (no more mid-character byte-slice panic / worker-abort DoS) and runs
  the client timestamp through sanitize_log_field (no newline log-forgery). Test:
  logs_endpoints_work_and_enforce_limits.
B-MED-2: OAuth refresh tokens now carry a family_id (schema v15) threaded through
  issuance and rotation. Presenting an already-rotated (revoked, non-expired)
  refresh token triggers reuse detection: revoke_oauth_token_family kills every
  access+refresh token in the lineage, so a stolen-token replay severs the thief's
  descendant tokens. Test: refresh_token_reuse_revokes_the_whole_family.
B-MED-8: restore_backup now validates the archive (backup::validate_backup_archive:
  openable + contains a regular-file database.db) BEFORE closing the live pool, and
  ALWAYS rebuilds+swaps the pool afterward (rebuild_sqlite_pool) even when the
  restore fails — so a corrupt/missing archive or a failed restore can't leave the
  app with a permanently closed pool. Tests updated (400 not 500) + corrupt-restore
  stays-alive test.
B-MED-11: get_workout_stats's workout_times CTE now filters
  `WHERE user_id = ? AND end_time > start_time`, so in-progress / abandoned
  workouts (end_time == start_time, zero duration) no longer count toward
  total_workouts, average duration, or the 0-30min duration bucket — which they
  previously deflated. sqlx offline cache regenerated. Test:
  workout_stats_exclude_in_progress_zero_duration_workouts.
B-MED-9: get_exercise_progress (2N+1 queries) and get_exercises_for_workout
  (N+1 queries) fanned out one sets/settings query per exercise row, so a user
  with a long history of an exercise type issued hundreds of round-trips per
  progress load. Added batched get_sets_for_exercises / get_settings_for_exercises
  (runtime sqlx::query_as with an IN(...) list, grouped by exercise_id in Rust);
  both callers now issue at most two extra queries regardless of exercise count.
  Covered by existing progress/exercise integration tests.
B-MED-10: get_progress_overview loaded the user's ENTIRE set history into a
  Vec<SetFact> and built two full Vec<PrEvent> lists, of which only 4 period
  counts + the last 20 events were used — memory grew unbounded with history.
  Refactored PR detection into incremental folds (AllTimePrDetector /
  RecentBestDetector push-per-fact) driven by a streaming query
  (for_each_progress_set_fact via sqlx fetch()), collecting into a bounded
  EventWindow (4 counters + a RECENT_PR_LIMIT ring). Running-best maps are
  bounded by distinct exercise types; the recent-best histories self-prune to
  the 90-day window. Output is unchanged (input already ordered by
  start_time,set_id so the ring's tail == the old rev().take(20)). Verified by
  progress_overview_reports_periods_timed_stats_and_pr_feed.
F-MED-8: the root layout rendered protected page content while auth status was
  still 'unknown' (the initial /auth/me check in flight), flashing cached
  IndexedDB data for a session that may already be revoked server-side. Added a
  `contentReady` gate: protected <main> now shows a spinner until status is
  'authenticated' — with an explicit exception for 'unknown' + offline so the
  offline-first cached experience is preserved (the server can't be reached to
  verify, so the cached session is trusted). svelte-check clean.
F-MED-9: the service worker's install handler called skipWaiting()
  unconditionally, so a deploy landing mid-session activated the new worker
  immediately; its activate handler then deleted the previous version's cache,
  404-ing the lazy chunks the running page still needed (breaking an in-progress
  workout). Removed the eager skipWaiting() — the new worker now waits until all
  tabs close — and added a SKIP_WAITING message listener so the page can opt into
  an immediate update when it's safe. svelte-check clean.
B-LOW-1: /api/auth/login was in should_skip_auth, so it bypassed the CSRF
  origin check entirely even though it establishes a session — a cross-site
  page could drive a login (login CSRF). The middleware now applies
  csrf_origin_ok to the login path (when enforce_csrf is on and the method is
  mutating) before the skip-auth early return, while leaving the genuinely
  cross-origin surfaces (/mcp, /oauth/*, /.well-known/*) exempt. The login_cookie
  test helper now sends same-origin headers (as a real browser does). Tests:
  csrf_blocks_login_without_origin_in_production_mode.
B-LOW-2: exchanging an authorization code detected a replay (used_at set) but
  only rejected the second exchange — the tokens already issued from the leaked
  code stayed live. Per RFC 6749 §4.1.2/§10.5, the token rotation family is now
  keyed by the auth-code id (auth_code_family_id) so a replay calls
  revoke_oauth_token_family and kills every token issued from that code before
  returning invalid_grant. Test: replaying_authorization_code_revokes_issued_tokens.
B-LOW-3: the OAuth authorize handler returned instantly for a disabled /
  must-change-password account (no argon2 work, no delay), while unknown users
  and wrong passwords ran a dummy/real verify + 200ms delay — a timing oracle
  for valid usernames. Moved the disabled/must-change check to AFTER password
  verification and folded it into the same failure path (record + IP failure +
  200ms sleep + identical access_denied), so all failure states are
  indistinguishable. Existing OAuth flow tests confirm valid logins still work;
  a timing assertion would be flaky so none was added.
B-LOW-4: request_ip trusted X-Real-IP unconditionally, so a client with direct
  network access could forge its source IP to evade/poison per-IP rate limiting.
  Now gated on a new TRUST_PROXY_HEADERS flag (default false = use the real peer
  address); the header is honoured only when explicitly trusted. docker-compose
  sets TRUST_PROXY_HEADERS=true (server is reachable only via the nginx client
  container) and server.env.example documents it. Test harness mirrors this;
  test: request_ip_trusts_x_real_ip_only_when_configured.
B-LOW-6: expose_internal_errors() used std::env::var(..).is_ok(), so
  EXPOSE_INTERNAL_ERRORS=0 (an operator trying to DISABLE leakage) actually
  enabled it since the var was merely present. Now parsed truthily (only
  1/true), matching the ENABLE_HSTS convention. Test:
  expose_flag_only_enabled_for_truthy_values.
B-LOW-7: already mitigated — services::exercises::replace_sets caps the array at
  MAX_SETS_PER_EXERCISE_REPLACE (100) for both the MCP tool and the REST endpoint
  (shared service layer). Added an explicit REST-layer regression assertion (101
  sets -> 400 "at most 100") to replace_sets_endpoint_replaces_existing_sets_and_validates_payload.
B-LOW-8: the authorize redirect was built as "{redirect_uri}?code=...", producing
  a malformed "...?foo=bar?code=..." when the registered redirect_uri already had
  a query string (allowed by RFC 6749 §3.1.2; fragments are rejected at
  registration). Now selects & vs ? based on whether the URI already contains a
  query. Test: authorize_appends_code_with_correct_separator_for_query_redirect.
B-LOW-9: rotate_mcp_token_for_user copied the original token's absolute
  expires_at into the replacement, so rotating a 30-day token on day 29 produced
  a token that expired in ~1 day, silently breaking the integration. Rotation now
  re-applies the original lifetime (expires_at - created_at) from now via a pure
  rotated_expiry() helper; non-expiring tokens stay non-expiring. Tests:
  rotation_reapplies_full_lifetime_from_now, rotation_keeps_non_expiring_tokens_non_expiring,
  mcp_personal_token_rotate_reapplies_lifetime (integration).
B-LOW-10: the login/OAuth IP limiter checked the counter (is_ip_rate_limited)
  and recorded the failure (record_ip_failure) under two separate lock
  acquisitions, so concurrent attempts could all pass the check before any
  recorded — overshooting LOGIN_RATE_LIMIT_ATTEMPTS by the in-flight concurrency.
  Replaced with a single atomic admit_login_attempt(ip, now) that evicts, checks,
  and records under one lock (mirroring the MCP admit_request), used at both the
  login and OAuth authorize entry points; a successful login still clears the IP.
  Behavior verified identical by login_is_rate_limited_by_ip_across_usernames and
  oauth_authorize_is_rate_limited_by_ip.
B-LOW-11: create_backup streamed the tar.gz straight into its final path, so a
  crash/error mid-write left a truncated .tar.gz that list/restore would treat as
  valid. Now writes to a "<name>.partial" temp file (ignored by list_backups'
  .tar.gz filter), finishes the tar + gzip, fsyncs, and atomically renames into
  place — removing the temp on any failure. Covered by existing backup tests.
B-LOW-12: restore_backup unpacked archive entries by name without checking their
  type, so a symlink/hardlink/dir entry named database.db(-wal/-shm) would be
  recreated and renamed onto the live DB path, redirecting later writes. The
  extraction loop now rejects any non-regular-file entry (entry_type().is_file()),
  defense-in-depth alongside the B-MED-8 pre-restore validation. Covered by
  existing backup restore tests.
B-LOW-13: start_workout_from_template checked for an active session with a
  COUNT(*) on the pool, then inserted the new workout in a separate transaction —
  two concurrent starts could both pass the check and create two active sessions.
  The workout INSERT is now an atomic INSERT ... SELECT ... WHERE NOT EXISTS (no
  active session) that holds the write lock during evaluation; a lost race returns
  the same 409 Conflict. The COUNT check is kept as a fast-path/precedence. Test:
  starting_a_second_template_session_while_one_is_active_conflicts.
B-LOW-14: detect_pr_events / detect_recent_best_events keyed their running-best
  maps on the raw exercise_type, while the volume/detail pages match with
  LOWER(exercise_type) — so "Bench Press" and "bench press" produced two separate
  PR streams in the overview but one on the detail page, and each new casing was
  re-flagged as a fresh PR. Both detectors now key on exercise_group_key() =
  to_ascii_lowercase() (matching SQLite LOWER, no trim); the displayed casing on
  each event is unchanged. Tests: pr_detection_groups_exercise_type_case_insensitively,
  exercise_group_key_matches_sqlite_lower_semantics.
F-LOW-1: toggling split-weight OFF collapsed each set's two side weights with
  Math.max(left, right); since perSideWeight stays on (so the single weight is
  doubled for volume), a 40/60 set became 60 -> volume jumped 20% and the lighter
  side was silently discarded and pushed to the backend. Now collapses to the
  average (left + right) / 2, which preserves the total lifted load and volume
  exactly. Test: collapses split sides to their average to preserve volume.
F-LOW-2: scopedKey fell back to the bare base key when the active user id was
  unknown, so an offline session written during the cold-start null window landed
  under an unscoped key — orphaned once the id resolved (scoped reads use `u<id>:`)
  and briefly readable by another account on a shared device. scopedKey now uses an
  explicit `anon:` namespace (never collides with real users' scoped reads), and
  saveOfflineSession refuses to persist without an active user scope (the in-memory
  session survives; a later persist writes it under the right key). Tests:
  offline-session-scope-guard.test.ts.
F-LOW-3: (a) handleResponse returned undefined (success) for ANY 2xx non-JSON
  body, so a proxy's HTML 200 on POST /workouts/{id}/end was accepted as a
  completed write and syncOne then deleted the un-retryable offline record. Now a
  non-EMPTY non-JSON 2xx throws ApiError (the backend always returns JSON for these
  endpoints); genuinely empty bodies and 204 still resolve void. (b) The remote
  logger dropped the in-flight batch on 401/403 before disabling; it now unshifts
  the batch back onto the queue so a transient auth lapse (token refresh) doesn't
  lose diagnostics — they resend once remote logging is re-enabled. Tests:
  api-client.test.ts (non-JSON rejects), logger.test.ts (401 preserves batch).
F-LOW-4: on login the layout's authenticated-while-on-/login effect always
  goto('/'), racing and overriding the login page's goto('/settings') for a
  must_change_password user (isLogin stays true until navigation resolves, so the
  effect fires last and wins). The layout effect now routes must_change_password
  users to /settings itself, making it the single source of truth. (No layout unit
  tests exist; verified via svelte-check + logic review. Backend still enforces the
  change on sensitive actions regardless.)
F-LOW-5: (b) the startup log recorded window.location.href (full URL incl.
  query/hash) and navigator.userAgent; now logs only location.pathname and omits
  the UA, so the line can't ship a sensitive URL param or fingerprint off-device
  if the remote-send gate is ever loosened. (a) the stale-auth.lastUser cross-user
  identity/data leak on a shared device: the ONLINE vector is already closed by
  F-MED-8 (protected content is gated behind a spinner during 'unknown' auth until
  authMe resolves to a 401 -> login), and handleUnauthorized already clears
  lastUser + activeUserId on logout/401. The residual offline-on-shared-device
  window is inherent to the offline-first design and accepted per the audit's own
  "bounded / personal-use PWA / httpOnly cookie is the real gate" assessment.
F-LOW-6: the service worker's cache-bypass covered only /api/, so same-origin GETs
  to /oauth, /mcp and /.well-known (exposed via the proxy) fell through to the
  shared cacheFirst bucket (keyed only by URL, no per-user scoping) and could be
  served to a later request — including a different user on a shared device — and
  /oauth* navigations were cached under the '/' shell key. The bypass now covers
  the whole backend seam (/api/, /oauth, /mcp, /.well-known) and runs before the
  navigate branch so those responses are never cached or served from the SW.
  (No SW unit-test harness; verified via svelte-check + review.)
