#!/bin/bash

# TrackEx Agent - Development Build Script

set -e  # Exit on any error

echo "🔨 Building TrackEx Agent (Development)"
echo "======================================"

# Check for required tools
command -v npm >/dev/null 2>&1 || { echo "❌ npm is required but not installed"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "❌ Rust/Cargo is required but not installed"; exit 1; }

# Set development environment
export TRACKEX_DEV_SHORT_INTERVALS=1
export TAURI_DEBUG=1

echo "📦 Installing frontend dependencies..."
npm install

echo "🦀 Installing Rust dependencies..."
cargo build

echo "🔧 Building Tauri application..."
npm run tauri build

echo "✅ Development build completed!"
echo ""
echo "📁 Build artifacts:"
echo "   • macOS: src-tauri/target/release/bundle/macos/"
echo "   • Logs: src-tauri/target/release/build/"
echo ""
echo "🚀 To run in development mode:"
echo "   npm run tauri dev"

