# SwoleMate Backend — Launch-Readiness Security & Correctness Rollup

## Findings Summary

| Severity | Type | Category | Title | Location |
|----------|------|----------|-------|----------|
| **HIGH** | security | Auth Session | Unauthenticated attacker can permanently lock out any account via failed-login lockout | `server/src/routes/auth.rs:58-64, 69`; `server/src/db/auth.rs:245-260` |
| **MEDIUM** | security | Auth Session | Username enumeration via divergent login responses (locked 429, disabled fast 401, unknown delayed 401) | `server/src/routes/auth.rs:44-74` |
| **MEDIUM** | security | OAuth | Refresh-token rotation has no reuse-detection / family revocation | `server/src/oauth/routes.rs:778-830` |
| **MEDIUM** | security | MCP Auth | MCP auth middleware performs unrated DB reads/writes per request; rate limiter only guards post-auth dispatch | `server/src/middleware/mcp_auth.rs:99-133` |
| **MEDIUM** | security | MCP Auth | Password change does not revoke MCP (or OAuth) bearer tokens, so stolen token survives reset | `server/src/routes/auth.rs:150-152` |
| **MEDIUM** | security | Rate Limiting | MCP rate limiter keyed per-token/per-client, not per-user — user multiplies budget by minting extra tokens | `server/src/mcp/routes.rs:825` |
| **MEDIUM** | security | Error Handling | Panic (worker request abort / DoS) on non-char-boundary byte slice of request-controlled metadata | `server/src/routes.rs:313` |
| **MEDIUM** | correctness | Authorization | disable_user lacks the last-admin guard that delete_user has; no re-enable route exists | `server/src/routes/admin.rs:68–80` |
| **MEDIUM** | correctness | Backup | Failed backup restore leaves the global DB connection pool permanently closed (app-wide outage until restart) | `server/src/routes.rs:438-449, 526-530` |
| **MEDIUM** | correctness | SQL Layer | Unbounded N+1 query fan-out in progress/exercise reads | `server/src/db/progress.rs:376–399` |
| **MEDIUM** | correctness | SQL Layer | Progress overview loads entire user set history into memory and recomputes PRs in Rust on every call | `server/src/db/progress.rs:968–1022` |
| **MEDIUM** | correctness | Progress Logic | get_workout_stats counts never-ended (in-progress/abandoned) workouts as zero-duration, deflating average duration | `server/src/db/progress.rs:411-522` |
| **LOW** ᵘ | security | Auth Session | Login endpoint is exempt from CSRF origin check, enabling cross-site forced login | `server/src/middleware/session_auth.rs:86-97` |
| **LOW** ᵘ | security | OAuth | Authorization-code replay does not revoke tokens already issued from that code | `server/src/oauth/routes.rs:643-683` |
| **LOW** ᵘ | security | OAuth | Username enumeration via differential response/timing in authorize_post | `server/src/oauth/routes.rs:462-494` |
| **LOW** ᵘ | security | Rate Limiting | request_ip trusts X-Real-IP unconditionally — enables login rate-limit bypass and unbounded map growth | `server/src/auth/rate_limit.rs:26` |
| **LOW** ᵘ | security | Error Handling | Log injection via unsanitized client-supplied timestamp field | `server/src/routes.rs:302` |
| **LOW** ᵘ | security | Error Handling | Internal error exposure toggle enabled by mere presence of env var | `server/src/errors.rs:6` |
| **LOW** ᵘ | security | Error Handling | Unbounded set array in replace_sets allows DB write amplification within body limit | `server/src/routes.rs:229` |
| **LOW** ᵘ | correctness | OAuth | Authorization response builder blindly appends '?code=', corrupting URIs that already contain query string | `server/src/oauth/routes.rs:549-555` |
| **LOW** ᵘ | correctness | MCP Auth | Token rotation copies the original absolute expires_at instead of issuing fresh lifetime | `server/src/db/mcp_tokens.rs:172-206` |
| **LOW** ᵘ | correctness | Rate Limiting | Login limiter check-then-record is non-atomic (two separate lock acquisitions) | `server/src/routes/auth.rs:37` |
| **LOW** ᵘ | correctness | Backup | Backups are written non-atomically to final path, leaving truncated archives in UI | `server/src/backup.rs:94` |
| **LOW** ᵘ | correctness | Backup | Restore extraction relies on entry-name whitelist; would follow symlink/hardlink entries if untrusted archive ingress added | `server/src/backup.rs:250-265` |
| **LOW** ᵘ | correctness | SQL Layer | TOCTOU on single-active-session check in start_workout_from_template | `server/src/db/templates.rs:396–447` |
| **LOW** ᵘ | correctness | Progress Logic | PR / recent-best detection groups by exact-case exercise_type while volume pages match case-insensitively | `server/src/db/progress.rs:95-99` |

*ᵘ = unverified (low-severity findings marked as unverified in source)*

---

## Top Launch Blockers

**CRITICAL TO FIX BEFORE LAUNCH:**

1. **[HIGH - CRITICAL]** Permanent account lockout DoS — Attacker can lock out any account indefinitely via trivial login attempts (`server/src/routes/auth.rs:58-64`). Single admin account exposure. Fix required.

**HIGH-SEVERITY ITEMS (Fix strongly recommended):**

2. **[MEDIUM]** Username enumeration leaks valid accounts via login response timing/status codes, feeding targeted attacks (`server/src/routes/auth.rs:44-74`).

3. **[MEDIUM]** Stolen MCP bearer tokens survive password reset, granting 30–365 day persistent access to training data (`server/src/routes/auth.rs:150-152`).

4. **[MEDIUM]** MCP auth rate limiting bypassed by creating multiple tokens; per-token keying allows user to multiply request budget (`server/src/mcp/routes.rs:825`).

5. **[MEDIUM]** Failed backup restore permanently closes DB pool, triggering app-wide outage until restart (`server/src/routes.rs:438-449`).

6. **[MEDIUM]** Panic (DoS) on multibyte UTF-8 characters at byte 2048 boundary in request log metadata (`server/src/routes.rs:313`).

7. **[MEDIUM]** Admin account can be irreversibly disabled with no recovery path, bricking all admin access (`server/src/routes/admin.rs:68–80`).

8. **[MEDIUM]** Unbounded N+1 queries on exercise progress; MCP agents can trigger thousands of sequential DB reads, exhausting pool (`server/src/db/progress.rs:376–399`).

---

## Recommendation

**Do not launch to production until the HIGH and the 1–7 MEDIUM items are remediated.** The HIGH finding alone (permanent account lockout) is a critical DoS vulnerability. Items 3, 5, 7, and 8 are correctness/availability blockers affecting core workflows. Items 2, 4, 6 pose secondary but real security/resilience risks.

The LOW unverified findings should be addressed in a follow-up hardening phase post-launch unless the deployment topology (e.g., nginx proxy for X-Real-IP) or feature set (e.g., no backup import) already mitigates them.
