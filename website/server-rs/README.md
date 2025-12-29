# Repomix Rust Server

High-performance web server for Repomix, written in Rust with Axum framework.

## Features

- ✅ **Remote Repository Processing** - Clone and pack GitHub/GitLab repositories
- ✅ **ZIP File Upload** - Upload and process ZIP archives
- ✅ **Chunked Uploads** - Support for large file uploads (Phase 4)
- ✅ **Multiple Output Formats** - XML, Markdown, and Plain text
- ✅ **Security** - ZIP bomb protection, path traversal prevention
- ✅ **Performance** - Async I/O with Tokio, CPU-intensive work offloaded to thread pool
- ✅ **Observability** - Structured JSON logging, request tracing

## Quick Start

### Prerequisites

- Rust 1.83+ (edition 2021)
- Cargo
- Git (required for remote repository cloning)

### Build & Run

```bash
# Build the server
cargo build --release

# Run the server
cargo run --release

# Server will start on http://localhost:8080
```

### Configuration

Set environment variables:

- `PORT` - Server port (default: 8080)
- `RUST_LOG` - Log level (e.g., `repomix_server=debug,tower_http=debug`)

Example:
```bash
PORT=3000 RUST_LOG=info cargo run --release
```

## API Endpoints

### Health Check
```bash
curl http://localhost:8080/health
# Response: "OK"
```

### Pack Repository (Remote URL)
```bash
curl -X POST \
  -F "url=yamadashy/repomix" \
  -F "format=xml" \
  http://localhost:8080/api/pack
```

### Pack ZIP File
```bash
curl -X POST \
  -F "file=@myproject.zip" \
  -F "format=markdown" \
  -F 'options={"compress":true,"removeComments":true}' \
  http://localhost:8080/api/pack
```

### Pack from Chunked Upload (Phase 4)
```bash
curl -X POST \
  -F "uploadId=<uuid>" \
  -F "format=plain" \
  http://localhost:8080/api/pack
```

## Response Format

```json
{
  "content": "...", // Generated output
  "format": "xml",
  "metadata": {
    "repository": "repomix",
    "timestamp": "2025-12-29T15:20:56.722Z",
    "summary": {
      "totalFiles": 907,
      "totalCharacters": 3802373,
      "totalTokens": 904505
    },
    "topFiles": [
      {
        "path": "README.md",
        "charCount": 0,
        "tokenCount": 16576
      }
      // ... top 10 files by token count
    ]
  }
}
```

## Pack Options

Available options for pack requests (JSON string in `options` field):

```json
{
  "removeComments": false,
  "removeEmptyLines": false,
  "showLineNumbers": false,
  "compress": false,
  "includePatterns": "*.rs,*.toml",
  "ignorePatterns": "target/**,*.log"
}
```

## Security

The server implements multiple security measures:

- **File size limits**: ZIP files limited to 100MB
- **Decompression limits**: 2GB max uncompressed size
- **Compression ratio check**: Prevents ZIP bombs (max ratio 100:1)
- **Path traversal protection**: Sanitizes all file paths
- **File count limit**: Max 50,000 files per archive
- **Path length limit**: Max 300 characters
- **Nesting depth limit**: Max 50 directory levels

## Project Structure

```
website/server-rs/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point, router setup
│   ├── handlers/
│   │   ├── mod.rs        # Handler module exports
│   │   ├── health.rs     # Health check handler (✅)
│   │   ├── pack.rs       # Pack handler (✅ Phase 3)
│   │   └── upload.rs     # Chunked upload handlers (Phase 4)
│   ├── types.rs          # Request/Response types
│   ├── error.rs          # Error handling
│   └── state.rs          # Application state (upload sessions)
└── README.md
```

## Development

```bash
# Run in development mode
cargo run

# Run tests
cargo test

# Check code with clippy
cargo clippy

# Format code
cargo fmt

# Build for production
cargo build --release
```

## Performance

Typical performance for a medium-sized repository (yamadashy/repomix):

- **Files**: 907
- **Characters**: 3.8M
- **Tokens**: 904K
- **Processing time**: ~9-10 seconds (includes git clone)

## Implementation Status

- ✅ **Phase 1**: Library API (Complete)
- ✅ **Phase 2**: Server Structure (Complete)
- ✅ **Phase 3**: Pack Endpoint (Complete)
- 🚧 **Phase 4**: Chunked Upload (Pending)
- 🚧 **Phase 5**: Docker & Deploy (Pending)

## Architecture

- **Axum 0.8** - Modern async web framework
- **Tokio** - Async runtime
- **Tower-HTTP** - Middleware (CORS, compression, tracing)
- **repomix-rs** - Core library for file processing
- **zip** - ZIP archive handling
- **serde_json** - JSON serialization
- **tracing** - Structured logging

## See Also

- [Implementation Blueprint](../IMPLEMENTATION-BLUEPRINT.md) - Full development plan
- [TypeScript Server](../server/) - Original TypeScript implementation
- [repomix-rs](../../repomix-rs/) - Core Rust library

## License

Same as repomix project.
