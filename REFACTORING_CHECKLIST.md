# Desktop Agent Refactoring Checklist

## Critical Issues to Fix

### Authentication & Session Guards
- [x] **App Focus Service**: Only run when user is authenticated AND clocked in
- [x] **Heartbeat Service**: Only run when user is authenticated AND clocked in
- [x] **Job Polling Service**: Only run when user is authenticated AND clocked in
- [x] **Idle Detection Service**: Only run when user is authenticated AND clocked in
- [x] **Queue Processing Service**: Only run when user is authenticated
- [x] **Sync Service**: Only run when user is authenticated

### Code Duplication Cleanup
- [x] **Remove duplicate `send_heartbeat_to_backend`**: Keep only in `sampling/mod.rs`, remove from `sampling/heartbeat.rs`
- [x] **Consolidate event sender**: Ensure all modules use `sampling::send_event_to_backend`
- [ ] **Refactor manual sync in commands.rs**: Use shared sync functions from `sampling/mod.rs`
- [ ] **Centralize queue operations**: Access `offline_queue` only from sampling layer

### Authentication State Management
- [x] **Add authentication check helper**: `is_authenticated()` function
- [x] **Add session check helper**: `is_clocked_in()` function that checks both local and remote
- [x] **Add unified service check helper**: `should_services_run()` function
- [ ] **Sync local and remote session state**: Ensure both DBs stay in sync
- [ ] **Add service lifecycle hooks**: Start/stop services on login/logout and clock-in/clock-out

### Service Lifecycle Improvements
- [x] **Proper service initialization**: Wait for auth AND clock-in before starting services
- [ ] **Graceful service shutdown**: Clean shutdown on logout
- [x] **Service restart on clock-in**: Ensure services start when user clocks in
- [x] **Service pause on clock-out**: Services automatically pause when user clocks out
- [ ] **Handle service errors**: Proper error recovery and retry logic

### Offline/Online Handling
- [ ] **Queue all events when offline**: Ensure no data loss
- [ ] **Sync queued data on reconnect**: Process offline queue when online
- [ ] **Handle partial failures**: Retry failed items
- [ ] **Sync local and remote sessions**: Keep work session state consistent
- [ ] **Sync app usage data**: Upload local app usage sessions to backend

### Best Practices Application
- [ ] **Remove unsafe global state**: Replace `LAST_IDLE_STATE` with safer pattern
- [ ] **Add proper error handling**: Handle all Result types properly
- [ ] **Improve logging**: Add structured logging with context
- [ ] **Add telemetry**: Track service health and errors
- [ ] **Document all public APIs**: Add comprehensive doc comments
- [ ] **Add unit tests**: Test core logic in isolation
- [ ] **Add integration tests**: Test service interactions

### Performance Optimizations
- [ ] **Batch event sending**: Send multiple events in one request
- [ ] **Optimize database queries**: Add indexes where needed
- [ ] **Reduce polling frequency when idle**: Dynamic intervals based on activity
- [ ] **Cache authentication state**: Avoid repeated auth checks
- [ ] **Debounce app focus changes**: Reduce noise from rapid app switches

### Security Improvements
- [ ] **Secure token storage**: Use OS keychain for device token
- [ ] **Validate all inputs**: Sanitize data before storage/transmission
- [ ] **Add request signing**: HMAC signatures for critical requests
- [ ] **Rate limiting**: Prevent service abuse
- [ ] **Audit logging**: Log all security-relevant events

### User Experience Enhancements
- [ ] **Clear error messages**: User-friendly error descriptions
- [ ] **Notification system**: Alert user of important events
- [ ] **Connection status indicator**: Show online/offline state
- [ ] **Sync progress indicator**: Show when syncing data
- [ ] **Manual sync trigger**: Allow user to force sync

## Testing Checklist
- [ ] Test login/logout flow
- [ ] Test clock-in/clock-out flow
- [ ] Test offline mode
- [ ] Test sync after coming back online
- [ ] Test service pause/resume
- [ ] Test app focus tracking
- [ ] Test idle detection
- [ ] Test heartbeat sending
- [ ] Test job polling
- [ ] Test screenshot capture
- [ ] Test queue processing
- [ ] Test error recovery

## Documentation Updates
- [ ] Update README with architecture overview
- [ ] Document service lifecycle
- [ ] Document authentication flow
- [ ] Document offline/online handling
- [ ] Add troubleshooting guide
- [ ] Add development setup guide

## Deployment Checklist
- [ ] Version bump
- [ ] Changelog update
- [ ] Build and test on all platforms
- [ ] Create release notes
- [ ] Deploy to staging
- [ ] Test in production-like environment
- [ ] Deploy to production
- [ ] Monitor for errors

---

## Progress Tracking

**Started:** [Date]
**Completed:** [Date]
**Last Updated:** [Date]

### Notes
- Add any important notes or decisions made during refactoring
- Document breaking changes
- Note any technical debt items to address later

