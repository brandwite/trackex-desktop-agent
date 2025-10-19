import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface AuthStatus {
    is_authenticated: boolean;
    email?: string;
    device_id?: string;
}

interface WorkSessionInfo {
    is_active: boolean;
    started_at?: string;
    current_app?: string;
    idle_time_seconds: number;
    is_paused: boolean;
}

interface TrackingStatus {
    is_tracking: boolean;
    is_paused: boolean;
    current_app?: string;
    idle_time_seconds: number;
}

interface AppInfo {
    name: string;
    app_id: string;
    window_title?: string;
}

interface MainViewProps {
    authStatus: AuthStatus;
    onLogout: () => Promise<void>;
}

interface RecentSession {
    id: string;
    started_at: string;
    ended_at?: string;
    duration: number;
    date: string;
}

// Removed unused SessionHistory interface

function MainView({ authStatus, onLogout }: Readonly<MainViewProps>) {
    const [workSession, setWorkSession] = useState<WorkSessionInfo | null>(null);
    const [trackingStatus, setTrackingStatus] = useState<TrackingStatus | null>(null);
    const [currentApp, setCurrentApp] = useState<AppInfo | null>(null);
    const [recentSessions, setRecentSessions] = useState<RecentSession[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState("");
    const [currentTime, setCurrentTime] = useState(new Date());

    useEffect(() => {

        fetchStatus();

        // Update time every second for the timer
        const timeInterval = setInterval(() => {
            setCurrentTime(new Date());
        }, 1000);

        // Note: do not schedule heartbeats or DB clearing from the UI.
        // Backend services handle heartbeats/app focus/job polling when clocked in.

        // Update current app every 3 seconds for live tracking when clocked in
        const appUpdateInterval = setInterval(async () => {
            if (authStatus.is_authenticated && workSession?.is_active) {
                try {
                    const app = await invoke<AppInfo | null>("get_current_app");
                    setCurrentApp(app);
                } catch (error) {
                    console.error('Failed to get current app:', error);
                }
            }
        }, 3000);

        // Update full status every 10 seconds
        const statusUpdateInterval = setInterval(() => {
            if (authStatus.is_authenticated) {
                fetchStatus();
            }
        }, 10000);

        // Note: Heartbeat, app focus, job polling, and idle detection are now handled by backend services
        // They automatically start when user clocks in and stop when user clocks out
        // No need for frontend intervals - backend services handle this efficiently

        return () => {
            clearInterval(timeInterval);
            clearInterval(appUpdateInterval);
            clearInterval(statusUpdateInterval);
        };
    }, [authStatus.is_authenticated, workSession?.is_active]);

    const fetchStatus = async () => {
        try {
            const [session, tracking, app, sessionsData] = await Promise.all([
                invoke<WorkSessionInfo>("get_work_session"),
                invoke<TrackingStatus>("get_tracking_status"),
                invoke<AppInfo | null>("get_current_app"),
                // eslint-disable-next-line @typescript-eslint/no-explicit-any
                invoke<any>("get_recent_sessions")
            ]);
            setWorkSession(session);
            setTrackingStatus(tracking);
            setCurrentApp(app);

            // Update recent sessions
            if (sessionsData?.sessions) {
                setRecentSessions(sessionsData.sessions);
            }
        } catch (error) {
            console.error("Failed to fetch status:", error);
        }
    };

    const handleClockIn = async () => {
        setLoading(true);
        setError("");

        try {
            await invoke("clock_in");
            await fetchStatus();
        } catch (error) {
            setError(error as string);
        } finally {
            setLoading(false);
        }
    };

    const handleClockOut = async () => {
        setLoading(true);
        setError("");

        try {
            await invoke("clock_out");
            await fetchStatus();
        } catch (error) {
            setError(error as string);
        } finally {
            setLoading(false);
        }
    };

    const handleLogout = async () => {
        setLoading(true);
        try {
            await invoke("logout");
            // Use the onLogout prop to properly reset the app state
            await onLogout();
        } catch (error) {
            setError(error as string);
        } finally {
            setLoading(false);
        }
    };

    const formatTimer = (start?: string) => {
        if (!start) return "00:00:00";
        const startTime = new Date(start);
        const diff = currentTime.getTime() - startTime.getTime();
        const hours = Math.floor(diff / (1000 * 60 * 60));
        const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));
        const seconds = Math.floor((diff % (1000 * 60)) / 1000);
        return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
    };

    const formatDate = (date: Date) => {
        return date.toLocaleDateString('en-US', {
            weekday: 'short',
            month: 'short',
            day: 'numeric',
            year: 'numeric'
        });
    };

    const formatTime = (date: Date) => {
        return date.toLocaleTimeString('en-US', {
            hour12: false,
            hour: '2-digit',
            minute: '2-digit'
        });
    };

    const formatStartTime = (timestamp?: string) => {
        if (!timestamp) return "N/A";
        return new Date(timestamp).toLocaleTimeString('en-US', {
            hour12: true,
            hour: 'numeric',
            minute: '2-digit'
        });
    };

    const formatSessionDuration = (seconds: number) => {
        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${(seconds % 60).toString().padStart(2, '0')} h`;
    };

    return (
        <div className="trackex-main">
            {/* Header */}
            <div className="trackex-header">
                <div className="trackex-logo">
                    <h1>TrackEx</h1>
                </div>
                <div className="trackex-user">
                    <span className="user-email">{authStatus.email}</span>
                    <button onClick={handleLogout} className="logout-btn">
                        Logout
                    </button>
                </div>
            </div>

            {/* Date */}
            <div className="trackex-date">
                {formatDate(currentTime)}
            </div>

            {/* Main Timer Section */}
            <div className="trackex-timer-section">
                {workSession?.is_active ? (
                    <>
                        <div className="timer-display">
                            {formatTimer(workSession.started_at)} <span className="timer-unit">h</span>
                        </div>
                        <div className="timer-subtitle">
                            Started at {formatStartTime(workSession.started_at)}
                        </div>
                        <button
                            onClick={handleClockOut}
                            disabled={loading}
                            className="clock-button clock-out"
                        >
                            {loading ? "Clocking Out..." : "Clock Out"}
                        </button>
                    </>
                ) : (
                    <>
                        <div className="timer-display">
                            {formatTime(currentTime)}
                        </div>
                        <div className="timer-subtitle">
                            Ready to start tracking
                        </div>
                        <button
                            onClick={handleClockIn}
                            disabled={loading}
                            className="clock-button clock-in"
                        >
                            {loading ? "Clocking In..." : "Clock In"}
                        </button>
                    </>
                )}
            </div>

            {/* Error Display */}
            {error && (
                <div className="trackex-error">
                    {error}
                </div>
            )}

            {/* Activity Status */}
            {workSession?.is_active && (
                <div className="trackex-activity">
                    <div className="activity-item">
                        <span className="activity-label">Current App</span>
                        <span className="activity-value">{currentApp?.name || "--"}</span>
                    </div>
                    <div className="activity-item">
                        <span className="activity-label">Status</span>
                        <span className="activity-value">
                            {trackingStatus?.is_paused ? "Paused" : "Active"}
                        </span>
                    </div>
                </div>
            )}

            {/* Recent Sessions Preview */}
            <div className="trackex-recent">
                <h3>Recent Sessions</h3>
                {recentSessions.length > 0 ? (
                    <div className="recent-sessions-list">
                        {recentSessions.slice(0, 3).map((session) => (
                            <div key={session.id} className="recent-session-item">
                                <div className="session-date">{session.date}</div>
                                <div className="session-details">
                                    <div className="session-times">
                                        <span>Clock In</span>
                                        <span>Clock Out</span>
                                        <span></span>
                                    </div>
                                    <div className="session-values">
                                        <span>{formatStartTime(session.started_at)}</span>
                                        <span>{session.ended_at ? formatStartTime(session.ended_at) : "---"}</span>
                                        <span className="session-duration">{formatSessionDuration(session.duration)}</span>
                                    </div>
                                </div>
                            </div>
                        ))}
                    </div>
                ) : (
                    <div className="recent-placeholder">
                        No recent sessions to display
                    </div>
                )}
            </div>
        </div>
    );
}

export default MainView;