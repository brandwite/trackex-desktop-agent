import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import LoginScreen from "./components/LoginScreen";
import ConsentWizard from "./components/ConsentWizard";
import MainView from "./components/MainView";
import PermissionsHelper from "./components/PermissionsHelper";
import "./App.css";

interface AuthStatus {
  is_authenticated: boolean;
  email?: string;
  device_id?: string;
}

interface ConsentStatus {
  accepted: boolean;
  accepted_at?: string;
  version: string;
}

interface PermissionsStatus {
  screen_recording: boolean;
  accessibility: boolean;
}

function App() {
  const [authStatus, setAuthStatus] = useState<AuthStatus | null>(null);
  const [consentStatus, setConsentStatus] = useState<ConsentStatus | null>(null);
  const [permissionsStatus, setPermissionsStatus] = useState<PermissionsStatus | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    checkStatus();
  }, []);

  const checkStatus = async () => {
    try {
      // ONLY check auth and consent on startup - NO PERMISSIONS
      const [auth, consent] = await Promise.all([
        invoke<AuthStatus>("get_auth_status"),
        invoke<ConsentStatus>("get_consent_status")
      ]);

      setAuthStatus(auth);
      setConsentStatus(consent);
      
      // Don't check permissions on startup - only after explicit user action
      setPermissionsStatus(null);
    } catch (error) {
      console.error("Failed to check status:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleLogin = async () => {
    // Force refresh auth status after login
    setLoading(true);
    await checkStatus();
  };

  const handleConsent = async () => {
    // Force refresh status after consent
    setLoading(true);
    await checkStatus();
  };

  const handleLogout = async () => {
    // Clear auth status and refresh
    setAuthStatus(null);
    setConsentStatus(null);
    setPermissionsStatus(null);
    setLoading(true);
    await checkStatus();
  };

  if (loading) {
    return (
      <div className="loading-container">
        <div className="loading-spinner"></div>
        <p>Loading TrackEx Agent...</p>
      </div>
    );
  }

  // Show login if not authenticated
  if (!authStatus?.is_authenticated) {
    return <LoginScreen onLogin={handleLogin} />;
  }

  // After login: Show consent wizard if consent not given
  if (!consentStatus?.accepted) {
    return <ConsentWizard onConsent={handleConsent} />;
  }

  // Check permissions only when user has completed login and consent
  if (permissionsStatus === null) {
    // Check permissions after login - this is when we first check them
    const checkPermissions = async () => {
      try {
        const permissions = await invoke<PermissionsStatus>("get_permissions_status");
        setPermissionsStatus(permissions);
      } catch (error) {
        console.error("Failed to check permissions:", error);
        // If permission check fails, show main app anyway
        setPermissionsStatus({ screen_recording: false, accessibility: false });
      }
    };
    checkPermissions();
    
    return (
      <div className="loading-container">
        <div className="loading-spinner"></div>
        <p>Checking system permissions...</p>
      </div>
    );
  }

  // Show permissions setup if screen recording not granted
  if (!permissionsStatus.screen_recording) {
    const handlePermissionsComplete = async () => {
      const permissions = await invoke<PermissionsStatus>("get_permissions_status");
      setPermissionsStatus(permissions);
    };
    
    return <PermissionsHelper 
      permissionsStatus={permissionsStatus} 
      onPermissionsGranted={handlePermissionsComplete} 
    />;
  }

  // Show main application only after all requirements met
  return <MainView authStatus={authStatus} onLogout={handleLogout} />;
}

export default App;