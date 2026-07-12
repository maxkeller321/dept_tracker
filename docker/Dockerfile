# Build backend (cargo-chef caches dependency layers across rebuilds)
FROM rust:1-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app/backend

FROM chef AS planner
COPY backend/ .
RUN cargo chef prepare --recipe-path ../recipe.json

FROM chef AS cacher
COPY backend/.cargo/ .cargo/
COPY --from=planner /app/recipe.json /app/recipe.json
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/crates/domain/Cargo.toml crates/domain/
COPY backend/crates/db/Cargo.toml crates/db/
COPY backend/crates/api/Cargo.toml crates/api/
ENV CARGO_NET_RETRY=10
RUN cargo chef cook --release --locked --recipe-path /app/recipe.json -p api

FROM rust:1-bookworm AS rust-builder
WORKDIR /app/backend
COPY backend/.cargo/ .cargo/
COPY --from=cacher /app/backend/target /app/backend/target
COPY --from=cacher /usr/local/cargo/registry /usr/local/cargo/registry
COPY backend/ .
ENV CARGO_NET_RETRY=10
RUN cargo build --release --locked -p api

# Build frontend
FROM node:22-bookworm AS frontend-builder
WORKDIR /app
COPY frontend/package*.json frontend/
RUN cd frontend && npm ci
COPY frontend/ frontend/
RUN cd frontend && npm run build

# Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=rust-builder /app/backend/target/release/dept-tracker /app/dept-tracker
COPY --from=frontend-builder /app/frontend/build /app/static
ENV DATA_DIR=/data STATIC_DIR=/app/static
EXPOSE 8080
VOLUME ["/data"]
CMD ["/app/dept-tracker"]
