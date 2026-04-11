# Repomix

Rust-first monorepo for Repomix.

## Layout

- `crates/repomix` — primary Rust CLI/library
- `apps/web/client` — active web frontend
- `apps/web/server` — active Rust web backend
- `legacy/repomix-ts` — legacy TypeScript CLI/library
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

The Rust CLI is the default workspace member, so root-level `cargo build`, `cargo test`, and `cargo run -- ...` target `crates/repomix`.

## Web app

- Frontend source: `apps/web/client`
- Backend source: `apps/web/server`
- Local stack: `docker-compose.yml`

The repository compose file starts the frontend on `127.0.0.1:93` and keeps the Rust backend internal to Docker.

## Legacy TypeScript code

The old TypeScript implementation is preserved under `legacy/`.

```bash
cd legacy/repomix-ts
npm ci
npm run build
```

Legacy TS web stack:

```bash
docker compose -f legacy/compose.yml up --build
```
