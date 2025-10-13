# TrackEx Agent for macOS

A secure, privacy-focused desktop agent for employee time tracking and productivity monitoring.

## Overview

TrackEx Agent is a native macOS application built with Tauri (Rust + React) that provides:

- **Time Tracking**: Clock in/out functionality with session management
- **Application Monitoring**: Track which applications are used during work hours
- **Idle Detection**: Automatically detect when users are away from their computers
- **Screenshots**: Policy-driven screen capture for work verification
- **Privacy Controls**: Domain-only mode for browsers, title redaction, and pause controls

## Installation

### System Requirements

- macOS 12.6 or later (Intel or Apple Silicon)
- 150MB RAM
- 100MB disk space
- Network connection to TrackEx server

### Install from Package

1. Download `TrackEx-Agent-v1.0.0.pkg` from your organization's distribution
2. Double-click the package to install
3. Follow the installation prompts
4. Launch from Applications folder or via Spotlight

### Security & Permissions

TrackEx Agent requires certain macOS permissions to function:

#### Required Permissions
- **Screen Recording**: Enables screenshot capture and application monitoring
  - Go to: System Preferences → Security & Privacy → Privacy → Screen Recording
  - Check the box next to "TrackEx Agent"

#### Optional Permissions
- **Accessibility**: Enhances window title detection
  - Go to: System Preferences → Security & Privacy → Privacy → Accessibility
  - Check the box next to "TrackEx Agent"

## First-Time Setup

### 1. Consent Wizard
On first launch, you'll be guided through a consent wizard that explains:
- What data is collected
- How data is used
- Your privacy controls
- Data retention policies

You must accept the consent terms to proceed.

### 2. Login
Enter your TrackEx credentials:
- **Server URL**: Your organization's TrackEx server (e.g., `https://trackex.yourcompany.com`)
- **Email**: Your TrackEx account email
- **Password**: Your TrackEx account password

### 3. Permissions
Grant the required macOS permissions when prompted. The app will guide you through the process.

## Usage

### Basic Operations

#### Clock In/Out
- **Clock In**: Click "Clock In" to start a work session
- **Clock Out**: Click "Clock Out" to end your work session
- Only active work sessions are monitored

#### Taking Breaks
Use the pause controls to take breaks:
- **15/30/60 minute breaks**: Temporary pause with auto-resume
- **Manual resume**: End break early
- **Tracking paused**: All monitoring stops during breaks

#### System Tray
TrackEx Agent runs in your menu bar with these options:
- **Show TrackEx**: Open the main application window
- **Pause/Resume**: Quick break controls
- **Send Diagnostics**: Report issues to support
- **Quit**: Close the application

### What's Monitored

#### During Work Sessions
- **Application Usage**: Which apps you use and for how long
- **Activity Status**: Whether you're actively working or idle
- **Screenshots**: Periodic or on-demand captures (policy dependent)
- **Work Hours**: Total time worked and session duration

#### Privacy Protections
- **Browser Privacy**: Only domain names captured, not full URLs
- **Title Filtering**: Sensitive window titles are redacted
- **No Keylogging**: Keystrokes and clipboard content never monitored
- **Work Hours Only**: No monitoring outside of active work sessions

## Configuration

### Environment Variables

For development or custom configurations:

```bash
# Development mode (shorter intervals for testing)
export TRACKEX_DEV_SHORT_INTERVALS=1

# Screenshot settings
export TRACKEX_SCREENSHOT_ENABLED=true
export TRACKEX_SCREENSHOT_INTERVAL=30  # minutes

# Privacy settings
export TRACKEX_DOMAIN_ONLY=true
export TRACKEX_TITLE_REDACTION=true

# Idle detection
export TRACKEX_IDLE_THRESHOLD=300  # seconds
```

### Policy Configuration

Most settings are controlled by your organization's TrackEx policy:
- Screenshot frequency (off/15/30/60 minutes)
- Domain-only mode for browsers
- Title redaction patterns
- Idle time thresholds

## Troubleshooting

### Common Issues

#### App Won't Start
- Check that you're running macOS 12.6 or later
- Verify the app is properly code-signed: `codesign -v /Applications/TrackEx\ Agent.app`
- Check Console.app for error messages

#### "App is damaged" Error
- The app may not be properly notarized
- Try downloading a fresh copy from your IT department
- Contact support if the issue persists

#### Screenshots Not Working
- Verify Screen Recording permission is granted
- Restart the app after granting permissions
- Check that screenshots are enabled in your organization's policy

#### Network Connection Issues
- Verify the server URL is correct
- Check firewall settings for HTTPS traffic
- Ensure your organization's certificate is trusted

#### High CPU/Memory Usage
- Restart the application
- Check for other monitoring software conflicts
- Report performance issues to support

### Performance Tuning

#### For IT Administrators
```bash
# Reduce monitoring frequency (increases intervals)
export TRACKEX_APP_FOCUS_INTERVAL=5      # seconds between app checks
export TRACKEX_HEARTBEAT_INTERVAL=60     # seconds between heartbeats
export TRACKEX_JOB_POLLING_INTERVAL=30   # seconds between job polls

# Adjust idle threshold
export TRACKEX_IDLE_THRESHOLD=600        # 10 minutes
```

### Logs & Diagnostics

#### Log Locations
- Application logs: `~/Library/Logs/TrackEx-Agent/`
- Database: `~/Library/Application Support/TrackEx/agent.db`
- Crash reports: `~/Library/Application Support/CrashReporter/TrackEx Agent_*.crash`

#### Collecting Diagnostics
1. Click "Send Diagnostics" in the system tray menu
2. Or manually collect logs:
   ```bash
   # View recent logs
   tail -f ~/Library/Logs/TrackEx-Agent/app.log
   
   # Check database status
   sqlite3 ~/Library/Application\ Support/TrackEx/agent.db ".tables"
   ```

#### Privacy Note
Diagnostic logs automatically redact sensitive information including:
- Authentication tokens
- Passwords
- Personal identifiers
- Window titles (when redaction is enabled)

## Network Requirements

### Firewall Configuration

TrackEx Agent requires outbound HTTPS access to:
- Your TrackEx server (configured server URL)
- Apple's notarization services (for updates)

#### Required Domains
```
# Your organization's TrackEx server
your-domain.trackex.com

# For auto-updates (optional)
*.apple.com
```

#### Ports
- **443 (HTTPS)**: All TrackEx API communication
- **80 (HTTP)**: Redirect to HTTPS only

### Corporate Proxy
If using a corporate proxy, ensure it supports:
- TLS 1.2 or later
- Certificate transparency
- WebSocket connections (for real-time features)

## Privacy & Security

### Data Collection
TrackEx Agent collects only work-related data during active sessions:
- Application names and usage duration
- Work session start/end times
- Activity vs. idle status
- Optional screenshots (policy dependent)

### Data NOT Collected
- Keystrokes or clipboard content
- File contents or document data
- Personal applications outside work hours
- Network traffic or browsing history
- Location data or device identifiers

### Security Measures
- **Encryption**: All data transmitted via HTTPS
- **Secure Storage**: Credentials stored in macOS Keychain
- **Code Signing**: App signed with Apple Developer ID
- **Notarization**: Verified by Apple for security
- **Hardened Runtime**: Enhanced security protections

### Data Retention
- Local data retained for 7 days maximum
- Server retention per your organization's policy
- Request data deletion through your system administrator

## Development

### Building from Source

#### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js (via nvm recommended)
nvm install 18
nvm use 18

# Install Tauri CLI
npm install -g @tauri-apps/cli
```

#### Build Commands
```bash
# Development build
./scripts/build.sh

# Development mode (auto-reload)
npm run tauri dev

# Production build
./scripts/package.sh

# Code signing
./scripts/codesign.sh

# Notarization
./scripts/notarize.sh
```

#### Testing
```bash
# Unit tests
cargo test

# Manual testing
# See tests/manual_qa_checklist.md
```

## Support

### Getting Help
1. Check this README for common solutions
2. Review the manual QA checklist for known issues
3. Contact your IT department or system administrator
4. For critical issues, use "Send Diagnostics" in the app

### Reporting Issues
When reporting issues, include:
- macOS version and hardware (Intel/Apple Silicon)
- TrackEx Agent version
- Steps to reproduce the issue
- Relevant log excerpts (redacted)
- Screenshot if applicable

### Enterprise Support
For enterprise customers:
- Dedicated support portal
- Priority response times
- Custom configuration assistance
- Deployment automation support

## License

Copyright (c) 2024 TrackEx. All rights reserved.

This software is proprietary and confidential. Unauthorized distribution, modification, or reverse engineering is prohibited.