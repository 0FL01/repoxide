# Project: Repomix

Repomix packs repositories into AI-friendly outputs. This fork is Rust-first: the repo root targets the Rust workspace by default, while the historical TypeScript implementations live under `legacy/`.

Tech stack: Rust 2021 workspace, Axum/Tokio backend, VitePress/Vue frontend, legacy Node/TypeScript packages, Docker Compose, GitHub Actions.

## Workspace Overview

- `Cargo.toml` - root workspace; default member is `crates/repomix`; shared release profile and single root `Cargo.lock`.
- `crates/repomix` - primary Rust CLI/library for packing, config loading, compression, metrics, and output generation.
- `apps/web/client` - active frontend; VitePress app and API client for the Rust backend.
- `apps/web/server` - active Rust web backend; Axum server for `/health`, `/api/pack`, and chunked upload endpoints.
- `legacy/repomix-ts` - legacy TypeScript CLI/library, browser extension, memory benchmark, npm publish target.
- `legacy/website-server-ts` - legacy TypeScript web backend kept for compatibility and security parity.
- `docker-compose.yml` - active local stack.
- `legacy/compose.yml` - legacy local stack.

## Where To Look

- `crates/repomix/src/cli` - CLI entrypoints and argument handling.
- `crates/repomix/src/config` - config loading and schema handling.
- `crates/repomix/src/core` - file collection, compression, metrics, and output generation.
- `apps/web/server/src/handlers` - active API handlers.
- `apps/web/client/components` - UI components.
- `apps/web/client/composables` - upload and request flows.
- `apps/web/client/.vitepress/config.ts` - frontend proxy and VitePress shell.
- `legacy/repomix-ts/src` - old Node implementation.
- `legacy/website-server-ts/src` - old TS backend.

## Architectural Invariants

- Root `cargo build`, `cargo test`, and `cargo run -- ...` must stay usable without extra package flags.
- Keep `Cargo.lock` only at the repository root; do not reintroduce per-member lockfiles.
- The active runtime path is `apps/web/client` -> proxy `/api` to `http://server:8080` -> `apps/web/server`.
- The legacy TS backend is not part of the main deployment path.
- `docker-compose.yml` publishes only the frontend on `127.0.0.1:93`; the Rust backend stays internal to Docker.
- Public nginx / Cloudflare routing may intentionally sit in front of the frontend port instead of the backend directly.
- Config schema generation is coupled: `legacy/repomix-ts/src/config/configSchema.ts` feeds `apps/web/client/scripts/generateSchema.ts`.
- ZIP/archive hardening exists in both active Rust and legacy TS paths; keep traversal, duplicate-entry, and size/nesting protections aligned when editing extraction code.
- Prefer rootless container settings and preserve `no-new-privileges`, `cap_drop`, `tmpfs`, and readonly patterns unless there is a concrete runtime reason to change them.

## Development Workflow

### Rust

- `cargo run -- --help`
- `cargo test --workspace`
- `cargo build --release -p repomix -p repomix-server`
- `cargo run -p repomix-server`

### Active web app

- `npm ci --prefix apps/web/client`
- `npm --prefix apps/web/client run docs:build`
- `docker compose config`
- `docker compose up --build`

### Legacy TypeScript

- `npm ci --prefix legacy/repomix-ts`
- `npm --prefix legacy/repomix-ts run build`
- `npm --prefix legacy/repomix-ts test -- --run`
- `npm ci --prefix legacy/website-server-ts`
- `npm --prefix legacy/website-server-ts run build`
- `docker compose -f legacy/compose.yml config`

### Schema updates

- `npm --prefix legacy/repomix-ts run website-generate-schema`
- output is written to `apps/web/client/src/public/schemas`

## Safe Edit Guidance

- If you move packages again, retarget `.github/workflows/*`, `.devcontainer/*`, `docker-compose.yml`, and `legacy/compose.yml` together.
- Do not reintroduce root-level `package.json` assumptions; Node workflows now target `legacy/*` and `apps/web/client` explicitly.
- The active client bind mount in `docker-compose.yml` expects writable files for UID/GID `1000`; local Docker issues can come from host ownership, not from application code.
- Keep root docs Rust-first and keep `legacy/*` docs explicit about being legacy.

## Where To Find Details

- `README.md` - repo overview and quick start.
- `apps/web/server/README.md` - active backend endpoints and Docker usage.
- `apps/web/server/CHUNKED_UPLOAD.md` - upload flow details.
- `legacy/repomix-ts/README.md` - legacy CLI and deployment notes.
- `.github/workflows/ci.yml` - current verification matrix across Rust, active client, and legacy TS.
