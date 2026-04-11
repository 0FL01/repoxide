# Repomix Web App

Active web application for the Rust-based Repomix stack.

## Layout

- `client/` — VitePress frontend
- `server/` — Rust API server

## Development

From the repository root:

```bash
docker compose up --build
```

This starts:

- frontend on `http://127.0.0.1:93`
- Rust API behind the frontend's `/api` proxy inside Docker

You can also work on each part directly:

```bash
# Frontend
cd apps/web/client
npm ci
npm run docs:dev -- --host 0.0.0.0

# Backend
cargo run -p repomix-server
```

## Documentation

Update the English source under `client/src/en/`. Translations live alongside it under `client/src/`.
