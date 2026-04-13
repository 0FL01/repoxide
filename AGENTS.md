# Project: Repoxide

Repoxide packs repositories into AI-friendly outputs. This repository is Rust-first: the repo root targets the Rust workspace by default.

Tech stack: Rust 2021 workspace, Axum/Tokio web app/API, server-rendered Rust UI, retired VitePress/Vue client assets, Docker Compose, GitHub Actions.

## Workspace Overview

- `Cargo.toml` - root workspace; default member is `crates/repoxide`; shared release profile and single root `Cargo.lock`.
- `crates/repoxide` - primary Rust CLI/library for packing, config loading, compression, metrics, and output generation.
- `apps/web/client` - retired VitePress/docs workspace; retained for docs/dev tooling and static assets consumed by the Rust server.
- `apps/web/server` - active Rust web app and API; Axum server for `/`, `/en`, `/ru`, `/pack`, `/health`, `/api/pack`, and chunked upload endpoints.
- `docker-compose.yml` - active local stack.

## Where To Look

- `crates/repoxide/src/cli` - CLI entrypoints and argument handling.
- `crates/repoxide/src/config` - config loading and schema handling.
- `crates/repoxide/src/core` - file collection, compression, metrics, and output generation.
- `apps/web/server/src/handlers` - active UI/page/API handlers.
- `apps/web/server/src/views` - server-rendered page templates, component helpers, and form state.
- `apps/web/server/static` - bundled CSS/JS for the web UI.
- `apps/web/client/scripts` - schema generation and retained client tooling.
- `apps/web/client/src/public` - static assets (logo + schemas) consumed by the Rust server.
- `apps/web/client/.vitepress/config.ts` - retired docs/dev shell and local proxy config.

## Architectural Invariants

- Root `cargo build`, `cargo test`, and `cargo run -- ...` must stay usable without extra package flags.
- Keep `Cargo.lock` only at the repository root; do not reintroduce per-member lockfiles.
- The active runtime is `apps/web/server`; it serves both the UI and API. `apps/web/client` is retained for docs/dev tooling and static assets.
- `docker-compose.yml` builds and publishes the Rust web app on `127.0.0.1:93` (`127.0.0.1:93:8080`); there is no separate runtime backend service.
- Public nginx / Cloudflare routing may intentionally sit in front of the published web-app port instead of the app directly.
- The checked-in schema assets live under `apps/web/client/src/public/schemas`; `apps/web/server` serves them from `/schemas/*`.
- ZIP/archive hardening in the active Rust paths must keep traversal, duplicate-entry, and size/nesting protections intact when editing extraction code.
- Prefer rootless container settings and preserve `no-new-privileges`, `cap_drop`, `tmpfs`, and readonly patterns unless there is a concrete runtime reason to change them.

## Development Workflow

### Rust

- `cargo run -- --help`
- `cargo test --workspace`
- `cargo build --release -p repoxide -p repoxide-server`
- `cargo run -p repoxide-server`

### Active web app

- `cargo test -p repoxide-server`
- `cargo clippy -p repoxide-server --all-targets`
- `docker compose config`
- `docker compose up --build`

### Schema updates

- schema assets are currently checked in under `apps/web/client/src/public/schemas`

## Safe Edit Guidance

- If you move packages again, retarget `.github/workflows/*`, `.devcontainer/*`, and `docker-compose.yml` together.
- Docker builds copy schema and logo assets from `apps/web/client/src/public`; regenerate schemas before rebuilding if those assets change.
- Keep root docs Rust-first and describe `apps/web/client` as retired/docs tooling.

## Where To Find Details

- `README.md` - repo overview and quick start.
- `apps/web/server/README.md` - active web app endpoints and Docker usage.
- `apps/web/server/CHUNKED_UPLOAD.md` - upload flow details.
- `apps/web/client/scripts/generateSchema.ts` - schema asset generation for the Rust server.
- `.github/workflows/ci.yml` - current verification matrix across Rust, web app, retained client tooling, and GitHub Action coverage.
