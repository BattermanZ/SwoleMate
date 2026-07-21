# Rate limiting (login & MCP)

3 confirmed (1 medium security, 2 low unverified), 0 refuted.

## Confirmed findings

### MEDIUM [security]: MCP rate limiter keyed per-token/per-client, not per-user — a user multiplies their budget by minting extra MCP tokens

- **Attack/trigger**: An authenticated user creates N personal MCP tokens from /settings (or, with allow_dynamic_client_registration, registers N OAuth clients and authorizes each). Each token/client yields a different client_id, hence a separate bucket. Their effective request ceiling becomes N x MCP_RATE_LIMIT_PER_MINUTE (default 60) instead of a single per-user 60/min, defeating the throttle whose purpose is to cap a single account's/agent's request volume against the DB.
- **Location**: `server/src/mcp/routes.rs:825`
- **What happens**: The MCP per-minute limiter keys the counter on `format!("{}:{}", principal.client_id, principal.user_id)`, then admits against that key in mcp/rate_limit.rs:admit_request. For personal MCP tokens the middleware sets `client_id = format!("mcp_token:{}", token_row.id)`, so every distinct personal token produces a distinct rate-limit bucket even though they all belong to the same user. Personal MCP tokens are self-service with no visible per-user cap. The same is true for OAuth clients when dynamic client registration is enabled.
- **Why**: The limiter is intended to bound how hard one authenticated principal (an AI agent) can hammer the backend. Keying on the token/client id rather than user_id makes the cap trivially multipliable by the same actor, so the DoS protection it provides per account is only as strong as one token's worth.
- **Fix sketch**: Key the MCP limiter on principal.user_id alone (or add a second per-user bucket alongside the per-token one) so all of a user's tokens share one budget, and/or cap the number of active MCP tokens per user.

### LOW [security] (unverified): request_ip trusts X-Real-IP unconditionally — direct exposure allows login rate-limit bypass and unbounded map growth

- **Attack/trigger**: With the port reachable without the bundled nginx, an attacker sends each brute-force login with a fresh forged X-Real-IP. Because is_ip_rate_limited/record_ip_failure key on that value, every request is a brand-new IP: the per-IP failure counter never accumulates, so the limiter is fully bypassed for password spraying and username enumeration. Simultaneously, each distinct forged IP inserts a new HashMap entry in LOGIN_FAILURES_BY_IP; evict_stale only drops entries older than the window (default 10 min), so an attacker holds unbounded distinct keys within a window -> memory-exhaustion DoS.
- **Location**: `server/src/auth/rate_limit.rs:26`
- **What happens**: request_ip prefers the client-supplied X-Real-IP header over the real peer socket with no trusted-proxy gate. This value is the sole key for the login/OAuth per-IP failure limiter. The bundled nginx overwrites the header with $remote_addr, so behind that proxy it is safe. But the server binds 0.0.0.0; if the port is ever published directly (or a proxy that does not overwrite the header is used), the header becomes attacker-controlled.
- **Why**: The per-IP limiter is the only brute-force defense for non-existent usernames and for password spraying across many accounts. Trusting a forgeable header removes that defense and adds an unbounded-map DoS the moment the deployment topology differs from the assumed nginx setup. Prior audit SEC-3 flagged the same trust.
- **Fix sketch**: Gate X-Real-IP trust behind an explicit TRUSTED_PROXY env flag (or verify req.peer_addr() is within a configured trusted-proxy CIDR before honoring the header); otherwise fall back to peer_addr().

### LOW [correctness] (unverified): Login limiter check-then-record is non-atomic (two separate lock acquisitions), unlike the MCP limiter

- **Attack/trigger**: Multiple concurrent login attempts from the same IP each acquire the read lock, all observe a count below max_attempts before any has recorded its failure, and all proceed. Under concurrency the effective number of admitted failing attempts can exceed the configured LOGIN_RATE_LIMIT_ATTEMPTS by roughly the in-flight concurrency.
- **Location**: `server/src/routes/auth.rs:37`
- **What happens**: The login/OAuth flow reads the counter with is_ip_rate_limited (its own Mutex lock) at routes/auth.rs:37 / oauth/routes.rs:454, and only later records a failure via record_ip_failure (a second, separate lock) at routes/auth.rs:47/71 and oauth/routes.rs:466/503. There is no single critical section spanning check-and-record. By contrast the MCP limiter's admit_request does check-and-push atomically under one lock.
- **Why**: A genuine time-of-check/time-of-use gap that lets the configured limit be overshot; impact is bounded because each attempt still performs an expensive argon2 verify (self-throttling on CPU) and per-user DB lockout is a second layer, so it is a minor hardening/correctness gap.
- **Fix sketch**: Fold the check and increment into a single admit-style function that takes one lock and both counts and records (mirroring MCP admit_request).

## Refuted (not real / already handled)

None.
