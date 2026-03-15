import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";
import { Onboarding } from "./components/Onboarding";
import { Dashboard } from "./components/Dashboard";
import { SettingsView } from "./components/SettingsView";
import { StatusIndicator } from "./components/StatusIndicator";
import { Sidebar } from "./components/Sidebar";

function App() {
  const [isOnboarded, setIsOnboarded] = useState(false);
  const [isOverlay, setIsOverlay] = useState(false);
  const [activeView, setActiveView] = useState<'dashboard' | 'settings'>('dashboard');

  useEffect(() => {
    const appWindow = getCurrentWindow();
    if (appWindow.label === 'overlay') {
      setIsOverlay(true);
      document.documentElement.classList.add('overlay-mode');
      document.body.classList.add('overlay-mode');
      const rootEl = document.getElementById('root');
      if (rootEl) rootEl.classList.add('overlay-mode-root');
    }
  }, []);

  if (isOverlay) {
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
