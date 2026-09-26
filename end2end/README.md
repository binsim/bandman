# End-to-end tests (Playwright)

**Status:** Foundation coverage in place

## Purpose

Browser tests for critical flows. Prefer waiting for WASM hydration (`networkidle` + visible shell) before interacting — Leptos SSR HTML is not interactive until hydrate finishes.

## Current coverage

- Home loads with brand + login link
- Name-based login as seeded **Admin**
- Login validation when no member selected
- Theme toggle (`data-theme`)
- EN ↔ DE language switch on login title

## Run locally

```bash
# Terminal 1 — Postgres + app
docker compose up -d db
cp -n ../.env.example ../.env
cargo leptos watch

# Terminal 2 — Playwright
cd end2end
npm install
npx playwright install chromium
npx playwright test
```

Or with cargo-leptos (builds app, starts server, runs tests):

```bash
cd end2end && npm install && npx playwright install chromium && cd ..
DATABASE_URL=postgres://bandman:bandman@127.0.0.1:5432/bandman \
  SESSION_SECRET=dev \
  cargo leptos end-to-end
```

## Adding tests

1. Prefer `data-testid` hooks over brittle CSS/text-only selectors when possible.
2. Use `gotoHydrated()` from `tests/helpers.ts`.
3. Keep `workers: 1` for Leptos apps unless you have proven parallel safety.
