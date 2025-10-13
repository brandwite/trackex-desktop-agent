#!/bin/bash

# TrackEx Agent Notarization Script
set -e

# Configuration
APP_PATH="src-tauri/target/release/bundle/macos/TrackEx Agent.app"
DMG_PATH="src-tauri/target/release/bundle/dmg/TrackEx Agent_1.0.0_aarch64.dmg"
APPLE_ID="${APPLE_ID:-your-apple-id@example.com}"
TEAM_ID="${APPLE_TEAM_ID:-YOUR_TEAM_ID}"
APP_PASSWORD="${APPLE_APP_PASSWORD:-@keychain:AC_PASSWORD}"

echo "📤 Notarizing TrackEx Agent..."

# Check if the DMG exists
if [ ! -f "$DMG_PATH" ]; then
    echo "❌ DMG not found at: $DMG_PATH"
    echo "Please run the build and code signing scripts first."
    exit 1
fi

echo "📝 Using Apple ID: $APPLE_ID"
echo "📝 Using Team ID: $TEAM_ID"
echo "📝 Notarizing: $DMG_PATH"

# Submit for notarization
echo "📤 Submitting for notarization..."
xcrun notarytool submit "$DMG_PATH" \
    --apple-id "$APPLE_ID" \
    --team-id "$TEAM_ID" \
    --password "$APP_PASSWORD" \
    --wait

# Staple the notarization to the DMG
echo "📎 Stapling notarization to DMG..."
xcrun stapler staple "$DMG_PATH"

# Verify the stapling
echo "🔍 Verifying stapled notarization..."
xcrun stapler validate "$DMG_PATH"

# Also staple the app bundle if it exists
if [ -d "$APP_PATH" ]; then
    echo "📎 Stapling notarization to app bundle..."
    xcrun stapler staple "$APP_PATH"
    xcrun stapler validate "$APP_PATH"
fi

echo "✅ Notarization completed successfully!"
echo "📍 Notarized DMG: $DMG_PATH"
echo "📍 Notarized App: $APP_PATH"


