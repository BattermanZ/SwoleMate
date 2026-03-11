# SwoleMate client

SvelteKit + Skeleton UI frontend for SwoleMate.

## Development

```bash
npm install
npm run dev
```

The dev server runs on `http://127.0.0.1:2470` and proxies `/api`, `/mcp`, `/oauth`, and `/.well-known` to the backend by default.
If the Rust backend is running elsewhere, set `VITE_API_PROXY_TARGET` to that origin.

## Building

To create a production version of your app:

```bash
npm run build
```

You can preview the production build with `npm run preview`.

## Notes

- Auth uses a long-lived HttpOnly cookie session; the client always calls the API with `credentials: 'include'`.
- SSR is disabled (`src/routes/+layout.ts`) so auth + offline-first behavior stays consistent in SPA/PWA mode.

## Testing

Unit/component tests (Vitest):

```bash
npm run test:unit
```
