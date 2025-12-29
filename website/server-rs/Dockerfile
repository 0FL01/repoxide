# === Build Stage ===
FROM rust:1.92-alpine AS builder
RUN apk add --no-cache musl-dev git pkgconfig openssl-dev

WORKDIR /build
# Copy workspace for path dependency
COPY repomix-rs ./repomix-rs
COPY website/server-rs ./website/server-rs

WORKDIR /build/website/server-rs
RUN cargo build --release

# === Runtime Stage ===
FROM alpine:3.23
RUN apk add --no-cache git ca-certificates libgcc

COPY --from=builder /build/website/server-rs/target/release/repomix-server /usr/local/bin/

ENV PORT=8080 \
    RUST_LOG=repomix_server=info,tower_http=info

EXPOSE 8080
CMD ["repomix-server"]
