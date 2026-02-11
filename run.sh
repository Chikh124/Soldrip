#!/bin/bash

echo "================================================"
echo "   SOLdrip Automation Tool - Quick Start"
echo "================================================"
echo ""

# Перевірка наявності Rust
if ! command -v cargo &> /dev/null; then
    echo "[ERROR] Rust is not installed!"
    echo ""
    echo "Please install Rust:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
    echo "After installation, restart this script."
    exit 1
fi

echo "[OK] Rust detected:"
cargo --version
echo ""

echo "Building project in release mode..."
cargo build --release

if [ $? -ne 0 ]; then
    echo ""
    echo "[ERROR] Build failed!"
    exit 1
fi

echo ""
echo "================================================"
echo "   Starting SOLdrip Automation Tool..."
echo "================================================"
echo ""

cargo run --release
