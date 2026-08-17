# ==============================================================================
# Multi-stage Dockerfile for Foundry Platform Server & Admin UI
# ==============================================================================

# Stage 1: Build Frontend Admin SPA
FROM node:24-alpine AS frontend-builder
WORKDIR /app/apps/admin
COPY apps/admin/package*.json ./
RUN npm ci || npm install
COPY apps/admin ./
RUN npm run build

# Stage 2: Build Rust Backend Binaries
FROM rust:1.97-bookworm AS backend-builder
WORKDIR /app

# Copy Cargo configs and sources
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY systems ./systems
COPY apps ./apps
COPY migrations ./migrations

# Build release server binary
RUN cargo build --release --bin foundry-server --bin foundry-cli

# Stage 3: Minimal Runtime Container
FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=backend-builder /app/target/release/foundry-server /usr/local/bin/foundry-server
COPY --from=backend-builder /app/target/release/foundry-cli /usr/local/bin/foundry-cli
COPY --from=frontend-builder /app/apps/admin/dist /app/static/admin
COPY migrations /app/migrations

ENV PORT=8080
ENV HOST=0.0.0.0
ENV RUST_LOG=info

EXPOSE 8080

CMD ["foundry-server"]
