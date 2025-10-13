# TrackEx Agent - Manual QA Checklist

## Pre-Testing Setup
- [ ] macOS 12.6+ (Intel or Apple Silicon)
- [ ] Valid TrackEx server URL and credentials
- [ ] Screen Recording permission available to grant
- [ ] Network connectivity to TrackEx backend

## Installation & First Run
- [ ] Download and install .pkg without errors
- [ ] App launches after installation
- [ ] Gatekeeper allows execution (no "unidentified developer" error)
- [ ] Menu bar icon appears immediately
- [ ] First-run consent wizard displays

## Consent Wizard
- [ ] All 4 consent steps display correctly
- [ ] Progress bar updates properly
- [ ] "Back" button works (disabled on first step)
- [ ] "Next" button advances through steps
- [ ] Final step shows "Accept & Continue"
- [ ] Consent acceptance is stored and not re-shown
- [ ] Can proceed to login after consent

## Authentication
- [ ] Login screen appears after consent
- [ ] Server URL field accepts valid URLs
- [ ] Email and password fields work correctly
- [ ] Invalid credentials show error message
- [ ] Valid credentials proceed to permissions check
- [ ] Credentials stored securely (persist after restart)
- [ ] Logout clears stored credentials

## Permissions
- [ ] Permissions helper shows if permissions missing
- [ ] Screen Recording permission status accurate
- [ ] Accessibility permission status accurate
- [ ] "Open System Preferences" button works
- [ ] Can continue with partial permissions
- [ ] Permissions status updates after granting

## Clock In/Out Flow
- [ ] Initial state shows "Clock In" button
- [ ] Clock In starts work session
- [ ] Timer begins counting after clock in
- [ ] Current app detection works
- [ ] Idle time calculation accurate
- [ ] Clock Out ends work session
- [ ] Can clock in again after clocking out

## Application Monitoring
- [ ] Current app updates every 2-5 seconds (dev mode: 1 second)
- [ ] App name displays correctly
- [ ] Bundle ID captured when available
- [ ] Browser detection works (Chrome, Safari, Firefox)
- [ ] Domain-only mode for browsers
- [ ] Window title redaction working
- [ ] App focus events queued properly

## Idle Detection
- [ ] Idle time increases when inactive
- [ ] Idle threshold respected (default 5 minutes)
- [ ] Status changes to "idle" when threshold reached
- [ ] Activity resumes when user returns
- [ ] Idle time resets on activity

## Heartbeats
- [ ] Heartbeats sent every 30 seconds (dev mode: 10 seconds)
- [ ] Current app included in heartbeat
- [ ] Idle status included in heartbeat
- [ ] Heartbeats queue when offline
- [ ] Queued heartbeats sent when online

## Screenshots
- [ ] Screenshot function works (test via command)
- [ ] Screen capture permission required
- [ ] Image quality acceptable
- [ ] File size reasonable (<2MB typical)
- [ ] Base64 encoding works
- [ ] Screenshot upload integration

## System Tray/Menu Bar
- [ ] Icon always visible in menu bar
- [ ] Left click shows main window
- [ ] Right click shows context menu
- [ ] "Show TrackEx" menu item works
- [ ] "Pause Tracking" pauses services
- [ ] "Resume Tracking" resumes services
- [ ] "Send Diagnostics" option available
- [ ] "Quit" closes application

## Pause/Resume Controls
- [ ] 15-minute pause button works
- [ ] 30-minute pause button works
- [ ] 60-minute pause button works
- [ ] Pause stops all monitoring
- [ ] Resume restarts monitoring
- [ ] Auto-resume after timeout works
- [ ] Manual resume works during pause

## Offline Operation
- [ ] Events queue when network unavailable
- [ ] Heartbeats queue when network unavailable
- [ ] Queued items process when network returns
- [ ] No data loss during offline periods
- [ ] Retry logic works with exponential backoff
- [ ] Queue cleanup removes old entries

## Job Polling
- [ ] Polls for jobs every 25 seconds (dev mode: 5 seconds)
- [ ] Processes screenshot job requests
- [ ] Uploads screenshot via presigned URL
- [ ] Sends completion events
- [ ] Handles job polling errors gracefully

## Performance
- [ ] CPU usage <1% during normal operation
- [ ] Memory usage <150MB typical
- [ ] No memory leaks during extended use
- [ ] App responsive during monitoring
- [ ] Battery impact minimal
- [ ] No UI freezing or delays

## Privacy & Security
- [ ] Device token stored in macOS Keychain
- [ ] Logs redact sensitive information
- [ ] Only HTTPS connections used
- [ ] No data collection when paused
- [ ] No data collection when clocked out
- [ ] Domain-only mode for browsers working
- [ ] Title redaction patterns working

## Error Handling
- [ ] Network errors handled gracefully
- [ ] Server errors don't crash app
- [ ] Permission denied handled properly
- [ ] Invalid server responses handled
- [ ] Malformed data handled safely
- [ ] Recovery from temporary failures

## Multi-Monitor Support
- [ ] App works with multiple displays
- [ ] Screenshot captures correct display
- [ ] Window detection works across monitors
- [ ] Menu bar icon visible on primary display

## macOS Integration
- [ ] Respects system dark/light mode
- [ ] Native macOS UI patterns
- [ ] Accessibility features compatible
- [ ] VoiceOver compatibility (basic)
- [ ] Keyboard navigation works
- [ ] App follows macOS HIG

## Development Mode
- [ ] TRACKEX_DEV_SHORT_INTERVALS env var works
- [ ] Shorter polling intervals in dev mode
- [ ] Debug logging more verbose
- [ ] Dev features don't appear in production

## Auto-Update (if implemented)
- [ ] Update notifications appear
- [ ] Update download works
- [ ] Update installation works
- [ ] App restarts properly after update
- [ ] No data loss during update

## Edge Cases
- [ ] App handles system sleep/wake
- [ ] Network switching (WiFi to Ethernet)
- [ ] User switching (macOS fast user switching)
- [ ] Disk space full scenarios
- [ ] Clock changes (timezone, DST)
- [ ] Very long window titles
- [ ] Special characters in app names

## Cleanup Testing
- [ ] Uninstall removes all files
- [ ] Keychain entries removed on logout
- [ ] Database files in expected location
- [ ] Log files don't grow indefinitely
- [ ] Temp files cleaned up properly

## Security Testing
- [ ] Code signature valid
- [ ] Notarization ticket present
- [ ] Hardened runtime enabled
- [ ] Entitlements minimal and correct
- [ ] No sensitive data in logs
- [ ] HTTPS certificate validation

## Pass Criteria
✅ All core functionality items must pass
✅ No crashes or data loss
✅ Performance within acceptable limits
✅ Security requirements met
⚠️ Edge cases should be documented if failing
⚠️ Non-critical features can be addressed in future versions

## Notes
- Test on both Intel and Apple Silicon Macs
- Test with different macOS versions (12.6, 13.x, 14.x)
- Test with corporate network policies
- Document any behavioral differences
- Report performance metrics for baseline


