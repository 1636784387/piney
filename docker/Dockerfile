# syntax=docker/dockerfile:1

# ============================================================================
# 极简 Dockerfile - 使用原生架构构建，GitHub Actions 原生 ARM64 Runner
# ============================================================================

# Stage 0: 从官方镜像获取预编译的 cargo-chef
FROM lukemathwalker/cargo-chef:latest-rust-slim-bookworm AS cargo-chef

# Stage 1: Chef - 基础环境与工具安装
FROM rust:slim-bookworm AS chef
# 复制预编译的 cargo-chef 二进制文件
COPY --from=cargo-chef /usr/local/cargo/bin/cargo-chef /usr/local/cargo/bin/cargo-chef
WORKDIR /app

# 安装构建工具
RUN apt-get update && apt-get install -y \
    pkg-config git cmake perl make build-essential \
    libssl-dev libwebp-dev \
    && rm -rf /var/lib/apt/lists/*

# Stage 2: Planner - 计算依赖配方
FROM chef AS planner
COPY . .
RUN sed -i 's/"src-tauri",//' Cargo.toml
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder - 编译
FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

# 构建依赖层
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    cargo chef cook --release --recipe-path recipe.json

# 构建应用层
COPY . .
RUN sed -i 's/"src-tauri",//' Cargo.toml

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    cargo build --release

# Stage 4: Frontend
FROM node:20-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend ./
RUN npx svelte-kit sync
RUN npm run build

# Stage 5: Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libwebp7 \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/piney-server /app/piney
COPY --from=frontend-builder /app/frontend/build /app/static
RUN mkdir -p /app/data

ENV RUN_MODE=server \
    PORT=9696 \
    DATA_DIR=/app/data

EXPOSE 9696
VOLUME ["/app/data"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:9696/api/health || exit 1

CMD ["/app/piney"]
