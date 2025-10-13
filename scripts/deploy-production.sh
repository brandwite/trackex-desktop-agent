#!/bin/bash

# TrackEx Agent Complete Production Deployment Script
set -e

echo "🚀 TrackEx Agent Production Deployment"
echo "======================================"

# Check prerequisites
echo "🔍 Checking prerequisites..."

# Check if we have the required environment variables
if [ -z "$APPLE_DEVELOPER_ID" ]; then
    echo "⚠️  APPLE_DEVELOPER_ID not set. Using placeholder."
    export APPLE_DEVELOPER_ID="Developer ID Application: Your Name (TEAM_ID)"
fi

if [ -z "$APPLE_ID" ]; then
    echo "⚠️  APPLE_ID not set. Notarization will be skipped."
fi

if [ -z "$APPLE_TEAM_ID" ]; then
    echo "⚠️  APPLE_TEAM_ID not set. Notarization will be skipped."
fi

# Step 1: Build
echo ""
echo "📦 Step 1: Building application..."
./scripts/build-production.sh

# Step 2: Code Sign (if developer ID is available)
echo ""
echo "🔐 Step 2: Code signing..."
if command -v codesign &> /dev/null; then
    ./scripts/codesign-production.sh
else
    echo "⚠️  codesign not available. Skipping code signing."
fi

# Step 3: Notarize (if credentials are available)
echo ""
echo "📤 Step 3: Notarization..."
if [ -n "$APPLE_ID" ] && [ -n "$APPLE_TEAM_ID" ] && command -v xcrun &> /dev/null; then
    ./scripts/notarize-production.sh
else
    echo "⚠️  Apple credentials or xcrun not available. Skipping notarization."
fi

# Step 4: Create distribution package
echo ""
echo "📦 Step 4: Creating distribution package..."
DIST_DIR="dist"
mkdir -p "$DIST_DIR"

# Copy the DMG to distribution directory
if [ -f "src-tauri/target/release/bundle/dmg/TrackEx Agent_1.0.0_aarch64.dmg" ]; then
    cp "src-tauri/target/release/bundle/dmg/TrackEx Agent_1.0.0_aarch64.dmg" "$DIST_DIR/"
    echo "✅ DMG copied to $DIST_DIR/"
fi

# Copy the app bundle to distribution directory
if [ -d "src-tauri/target/release/bundle/macos/TrackEx Agent.app" ]; then
    cp -R "src-tauri/target/release/bundle/macos/TrackEx Agent.app" "$DIST_DIR/"
    echo "✅ App bundle copied to $DIST_DIR/"
fi

# Create a zip of the app bundle for easy distribution
if [ -d "$DIST_DIR/TrackEx Agent.app" ]; then
    cd "$DIST_DIR"
    zip -r "TrackEx-Agent-1.0.0-macos.zip" "TrackEx Agent.app"
    cd ..
    echo "✅ App bundle zipped as TrackEx-Agent-1.0.0-macos.zip"
fi

# Generate checksums
echo ""
echo "🔍 Generating checksums..."
cd "$DIST_DIR"
if command -v shasum &> /dev/null; then
    shasum -a 256 *.dmg *.zip > checksums.txt 2>/dev/null || true
    echo "✅ Checksums generated in $DIST_DIR/checksums.txt"
fi
cd ..

# Summary
echo ""
echo "🎉 Deployment Complete!"
echo "======================"
echo "📍 Distribution files available in: $DIST_DIR/"
ls -la "$DIST_DIR/"

echo ""
echo "📋 Next Steps:"
echo "1. Test the application on a clean macOS system"
echo "2. Upload to your distribution server or app store"
echo "3. Update your website download links"
echo "4. Notify users of the new version"


