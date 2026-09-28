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
docker compose up -d db
cp .env.example .env
```

### Run the app

```bash
cargo leptos watch
```

The app server-renders each page and hydrates it in the browser, so forms and controls
remain interactive. Open [http://127.0.0.1:3000](http://127.0.0.1:3000) and log in as
**Admin** (seeded by migration).

### Full stack in Docker

```bash
docker compose up --build
```
