# Services Lifecycle Fix - Critical Issue Resolved

## Problem Identified

From the logs, we discovered that background services were **continuing to run even when not clocked in**:

```
Service check: auth=true, clocked_in=false, running=true, paused=false, should_run=false
```

The issue was:
- `SERVICES_RUNNING` was set to `true` when services started
- **But it was NEVER set back to `false` when user clocked out**
- Services were checking `should_services_run()` every cycle, but the flag remained true
- This caused unnecessary service loops even when `clocked_in=false`

## Root Cause

In `commands.rs`, the `clock_out` function was:
```rust
// ❌ OLD CODE - Services not explicitly stopped
log::info!("Clock out: Services will automatically pause on next check");
```

The comment said services would "automatically pause," but `SERVICES_RUNNING` flag was never set to `false`, so services kept running their check loops.

## Solution Applied

### 1. Clock Out - Explicitly Stop Services

**File**: `desktop-agent/trackex-agent/src-tauri/src/commands.rs`

```rust
// ✅ NEW CODE - Services explicitly stopped
pub async fn clock_out(state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    log::info!("Clock out: Ending local session");
    
    // ✅ 1. Stop background services immediately
    crate::sampling::stop_services().await;
    log::info!("Clock out: Background services stopped");
    
    // ✅ 2. End LOCAL session
    crate::storage::work_session::end_session().await
        .map_err(|e| format!("Failed to end local session: {}", e))?;
    
    // ... send clock_out event to backend ...
}
```

**What this does**:
- Calls `sampling::stop_services()` which sets `SERVICES_RUNNING.store(false, Ordering::Relaxed)`
- This causes all service loops to exit immediately on their next check
- Prevents services from continuing to run when not clocked in

### 2. Clock In - Explicitly Start Services

**File**: `desktop-agent/trackex-agent/src-tauri/src/commands.rs`

```rust
// ✅ Explicitly start services on clock in
log::info!("Clock in: Starting background services");
tokio::spawn(async move {
    crate::sampling::start_services().await;  // Sets SERVICES_RUNNING = true
    crate::sampling::start_all_background_services(app_handle).await;
});
```

**What this does**:
- Calls `sampling::start_services()` which sets `SERVICES_RUNNING.store(true, Ordering::Relaxed)`
- Then starts all background service loops (app focus, heartbeat, idle detection, etc.)

### 3. Enhanced Database Clearing

**File**: `desktop-agent/trackex-agent/src-tauri/src/commands.rs`

```rust
// Clear event and heartbeat queues to prevent residual sends
conn.execute("DELETE FROM event_queue", [])
    .map_err(|e| format!("Failed to clear event_queue: {}", e))?;
conn.execute("DELETE FROM heartbeat_queue", [])
    .map_err(|e| format!("Failed to clear heartbeat_queue: {}", e))?;

// Reset auto-increment counters including queues
conn.execute("DELETE FROM sqlite_sequence WHERE name IN ('app_usage_sessions', 'work_sessions', 'offline_queue', 'event_queue', 'heartbeat_queue')", [])
    .map_err(|e| format!("Failed to reset auto-increment counters: {}", e))?;

log::info!("Local database cleared successfully - all tables and sequences reset");
```

**What this does**:
- Clears `event_queue` and `heartbeat_queue` tables
- Resets their auto-increment sequences
- Ensures no residual queued events remain after clearing

## Expected Behavior After Fix

### When Not Clocked In:
```
Service check: auth=true, clocked_in=false, running=false, paused=false, should_run=false
```
- ✅ `running=false` - Services are stopped
- ✅ No app focus tracking
- ✅ No heartbeats sent
- ✅ No job polling
- ✅ No events generated

### When Clocked In:
```
Service check: auth=true, clocked_in=true, running=true, paused=false, should_run=true
```
- ✅ `running=true` - Services are active
- ✅ App focus tracked every 3 seconds
- ✅ Heartbeats sent every 30 seconds
- ✅ Jobs polled every 30 seconds
- ✅ Events sent to backend

## Testing Steps

1. **Rebuild the app**: `cd desktop-agent/trackex-agent && npm run tauri build`

2. **Clear remote data**: Clear the device data in the remote database (as planned)

3. **Test Not Clocked In**:
   - Launch the app
   - Login
   - **Do NOT clock in**
   - Check logs: Should see `running=false` in service checks
   - Verify no events are sent to backend

4. **Test Clock In**:
   - Click "Clock In"
   - Check logs: Should see "Clock in: Starting background services"
   - Check logs: Should see `running=true` in service checks
   - Verify events ARE sent to backend

5. **Test Clock Out**:
   - Click "Clock Out"
   - Check logs: Should see "Clock out: Background services stopped"
   - Check logs: Should see `running=false` in service checks
   - Verify no more events are sent

## Files Changed

1. ✅ `desktop-agent/trackex-agent/src-tauri/src/commands.rs`
   - Added explicit `stop_services()` call in `clock_out`
   - Added explicit `start_services()` call in `clock_in`
   - Enhanced `clear_local_database` to clear queues

2. ✅ `desktop-agent/trackex-agent/src/components/MainView.tsx`
   - Added temporary one-time `clear_local_database` call on mount when authenticated
   - Added console logging for debugging

## Summary

The critical issue was that `SERVICES_RUNNING` flag was never set to `false` on clock out. This caused service loops to continue running even when `clocked_in=false`. The fix explicitly calls `stop_services()` on clock out and `start_services()` on clock in, ensuring the service lifecycle is properly managed based on work session state.



