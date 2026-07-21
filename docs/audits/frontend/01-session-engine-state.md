# Live workout-session state engine

**Summary:** 3 confirmed (1 high, 1 medium, 1 low/unverified), 0 refuted.

## Confirmed findings

### HIGH [data-integrity]: Online action reducers capture a stale session snapshot across an await and overwrite the whole store with .set(), silently discarding concurrent mutations

- **Trigger:** Online. User types a brand-new exercise name (not in recentSessions cache, not yet in lastTimeByExercise) and taps add. addExercise awaits loadLastTimeForExercise() over the network while loading is still false and nothing is disabled. During that window the user edits an existing exercise's notes / a setting / tracking (settings.ts synchronously .set the store) or logs a set. When createExercise resolves, core.ts:211 does .set({...stale session}) and wipes that edit from currentSession. For debounce-synced fields (notes/settings/tracking) the loss is permanent: the 650ms debounced backend push syncExercise() (settings.ts:45-66) reads get(state.currentSession) at fire time (settings.ts:47), which is now the clobbered snapshot, so it persists the OLD value to the backend.

- **Location:** `client/src/lib/today/controller/actions/exercise/core.ts:106,118,179,211`; also `sets.ts:163→204`, `sets.ts:98→135`, `core.ts:245→271`

- **What happens:** addExercise() captures `const session = get(state.currentSession)` at line 106, performs network awaits (loadLastTimeForExercise at line 118 runs *before* state.loading.set(true) at line 179), and at line 211 rebuilds the whole store with `state.currentSession.set({ ...session, exercises: [...session.exercises, newExercise] })` using that stale pre-await snapshot. Any mutation applied to currentSession between the snapshot and the .set is overwritten and lost. The same stale-snapshot-then-.set pattern appears in addSet online (sets.ts:163 → sets.ts:204), markExerciseDone online (sets.ts:98 → sets.ts:135), and removeExercise online (core.ts:245 → core.ts:271).

- **Why:** A logged set or an exercise-notes/settings edit silently vanishes from the live in-progress session; for debounce-synced fields it is written back to the backend as the old value, so the user's data is permanently lost, not just visually. The user sees their edit disappear mid-workout and is likely to re-enter it, which then races/duplicates.

- **Fix sketch:** In every online reducer, do not build the next session from the pre-await snapshot. After the await, use `state.currentSession.update((current) => ...)` and merge into `current`, exactly as replaceExerciseSets already does at sets.ts:73-81. Also gate input (loading.set(true)) before the first network await in addExercise so the pre-load fetch does not run with the UI live.

### MEDIUM [data-integrity]: Network failure during the final endWorkout of submitEndSession drops the entered mood/notes and leaves the workout un-ended forever on the next sync

- **Trigger:** Online, user opens the end-session modal, picks a mood/notes, taps End. endExercise calls succeed, then endWorkout fails with a network error. App flips offline and persists an in_progress record with no mood. On reconnect syncOne finishes the exercises and deletes the record without calling endWorkout. The workout stays active on the server and the entered mood/notes are gone.

- **Location:** `client/src/lib/today/controller/actions/session.ts:345`; check offline.ts:97-108, offline.ts:269, offline.ts:283-285

- **What happens:** In submitEndSession online path, exercises are ended via Promise.all (session.ts:309) then endWorkout is awaited (session.ts:330). If connectivity drops during endWorkout, the catch at session.ts:345-352 calls persistInProgressSession(state), which saves the offline record with status 'in_progress' and NO end_mood / end_notes / endedAt. Contrast the offline branch (session.ts:291-301) which saves status 'pending_sync' WITH end_mood and endedAt. On reconnect, syncOne only calls endWorkout when `record.status === 'pending_sync' && record.end_mood && record.session.endedAt` (offline.ts:269); for a positive-id in_progress record it replays exercises then deletes the record (offline.ts:283-285) without ending the workout.

- **Why:** The user believes they finished and rated their workout, but the session is never actually ended on the backend and their end-of-session mood/notes are silently discarded. They must notice the session is still 'in progress' and end it again.

- **Fix sketch:** In the submitEndSession network-failure catch, persist an end record analogous to the offline branch: status 'pending_sync' with end_mood, end_notes and session.endedAt set, so syncOne's endWorkout branch (offline.ts:269) fires on reconnect.

### LOW [data-integrity]: Turning off split-weight with unequal left/right silently rewrites the logged weight to max(left,right), changing the recorded set and volume (unverified)

- **Trigger:** During a live workout the user logs a dumbbell exercise with different left/right loads while split-weight is on, then toggles split-weight off. Every asymmetric set is rewritten to the heavier side and exercise volume jumps.

- **Location:** `client/src/lib/today/controller/actions/exercise/weightModes.ts:138`

- **What happens:** toggleExerciseSplitWeight(enabled=false) collapses each set's two side weights into a single weight using Math.max(left, right) at weightModes.ts:138, then persists via replaceSets. With per-side weight still on (perSideWeight true, splitWeight now false), calculateExerciseVolumeKg treats the single weight as per-side and doubles it (metrics.ts:20-21: reps * weight * 2). A set logged as left=40/right=60 (volume reps*100) becomes weight=60 (volume reps*120). The lighter side is discarded, total volume inflated, and the change pushed to the backend.

- **Why:** The user's actually-lifted weights are silently altered and session volume mis-computed; the corrupted values are persisted to the backend.

- **Fix sketch:** Preserve volume when collapsing sides, e.g. weight = (left + right) / 2 rather than Math.max, or confirm before replacing both sides with a single value.

## Refuted (not real / already handled)

None.
