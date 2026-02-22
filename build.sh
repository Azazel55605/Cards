#!/bin/bash
set -e

echo "==> Linux x86_64"
cargo build --release

echo "==> Linux ARM64"
cross build --release --target aarch64-unknown-linux-gnu

echo "==> Windows x86_64"
cargo build --release --target x86_64-pc-windows-gnu

echo "==> macOS x86_64"
cargo build --release --target x86_64-apple-darwin

echo "==> macOS ARM64"
cargo build --release --target aarch64-apple-darwin

echo ""
echo "Done!