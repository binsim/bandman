# Bandman

[![CI](https://github.com/binsim/bandman/actions/workflows/ci.yml/badge.svg)](https://github.com/binsim/bandman/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)
[![Leptos](https://img.shields.io/badge/leptos-0.8-blue.svg)](https://leptos.dev/)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](#license)

Band management web app written in **Rust** (Leptos + Axum), running on **Docker** with **PostgreSQL**.

Members log in by **name** (no email/passwords). UI supports **English / German** and **light / dark** themes.

## Status

| Area | Status | Notes |
| --- | --- | --- |
| Project foundation (Leptos/Axum, SCSS, Docker) | Done | See `docker-compose.yml` |
| Database + `members` types | Done | SQLx migrations + [`src/models/member.rs`](src/models/member.rs) |
| Name-based login | Done | [`src/pages/login/`](src/pages/login/) |
| Theming + color palette | Done | [`style/`](style/) — teal accent, light/dark via `data-theme` |
| i18n (EN / DE) | Done | [`locales/`](locales/) via leptos-fluent |
| CI (GitHub Actions) | Done | [`.github/workflows/ci.yml`](.github/workflows/ci.yml) |
| Playwright E2E | Done | [`end2end/`](end2end/) — login, theme, i18n |
| Wishlist | Not started | [Plan](src/pages/wishlist/README.md) |
| Program (PDFs + live sync) | Not started | [Plan](src/pages/program/README.md) |
| Rehearsal plan | Not started | [Plan](src/pages/plan/README.md) |
| Finance | Deferred | [Plan](src/pages/finance/README.md) |
| Admin (members) | Not started | [Plan](src/pages/admin/README.md) |

Each page folder under `src/pages/*/` has a **README** describing intended behaviour so features are not forgotten.

## Stack

- **Leptos 0.8** (SSR + WASM hydrate) + **Axum**
- **PostgreSQL** + **SQLx** migrations
- **SCSS** + CSS variables (no Tailwind)
- **leptos-fluent** for EN/DE
- **Docker Compose** (`app` + `db`)

## Quick start (local)

### Prerequisites

- Rust (stable), `wasm32-unknown-unknown` target
- [`cargo-leptos`](https://github.com/leptos-rs/cargo-leptos)
- Dart Sass (`sass` on `PATH`) — used by cargo-leptos for SCSS
- Docker (for Postgres)

```bash
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos --locked
# optional: npm i -g sass
```

### Database

```bash
docker compose up -d db
cp .env.example .env
```

### Run the app

```bash
cargo leptos watch
```

Open [http://127.0.0.1:3000](http://127.0.0.1:3000). Log in as **Admin** (seeded by migration).

### Full stack in Docker

```bash
docker compose up --build
```

## Continuous integration

On every push/PR to `main` or `master`, GitHub Actions runs:

| Job | What it does |
| --- | --- |
| `fmt` | `cargo fmt --check` |
| `clippy` | Clippy for SSR and WASM hydrate (`-D warnings`) |
| `check` | `cargo check` for SSR + WASM |
| `test` | `cargo test` (SSR) with a Postgres service |
| `e2e` | Playwright (Chromium) against a built Leptos server + Postgres |

Locally:

```bash
cargo fmt --all -- --check
cargo clippy --features ssr --no-default-features -- -D warnings
cargo test --features ssr --no-default-features

# E2E (app must be reachable on :3000, or use cargo leptos end-to-end)
cd end2end && npm install && npx playwright install chromium && npx playwright test
```

## Branch protection (GitHub settings)

Branch protection is configured in the GitHub UI (not in this repo). After the first successful CI run on the default branch:

1. Open the repo → **Settings** → **Branches** (or **Rules** → **Rulesets** on newer GitHub).
2. Add a rule for `main` (or `master`).
3. Enable:
   - **Require a pull request before merging** (recommended)
   - **Require status checks to pass before merging**
   - **Require branches to be up to date before merging** (strict; recommended)
4. In the status checks search box, select these job names exactly (they appear after CI has run at least once):
   - `fmt`
   - `clippy`
   - `check`
   - `test`
   - `e2e`
5. Optionally enable **Do not allow bypassing the above settings** (admins included) and block force pushes / deletions.

If you use **Rulesets** instead of classic branch protection: create a ruleset targeting `main`/`master`, add the same required status checks, and set the enforcement to **Active**.

## Project layout

```
src/
  auth/           # login server functions + session cookie
  components/     # layout, theme toggle, language switcher
  models/         # Member types
  pages/          # one folder per page (+ README plans)
  db.rs           # Postgres pool + migrations (SSR)
  app.rs          # routes + shell
migrations/       # SQLx SQL
locales/en|de/    # Fluent translations
style/            # SCSS tokens + layout
.github/workflows # CI
```

## Environment

| Variable | Purpose |
| --- | --- |
| `DATABASE_URL` | Postgres connection string |
| `SESSION_SECRET` | HMAC secret for session cookies |
| `COOKIE_SECURE` | Set `true` behind HTTPS |
| `LEPTOS_SITE_ADDR` | Bind address (Docker: `0.0.0.0:3000`) |

## License

Private / undecided — add a license when you publish the repo.
