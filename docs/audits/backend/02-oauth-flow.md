# OAuth authorization flow

4 confirmed (1 medium, 3 low unverified), 0 refuted.

## Confirmed findings

### MEDIUM [security]: Refresh-token rotation has no reuse-detection / family revocation

- **Attack/trigger:** Attacker steals refresh token RT1, redeems it at POST /oauth/token (grant_type=refresh_token), gets RT2 + live access token; RT1 becomes revoked. The legit client later redeems its copy of RT1, gets invalid_grant and re-authenticates, but the server takes no action against the attacker's RT2/access token, so the thief retains persistent silent access (the RFC 6819 5.2.2.3 replay scenario rotation is meant to catch).
- **Location:** server/src/oauth/routes.rs:778-830
- **What happens:** `exchange_refresh_token` rotates the refresh token (revokes the presented token and issues a new access+refresh pair). When a previously-rotated (already revoked) refresh token is presented again, the code returns invalid_grant because `stored.revoked_at.is_some()`, but never revokes the still-active descendant tokens issued from that same grant chain. `rotate_oauth_refresh_token` in server/src/db/oauth.rs:425 only revokes the single presented token id; there is no grant/family id linking successive refresh tokens.
- **Why:** Refresh tokens default to a 30-day TTL. Without reuse detection the core benefit of rotation (bounding damage from a stolen refresh token) is lost, giving a thief indefinite access to the victim's MCP data.
- **Fix sketch:** Add a shared grant/family id across rotated refresh tokens; on presentation of an already-revoked but non-expired refresh token, revoke every access+refresh token in that family (inside the existing rotate transaction) and return invalid_grant.

### LOW [security]: Authorization-code replay does not revoke tokens already issued from that code (unverified)

- **Attack/trigger:** If a code leaks (referrer/logs) and is redeemed twice, the second attempt is rejected but the first redemption's tokens stay valid. PKCE (S256, enforced) requires the code_verifier, so a bare code leak is not sufficient to redeem, which keeps this low.
- **Location:** server/src/oauth/routes.rs:643-683
- **What happens:** Single-use is enforced via `stored.used_at` check and an atomic UPDATE that rejects reuse. But on a detected replay of an already-used code the handler only returns invalid_grant; it never revokes the access/refresh tokens minted during the first exchange, contrary to RFC 6749 4.1.2.
- **Why:** Defense-in-depth. PKCE materially reduces exploitability, but a replayed code should still invalidate any parallel malicious issuance.
- **Fix sketch:** When `stored.used_at.is_some()` for an otherwise-matching code, revoke all access/refresh tokens for (client_id, user_id, code) and return invalid_grant.

### LOW [security]: Username enumeration via differential response/timing in authorize_post (unverified)

- **Attack/trigger:** Attacker POSTs to the public /oauth/authorize with a valid client_id/redirect_uri/PKCE, approve=yes, a guessed username and arbitrary password. Near-instant 401 => existing disabled account; 429 => existing locked account; ~200ms 401 => username does not exist. Iterating enumerates valid usernames.
- **Location:** server/src/oauth/routes.rs:462-494
- **What happens:** The combined login+consent POST /oauth/authorize leaks account state before verifying the password. A non-existent username runs a dummy argon2 verify plus a fixed 200ms sleep. A disabled/must_change_password account returns 401 immediately with no dummy hash and no delay; a locked account returns 429. These divergent status codes and latencies distinguish 'no such user' from 'user exists'.
- **Why:** Enables reliable account enumeration on the MCP OAuth login surface, aiding targeted credential attacks. Low because it does not bypass auth.
- **Fix sketch:** Move the disabled/must_change_password/locked checks to after password verification (or apply the same dummy-hash + fixed-delay path) and return a single generic access_denied for all failure modes.

### LOW [correctness]: Authorization response builder blindly appends '?code=', corrupting redirect URIs that already contain a query string (unverified)

- **Attack/trigger:** A legitimately registered client whose redirect_uri includes a query string completes login/consent; the Location header has a double '?', so its callback cannot parse the code and the flow breaks. Interop/functional defect, not attacker-driven.
- **Location:** server/src/oauth/routes.rs:549-555
- **What happens:** The success redirect is built with `format!("{}?code={}", form.redirect_uri, ...)` then `&state=` is appended. `redirect_uri` is exact-matched against a registered value, but registered URIs may contain query components (validate_redirect_uri only rejects fragments/userinfo). A registered `https://client.example/cb?tenant=42` yields `https://client.example/cb?tenant=42?code=...&state=...`, a malformed response with two '?'.
- **Why:** Breaks otherwise-valid clients and violates RFC 6749 3.1.2 (append params preserving existing query). No injection risk (code/state are urlencoded), so low, but real correctness impact.
- **Fix sketch:** Parse redirect_uri with Url and use `query_pairs_mut().append_pair("code", &code)/append_pair("state", state)` so the '?' vs '&' separator is chosen correctly.
