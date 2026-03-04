# SwoleMate (v2.1.0)

SwoleMate is a self-hosted workout logging web app designed for fast, phone-first session tracking:

- Log a workout as you train (exercises, sets, notes, equipment settings).
- Review recent sessions in History (including details) to quickly remember what you did.
- Use Progress charts to see trends (strength, volume, time-of-day patterns).
- Keep your data local in a SQLite database and export/restore backups.

It runs as:
- A frontend (SvelteKit SPA/PWA) served as static files.
- A backend (Rust + Actix) that stores data in SQLite and provides a JSON API.

## How the app works (non-technical overview)

- You sign in with a username and password.
- You start a session, add exercises, then add sets and notes.
- When you mark an exercise done, its inputs are locked unless you explicitly edit it again.
- If you forget to end a session, the server can automatically close it after inactivity (configurable). Auto-closed sessions are marked so you can adjust the times later.
- Your data stays on your server. Backups are available to restore or download.

## Key features

- Phone-first session logging (sets, notes, optional equipment settings, per-side weights).
- History and session details (including the ability to edit start/end times, notes, and rating).
- Progress charts (exercise focus and overall trends).
- Admin-only user management and admin-only backups.
- Automatic backups and a retention policy designed to keep a useful set of restore points.
- Optional offline logging on the Today page (local-first queue + later sync).

## Tech stack (for developers)

**Frontend**
- SvelteKit SPA (SSR disabled for consistency with cookie auth + PWA/offline behavior)
- Skeleton UI + Tailwind
- Chart.js
- Service worker (production only)

**Backend**
- Rust + Actix-web
- SQLite via SQLx (offline query metadata checked into `server/.sqlx/`)
- Cookie-based sessions (HttpOnly, SameSite=Lax; Secure in production)
- Logs to stdout (Docker-friendly)

## Running locally (development)

**Backend**
```bash
cd server
cargo run
```

**Frontend**
```bash
cd client
npm install
npm run dev -- --host 0.0.0.0
```

Default ports:
- Frontend: `http://localhost:2470`
- Backend: `http://localhost:2469`

### Development API routing (Vite proxy)

In development, the frontend can call the backend via same-origin `/api/...` and Vite will proxy it to the backend.

- `client/vite.config.ts` proxies `/api` to `VITE_API_PROXY_TARGET` (default `http://127.0.0.1:2469`).
- `client/.env.local` is a convenient place for local-only settings:
  - `VITE_API_URL=` (empty means the frontend uses same-origin `/api`)
  - `VITE_API_PROXY_TARGET=http://127.0.0.1:2469`

`client/.env.local` is for development only and should not be baked into production images.

## Auth and roles

SwoleMate uses username/password authentication with cookie sessions.

- **Admin** users can manage users and backups.
- **Normal** users can log and view their own data, but cannot access admin endpoints.

Password rules:
- Passwords must be at least 12 characters.

First-login password change:
- On a brand-new database, the bootstrap admin account is created with a forced password change on first login.
- When an admin resets another user’s password, that user is also forced to change their password on the next login.
- For existing (legacy) databases upgraded to v2.1.0, users are not forced to change passwords by default.

## Data and backups

**Database**
- SQLite file (default `database/swolemate.db`).
- The backend writes WAL/SHM files next to the DB when WAL is enabled.

**Backups**
- Backups are `.tar.gz` archives containing a consistent DB snapshot.
- Auto backups run weekly (Monday at 01:00 local time).
- Auto backup retention (max 12):
  - Keep the most recent 6 weekly auto backups.
  - Keep 1 monthly checkpoint for each of the last 6 months (earliest auto backup in the month).

## Configuration

Backend configuration is via environment variables. See `server.env.example` for the full list.

Common variables:
- `SERVER_PORT` (default `2469`)
- `DATABASE_URL` (default `sqlite:database/swolemate.db`)
- `APP_ENV` (`development` or `production`)
- `BOOTSTRAP_ADMIN_USERNAME` (default `admin`)
- `BOOTSTRAP_ADMIN_PASSWORD` (required in production)
- `CORS_ALLOWED_ORIGINS` (recommended for production behind HTTPS)
- `ENABLE_HSTS` / `HSTS_MAX_AGE` (only if you always serve HTTPS)
- `AUTO_CLOSE_INACTIVITY_MINUTES` and `AUTO_CLOSE_POLL_SECONDS` (session auto-close)

## Docker and reverse proxy (production)

Recommended production routing is same-origin:

- Serve the frontend at `https://your-domain/`
- Proxy `https://your-domain/api/...` to the backend container on port `2469`

This avoids hardcoding a backend URL in the frontend build and keeps cookies and CORS simple.

Notes:
- In production (`APP_ENV=production`), session cookies are Secure, so you must serve the app via HTTPS.
- Distroless backend images run without a shell; make sure you mount writable `database/` and `backups/` volumes.

## Docker Compose (example)

This repo includes an example `docker-compose.yml` that runs:

- `client` on port `2470` (static frontend served by nginx)
- `server` on port `2469` (internal; proxied by the client container or your reverse proxy)

Important details:

- The backend container is built to run as a distroless image (no shell, no package manager). It runs from the working directory `/data`.
- The compose file mounts the host directories into `/data/database` and `/data/backups` so the server can create the SQLite DB and write backups.
- If you are bind-mounting an existing (legacy) database created on the host, you may need to run the server container as your host UID/GID (example: `user: "1000:1000"`) so SQLite can create WAL/SHM files and migrations can write.
- The client container’s nginx config proxies `/api/...` requests to the backend container (so the frontend can call `/api` on the same origin).

If you use an external reverse proxy (recommended for HTTPS), use the same-origin approach:

- `https://your-domain/` serves the client
- `https://your-domain/api/...` proxies to the server on port `2469`

## Testing

**Backend**
```bash
cd server
cargo test
```

**Frontend**
```bash
cd client
npm run test:unit
npm run check
npm run lint
```

## Project structure

```
SwoleMate/
├── client/                 # SvelteKit frontend (static build + nginx container)
├── server/                 # Rust backend
│   ├── src/                # app code (routes, db modules, middleware)
│   ├── tests/              # integration tests
│   └── .sqlx/              # SQLx offline metadata (checked into git)
├── scripts/                # helper scripts (node)
├── docker-compose.yml      # example compose layout
├── server.env.example      # documented backend env vars
└── README.md
```
