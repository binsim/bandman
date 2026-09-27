# Build stage: compile Leptos SSR binary + WASM/site assets
FROM rust:1.98-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev clang lld \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown \
    && cargo install cargo-leptos --locked

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY style ./style
COPY locales ./locales
COPY migrations ./migrations
COPY public ./public

ENV DATABASE_URL=postgres://bandman:bandman@localhost:5432/bandman
RUN cargo leptos build --release

# Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/bandman /app/bandman
COPY --from=builder /app/target/site /app/site
COPY migrations /app/migrations

ENV LEPTOS_SITE_ROOT=site \
    LEPTOS_SITE_ADDR=0.0.0.0:3000 \
    RUST_LOG=info

EXPOSE 3000
CMD ["/app/bandman"]
