#!/bin/bash

# TrackEx Agent Code Signing Script
set -e

# Configuration
APP_PATH="src-tauri/target/release/bundle/macos/TrackEx Agent.app"
DEVELOPER_ID="${APPLE_DEVELOPER_ID:-Developer ID Application: Your Name (TEAM_ID)}"
ENTITLEMENTS_PATH="src-tauri/entitlements.plist"

echo "🔐 Code Signing TrackEx Agent..."

# Check if the app exists
if [ ! -d "$APP_PATH" ]; then
    echo "❌ App bundle not found at: $APP_PATH"
    echo "Please run the build script first."
    exit 1
fi

# Check if entitlements file exists
if [ ! -f "$ENTITLEMENTS_PATH" ]; then
    echo "❌ Entitlements file not found at: $ENTITLEMENTS_PATH"
    exit 1
fi

echo "📝 Using Developer ID: $DEVELOPER_ID"
echo "📝 Using Entitlements: $ENTITLEMENTS_PATH"

# Sign the application
echo "✍️  Signing application..."
codesign --force \
    --options runtime \
    --entitlements "$ENTITLEMENTS_PATH" \
    --sign "$DEVELOPER_ID" \
    --deep \
    "$APP_PATH"

# Verify the signature
echo "🔍 Verifying signature..."
codesign --verify --verbose=2 "$APP_PATH"

# Check if Gatekeeper will accept it
echo "🛡️  Checking Gatekeeper compatibility..."
spctl --assess --type execute --verbose "$APP_PATH"

echo "✅ Code signing completed successfully!"
echo "📍 Signed app: $APP_PATH"


