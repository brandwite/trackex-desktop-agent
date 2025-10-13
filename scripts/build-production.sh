#!/bin/bash

# TrackEx Agent Production Build Script
set -e

echo "🔨 Building TrackEx Agent for Production..."

# Set environment variables for production
export TAURI_PRIVATE_KEY=""
export TAURI_KEY_PASSWORD=""

# Build the application
echo "📦 Building application..."
cargo tauri build

echo "✅ Build completed successfully!"
echo "📍 Application bundle: src-tauri/target/release/bundle/macos/TrackEx Agent.app"
echo "📍 DMG installer: src-tauri/target/release/bundle/dmg/TrackEx Agent_1.0.0_aarch64.dmg"

# Verify the build
if [ -d "src-tauri/target/release/bundle/macos/TrackEx Agent.app" ]; then
    echo "✅ macOS app bundle created successfully"
else
    echo "❌ Failed to create macOS app bundle"
    exit 1
fi

if [ -f "src-tauri/target/release/bundle/dmg/TrackEx Agent_1.0.0_aarch64.dmg" ]; then
    echo "✅ DMG installer created successfully"
else
    echo "❌ Failed to create DMG installer"
    exit 1
fi

echo "🎉 Production build completed!"


