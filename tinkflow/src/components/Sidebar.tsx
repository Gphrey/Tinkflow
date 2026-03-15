import '../styles/sidebar.css';

interface SidebarProps {
    activeView: 'dashboard' | 'settings';
    onNavigate: (view: 'dashboard' | 'settings') => void;
}

export function Sidebar({ activeView, onNavigate }: SidebarProps) {
    return (
        <nav className="sidebar" aria-label="Main navigation">
            <div className="sidebar-top">
                {/* Logo */}
                <div className="sidebar-logo">
                    <img src="/logo.png" alt="Tinkflow" width="22" height="22" style={{ objectFit: 'contain' }} />
                </div>

                {/* Navigation Items */}
                <div className="sidebar-nav">
                    <button
                        className={`sidebar-nav-item ${activeView === 'dashboard' ? 'active' : ''}`}
                        onClick={() => onNavigate('dashboard')}
                        title="Dashboard"
                        aria-label="Dashboard"
                        aria-current={activeView === 'dashboard' ? 'page' : undefined}
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                            <rect x="3" y="3" width="7" height="7" rx="1" />
                            <rect x="14" y="3" width="7" height="7" rx="1" />
                            <rect x="3" y="14" width="7" height="7" rx="1" />
                            <rect x="14" y="14" width="7" height="7" rx="1" />
                        </svg>
                    </button>

                    <button
                        className={`sidebar-nav-item ${activeView === 'settings' ? 'active' : ''}`}
                        onClick={() => onNavigate('settings')}
                        title="Settings"
                        aria-label="Settings"
                        aria-current={activeView === 'settings' ? 'page' : undefined}
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
                            <circle cx="12" cy="12" r="3" />
                        </svg>
                    </button>
                </div>
            </div>

            <div className="sidebar-bottom">
                <div className="sidebar-version">v0.1</div>
            </div>
        </nav>
    );
}
