# Password authentication & session lifecycle

**3 confirmed (1 high, 1 medium, 1 unverified low), 0 refuted**

## Confirmed findings

### HIGH security: Unauthenticated attacker can permanently lock out any account (incl. admin) via failed-login lockout

- **Attack/trigger:** Attacker POSTs /api/auth/login (route skipped by SessionAuth via should_skip_auth, no session/CSRF gate) with {username:'admin', password:'x'} five times to lock, then one bad attempt every 5 min. IP limiter defaults to 30/10min (auth/rate_limit.rs:16), far above the ~1/5min needed, so a single IP sustains a permanent admin lockout.
- **Location:** `server/src/routes/auth.rs:58-64, 69` and `server/src/db/auth.rs:245-260`
- **What happens:** Login enforces a per-account lockout: db.record_failed_login increments failed_login_count and, once it reaches 5, sets locked_until = now+5min. While locked, the handler returns TooManyRequests at lines 58-64 BEFORE the password is checked, so even the legitimate owner with the correct password is bounced. failed_login_count is NOT reset when the lock expires (only reset on a successful login at line 77, which is unreachable while locked). Because the counter stays at 5, one further failed attempt after the window immediately re-locks (5+1 >= 5).
- **Why:** Remote, unauthenticated, sustained DoS of authentication for arbitrary named accounts including the sole admin, at trivial cost. Missing counter-reset-on-expiry turns brute-force defense into a permanent account-freeze primitive.
- **Fix sketch:** Reset failed_login_count when locked_until has elapsed, or use time-based exponential backoff instead of a sticky counter; don't block a correct-password login solely on lock state; prefer per-IP tarpitting over hard account lock.

### MEDIUM security: Username enumeration via divergent login responses (locked 429, disabled fast 401, unknown delayed 401)

- **Attack/trigger:** Send 5 bad-password POSTs to /api/auth/login for a candidate username; if responses become 429 'Too many login attempts. Try again later.' the username exists, else it does not. A near-zero-latency 401 (no argon2 cost) reveals a disabled account.
- **Location:** `server/src/routes/auth.rs:44-74`
- **What happens:** Login leaks account existence via content and timing. Unknown users (45-52) get 401 after verify_dummy_password + 200ms sleep. Disabled users (54-56) return 401 IMMEDIATELY with no argon2 and no sleep, so measurably faster. Locked users (58-64) return 429 with distinct body 'Too many login attempts. Try again later.' (different from the IP-limit message at 38-40). Attacker can distinguish disabled accounts by timing and confirm a username exists by sending 5 bad passwords and watching the account flip to the locked 429 state; a nonexistent username never locks and keeps returning delayed 401s.
- **Why:** Defeats the deliberate generic-response design (dummy hash + 200ms sleep on the unknown path shows enumeration was meant to be blocked). Enumerated usernames feed the targeted lockout DoS above and credential stuffing.
- **Fix sketch:** Make the disabled and locked paths return the same status/body and same argon2+sleep timing as the unknown/wrong-password paths (run verify_dummy_password + 200ms delay, return uniform 401); do not surface lock state to unauthenticated callers.

### LOW security: Login endpoint is exempt from CSRF origin check, enabling cross-site forced login (unverified)

- **Attack/trigger:** Victim visits evil.com which auto-submits a POST to /api/auth/login with attacker-known credentials; the victim's browser is now authenticated as the attacker's account, so workouts the victim logs land in the attacker's account, which the attacker later reads.
- **Location:** `server/src/middleware/session_auth.rs:86-97`
- **What happens:** should_skip_auth returns true for '/api/auth/login', so SessionAuth (including csrf_origin_ok at 129-150) never runs for the login POST. Login is state-changing and sets an authenticated session cookie but has no Origin/Referer validation. A malicious site can trigger a cross-site POST to /api/auth/login with the attacker's own credentials; the browser sends the request (CORS blocks reading the response, not sending) and the SameSite=Lax response cookie is still set on the victim's browser, silently logging the victim into an attacker-controlled account (login CSRF).
- **Why:** Impact is bounded (victim data flows into an attacker-controlled account) so low severity, but the CSRF origin check that guards every other mutating route is deliberately bypassed for login.
- **Fix sketch:** Run the csrf_origin_ok Origin/Referer check for /api/auth/login even though the session gate is skipped, rejecting login POSTs whose Origin/Referer does not match the server origin when enforce_csrf is on.

## Refuted (not real / already handled)

(none)
