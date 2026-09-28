# End-to-end tests (Playwright)

Browser tests for critical server-rendered and hydrated flows.

## Current coverage

- Server-rendered home page and member login
- Theme and language controls
- Admin access, member validation and CRUD, last-admin protections, sorting, and delete confirmation

## Run locally

```bash
# Terminal 1 — Postgres + app
docker compose up -d db
cp -n .env.example .env
cargo leptos watch
```

```bash
# Terminal 2 — Playwright
cd end2end
npm install
npx playwright install chromium
npx playwright test
```

Or let cargo-leptos build the app, start it, and run the tests:

```bash
cd end2end && npm install && npx playwright install chromium && cd ..
DATABASE_URL=******127.0.0.1:5432/bandman \
  SESSION_SECRET=dev \
  cargo leptos end-to-end
```

## Adding tests

1. Prefer `data-testid` hooks over brittle CSS/text-only selectors when possible.
2. Use `gotoHydrated()` from `tests/helpers.ts`.
3. Keep `workers: 1` to avoid shared-database test interference.
