# SwoleMate Remote MCP Architecture

## Goal

Bundle a remote Model Context Protocol (MCP) server into SwoleMate so users can connect to it from AI services over the public app domain.

Primary targets:

- OpenAI-compatible remote MCP clients
- Anthropic-compatible remote MCP clients
- Google MCP-capable clients

This document defines the recommended architecture for a first production-ready implementation.

## Non-Goals

Version 1 does not aim to expose:

- admin user management over MCP
- backup creation, deletion, or restore over MCP
- destructive delete operations for workouts or exercises
- raw database or filesystem access

## Current App Structure

SwoleMate currently consists of:

- a SvelteKit frontend served by nginx
- a Rust Actix backend
- a SQLite database
- cookie-based browser authentication

Current public routing:

- `/` serves the web app
- `/api/...` serves the existing JSON API

This is a good base for MCP, but the current browser session model should not be reused for remote MCP clients.

## Recommended Architecture

Implement MCP inside the Rust backend and expose it over the same HTTPS domain as the app.

Public routes:

- `/` -> existing frontend
- `/api/...` -> existing JSON API
- `/mcp` -> primary MCP endpoint
- `/sse` -> optional compatibility transport
- `/oauth/authorize`
- `/oauth/token`
- `/.well-known/oauth-authorization-server`
- `/.well-known/oauth-protected-resource`

The frontend nginx container should proxy these routes to the Rust backend.

## Why MCP Should Live in the Backend

The backend already owns:

- user identities
- authorization rules
- workout data
- progress data
- audit-worthy operations

Keeping MCP in the backend avoids:

- duplicating business logic in another service
- tunneling machine clients through browser cookie auth
- adding a separate deployment unless it becomes necessary later

## Transport

Primary transport:

- Streamable HTTP on `/mcp`

Optional compatibility transport:

- SSE on `/sse`

The initial implementation should be stateless:

- every MCP request is authenticated independently
- no backend MCP session affinity is required

This is simpler to operate behind a reverse proxy and more robust across client products.

## Authentication Model

Remote MCP must use OAuth bearer tokens, not browser cookies.

Why:

- browser cookie auth is designed for the web UI
- current session middleware includes browser-oriented assumptions
- remote MCP clients need a standard delegated authorization flow

OAuth flow:

1. User adds the SwoleMate MCP server URL to an AI client.
2. The client discovers the authorization requirements.
3. The user is redirected to SwoleMate login and consent pages.
4. The user approves the requested scopes.
5. The AI client receives an access token.
6. The AI client calls `/mcp` using `Authorization: Bearer <token>`.

## Separation of Auth Concerns

The app should maintain two auth systems side by side:

- browser auth for the web app using existing cookie sessions
- MCP auth using OAuth access tokens

These should not share middleware.

Current browser auth remains responsible for:

- `/`
- `/api/...`
- current user login/logout/password flows

New MCP auth is responsible for:

- `/mcp`
- `/sse`
- OAuth discovery and token flows

## Internal Refactor Required

Before adding MCP tools, move business logic out of HTTP route handlers into shared services.

Recommended new modules:

- `server/src/services/mod.rs`
- `server/src/services/workouts.rs`
- `server/src/services/exercises.rs`
- `server/src/services/progress.rs`
- `server/src/services/authz.rs`

Pattern:

- HTTP route parses request
- route calls service
- service validates input and permissions
- service performs DB work
- route serializes result

The MCP layer should call the same services.

This prevents drift between `/api` behavior and `/mcp` behavior.

## New Backend Modules

Recommended additions under `server/src/`:

- `mcp/mod.rs`
- `mcp/routes.rs`
- `mcp/tools.rs`
- `mcp/resources.rs`
- `mcp/auth.rs`
- `oauth/mod.rs`
- `oauth/models.rs`
- `oauth/routes.rs`
- `oauth/service.rs`
- `audit.rs`

Likely existing files to update:

- `server/src/lib.rs`
- `server/src/main.rs`
- `server/src/routes.rs`
- `server/src/schema.rs`
- `server/src/db/mod.rs`

## OAuth Data Model

Add the following tables.

### `oauth_clients`

- `id`
- `client_id`
- `client_secret_hash`
- `client_name`
- `redirect_uris_json`
- `scopes_json`
- `created_at`
- `disabled_at`

Notes:

- `client_secret_hash` may be nullable for public clients

### `oauth_authorization_codes`

- `id`
- `code_hash`
- `client_id`
- `user_id`
- `redirect_uri`
- `scopes_json`
- `pkce_code_challenge`
- `pkce_method`
- `expires_at`
- `used_at`
- `created_at`

### `oauth_access_tokens`

- `id`
- `token_hash`
- `client_id`
- `user_id`
- `scopes_json`
- `expires_at`
- `revoked_at`
- `created_at`

### `oauth_refresh_tokens`

- `id`
- `token_hash`
- `client_id`
- `user_id`
- `scopes_json`
- `expires_at`
- `revoked_at`
- `created_at`

### `oauth_consents`

- `id`
- `user_id`
- `client_id`
- `scopes_json`
- `granted_at`
- `revoked_at`

### `mcp_audit_log`

- `id`
- `timestamp`
- `user_id`
- `client_id`
- `tool_name`
- `success`
- `error_code`
- `input_summary_json`
- `ip_address`
- `user_agent`

## OAuth Endpoints

Required endpoints:

- `GET /oauth/authorize`
- `POST /oauth/authorize`
- `POST /oauth/token`
- `GET /.well-known/oauth-authorization-server`
- `GET /.well-known/oauth-protected-resource`

Version 1 behavior:

- use SwoleMate login for user authentication
- show a consent page before granting access
- issue short-lived access tokens
- store tokens server-side as hashed opaque tokens

JWTs are not required for version 1.

## Scope Model

Start with a small scope set:

- `workouts.read`
- `progress.read`
- `workouts.write`

Rules:

- a token must include the required scope
- the authenticated user still only accesses their own data
- admin MCP scopes should not exist in version 1

## MCP Surface

Version 1 should prioritize tools over richer MCP features for maximum compatibility.

### Version 1 Read-Only Tools

#### `list_workouts`

Input:

- `limit?: number`
- `from_date?: string`
- `to_date?: string`

Required scope:

- `workouts.read`

#### `get_workout`

Input:

- `id: number`

Required scope:

- `workouts.read`

#### `get_last_exercise_data`

Input:

- `exercise_type: string`

Required scope:

- `workouts.read`

#### `get_exercise_progress`

Input:

- `exercise_type: string`

Required scope:

- `progress.read`

#### `get_workout_stats`

Input:

- none

Required scope:

- `progress.read`

#### `get_volume_stats`

Input:

- `exercise_type: string`

Required scope:

- `progress.read`

### Version 2 Write Tools

#### `create_workout`

Input:

- `date: string`
- `start_time: string`
- `notes?: string`
- `timezone_offset_minutes?: number`

Required scope:

- `workouts.write`

#### `add_exercise`

Input:

- `workout_id: number`
- `exercise_type: string`
- `start_time: string`
- `notes?: string`
- `per_side_weight?: boolean`
- `split_weight?: boolean`
- `settings?: Array<{ key: string; value: string }>`

Required scope:

- `workouts.write`

#### `replace_sets`

Input:

- `exercise_id: number`
- `sets: Array<{ reps: number; weight: number; weight_left?: number; weight_right?: number; notes?: string }>`

Required scope:

- `workouts.write`

#### `end_exercise`

Input:

- `id: number`
- `end_time: string`
- `notes?: string`
- `per_side_weight?: boolean`
- `split_weight?: boolean`
- `settings?: Array<{ key: string; value: string }>`

Required scope:

- `workouts.write`

#### `end_workout`

Input:

- `id: number`
- `end_time: string`
- `notes?: string`
- `feedback?: string`

Required scope:

- `workouts.write`

## Excluded MCP Operations in Version 1

Do not expose:

- user creation, disable, deletion, or password reset
- backup listing, creation, deletion, or restore
- exercise delete
- workout delete
- client log ingestion

These are either high-risk or not useful enough for a first public MCP release.

## MCP Authorization Middleware

Add dedicated bearer-token middleware for MCP requests.

Responsibilities:

- read `Authorization: Bearer ...`
- hash the supplied token
- load token record
- verify expiry
- verify revocation
- load user
- load granted scopes
- inject principal into request context

Suggested principal model:

- `user_id`
- `client_id`
- `scopes`

This middleware must be separate from the current cookie session middleware.

## Reverse Proxy Changes

Update nginx to proxy:

- `/api/`
- `/mcp`
- `/sse`
- `/oauth/`
- `/.well-known/`

Additional notes:

- forward `Host`
- forward `X-Forwarded-Proto`
- forward client IP headers
- disable buffering on `/sse` if that transport is enabled

## Configuration Additions

Extend backend configuration with:

- `MCP_PUBLIC_BASE_URL`
- `MCP_ENABLE_SSE`
- `MCP_REQUIRE_OAUTH`
- `OAUTH_ACCESS_TOKEN_TTL_SECONDS`
- `OAUTH_REFRESH_TOKEN_TTL_SECONDS`
- `OAUTH_DEFAULT_SCOPES`
- `OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION`
- `MCP_RATE_LIMIT_PER_MINUTE`

Suggested version 1 defaults:

- `MCP_ENABLE_SSE=true`
- `MCP_REQUIRE_OAUTH=true`
- `OAUTH_ACCESS_TOKEN_TTL_SECONDS=3600`
- `OAUTH_REFRESH_TOKEN_TTL_SECONDS=2592000`
- `OAUTH_DEFAULT_SCOPES=workouts.read,progress.read`
- `OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION=false`

## Audit and Security Requirements

Every MCP tool call should be audit-logged.

Do not log:

- raw access tokens
- refresh tokens
- client secrets
- full sensitive payloads

Security controls for version 1:

- HTTPS only in production
- short-lived access tokens
- hashed token storage
- route-level scope checks
- user ownership checks
- MCP-specific rate limiting
- no admin MCP capabilities

## Delivery Plan

### Phase 1

- extract shared services from existing routes
- add OAuth schema and DB accessors
- add OAuth endpoints
- add MCP bearer auth middleware
- add read-only MCP tools
- add audit logging
- add reverse proxy routes

### Phase 2

- add write MCP tools
- improve scope enforcement and error reporting
- test against OpenAI and Anthropic remote MCP clients
- test against Google MCP-capable clients

### Phase 3

- improve consent UX
- add optional SSE transport if not included earlier
- consider dynamic client registration only if required by target clients

## Version 1 Acceptance Criteria

Version 1 is complete when:

1. A remote MCP client can connect to `https://<your-domain>/mcp`.
2. The client can complete OAuth authorization.
3. The client can call read-only workout and progress tools successfully.
4. Results are scoped to the authenticated SwoleMate user only.
5. MCP requests are bearer-token authenticated and audit-logged.
6. Existing browser login and `/api` behavior remain unchanged.

## Recommended Next Step

Implement Phase 1 only.

That gives the smallest useful public MCP:

- remote connectivity
- delegated auth
- user-scoped read access
- compatibility testing with target AI platforms

Only after that should write tools be added.
