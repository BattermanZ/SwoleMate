# MCP token authentication & the MCP tool surface

**3 confirmed (2 medium-severity verified, 1 low-severity unverified), 0 refuted.**

## Confirmed findings

### MEDIUM [security]: MCP auth middleware performs unrated DB reads/writes per request; rate limiter only guards post-auth RPC dispatch

- **Attack/trigger:** Unauthenticated: flood POST /mcp with a bogus 'Authorization: Bearer x' header; each request forces two DB reads, no rate limiting or auth needed. Authenticated (any registered user can mint a token): flood /mcp with a valid token; every request forces a mcp_tokens UPDATE before the rate limiter is consulted. On single-writer SQLite these writes serialize and contend with all app writes, degrading the whole service.
- **Location:** `server/src/middleware/mcp_auth.rs:99-133`
- **What happens:** For every POST /mcp request with any Authorization: Bearer value, the middleware issues `get_oauth_access_token_by_hash` then `get_mcp_token_by_hash` (two SQLite SELECTs, mcp_auth.rs:100-121). A valid token additionally triggers `touch_mcp_token_last_used` — a SQLite UPDATE — on every request (mcp_auth.rs:131-133). The only rate limiter, `rate_limit::admit_request`, runs later inside `handle_mcp_message` (server/src/mcp/routes.rs:826), AFTER authentication and the per-request UPDATE, and only short-circuits JSON-RPC dispatch. None of the auth-layer DB work is rate limited.
- **Why:** The rate limiter protects tool execution but not the authentication path in front of it, leaving the most abuse-amplifying layer (DB lookups plus a guaranteed per-request write) unthrottled — a low-cost remote DoS against a single-writer datastore.
- **Fix sketch:** Apply IP/token rate limiting inside McpBearerAuth before the DB lookups, and debounce `touch_mcp_token_last_used` (update at most once per N minutes per token) or move it behind the rate-limit gate.

### MEDIUM [security]: Password change does not revoke MCP (or OAuth) bearer tokens, so a leaked token survives the reset

- **Attack/trigger:** An attacker steals a user's `smcp_` token. The user notices and changes their password to lock the attacker out. All web sessions are killed, but the stolen bearer token keeps full `workouts.read/write` and `progress.read` access via POST /mcp until token expiry (default 30, up to 365 days) or manual per-token revoke in /settings.
- **Location:** `server/src/routes/auth.rs:150-152`
- **What happens:** `change_password` updates the password hash and calls `db.revoke_all_sessions_for_user` (auth.rs:151-152) but never revokes the user's `mcp_tokens` or `oauth_access_tokens`. The MCP auth middleware (server/src/middleware/mcp_auth.rs:122-129) only rejects a token when `revoked_at` is set, it is expired, the user is disabled, or `must_change_password` is set — none of which a voluntary password change triggers (`update_password_hash` sets `must_change_password` to false).
- **Why:** Password rotation is the canonical response to compromise; users expect it to sever all active access. Here it severs cookie sessions but leaves bearer-token access — full read/write authority over training data for an AI agent — fully intact.
- **Fix sketch:** In `change_password`, after `update_password_hash`, also revoke the user's live MCP tokens and OAuth access/refresh tokens (`UPDATE mcp_tokens SET revoked_at=CURRENT_TIMESTAMP WHERE user_id=? AND revoked_at IS NULL`, plus OAuth equivalents).

### LOW [correctness]: Token rotation copies the original absolute expires_at instead of issuing a fresh lifetime (unverified)

- **Attack/trigger:** A user created a token with the default 30-day expiry, then rotates it 29 days later (a common reaction to a suspected leak). The new token expires in ~1 day rather than getting a fresh 30-day window, silently breaking the integration shortly after rotation.
- **Location:** `server/src/db/mcp_tokens.rs:172-206`
- **What happens:** `rotate_mcp_token_for_user` reads the existing row's `expires_at` as a stored absolute timestamp (mcp_tokens.rs:172-174) and binds that same value into the new token's INSERT (line 186, returned at 206). Rotation does not reset the expiry window; the replacement token inherits the original's absolute expiry moment.
- **Why:** Rotation is meant to hand out a durable replacement credential; inheriting a nearly-elapsed absolute expiry defeats that and is inconsistent with `create_mcp_token`, which always computes a fresh window (server/src/routes/mcp_tokens.rs:74-82).
- **Fix sketch:** On rotate, recompute `expires_at` from now using the token's intended duration (or re-apply the default window) rather than copying the stored absolute timestamp.

## Refuted (not real / already handled)

*(None.)*
