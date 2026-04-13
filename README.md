# Repoxide

Rust-first monorepo for Repoxide.

## Layout

- `crates/repoxide` — primary Rust CLI/library
- `apps/web/server` — active Rust web app and API
- `apps/web/client` — retired VitePress/Vue frontend kept in-tree for docs/dev tooling and static assets

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

## Notes

The historical TypeScript implementation has been removed. The maintained runtime and CLI are the Rust targets in this repository.
