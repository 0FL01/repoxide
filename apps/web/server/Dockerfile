# === Build Stage ===
FROM rust:1.92-alpine AS builder
RUN apk add --no-cache musl-dev git pkgconfig openssl-dev

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY apps/web/server ./apps/web/server

RUN cargo build --release -p repomix-server

# === Runtime Stage ===
FROM alpine:3.23
RUN apk add --no-cache git ca-certificates libgcc

RUN addgroup -S repomix && adduser -S -G repomix -h /app repomix && mkdir -p /app && chown repomix:repomix /app

COPY --from=builder /build/target/release/repomix-server /usr/local/bin/

WORKDIR /app

ENV PORT=8080 \
    RUST_LOG=repomix_server=info,tower_http=info

EXPOSE 8080
USER repomix
CMD ["repomix-server"]
