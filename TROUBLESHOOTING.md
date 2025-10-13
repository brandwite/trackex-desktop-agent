# Troubleshooting: Services Still Running

## Problem
Events are still being sent even after applying the fixes.

## Root Cause
The desktop agent application needs to be **completely restarted** for the code changes to take effect. The old service instances are still running from before the code was modified.

## Solution

### Step 1: Completely Stop the App
You must stop the running desktop agent completely:

**On Windows:**
1. Close the app window
2. Right-click system tray icon → Quit
3. Open Task Manager (Ctrl+Shift+Esc)
4. Look for "trackex" or "trackex-agent" process
5. If found, End Task
6. Verify it's gone from the Processes list

**On macOS:**
1. Close the app window
2. Right-click Dock icon → Quit
3. Open Activity Monitor
4. Search for "trackex"
5. If found, Force Quit
6. Verify it's terminated

**On Linux:**
1. Close the app
2. Run: `ps aux | grep trackex`
3. If found, `kill -9 <PID>`
4. Verify with `ps aux | grep trackex`

### Step 2: Rebuild the App
The code has been modified, so you need to rebuild:

```bash
# Navigate to the desktop agent directory
cd desktop-agent/trackex-agent

# Rebuild the Tauri app
npm run tauri build
# OR for development
npm run tauri dev
```

### Step 3: Start Fresh
1. Launch the rebuilt app
2. Login (if not already logged in)
3. **DO NOT clock in yet**
4. Check the logs - you should see:
   ```
   "User is not authenticated or not clocked in, services will start after clock-in"
   ```
5. Check network tab - should be SILENT (no events)

### Step 4: Verify Fix
1. Now click "Clock In"
2. Check logs - should see:
   ```
   "Clock in: Local session started with ID X"
   "Clock in: Starting background services"
   ```
3. Wait 3 seconds
4. Check network - events should START appearing now

### Step 5: Verify Clock Out
1. Click "Clock Out"
2. Check logs - should see:
   ```
   "Clock out: Ending local session"
   "Clock out: Services will automatically pause"
   "Service check: auth=true, clocked_in=false, running=true, paused=false, should_run=false"
   ```
3. Wait 3 seconds
4. Check network - events should STOP

## Debug Logging Added

The code now logs service check decisions. Look for these log messages:

```
Service check: auth={bool}, clocked_in={bool}, running={bool}, paused={bool}, should_run={bool}
```

This will help you understand why services are or aren't running.

## Quick Check: Are You Clocked In?

The services will run if you have an active work session. To check:

1. Look at the app UI - does it show "Clock Out" button? (means you're clocked in)
2. Check the database:
   ```sql
   SELECT * FROM work_sessions WHERE is_active = 1;
   ```
3. If you see an active session, that's why events are being sent!

## To Stop Services Without Clock Out

If you want to stop the services without clocking out (for testing):

```javascript
// In the app console or via Tauri command
invoke('stop_background_services')
```

## Common Issues

### Issue 1: "I restarted but it's still sending"
**Cause**: The app process didn't fully terminate
**Fix**: Use Task Manager/Activity Monitor to force quit, then restart

### Issue 2: "Logs show should_run=true but I'm not clocked in"
**Cause**: There's an orphaned work session in the database
**Fix**: 
1. Check: `SELECT * FROM work_sessions WHERE is_active = 1;`
2. If found: `UPDATE work_sessions SET is_active = 0, ended_at = CURRENT_TIMESTAMP WHERE is_active = 1;`
3. Restart app

### Issue 3: "Services won't start even after clock in"
**Cause**: Services global flag not set
**Fix**: Check logs for "Starting background services". If missing, the clock_in command might have failed.

### Issue 4: "Events sending before I even log in"
**Cause**: Old build running
**Fix**: Rebuild the app completely, verify the new binary is running

## Verification Checklist

- [ ] App completely stopped (verified in Task Manager)
- [ ] Code rebuilt (`npm run tauri build` or `tauri dev`)
- [ ] New binary launched
- [ ] Logged in successfully
- [ ] NOT clocked in yet
- [ ] Logs show: "User is not authenticated or not clocked in"
- [ ] Network tab silent (no events)
- [ ] Clock in
- [ ] Logs show: "Starting background services"
- [ ] Events start appearing in network tab
- [ ] Clock out
- [ ] Logs show: "Services will automatically pause"
- [ ] Events stop appearing

## Expected Log Sequence

### At App Startup (Not Clocked In):
```
[INFO] User is not authenticated or not clocked in, services will start after clock-in
```

### At Clock In:
```
[INFO] Clock in: Local session started with ID 123
[INFO] Clock in: Starting background services
[DEBUG] Service check: auth=true, clocked_in=true, running=true, paused=false, should_run=true
```

### While Working (every 1-3 seconds in dev mode):
```
[DEBUG] Service check: auth=true, clocked_in=true, running=true, paused=false, should_run=true
[DEBUG] Event sent: 200 OK
```

### At Clock Out:
```
[INFO] Clock out: Ending local session
[INFO] Clock out: Services will automatically pause
[DEBUG] Service check: auth=true, clocked_in=false, running=true, paused=false, should_run=false
```

### After Clock Out (services paused):
```
[DEBUG] Service check: auth=true, clocked_in=false, running=true, paused=false, should_run=false
(no "Event sent" messages)
```

## Still Having Issues?

1. Check if you have dev mode enabled: `TRACKEX_DEV_SHORT_INTERVALS` environment variable
2. Look for multiple instances of the app running
3. Clear the local database and start fresh
4. Check the compiled binary date to ensure it's the latest build
5. Review all logs from app startup to see the full sequence

## Nuclear Option: Clean Rebuild

If nothing works:

```bash
# Stop the app completely
# Kill all processes

# Clean everything
cd desktop-agent/trackex-agent
rm -rf node_modules
rm -rf src-tauri/target
npm install
npm run tauri build

# Start fresh
# Launch the app
# Test the flow
```



