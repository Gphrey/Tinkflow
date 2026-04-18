import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Onboarding } from "./components/Onboarding";
import { Dashboard } from "./components/Dashboard";
import { SettingsView } from "./components/SettingsView";
import { Monitor } from "./components/Monitor";
import { StatusIndicator } from "./components/StatusIndicator";
import { Sidebar } from "./components/Sidebar";

// Determine synchronously at module load — before React renders a single frame.
// getCurrentWindow().label is a pure property read (no async) in Tauri 2.
// This prevents the isOverlay flip that caused useRecording to mount twice,
// missing the 'listening' event and leaving the overlay stuck.
const IS_OVERLAY = getCurrentWindow().label === 'overlay';

interface AppSettings {
  whisper_model: string;
  llm_model: string;
  audio_device_name: string;
  launch_at_startup: boolean;
  onboarding_completed: boolean;
}

function App() {
  const [isLoading, setIsLoading] = useState(true);
  const [isOnboarded, setIsOnboarded] = useState(false);
  const [activeView, setActiveView] = useState<'dashboard' | 'settings' | 'monitor'>('dashboard');

  // Apply overlay CSS classes once on mount (no longer drives rendering logic)
  if (IS_OVERLAY) {
    document.documentElement.classList.add('overlay-mode');
    document.body.classList.add('overlay-mode');
    const rootEl = document.getElementById('root');
    if (rootEl) rootEl.classList.add('overlay-mode-root');
  }

  // Check persisted onboarding state on mount
  useEffect(() => {
    if (IS_OVERLAY) return;
    (async () => {
      try {
        const settings = await invoke<AppSettings>('get_app_settings');
        if (settings.onboarding_completed) {
          setIsOnboarded(true);
        }
      } catch (e) {
        console.error('Failed to load settings:', e);
      } finally {
        setIsLoading(false);
      }
    })();
  }, []);

  const handleOnboardingComplete = async () => {
    try {
      const settings = await invoke<AppSettings>('get_app_settings');
      await invoke('update_app_settings', {
        newSettings: { ...settings, onboarding_completed: true },
      });
    } catch (e) {
      console.error('Failed to persist onboarding state:', e);
    }
    setIsOnboarded(true);
  };

  if (IS_OVERLAY) {
    return (
      <main className="overlay-container">
        <StatusIndicator isOverlay={true} />
      </main>
    );
  }

  // Brief loading state while we check settings — prevents flash of onboarding
  if (isLoading) {
    return null;
  }

  return (
    <>
      {!isOnboarded ? (
        <Onboarding onComplete={handleOnboardingComplete} />
      ) : (
        <div className="app-shell">
          <Sidebar activeView={activeView} onNavigate={setActiveView} />
          <main className="content-area">
            {activeView === 'dashboard' && <Dashboard />}
            {activeView === 'settings' && <SettingsView />}
            {activeView === 'monitor' && <Monitor />}
          </main>
        </div>
      )}
    </>
  );
}

export default App;

