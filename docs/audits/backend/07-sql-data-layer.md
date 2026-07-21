# SQL / sqlx data-access layer correctness & injection

**Summary:** 3 confirmed (2 medium, 1 low unverified), 0 refuted.

## Confirmed findings

### MEDIUM [correctness]: Unbounded N+1 query fan-out in progress/exercise reads

- **Attack/trigger:** An authenticated session user OR an MCP-token agent (routes.rs:559 → services/progress.rs:16 → db::get_exercise_progress; and mcp/routes.rs:1018 get_exercise_progress tool) bulk-creates thousands of exercises of one exercise_type, then requests progress for that type. The backend then executes ~2*N sequential SQLite queries on one pooled connection while holding it, blocking the pool.

- **Location:** `server/src/db/progress.rs:376–399`; see also `server/src/db/exercises.rs:391–440` (get_exercises_for_workout) and `server/src/db/exercises.rs:446, 565` (get_sets_for_exercise, get_settings_for_exercise)

- **What happens:** `get_exercise_progress()` fetches all exercises of a given type for a user (no LIMIT) and then, in a Rust for-loop, issues TWO additional round-trip queries per row: `get_sets_for_exercise()` and `get_settings_for_exercise()`. Similarly, `get_exercises_for_workout()` has the same per-exercise `get_settings_for_exercise()` N+1 problem.

- **Why:** Read is scoped to the caller's own user_id (no IDOR), but query count is proportional to attacker-controlled row count with no pagination/LIMIT, giving a self-service latency/connection-exhaustion vector amplified by MCP agents generating data volumes a human UI never would.

- **Fix sketch:** Replace per-row lookups with a single JOINed query (or batched IN (...) queries keyed by fetched exercise ids); add server-side LIMIT/pagination to `get_exercise_progress` and `get_exercises_for_workout`.

---

### MEDIUM [correctness]: Progress overview loads entire user set history into memory and recomputes PRs in Rust on every call

- **Attack/trigger:** An MCP-token agent or session user creates a very large number of sets (each `create_set` is a cheap authenticated write), then calls `get_progress_overview` (routes.rs:578 / mcp/routes.rs:1036). Each call re-materializes the entire history in memory and re-sorts/re-hashes it; repeated concurrent calls multiply the RSS spike and CPU burn.

- **Location:** `server/src/db/progress.rs:968–1022` (get_progress_set_facts) and `server/src/db/progress.rs:771–773` (get_progress_overview)

- **What happens:** `get_progress_set_facts()` selects EVERY set the user ever logged (JOIN sets/exercises/workouts, no LIMIT, no time window) into a `Vec<SetFact>`, and `get_progress_overview()` runs `detect_pr_events()`/`detect_recent_best_events()` building HashMaps and Vecs over the full set on each request. Memory and CPU cost are O(total sets), unbounded, uncached.

- **Why:** Scoped to own data (not IDOR), but fully unbounded in-memory materialization plus per-request recomputation is a real resource-exhaustion path for a public launch, reachable by automated MCP clients that inflate dataset size quickly.

- **Fix sketch:** Bound the fact query with a rolling time window and/or compute aggregates in SQL; cache computed PR events; avoid pulling complete lifetime set history on every overview request.

---

### LOW [correctness]: TOCTOU on single-active-session check in start_workout_from_template (unverified)

- **Attack/trigger:** A user or MCP agent fires two `start_workout_from_template` requests (routes.rs:173 / mcp/routes.rs:1126) nearly simultaneously. Both pass the active_count==0 check before either commits, producing two simultaneously-active sessions, violating the invariant and confusing downstream auto-close/active-session logic.

- **Location:** `server/src/db/templates.rs:396–447`

- **What happens:** The 'you already have an active session' guard runs as a standalone COUNT query on the pool (lines 396–405) BEFORE the transaction that inserts the workout is begun (tx = pool.begin() at line 415). The uniqueness invariant (at most one workout with end_time ≤ start_time per user) is checked outside the write transaction, so two concurrent requests can both observe active_count == 0 and both insert an active workout.

- **Why:** Own-data only, not corrupting other users, so low severity, but it defeats a stated business invariant and the non-atomic check-then-act pattern lets clients create inconsistent state under normal double-submit conditions.

- **Fix sketch:** Perform the active-session COUNT inside the same transaction that inserts the new workout, or enforce a partial unique index on user_id WHERE end_time ≤ start_time so concurrent inserts fail atomically.

---

## Refuted (not real / already handled)

(None.)
