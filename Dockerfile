FROM rust:1.92-alpine AS builder
RUN apk add --no-cache musl-dev git pkgconfig openssl-dev

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY apps/web/server ./apps/web/server

RUN cargo build --release -p repoxide

FROM alpine:3.23
RUN apk add --no-cache git ca-certificates libgcc

RUN addgroup -S repoxide && adduser -S -G repoxide -h /app repoxide && mkdir -p /app && chown repoxide:repoxide /app

COPY --from=builder /build/target/release/repoxide /usr/local/bin/repoxide

WORKDIR /app
USER repoxide

ENTRYPOINT ["repoxide"]
