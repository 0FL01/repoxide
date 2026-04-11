# Repomix Web App

Rust web app for Repomix. It serves both the browser UI and the JSON API.

## Location

- active web UI + API: `apps/web/server`
- retired VitePress frontend: `apps/web/client`
- core library: `crates/repomix`

## Features

- server-rendered Rust frontend
- remote repository packing
- ZIP upload, direct folder upload, and chunked uploads
- multiple output formats
- structured logging
- ZIP traversal and ZIP bomb protections

## Run locally

From the repository root:

```bash
cargo run -p repomix-server
```

The app listens on `http://localhost:8080` by default.

Environment variables:

- `PORT` — server port, defaults to `8080`
- `RUST_LOG` — tracing filter, for example `repomix_server=debug,tower_http=debug`
- `CORS_ALLOW_ORIGIN` — optional comma-separated origin allowlist for cross-origin browser access to `/api/*`; defaults to `repomix.com`, `www.repomix.com`, and local Vite dev origins

## API

- `GET /`
- `GET /en`
- `GET /ru`
- `POST /pack`
- `GET /health`
- `POST /api/pack`
- `POST /api/upload/init`
- `POST /api/upload/chunk`
- `GET /api/upload/status/{id}`

Example:

```bash
curl http://localhost:8080/health
curl -X POST -F "url=yamadashy/repomix" http://localhost:8080/api/pack
```

## Docker

From the repository root:

```bash
docker compose up --build web

docker build -f apps/web/server/Dockerfile -t repomix-server .
docker run -p 8080:8080 repomix-server
```

## Development

```bash
cargo test -p repomix-server
cargo clippy -p repomix-server --all-targets
cargo build -p repomix-server --release
```

## See also

- `../../../README.md`
- `../client`
- `../../../crates/repomix`
