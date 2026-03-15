import { useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";
import { Onboarding } from "./components/Onboarding";
import { Dashboard } from "./components/Dashboard";
import { SettingsView } from "./components/SettingsView";
import { StatusIndicator } from "./components/StatusIndicator";
import { Sidebar } from "./components/Sidebar";

// Determine synchronously at module load — before React renders a single frame.
// getCurrentWindow().label is a pure property read (no async) in Tauri 2.
// This prevents the isOverlay flip that caused useRecording to mount twice,
// missing the 'listening' event and leaving the overlay stuck.
const IS_OVERLAY = getCurrentWindow().label === 'overlay';

function App() {
  const [isOnboarded, setIsOnboarded] = useState(false);
  const [activeView, setActiveView] = useState<'dashboard' | 'settings'>('dashboard');

  // Apply overlay CSS classes once on mount (no longer drives rendering logic)
  if (IS_OVERLAY) {
    document.documentElement.classList.add('overlay-mode');
    document.body.classList.add('overlay-mode');
    const rootEl = document.getElementById('root');
    if (rootEl) rootEl.classList.add('overlay-mode-root');
  }

  if (IS_OVERLAY) {
    return (
      <main className="overlay-container">
        <StatusIndicator isOverlay={true} />
      </main>
    );
  }

  return (
    <>
      {!isOnboarded ? (
        <Onboarding onComplete={() => setIsOnboarded(true)} />
      ) : (
        <div className="app-shell">
          <Sidebar activeView={activeView} onNavigate={setActiveView} />
          <main className="content-area">
            {activeView === 'dashboard' && <Dashboard />}
            {activeView === 'settings' && <SettingsView />}
          </main>
        </div>
      )}
    </>
  );
}

export default App;
