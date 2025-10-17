import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import LoginScreen from "./components/LoginScreen";
import ConsentWizard from "./components/ConsentWizard";
import MainView from "./components/MainView";
// import PermissionsHelper from "./components/PermissionsHelper";
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

// interface PermissionsStatus {
//   screen_recording: boolean;
//   accessibility: boolean;
// }

function App() {
  const [authStatus, setAuthStatus] = useState<AuthStatus | null>(null);
  const [consentStatus, setConsentStatus] = useState<ConsentStatus | null>(null);
//   const [permissionsStatus, setPermissionsStatus] = useState<PermissionsStatus | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    checkStatus();
  }, []);

  const checkStatus = async () => {
    try {
      // ONLY check auth and consent on startup - NO PERMISSIONS
      // Add timeout to prevent hanging
      const timeoutPromise = new Promise<never>((_, reject) => 
        setTimeout(() => reject(new Error("Timeout checking status")), 15000)
      );
      
      const [auth, consent] = await Promise.race([
        Promise.all([
          invoke<AuthStatus>("get_auth_status"),
          invoke<ConsentStatus>("get_consent_status")
        ]),
        timeoutPromise
      ]);

      setAuthStatus(auth);
      setConsentStatus(consent);
      
      // Don't check permissions on startup - only after explicit user action
      // setPermissionsStatus(null);
    } catch (error) {
      console.error("Failed to check status:", error);
      // On error, assume not authenticated and not consented
      // This allows the app to show login screen instead of hanging
      setAuthStatus({ is_authenticated: false });
      setConsentStatus({ accepted: false, version: "1.0" });
      // setPermissionsStatus(null);
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
    // setPermissionsStatus(null);
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

  // Skip permissions check for now - go straight to main app
  // Show main application after login and consent
  return <MainView authStatus={authStatus} onLogout={handleLogout} />;
}

export default App;