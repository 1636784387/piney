#!/bin/bash
# cross-cargo.sh - Wrapper script for cross-compilation with proper environment setup
# This script ensures all environment variables are properly set for cargo build scripts

set -e

# Get target triple from xx
XX_TRIPLE=$(xx-info triple)
XX_SYSROOT=$(xx-info sysroot)

# Set PKG_CONFIG environment
export PKG_CONFIG_PATH="/usr/lib/${XX_TRIPLE}/pkgconfig"
export PKG_CONFIG_SYSROOT_DIR="${XX_SYSROOT}"
export PKG_CONFIG_ALLOW_CROSS=1

# Set OpenSSL environment for cross-compilation
# These are checked by openssl-sys build script
export OPENSSL_DIR="/usr"
export OPENSSL_LIB_DIR="/usr/lib/${XX_TRIPLE}"
export OPENSSL_INCLUDE_DIR="/usr/include"
export OPENSSL_NO_VENDOR=1
export OPENSSL_STATIC=0

# Also set target-prefixed variables (openssl-sys checks these first)
TARGET_UPPER=$(echo "${XX_TRIPLE}" | tr '[:lower:]-' '[:upper:]_')
export "${TARGET_UPPER}_OPENSSL_DIR=/usr"
export "${TARGET_UPPER}_OPENSSL_LIB_DIR=/usr/lib/${XX_TRIPLE}"
export "${TARGET_UPPER}_OPENSSL_INCLUDE_DIR=/usr/include"

# Debug output
echo "=== Cross-compilation environment ==="
echo "XX_TRIPLE: ${XX_TRIPLE}"
echo "PKG_CONFIG_PATH: ${PKG_CONFIG_PATH}"
echo "OPENSSL_LIB_DIR: ${OPENSSL_LIB_DIR}"
echo "====================================="

# Execute xx-cargo with all arguments
exec xx-cargo "$@"
