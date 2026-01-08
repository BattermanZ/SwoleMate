# SwoleMate - Your Personal Fitness Tracking App

SwoleMate is a modern fitness tracking application that helps you monitor and visualize your workout progress over time.

## Features

- Track exercises with detailed information (type, sets, reps, weights, notes)
- Visualize progress through interactive graphs
- Compare performance across different time periods
- PWA support for iOS devices
- Offline functionality

## Tech Stack

### Frontend
- Svelte + SvelteKit
- Skeleton UI
- TailwindCSS
- Chart.js for data visualization
- PWA support with service workers

### Backend
- Rust with Actix-web framework
- SQLite database
- JSON structured logging
- RESTful API

## Getting Started

### Prerequisites
- Node.js (v20.19+ / v22.12+ / v24+)
- Rust (latest stable)
- SQLite

### Backend Setup
1. Navigate to the server directory:
   ```bash
   cd server
   ```
2. Install Rust dependencies and run the server:
   ```bash
   cargo run
   ```

### Frontend Setup
1. Navigate to the client directory:
   ```bash
   cd client
   ```
2. Install dependencies:
   ```bash
   npm install
   ```
3. Start the development server:
   ```bash
   npm run dev
   ```

## Development

- Frontend runs on `http://localhost:2470`
- Backend API runs on `http://localhost:2469`

## Security / Production

When exposing the backend behind an HTTPS reverse proxy, configure API auth:

- `APP_ENV=production` (enables stricter startup checks)
- `SWOLEMATE_API_TOKEN=...` (required in production; sent as `Authorization: Bearer ...`)
- `SWOLEMATE_ADMIN_TOKEN=...` (optional; used for `/api/backups*` and `/api/logs*`)

Optional hardening knobs:

- `CORS_ALLOWED_ORIGINS=https://example.com,https://www.example.com` (override single `FRONTEND_URL`)
- `ENABLE_HSTS=1` + `HSTS_MAX_AGE=31536000` (adds `Strict-Transport-Security`; only enable if you always serve via HTTPS)
- `API_MAX_INFLIGHT`, `API_MAX_INFLIGHT_LOGS`, `API_MAX_INFLIGHT_BACKUPS`, `API_MAX_INFLIGHT_BACKUP_RESTORE`, `API_CONCURRENCY_TIMEOUT_MS`

## Project Structure

```
swolemate/
├── client/          # Frontend Svelte application
├── server/          # Rust backend
│   ├── src/
│   │   ├── models/  # Data models
│   │   ├── routes/  # API endpoints
│   │   └── db/      # Database operations
└── README.md
``` 
