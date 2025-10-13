# Desktop Agent Refactoring Summary

## Overview
This document summarizes the refactoring work completed to ensure proper authentication and session guards for all background services in the TrackEx desktop agent.

## Problem Statement
The desktop agent was sending app focus/usage events, heartbeats, and polling jobs even when:
1. User was not logged in/authenticated
2. User was not clocked in (no active work session)

This resulted in unnecessary API calls and potential data inconsistencies.

## Solution Implemented

### 1. Authentication & Session Guards

#### New Helper Functions (`sampling/mod.rs`)
Added three core helper functions to check service prerequisites:

```rust
// Check if user has valid authentication token
pub async fn is_authenticated() -> bool

// Check if user has active work session (clocked in)
pub async fn is_clocked_in() -> bool

// Combined check: authenticated AND clocked in AND services running AND not paused
pub async fn should_services_run() -> bool
```

### 2. Service Updates

All background services now check `should_services_run()` before performing any work:

#### App Focus Service (`sampling/app_focus.rs`)
- **Before**: Ran continuously, sent events regardless of session state
- **After**: Only tracks and sends app focus events when authenticated AND clocked in
- Removed redundant session checks within the loop

#### Heartbeat Service (`sampling/heartbeat.rs`)
- **Before**: Checked session state but still ran the loop
- **After**: Only sends heartbeats when authenticated AND clocked in
- Removed duplicate `send_heartbeat_to_backend()` function
- Now uses shared function from `sampling/mod.rs`

#### Job Polling Service (`api/job_polling.rs`)
- **Before**: Polled jobs regardless of authentication or session state
- **After**: Only polls for jobs when authenticated AND clocked in
- Prevents unnecessary API calls when user is not working

#### Idle Detection Service (`sampling/mod.rs`)
- **Before**: Ran when session was active
- **After**: Only runs when authenticated AND clocked in
- Properly resets idle state when service stops

#### Queue Processing Service (`sampling/mod.rs`)
- **Before**: Ran continuously
- **After**: Only processes offline queue when authenticated
- Prevents unauthorized API calls with queued data

#### Sync Service (`sampling/mod.rs`)
- **Before**: Ran continuously
- **After**: Only syncs data when authenticated and online
- Prevents sync attempts without valid credentials

### 3. Code Deduplication

#### Removed Duplicates
- **`send_heartbeat_to_backend()`**: Removed private copy from `heartbeat.rs`, now uses shared public function from `sampling/mod.rs`

#### Consolidated Functions
All modules now use the shared functions from `sampling/mod.rs`:
- `send_heartbeat_to_backend()`
- `send_event_to_backend()`
- `is_authenticated()`
- `is_clocked_in()`
- `should_services_run()`

### 4. Service Behavior

#### When User is Not Authenticated
- All services pause their work
- Services continue to check periodically and resume when authenticated
- No API calls are made
- No data is sent to backend

#### When User is Authenticated but Not Clocked In
- App focus tracking paused
- Heartbeat sending paused
- Idle detection paused
- Job polling paused
- Queue processing continues (to sync previous session data)
- Sync service continues (to upload pending data)

#### When User is Authenticated AND Clocked In
- All services run normally
- App focus tracked and sent to backend
- Heartbeats sent regularly
- Idle detection active
- Jobs polled and processed
- Offline queue processed
- Data synced

## Technical Details

### Service Loop Pattern
All services now follow this pattern:

```rust
loop {
    // Check if services should continue running (authenticated AND clocked in)
    if !should_services_run().await {
        // Stop if user is not authenticated or not clocked in
        if !is_services_running().await {
            break; // Service stopped completely
        }
        // Otherwise, just wait before checking again
        interval.tick().await;
        continue;
    }

    // Perform service work here...

    interval.tick().await;
}
```

### Benefits of This Pattern
1. **Clean separation**: Auth/session logic separate from business logic
2. **Consistent behavior**: All services behave the same way
3. **Easy to maintain**: Single source of truth for service prerequisites
4. **Graceful pause/resume**: Services don't need to restart, just pause
5. **No data loss**: Offline queue preserves data when services are paused

## Files Modified

### Core Changes
- `desktop-agent/trackex-agent/src-tauri/src/sampling/mod.rs`
  - Added helper functions
  - Updated idle detection service
  - Updated queue processing service
  - Updated sync service

- `desktop-agent/trackex-agent/src-tauri/src/sampling/app_focus.rs`
  - Added service guard checks
  - Removed redundant session checks

- `desktop-agent/trackex-agent/src-tauri/src/sampling/heartbeat.rs`
  - Added service guard checks
  - Removed duplicate `send_heartbeat_to_backend()`
  - Uses shared function from mod.rs

- `desktop-agent/trackex-agent/src-tauri/src/api/job_polling.rs`
  - Added service guard checks
  - Only polls when authenticated and clocked in

### Documentation
- `desktop-agent/trackex-agent/REFACTORING_CHECKLIST.md` - Detailed checklist
- `desktop-agent/trackex-agent/REFACTORING_SUMMARY.md` - This document

## Testing Recommendations

### Manual Testing Checklist
1. **Login Flow**
   - [ ] Login with valid credentials
   - [ ] Verify services don't start until clocked in
   - [ ] Logout and verify all services stop

2. **Clock In/Out Flow**
   - [ ] Clock in and verify all services start
   - [ ] Verify app focus tracking works
   - [ ] Verify heartbeats are sent
   - [ ] Clock out and verify services pause
   - [ ] Verify no events sent while clocked out

3. **Offline/Online Handling**
   - [ ] Disconnect network while clocked in
   - [ ] Verify events queued locally
   - [ ] Reconnect network
   - [ ] Verify queued events are sent

4. **Service Lifecycle**
   - [ ] Verify services pause when app loses focus
   - [ ] Verify services resume when app gains focus
   - [ ] Verify clean shutdown on app exit

### Expected Behavior

| User State | Services Running | API Calls Made |
|------------|-----------------|----------------|
| Not logged in | No | No |
| Logged in, not clocked in | Partially (queue/sync only) | Only sync calls |
| Logged in, clocked in | Yes | All normal calls |
| Logged in, clocked in, offline | Yes (queuing locally) | No (offline) |

## Security Improvements
- No unauthorized API calls
- No data sent without valid authentication
- Proper token validation before any network request

## Performance Improvements
- Reduced unnecessary API calls
- Lower battery consumption when not working
- Reduced network usage

## Future Enhancements
1. Add lifecycle hooks for explicit service start/stop on login/logout
2. Implement proper session sync between local and remote DB
3. Add retry logic with exponential backoff for failed requests
4. Add telemetry to track service health
5. Add unit tests for service guards
6. Add integration tests for service lifecycle

## Migration Notes
- No breaking changes for users
- Existing local data will be preserved
- Queued events will be synced on next clock-in
- No database schema changes required

## Rollback Plan
If issues are discovered:
1. Revert to previous commit before this refactoring
2. All services will behave as they did before
3. No data loss expected

## Conclusion
This refactoring ensures that the desktop agent only tracks and sends data when the user is authenticated and actively working (clocked in). This provides:
- Better security
- Reduced unnecessary API calls
- Improved user experience
- Consistent service behavior
- Easier maintenance and debugging

All critical services now properly check authentication and session state before performing any work, fixing the reported issue where events were being sent even when users were not clocked in.


