# TrackEx Agent - Production Deployment Guide

## Overview

This guide covers the complete production deployment process for the TrackEx macOS desktop agent, including building, code signing, notarization, and distribution.

## Prerequisites

### Development Environment
- macOS 12.6 or later
- Xcode Command Line Tools
- Rust and Cargo (latest stable)
- Node.js 18+ and npm
- Tauri CLI v2

### Apple Developer Account (for distribution)
- Apple Developer Program membership
- Developer ID Application certificate
- App-specific password for notarization

## Quick Start

### 1. Development Build
```bash
# Install dependencies
npm install

# Run in development mode
cargo tauri dev
```

### 2. Production Build
```bash
# Complete production deployment
./scripts/deploy-production.sh
```

## Environment Variables

Set these environment variables for production builds:

```bash
# Required for code signing
export APPLE_DEVELOPER_ID="Developer ID Application: Your Name (TEAM_ID)"

# Required for notarization
export APPLE_ID="your-apple-id@example.com"
export APPLE_TEAM_ID="YOUR_TEAM_ID"
export APPLE_APP_PASSWORD="@keychain:AC_PASSWORD"

# Optional: Development mode
export TRACKEX_DEV_SHORT_INTERVALS=1  # For testing with shorter intervals
```

## Build Scripts

### Individual Scripts

1. **Build Only**
   ```bash
   ./scripts/build-production.sh
   ```

2. **Code Sign**
   ```bash
   ./scripts/codesign-production.sh
   ```

3. **Notarize**
   ```bash
   ./scripts/notarize-production.sh
   ```

### Complete Deployment
```bash
./scripts/deploy-production.sh
```

## Code Signing Setup

### 1. Install Developer Certificate
1. Download your Developer ID Application certificate from Apple Developer portal
2. Install it in Keychain Access
3. Verify with: `security find-identity -v -p codesigning`

### 2. Configure Entitlements
The app uses these entitlements (in `src-tauri/entitlements.plist`):
- Screen recording access
- Accessibility features
- Network access
- File system access

## Notarization Setup

### 1. Create App-Specific Password
1. Go to [appleid.apple.com](https://appleid.apple.com)
2. Generate an app-specific password
3. Store it in Keychain: `xcrun notarytool store-credentials`

### 2. Verify Setup
```bash
xcrun notarytool history --apple-id your-apple-id@example.com --team-id YOUR_TEAM_ID
```

## Distribution

### Output Files
After successful deployment, find these files in the `dist/` directory:

- `TrackEx Agent_1.0.0_aarch64.dmg` - Installer for Apple Silicon Macs
- `TrackEx Agent.app` - Application bundle
- `TrackEx-Agent-1.0.0-macos.zip` - Zipped app bundle
- `checksums.txt` - SHA256 checksums for verification

### Installation Instructions for End Users

1. **Download** the DMG file
2. **Open** the DMG and drag TrackEx Agent to Applications
3. **First Launch**: Right-click → Open (to bypass Gatekeeper warning)
4. **Grant Permissions**: 
   - System Preferences → Security & Privacy → Privacy
   - Enable "Screen Recording" and "Accessibility" for TrackEx Agent

## Troubleshooting

### Common Build Issues

1. **Rust not found**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Tauri CLI missing**
   ```bash
   cargo install tauri-cli
   ```

3. **Node dependencies**
   ```bash
   npm install
   ```

### Code Signing Issues

1. **Certificate not found**
   - Verify certificate installation: `security find-identity -v -p codesigning`
   - Download and install from Apple Developer portal

2. **Entitlements errors**
   - Ensure `src-tauri/entitlements.plist` exists
   - Verify entitlements match app capabilities

### Notarization Issues

1. **Authentication failed**
   - Verify Apple ID and Team ID
   - Regenerate app-specific password
   - Check keychain credentials

2. **Notarization rejected**
   - Ensure app is properly code signed
   - Check for hardened runtime compatibility
   - Review notarization logs: `xcrun notarytool log`

## Security Considerations

### Privacy
- The agent only collects data when user is clocked in
- Screenshots are encrypted in transit
- Local data is stored securely
- User can pause/resume tracking anytime

### Permissions
- Screen recording: For productivity screenshots
- Accessibility: For window title detection
- Network: For API communication
- File system: For local data storage

### Data Handling
- All API communication uses HTTPS
- Device tokens stored in macOS Keychain
- Local database encrypted at rest
- Automatic cleanup of old data

## Testing

### Manual Testing Checklist
- [ ] Application launches successfully
- [ ] System tray icon appears
- [ ] Login flow works
- [ ] Consent wizard displays
- [ ] Clock in/out functionality
- [ ] Screenshot capture (with permissions)
- [ ] Idle detection
- [ ] App focus tracking
- [ ] Offline queue functionality
- [ ] Settings and preferences
- [ ] Graceful shutdown

### Automated Testing
```bash
# Run unit tests
cargo test

# Run integration tests
npm test
```

## Deployment Checklist

- [ ] All tests passing
- [ ] Version number updated
- [ ] Changelog updated
- [ ] Code signed successfully
- [ ] Notarization completed
- [ ] Tested on clean macOS system
- [ ] Distribution files generated
- [ ] Checksums verified
- [ ] Upload to distribution server
- [ ] Update download links
- [ ] User documentation updated

## Support

For issues with the TrackEx Agent:
1. Check the troubleshooting section above
2. Review application logs in Console.app
3. Contact support with system information and error details

## Version History

- **v1.0.0** - Initial production release
  - Complete Tauri v2 implementation
  - macOS-specific features
  - Production-ready build system
  - Code signing and notarization support


