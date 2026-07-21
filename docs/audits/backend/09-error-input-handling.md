# Error handling, input validation & information leakage

**4 confirmed findings (1 verified, 3 unverified). 0 rejected.**

## Confirmed findings

### MEDIUM security: Panic (worker request abort / DoS) on non-char-boundary byte slice of request-controlled log metadata

- **Attack/trigger:** Any authenticated session user (route guarded by CurrentUser) POSTs `[{"level":"debug","metadata":{"x":"éééé…"}}]` where the serialized metadata exceeds 2048 bytes and a multibyte char (é, emoji, CJK) straddles byte 2048. The handler future panics, aborting that request/connection.
- **Location:** `server/src/routes.rs:313`
- **What happens:** In write_logs (POST /api/logs), the debug-level branch serializes the client-supplied `metadata` JSON value and, when it exceeds 2048 bytes, truncates it with `&s[..2048]`. This is a byte-index slice on a String. serde_json emits non-ASCII characters as raw UTF-8 (it does not \u-escape them), so if byte offset 2048 lands in the middle of a multibyte character the slice panics with 'byte index is not a char boundary'. The truncation is computed eagerly inside `and_then`, before any log-level filtering, so it runs regardless of the configured RUST_LOG level.
- **Why:** Request-controlled input reaches a byte-boundary slice with no floor-to-char-boundary guard. Repeatable per-request DoS; each crafted request aborts its task and drops the connection.
- **Fix sketch:** Truncate on a char boundary (floor_char_boundary, or iterate chars while byte length < limit like sanitize_log_field already does). Never index a String with a fixed byte range from arbitrary input.

### LOW security: Log injection via unsanitized client-supplied timestamp field (unverified)

- **Attack/trigger:** Authenticated user POSTs `[{"timestamp":"2026-01-01\n[2026-01-01] ERROR error - forged event","message":"x"}]`; the newline is written into the server log, injecting a fake record.
- **Location:** `server/src/routes.rs:302`
- **What happens:** write_logs passes `target` and `message` through sanitize_log_field (strips control chars) but appends the client-supplied `timestamp` verbatim as `client_ts={}`. The log line is a single writeln, so embedded newlines/control chars let a caller forge additional log lines or corrupt structured log parsing.
- **Why:** Inconsistent sanitization — the other free-text fields are cleaned, so the mitigation is bypassed by moving payload into timestamp. Enables log forgery/audit confusion.
- **Fix sketch:** Run timestamp through sanitize_log_field (or strict RFC3339 validation) before appending.

### LOW security: Internal error exposure toggle enabled by mere presence of env var (unverified)

- **Attack/trigger:** Operator sets EXPOSE_INTERNAL_ERRORS=false/0 intending to disable exposure; production then returns full DB/internal error strings on any 500. Also always-on in debug builds via cfg!(debug_assertions).
- **Location:** `server/src/errors.rs:6`
- **What happens:** expose_internal_errors() returns true whenever `std::env::var("EXPOSE_INTERNAL_ERRORS").is_ok()` — i.e. the var is set to ANY value including `0`, `false`, or empty. When true, DatabaseError/InternalError responses include the raw underlying error (sqlx text with SQL fragments, table/column names, file paths).
- **Why:** Presence-based check inverts the intended safe default and is easy to trip; leaks DB internals aiding schema mapping.
- **Fix sketch:** Parse the value explicitly (only expose on `1`/`true`, case-insensitive) as ENABLE_HSTS is parsed in main.rs.

### LOW security: Unbounded set array in replace_sets allows DB write amplification within body limit (unverified)

- **Attack/trigger:** Authenticated user repeatedly PUTs ~512KB bodies of thousands of tiny set objects for an owned exercise, forcing large delete+insert transactions and table growth.
- **Location:** `server/src/routes.rs:229`
- **What happens:** PUT /api/exercises/{id}/sets accepts `web::Json<Vec<CreateSetRequest>>` with no element-count cap; each element is validated and written. Unlike template exercises (cap 64) and settings (cap 24), the only bound is the 512KB JSON body limit, permitting tens of thousands of minimal set objects per request, each an insert.
- **Why:** Missing the explicit element-count cap the rest of the model layer applies consistently; enables cheap write-amplification/storage abuse.
- **Fix sketch:** Add MAX_SETS_PER_EXERCISE and reject oversized arrays, consistent with template/settings caps.
