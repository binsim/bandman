# Bandman

[![CI](https://github.com/binsim/bandman/actions/workflows/ci.yml/badge.svg)](https://github.com/binsim/bandman/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)
[![Leptos](https://img.shields.io/badge/leptos-0.8-blue.svg)](https://leptos.dev/)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](#license)

## Quick start (local)

### Prerequisites

- Rust (stable), `wasm32-unknown-unknown` target
- [`cargo-leptos`](https://github.com/leptos-rs/cargo-leptos)
- Dart Sass (`sass` on `PATH`) — used by cargo-leptos for SCSS
- Docker (for Postgres)

```bash
npm install --global sass@1.105.0
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos --locked
```

### Database

```bash
cp .env.example .env
docker compose up -d db
```

For an optional database GUI, start only the database and pgAdmin with
`docker compose --profile dev up -d db pgadmin`.
Open [http://127.0.0.1:5050](http://127.0.0.1:5050) and sign in with
`admin@bandman.dev` / `bandman`. The `bandman` database is pre-registered and connects
automatically using the development database credentials.

### Run the app

```bash
cargo leptos watch
```

The app server-renders each page and hydrates it in the browser, so forms and controls
remain interactive. Open [http://127.0.0.1:3000](http://127.0.0.1:3000) and log in as
**Admin** (seeded by migration).

### Publish with Docker Compose

```bash
docker compose up -d
```

Before publishing, replace the development database credentials and `SESSION_SECRET` in
`.env` with secure values. PostgreSQL is bound to localhost; it is available to the app
over Docker's internal network.
