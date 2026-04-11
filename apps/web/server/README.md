# Repomix Web Server

Rust backend for the active Repomix web app.

## Location

- frontend: `apps/web/client`
- backend: `apps/web/server`
- core library: `crates/repomix`

## Features

- remote repository packing
- ZIP upload and chunked uploads
- multiple output formats
- structured logging
- ZIP traversal and ZIP bomb protections

## Run locally

From the repository root:

```bash
cargo run -p repomix-server
```

The server listens on `http://localhost:8080` by default.

Environment variables:

- `PORT` — server port, defaults to `8080`
- `RUST_LOG` — tracing filter, for example `repomix_server=debug,tower_http=debug`

## API

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
docker compose up --build server

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
