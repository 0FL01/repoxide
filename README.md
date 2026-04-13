# Repoxide

Rust-first monorepo for Repoxide.

## Layout

- `crates/repoxide` — primary Rust CLI/library
- `apps/web/server` — active Rust web app and API
- `apps/web/client` — retired VitePress/Vue frontend kept in-tree during migration
- `legacy/repoxide-ts` — legacy TypeScript CLI/library
- `legacy/website-server-ts` — legacy TypeScript web backend

## Quick start

```bash
# Rust CLI from the repo root
cargo run -- --help

# Release build
cargo build --release

# Active web stack
docker compose up --build
```

The Rust CLI is the default workspace member, so root-level `cargo build`, `cargo test`, and `cargo run -- ...` target `crates/repoxide`.

## Web app

- Active web UI and API: `apps/web/server`
- Retired frontend implementation: `apps/web/client`
- Local stack: `docker-compose.yml`

The repository compose file starts the Rust web app on `127.0.0.1:93`.

## Legacy TypeScript code

The old TypeScript implementation is preserved under `legacy/`.

```bash
cd legacy/repoxide-ts
npm ci
npm run build
```

Legacy TS web stack:

```bash
docker compose -f legacy/compose.yml up --build
```
