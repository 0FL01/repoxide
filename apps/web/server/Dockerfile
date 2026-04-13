# === Build Stage ===
FROM rust:1.92-alpine AS builder
RUN apk add --no-cache musl-dev git pkgconfig openssl-dev

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY apps/web/server ./apps/web/server
COPY apps/web/client/src/public/images/repoxide-logo.svg ./apps/web/client/src/public/images/repoxide-logo.svg
COPY apps/web/client/src/public/schemas ./apps/web/client/src/public/schemas

RUN cargo build --release -p repoxide-server

# === Runtime Stage ===
FROM alpine:3.23
RUN apk add --no-cache git ca-certificates libgcc

RUN addgroup -S repoxide && adduser -S -G repoxide -h /app repoxide && mkdir -p /app && chown repoxide:repoxide /app

COPY --from=builder /build/target/release/repoxide-server /usr/local/bin/
COPY --from=builder /build/apps/web/client/src/public/schemas /app/schemas

WORKDIR /app

ENV PORT=8080 \
    RUST_LOG=repoxide_server=info,tower_http=info \
    REPOXIDE_SCHEMA_DIR=/app/schemas

EXPOSE 8080
USER repoxide
CMD ["repoxide-server"]
