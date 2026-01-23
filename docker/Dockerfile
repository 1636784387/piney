# syntax=docker/dockerfile:1

# Stage 0: 引入 xx 跨平台工具
FROM --platform=$BUILDPLATFORM tonistiigi/xx:master AS xx

# Stage 1: Chef - 基础环境与工具安装
FROM --platform=$BUILDPLATFORM rust:slim-bookworm AS chef
# 复制 xx 工具
COPY --from=xx / /
WORKDIR /app
# 安装基础构建工具
RUN apt-get update && apt-get install -y \
    clang lld pkg-config git cmake perl make build-essential \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef

# Stage 2: Planner - 计算依赖配方
FROM chef AS planner
COPY . .
# 移除 cargo-chef 无法处理的 workspace members (如 src-tauri，Docker 不需要构建它)
RUN sed -i 's/"src-tauri",//' Cargo.toml
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder - 交叉编译依赖与应用
FROM chef AS builder
# 引入目标平台参数 (如 linux/arm64)
ARG TARGETPLATFORM

# 安装目标架构的系统依赖 (libwebp, openssl 等)
# xx-apt-get 会自动根据 TARGETPLATFORM 安装对应的架构包
RUN xx-apt-get update && xx-apt-get install -y \
    libssl-dev libwebp-dev gcc libc6-dev \
    && rm -rf /var/lib/apt/lists/*

# 配置交叉编译环境变量
# OPENSSL_NO_VENDOR=1: 强制使用系统库，禁止自己编译 OpenSSL
# PKG_CONFIG_ALLOW_CROSS=1: 允许 pkg-config 在交叉编译时工作
# 注意：PKG_CONFIG_PATH 和 OPENSSL_DIR 需要在 RUN 时动态设置，因为它们依赖于 xx-info 的输出
ENV OPENSSL_NO_VENDOR=1 \
    PKG_CONFIG_ALLOW_CROSS=1 \
    OPENSSL_STATIC=0

COPY --from=planner /app/recipe.json recipe.json

# 构建依赖层
# 使用 xx-clang 作为 C 编译器包装器，确保正确的 sysroot 和目标架构
# 显式设置 OPENSSL_DIR 和 PKG_CONFIG_SYSROOT_DIR 以便找到交叉编译的库
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    XX_TRIPLE=$(xx-info triple) && \
    export PKG_CONFIG_PATH="/usr/lib/${XX_TRIPLE}/pkgconfig" && \
    export PKG_CONFIG_SYSROOT_DIR="$(xx-info sysroot)" && \
    export OPENSSL_DIR="/usr" && \
    export OPENSSL_LIB_DIR="/usr/lib/${XX_TRIPLE}" && \
    export OPENSSL_INCLUDE_DIR="/usr/include" && \
    xx-cargo chef cook --release --recipe-path recipe.json

# 构建应用层
COPY . .
RUN sed -i 's/"src-tauri",//' Cargo.toml

# 执行交叉编译 (使用相同的环境变量设置)
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    XX_TRIPLE=$(xx-info triple) && \
    export PKG_CONFIG_PATH="/usr/lib/${XX_TRIPLE}/pkgconfig" && \
    export PKG_CONFIG_SYSROOT_DIR="$(xx-info sysroot)" && \
    export OPENSSL_DIR="/usr" && \
    export OPENSSL_LIB_DIR="/usr/lib/${XX_TRIPLE}" && \
    export OPENSSL_INCLUDE_DIR="/usr/include" && \
    xx-cargo build --release && \
    cp target/$(xx-cargo --print-target-triple)/release/piney-server /app/piney-server && \
    xx-verify /app/piney-server

# Stage 4: Frontend - 构建前端 (始终在构建平台运行，速度最快)
FROM --platform=$BUILDPLATFORM node:20-slim AS frontend-builder
WORKDIR /app/frontend
# 单独复制依赖定义文件以利用缓存
COPY frontend/package.json frontend/package-lock.json ./
# 使用 npm ci 进行纯净安装 (比 npm install 更快更可靠)
RUN npm ci
# 复制源码并构建
COPY frontend ./
RUN npx svelte-kit sync
RUN npm run build

# Stage 5: Runtime - 最终运行时环境
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# 安装运行时依赖 (原生架构)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libwebp7 \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 复制产物
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
