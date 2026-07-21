# Progress & consistency business-logic correctness

2 confirmed (1 medium, 1 low/unverified), 0 refuted.

## Confirmed findings

### MEDIUM correctness: get_workout_stats counts never-ended (in-progress/abandoned) workouts as zero-duration, deflating average duration and skewing distribution

**Attack / trigger:**  
Any authenticated web or MCP user calls GET /api/stats (services::progress::get_workout_stats → db.get_workout_stats) while having an in-progress workout, or after abandoning workouts with auto-close disabled. No special input; more open/abandoned sessions → more downward skew on average duration and the 0–30 min bucket.

**Location:**  
`server/src/db/progress.rs:411-522`

**What happens:**  
In get_workout_stats the base CTE `workout_times` selects FROM workouts with only `WHERE user_id = ?` and no `end_time > start_time` filter (lines 411–420). A freshly created workout is stored with end_time == start_time (server/src/db/workouts.rs:82–93 sets end_time = start_time on INSERT), and a workout abandoned without ending stays that way indefinitely when auto-close is disabled (inactivity_minutes ≤ 0). For those rows ROUND((julianday(end_time)−julianday(start_time))*24*60) = 0. total_workouts (line 492) counts them, avg_duration = AVG(duration) (line 493) averages the zeros in, and duration_ranges/duration_distribution (lines 473–489) bucket them into '0–30'. This is inconsistent even within the same function: avg_exercise_duration_series (line 636) and session_start_times (line 689) DO filter end_time > start_time, and get_calendar_workout_frequency (progress_consistency.rs:35) filters end_time > start_time, so frequency counts only completed workouts while total_workouts/avg_duration count all.

**Why:**  
Reported statistics are silently incorrect; no security impact, data loss, or escalation. Displayed average session duration and duration histogram are dragged toward 0 whenever a user has any open or abandoned workout.

**Fix sketch:**  
Add `AND end_time > start_time` to the workout_times CTE (or exclude zero-duration rows from avg_duration, the duration-based total, and duration_ranges) so duration stats only consider completed workouts, matching the other series in the same function and get_calendar_workout_frequency.

### LOW correctness: PR / recent-best detection groups by exact-case exercise_type while volume/records pages match case-insensitively, producing split and inconsistent PR streams (unverified)

**Attack / trigger:**  
A user or MCP agent creates exercises of the same movement with differing casing/whitespace in exercise_type (common with free-typed web input vs. AI-generated names via MCP), then loads GET /api/progress/overview vs. the exercise detail/volume endpoints; the two surfaces report different PRs/records for the logically same exercise.

**Location:**  
`server/src/db/progress.rs:95-99`

**What happens:**  
detect_pr_events and detect_recent_best_events key their running-best maps on the raw fact.exercise_type string (HashMap<String,...> at lines 95–99 and format!("{}::...", fact.exercise_type) at lines 191–238). exercise_type is stored verbatim from the client with no normalization on write (create_exercise inserts req.exercise_type as-is, server/src/db/exercises.rs:41–56). The per-exercise views match case-insensitively: get_exercise_progress uses LOWER(exercise_type)=LOWER(?) (line 363) and get_volume_stats uses LOWER(e.exercise_type)=LOWER(?) (lines 1079, 1149, 1211, 1241). So the same movement logged as 'Bench Press' and 'bench press' (or with trailing whitespace) is ONE exercise on the detail/volume/records page but TWO independent PR histories in the progress overview: recent_prs / pr_count / recent_best_count in get_progress_overview disagree with personal_records on the exercise page, and a lift the user knows is not a PR is re-flagged as a PR the first time each casing variant appears, inflating per-period pr_count deltas.

**Why:**  
Correctness/consistency defect in derived analytics only; no security impact and no persisted corruption. Impact is user confusion and slightly inflated PR counts, not data loss.

**Fix sketch:**  
Normalize the grouping key consistently: lowercase and trim fact.exercise_type when building the HashMap keys in detect_pr_events/detect_recent_best_events to match the LOWER(...) comparisons used elsewhere; ideally also normalize exercise_type on write in create_exercise so all code paths agree.

## Refuted (not real / already handled)

(None)
