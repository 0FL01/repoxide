# Project: Repomix

Repomix packs repositories into AI-friendly outputs. This fork is Rust-first: the repo root targets the Rust workspace by default, while the historical TypeScript implementations live under `legacy/`.

Tech stack: Rust 2021 workspace, Axum/Tokio web app/API, server-rendered Rust UI, retired VitePress/Vue client assets, legacy Node/TypeScript packages, Docker Compose, GitHub Actions.

## Workspace Overview

- `Cargo.toml` - root workspace; default member is `crates/repomix`; shared release profile and single root `Cargo.lock`.
- `crates/repomix` - primary Rust CLI/library for packing, config loading, compression, metrics, and output generation.
- `apps/web/client` - retired VitePress/docs workspace; retained for docs/dev tooling and static assets consumed by the Rust server.
- `apps/web/server` - active Rust web app and API; Axum server for `/`, `/en`, `/ru`, `/pack`, `/health`, `/api/pack`, and chunked upload endpoints.
- `legacy/repomix-ts` - legacy TypeScript CLI/library, browser extension, memory benchmark, npm publish target.
- `legacy/website-server-ts` - legacy TypeScript web backend kept for compatibility and security parity.
- `docker-compose.yml` - active local stack.
- `legacy/compose.yml` - legacy local stack.

## Where To Look

- `crates/repomix/src/cli` - CLI entrypoints and argument handling.
- `crates/repomix/src/config` - config loading and schema handling.
- `crates/repomix/src/core` - file collection, compression, metrics, and output generation.
- `apps/web/server/src/handlers` - active UI/page/API handlers.
- `apps/web/server/src/views` - server-rendered page templates, component helpers, and form state.
- `apps/web/server/static` - bundled CSS/JS for the web UI.
- `apps/web/client/scripts` - schema generation and retained client tooling.
- `apps/web/client/src/public` - static assets (logo + schemas) consumed by the Rust server.
- `apps/web/client/.vitepress/config.ts` - retired docs/dev shell and local proxy config.
- `legacy/repomix-ts/src` - old Node implementation.
- `legacy/website-server-ts/src` - old TS backend.

## Architectural Invariants

- Root `cargo build`, `cargo test`, and `cargo run -- ...` must stay usable without extra package flags.
- Keep `Cargo.lock` only at the repository root; do not reintroduce per-member lockfiles.
- The active runtime is `apps/web/server`; it serves both the UI and API. `apps/web/client` is retained for docs/dev tooling and static assets.
- The legacy TS backend is not part of the main deployment path.
- `docker-compose.yml` builds and publishes the Rust web app on `127.0.0.1:93` (`127.0.0.1:93:8080`); there is no separate runtime backend service.
- Public nginx / Cloudflare routing may intentionally sit in front of the published web-app port instead of the app directly.
- Config schema generation is coupled: `legacy/repomix-ts/src/config/configSchema.ts` feeds `apps/web/client/scripts/generateSchema.ts`, which writes `apps/web/client/src/public/schemas`; `apps/web/server` serves those assets from `/schemas/*`.
- ZIP/archive hardening exists in both active Rust and legacy TS paths; keep traversal, duplicate-entry, and size/nesting protections aligned when editing extraction code.
- Prefer rootless container settings and preserve `no-new-privileges`, `cap_drop`, `tmpfs`, and readonly patterns unless there is a concrete runtime reason to change them.

## Development Workflow

### Rust

- `cargo run -- --help`
- `cargo test --workspace`
- `cargo build --release -p repomix -p repomix-server`
- `cargo run -p repomix-server`

### Active web app

- `cargo test -p repomix-server`
- `cargo clippy -p repomix-server --all-targets`
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
- Do not reintroduce root-level `package.json` assumptions; Node workflows now target `legacy/*` and the retained `apps/web/client` tooling explicitly.
- Docker builds copy schema and logo assets from `apps/web/client/src/public`; regenerate schemas before rebuilding if those assets change.
- Keep root docs Rust-first, describe `apps/web/client` as retired/docs tooling, and keep `legacy/*` docs explicit about being legacy.

## Where To Find Details

- `README.md` - repo overview and quick start.
- `apps/web/server/README.md` - active web app endpoints and Docker usage.
- `apps/web/server/CHUNKED_UPLOAD.md` - upload flow details.
- `apps/web/client/scripts/generateSchema.ts` - schema asset generation for the Rust server.
- `legacy/repomix-ts/README.md` - legacy CLI and deployment notes.
- `.github/workflows/ci.yml` - current verification matrix across Rust, web app, retained client tooling, and legacy TS.
