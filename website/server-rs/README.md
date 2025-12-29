# Repomix Rust Server

High-performance Rust implementation of the repomix web server using Axum.

## Features

- ✅ **Health Check** - `/health` endpoint
- 🚧 **Pack API** - `/api/pack` (Phase 3)
- 🚧 **Chunked Upload** - `/api/upload/*` endpoints (Phase 4)
- ✅ **CORS Support** - Permissive CORS for development
- ✅ **Gzip Compression** - Automatic response compression
- ✅ **Structured Logging** - JSON-formatted tracing logs
- ✅ **Background Cleanup** - Automatic cleanup of expired upload sessions

## Quick Start

### Prerequisites

- Rust 1.83+ (edition 2021)
- Cargo

### Build

```bash
cargo build
```

### Run

```bash
cargo run
```

The server will start on `http://0.0.0.0:8080` by default.

### Configuration

Set environment variables:

- `PORT` - Server port (default: 8080)
- `RUST_LOG` - Log level (default: `repomix_server=debug,tower_http=debug`)

Example:

```bash
PORT=3000 RUST_LOG=info cargo run
```

## API Endpoints

### Health Check

```bash
curl http://localhost:8080/health
```

Response: `"OK"`

### Pack API (Phase 3 - Not Yet Implemented)

```bash
curl -X POST http://localhost:8080/api/pack \
  -F "url=yamadashy/repomix" \
  -F "format=markdown"
```

### Upload API (Phase 4 - Not Yet Implemented)

- `POST /api/upload/init` - Initialize chunked upload
- `POST /api/upload/chunk` - Upload a chunk
- `GET /api/upload/status/{id}` - Get upload status

## Development

### Project Structure

```
website/server-rs/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point, router setup
│   ├── handlers/
│   │   ├── mod.rs        # Handler module exports
│   │   ├── health.rs     # Health check handler
│   │   ├── pack.rs       # Pack handler (Phase 3)
│   │   └── upload.rs     # Chunked upload handlers (Phase 4)
│   ├── types.rs          # Request/Response types
│   ├── error.rs          # Error handling
│   └── state.rs          # Application state (upload sessions)
└── README.md
```

### Running Tests

```bash
cargo test
```

### Building for Production

```bash
cargo build --release
```

The optimized binary will be at `target/release/repomix-server`.

## Implementation Status

- ✅ **Phase 1**: Library API (Complete)
- ✅ **Phase 2**: Server Structure (Complete)
- 🚧 **Phase 3**: Pack Endpoint (Pending)
- 🚧 **Phase 4**: Chunked Upload (Pending)
- 🚧 **Phase 5**: Docker & Deploy (Pending)

## License

Same as repomix project.
