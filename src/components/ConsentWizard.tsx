import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ConsentWizardProps {
  onConsent: () => void;
}

const CONSENT_VERSION = "1.0.0";

const CONSENT_STEPS = [
  {
    title: "Welcome to TrackEx Agent",
    content: `
      TrackEx Agent helps you track your work time and productivity.
      
      This wizard will explain what data is collected and how it's used.
      
      Your privacy and data security are our top priorities.
    `
  },
  {
    title: "What We Track",
    content: `
      • Work Hours: Clock-in/out times and session duration
      • Application Usage: Which applications you use during work
      • Activity Status: Whether you're actively working or idle
      • Screenshots: Optional screen captures for work verification
      
      We DO NOT track:
      • Keystroke logging or clipboard content
      • Personal files or documents
      • Activity outside of work sessions
      • Private browsing or personal applications
    `
  },
  {
    title: "Privacy Controls",
    content: `
      You maintain control over your data:
      
      • Pause tracking anytime for breaks (15/30/60 minutes)
      • View all your activity data in the TrackEx dashboard
      • Screenshots are policy-controlled by your organization
      • Browser activity uses domain-only mode for privacy
      • Window titles are filtered to protect sensitive information
    `
  },
  {
    title: "Data Usage & Security",
    content: `
      Your data is:
      
      • Used solely for work productivity and time tracking
      • Secured with industry-standard encryption
      • Accessible only to authorized personnel
      • Retained according to your organization's data policy
      • Stored securely on TrackEx servers
      
      You can request data deletion by contacting your administrator.
    `
  }
];

function ConsentWizard({ onConsent }: ConsentWizardProps) {
  const [currentStep, setCurrentStep] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const isLastStep = currentStep === CONSENT_STEPS.length - 1;

  const handleNext = () => {
    if (isLastStep) {
      handleAccept();
    } else {
      setCurrentStep(currentStep + 1);
    }
  };

  const handleBack = () => {
    setCurrentStep(Math.max(0, currentStep - 1));
  };

  const handleAccept = async () => {
    setLoading(true);
    setError("");

    try {
      await invoke("accept_consent", { version: CONSENT_VERSION });
      onConsent();
    } catch (error) {
      setError(error as string);
    } finally {
      setLoading(false);
    }
  };

  const step = CONSENT_STEPS[currentStep];

  return (
    <div className="consent-container">
      <div className="consent-wizard">
        <div className="consent-header">
          <h1>Privacy & Data Collection Consent</h1>
          <div className="progress-bar">
            <div 
              className="progress-fill" 
              style={{ width: `${((currentStep + 1) / CONSENT_STEPS.length) * 100}%` }}
            />
          </div>
          <p>Step {currentStep + 1} of {CONSENT_STEPS.length}</p>
        </div>

        {error && (
          <div className="error-message">
            {error}
          </div>
        )}

        <div className="consent-content">
          <h2>{step.title}</h2>
          <div className="consent-text">
            {step.content.split('\n').map((line, index) => (
              <p key={index}>{line.trim()}</p>
            ))}
          </div>
        </div>

        <div className="consent-actions">
          <button
            type="button"
            onClick={handleBack}
            disabled={currentStep === 0}
            className="back-button"
          >
            Back
          </button>

          <button
            type="button"
            onClick={handleNext}
            disabled={loading}
            className="next-button"
          >
            {loading ? "Processing..." : isLastStep ? "Accept & Continue" : "Next"}
          </button>
        </div>

        {isLastStep && (
          <div className="consent-footer">
            <p>
              By clicking "Accept & Continue", you acknowledge that you have read and 
              understood the data collection practices and consent to the monitoring 
              described above.
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

export default ConsentWizard;
