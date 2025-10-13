import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface LoginScreenProps {
  onLogin: () => void;
}

interface LoginRequest {
  email: string;
  password: string;
  server_url: string;
}

// Removed AuthStatus interface since not using session restoration

function LoginScreen({ onLogin }: LoginScreenProps) {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  // Hardcoded server URL - users don't need to see or change this
  const serverUrl = "https://www.trackex.app";
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError("");

    try {
      const loginRequest: LoginRequest = {
        email,
        password,
        server_url: serverUrl,
      };

      await invoke("login", { request: loginRequest });
      
      // Start background services after successful login
      try {
        await invoke("start_background_services");
      } catch (error) {
        console.error("Failed to start background services:", error);
        // Don't fail the login if services fail to start
      }
      
      onLogin();
    } catch (error) {
      setError(error as string);
    } finally {
      setLoading(false);
    }
  };

  // No loading screen needed - direct to login

  return (
    <div className="login-container">
      <div className="login-form">
        <div className="login-header">
          <h1>TrackEx Agent</h1>
          <p>Welcome back! Please sign in to continue</p>
        </div>

        {error && (
          <div className="error-message">
            {error}
          </div>
        )}

        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label htmlFor="email">Email</label>
            <input
              id="email"
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="your@email.com"
              required
              autoFocus
            />
          </div>

          <div className="form-group">
            <label htmlFor="password">Password</label>
            <input
              id="password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="••••••••"
              required
            />
          </div>

          <button
            type="submit"
            className="login-button"
            disabled={loading}
          >
            {loading ? "Signing in..." : "Sign In"}
          </button>
        </form>

        <div className="login-footer">
          <p>Secure connection to TrackEx server</p>
        </div>
      </div>
    </div>
  );
}

export default LoginScreen;
