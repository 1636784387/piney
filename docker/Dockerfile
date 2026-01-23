# syntax=docker/dockerfile:1

# ============================================================================
# 使用 tonistiigi/xx 交叉编译 + Vendored OpenSSL
# openssl-sys 会自动从源码编译 OpenSSL，避免系统库检测问题
# ============================================================================

# Stage 0: 引入 xx 跨平台工具
FROM --platform=$BUILDPLATFORM tonistiigi/xx:master AS xx

# Stage 1: Chef - 基础环境与工具安装
FROM --platform=$BUILDPLATFORM rust:slim-bookworm AS chef
COPY --from=xx / /
WORKDIR /app

# 安装构建工具 (perl/make 是编译 OpenSSL 源码所必需的)
RUN apt-get update && apt-get install -y \
    clang lld pkg-config git cmake perl make build-essential \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef

# Stage 2: Planner - 计算依赖配方
FROM chef AS planner
COPY . .
RUN sed -i 's/"src-tauri",//' Cargo.toml
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder - 交叉编译
FROM chef AS builder
ARG TARGETPLATFORM

# 安装目标架构的基础依赖 (不需要 libssl-dev，OpenSSL 将从源码编译)
RUN xx-apt-get update && xx-apt-get install -y \
    libwebp-dev gcc libc6-dev \
    && rm -rf /var/lib/apt/lists/*

# 关键：不设置 OPENSSL_NO_VENDOR，让 openssl-sys 自己编译 OpenSSL
# 这会增加首次构建时间，但避免了所有交叉编译库检测问题
ENV PKG_CONFIG_ALLOW_CROSS=1

COPY --from=planner /app/recipe.json recipe.json

# 构建依赖层 (OpenSSL 会在这里从源码编译)
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    xx-cargo chef cook --release --recipe-path recipe.json

# 构建应用层
COPY . .
RUN sed -i 's/"src-tauri",//' Cargo.toml

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    xx-cargo build --release && \
    cp target/$(xx-cargo --print-target-triple)/release/piney-server /app/piney-server && \
    xx-verify /app/piney-server

# Stage 4: Frontend
FROM --platform=$BUILDPLATFORM node:20-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend ./
RUN npx svelte-kit sync
RUN npm run build

# Stage 5: Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# 运行时只需要基础库 (OpenSSL 已静态链接到二进制文件中)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libwebp7 \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/piney-server /app/piney
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
