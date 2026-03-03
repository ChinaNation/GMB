FROM rust:1.76-bookworm AS builder
WORKDIR /build

COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/cpms-backend /app/cpms-backend

EXPOSE 8080
ENTRYPOINT ["/app/cpms-backend"]
