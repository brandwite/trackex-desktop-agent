import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface PermissionsStatus {
  screen_recording: boolean;
  accessibility: boolean;
}

interface PermissionsHelperProps {
  permissionsStatus: PermissionsStatus;
  onPermissionsGranted: () => void;
}

function PermissionsHelper({ permissionsStatus, onPermissionsGranted }: PermissionsHelperProps) {
  const [isRequesting, setIsRequesting] = useState(false);
  const [hasStartedSetup, setHasStartedSetup] = useState(false);

  const handleStartPermissionSetup = async () => {
    setHasStartedSetup(true);
    setIsRequesting(true);
    
    try {
      // Request all permissions at once
      await invoke("request_permissions");
      
      // Give macOS time to show all permission dialogs
      setTimeout(() => {
        setIsRequesting(false);
        // Check status after user has had time to approve
        setTimeout(() => {
          onPermissionsGranted();
        }, 1000);
      }, 2000);
    } catch (error) {
      console.error("Failed to request permissions:", error);
      setIsRequesting(false);
    }
  };

  const handleRetryPermissions = () => {
    setHasStartedSetup(false);
    setIsRequesting(false);
  };

  // If permissions are still not granted after setup attempt
  if (hasStartedSetup && !permissionsStatus.screen_recording && !isRequesting) {
    return (
      <div className="permissions-container">
        <div className="permissions-helper">
          <div className="permissions-header">
            <h1>⚠️ Permissions Required</h1>
            <p>TrackEx cannot function without screen recording permission</p>
          </div>

          <div className="permissions-blocked">
            <div className="permission-status">
              <div className="permission-icon">❌</div>
              <div className="permission-text">
                <h3>Screen Recording: Denied</h3>
                <p>This permission is required for TrackEx to work</p>
              </div>
            </div>
          </div>

          <div className="permissions-instructions">
            <h3>To enable TrackEx:</h3>
            <ol>
              <li>Open <strong>System Preferences</strong> → <strong>Security & Privacy</strong></li>
              <li>Click the <strong>Privacy</strong> tab</li>
              <li>Select <strong>Screen Recording</strong> from the list</li>
              <li>Check the box next to <strong>TrackEx Agent</strong></li>
              <li>Restart TrackEx Agent</li>
            </ol>
          </div>

          <div className="permissions-actions">
            <button
              onClick={handleRetryPermissions}
              className="request-permissions-button secondary"
            >
              Try Permission Setup Again
            </button>
          </div>

          <div className="permissions-footer">
            <p>🔒 TrackEx requires these permissions to track work activity securely</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="permissions-container">
      <div className="permissions-helper">
        <div className="permissions-header">
          <h1>🔐 Final Setup Step</h1>
          <p>Great! You're logged in. Now let's enable work tracking permissions.</p>
        </div>

        <div className="permissions-explanation">
          <div className="permission-preview">
            <div className="permission-icon">📺</div>
            <div className="permission-text">
              <h3>What happens next:</h3>
              <p>macOS will ask you to allow TrackEx to record your screen for work tracking. After granting permissions, <strong>you'll need to restart the app once</strong> and then you're all set!</p>
            </div>
          </div>
        </div>

        <div className="permissions-benefits">
          <h3>✅ Once complete, TrackEx will:</h3>
          <ul>
            <li>Automatically track your work sessions</li>
            <li>Monitor application usage for insights</li>
            <li>Take screenshots when requested</li>
            <li>Remember your login (no need to re-enter credentials)</li>
          </ul>
        </div>

        <div className="permissions-actions">
          {isRequesting ? (
            <div className="requesting-permissions">
              <div className="loading-spinner"></div>
              <p>Grant permissions when macOS asks, then restart the app</p>
            </div>
          ) : (
            <button
              onClick={handleStartPermissionSetup}
              className="request-permissions-button primary large"
            >
              Enable Work Tracking
            </button>
          )}
        </div>

        <div className="permissions-footer">
          <p>🔒 After granting permissions, restart TrackEx to complete setup</p>
        </div>
      </div>
    </div>
  );
}

export default PermissionsHelper;