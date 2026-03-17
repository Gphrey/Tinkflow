import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import '../styles/dashboard.css';

interface AppSettings {
    whisper_model: string;
    llm_model: string;
    audio_device_name: string;
    onboarding_completed: boolean;
}

/** Curated list of recommended models for text polishing — lightweight and fast */
const RECOMMENDED_MODELS = [
    { name: 'phi3:mini', label: 'Phi-3 Mini', size: '2.3 GB', desc: 'Microsoft — fast, great at text cleanup', recommended: true },
    { name: 'llama3.2:1b', label: 'Llama 3.2 1B', size: '1.3 GB', desc: 'Meta — ultra lightweight, decent quality' },
    { name: 'llama3.2:3b', label: 'Llama 3.2 3B', size: '2.0 GB', desc: 'Meta — balanced speed & accuracy' },
    { name: 'gemma2:2b', label: 'Gemma 2 2B', size: '1.6 GB', desc: 'Google — compact and capable' },
    { name: 'mistral:7b', label: 'Mistral 7B', size: '4.1 GB', desc: 'Mistral AI — high quality, needs more RAM' },
    { name: 'qwen2.5:3b', label: 'Qwen 2.5 3B', size: '1.9 GB', desc: 'Alibaba — good multilingual support' },
];

export function Dashboard() {
    const [ollamaStatus, setOllamaStatus] = useState<'checking' | 'connected' | 'offline'>('checking');
    const [installedModels, setInstalledModels] = useState<string[]>([]);
    const [pullingModel, setPullingModel] = useState<string | null>(null);
    const [cancelling, setCancelling] = useState(false);
    const [ollamaProgress, setOllamaProgress] = useState<number>(0);
    const [pullError, setPullError] = useState<string | null>(null);
    const [settings, setSettings] = useState<AppSettings>({
        whisper_model: 'tiny.en',
        llm_model: '',
        audio_device_name: 'default',
        onboarding_completed: false,
    });

    // Derived: models installed by user that aren't in RECOMMENDED_MODELS
    const otherModels = installedModels.filter(m =>
        !RECOMMENDED_MODELS.some(curated => m.startsWith(curated.name.split(':')[0]))
    );

    useEffect(() => {
        loadSettings();
        checkOllama();

        const unlistenOllama = listen<number>('ollama-download-progress', (event) => {
            if (event.payload === -1) {
                // Cancellation sentinel
                setPullingModel(null);
                setCancelling(false);
                setOllamaProgress(0);
                setPullError('Download cancelled.');
            } else {
                setOllamaProgress(event.payload);
            }
        });

        return () => {
            unlistenOllama.then(f => f());
        };
    }, []);

    const loadSettings = async () => {
        try {
            const currentSettings = await invoke<AppSettings>('get_app_settings');
            setSettings(currentSettings);
        } catch (e) {
            console.error("Failed to load settings:", e);
        }
    };

    const updateSetting = async (key: keyof AppSettings, value: string) => {
        const newSettings = { ...settings, [key]: value };
        setSettings(newSettings);
        try {
            await invoke('update_app_settings', { newSettings });
        } catch (e) {
            console.error("Failed to save settings:", e);
        }
    };

    const checkOllama = async () => {
        setOllamaStatus('checking');
        try {
            const healthy = await invoke<boolean>('check_ollama_status');
            if (healthy) {
                setOllamaStatus('connected');
                await refreshModels();
            } else {
                setOllamaStatus('offline');
            }
        } catch {
            setOllamaStatus('offline');
        }
    };

    const refreshModels = async () => {
        try {
            const modelList = await invoke<string[]>('list_ollama_models');
            setInstalledModels(modelList);
            const currentSettings = await invoke<AppSettings>('get_app_settings');
            if (!currentSettings.llm_model) {
                const firstInstalled = RECOMMENDED_MODELS.find(m =>
                    modelList.some(installed => installed.startsWith(m.name.split(':')[0]))
                );
                if (firstInstalled) {
                    const match = modelList.find(installed => installed.startsWith(firstInstalled.name.split(':')[0]));
                    if (match) updateSetting('llm_model', match);
                } else if (modelList.length > 0) {
                    updateSetting('llm_model', modelList[0]);
                }
            }
        } catch { /* ignore */ }
    };

    const isModelInstalled = (modelName: string) => {
        const baseName = modelName.split(':')[0];
        return installedModels.some(m => m.startsWith(baseName));
    };

    const handleModelSelect = async (model: typeof RECOMMENDED_MODELS[0]) => {
        if (isModelInstalled(model.name)) {
            const match = installedModels.find(m => m.startsWith(model.name.split(':')[0]));
            if (match) updateSetting('llm_model', match);
        } else {
            setPullingModel(model.name);
            setCancelling(false);
            setOllamaProgress(0);
            setPullError(null);
            try {
                await invoke('pull_ollama_model', { modelName: model.name });
                await refreshModels();
                updateSetting('llm_model', model.name);
            } catch (e: any) {
                if (!e?.toString().includes('cancelled')) {
                    setPullError(`Failed to download ${model.label}: ${e}`);
                }
            } finally {
                setPullingModel(null);
                setCancelling(false);
            }
        }
    };

    const handleCancelPull = async () => {
        setCancelling(true);
        await invoke('cancel_download');
        // Rust emits -1.0 progress which resets state in the listener above
    };

    return (
        <div className="dashboard">
            {/* Header */}
            <div className="dash-header">
                <div className="dash-logo-row">
                    <img src="/logo.png" alt="Tinkflow" width="26" height="26" style={{ objectFit: 'contain' }} />
                    <h1 className="dash-title">Tinkflow</h1>
                </div>
                <p className="dash-subtitle">Voice-to-text for developers — local, private, fast.</p>
            </div>

            {/* Quick Start Card */}
            <div className="dash-card accent-card">
                <div className="dash-card-header">
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                        <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
                        <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
                        <line x1="12" x2="12" y1="19" y2="22" />
                    </svg>
                    <span>Quick Start</span>
                </div>
                <div className="hotkey-display-inline">
                    <span>Hold</span>
                    <kbd>Ctrl</kbd><span>+</span><kbd>Shift</kbd><span>+</span><kbd>Space</kbd>
                    <span>to dictate</span>
                </div>
                <p className="dash-hint">Speak naturally, then release. Text appears where your cursor is.</p>
            </div>

            {/* Status Cards Row */}
            <div className="dash-cards-row">
                <div className="dash-card">
                    <div className="dash-card-header">
                        <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                            <polyline points="7 10 12 15 17 10" />
                            <line x1="12" y1="15" x2="12" y2="3" />
                        </svg>
                        <span>Whisper Model</span>
                    </div>
                    <div className="dash-status-row">
                        <div className="status-dot connected-dot" />
                        <span className="dash-status-text connected-text">{settings.whisper_model} loaded</span>
                    </div>
                </div>

                <div className="dash-card">
                    <div className="dash-card-header">
                        <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M12 3l1.912 5.813a2 2 0 0 0 1.275 1.275L21 12l-5.813 1.912a2 2 0 0 0-1.275 1.275L12 21l-1.912-5.813a2 2 0 0 0-1.275-1.275L3 12l5.813-1.912a2 2 0 0 0 1.275-1.275L12 3z" />
                        </svg>
                        <span>LLM Polish</span>
                    </div>
                    {ollamaStatus === 'checking' && (
                        <div className="dash-status-row">
                            <div className="status-dot checking-dot" />
                            <span className="dash-status-text">Checking...</span>
                        </div>
                    )}
                    {ollamaStatus === 'connected' && (
                        <div className="dash-status-row">
                            <div className="status-dot connected-dot" />
                            <span className="dash-status-text connected-text">
                                {settings.llm_model ? settings.llm_model : 'Connected'}
                            </span>
                        </div>
                    )}
                    {ollamaStatus === 'offline' && (
                        <div className="dash-status-row">
                            <div className="status-dot not-found-dot" />
                            <span className="dash-status-text offline-text">Offline</span>
                        </div>
                    )}
                </div>
            </div>

            {/* LLM Model Selector — Curated List */}
            {ollamaStatus === 'connected' && (
                <div className="dash-card">
                    <div className="dash-card-header">
                        <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <circle cx="12" cy="12" r="3" />
                            <path d="M12 1v6m0 6v6m8.66-13l-5.2 3m-6.92 4l-5.2 3M20.66 17l-5.2-3M9.34 10l-5.2-3" />
                        </svg>
                        <span>LLM Models</span>
                    </div>

                    <div className="model-list">
                        {RECOMMENDED_MODELS.map(model => {
                            const installed = isModelInstalled(model.name);
                            const isSelected = settings.llm_model.startsWith(model.name.split(':')[0]);
                            const isPulling = pullingModel === model.name;

                            return (
                                <button
                                    key={model.name}
                                    className={`model-item ${isSelected ? 'model-selected' : ''} ${installed ? 'model-installed' : ''}`}
                                    onClick={() => handleModelSelect(model)}
                                    disabled={isPulling}
                                >
                                    <div className="model-item-left">
                                        <div className="model-item-name">
                                            {model.label}
                                            {model.recommended && <span className="model-badge">Recommended</span>}
                                        </div>
                                        <div className="model-item-desc">{model.desc}</div>
                                    </div>
                                    <div className="model-item-right">
                                        <span className="model-item-size">{model.size}</span>
                                        {isPulling ? (
                                            <div className="model-item-status pulling" style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-end', gap: '4px' }}>
                                                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                                                    <div className="mini-spinner" />
                                                    <span>{cancelling ? 'Cancelling…' : 'Downloading...'}</span>
                                                </div>
                                                <span className="font-mono text-xs" style={{ color: 'var(--accent-cyan)' }}>{ollamaProgress.toFixed(1)}%</span>
                                                <button
                                                    onClick={(e) => { e.stopPropagation(); handleCancelPull(); }}
                                                    disabled={cancelling}
                                                    style={{ fontSize: '0.7rem', padding: '2px 8px', marginTop: '2px', borderRadius: '4px', border: '1px solid rgba(239,68,68,0.4)', background: 'rgba(239,68,68,0.08)', color: 'var(--accent-red, #ef4444)', cursor: cancelling ? 'not-allowed' : 'pointer', opacity: cancelling ? 0.5 : 1 }}
                                                >
                                                    {cancelling ? 'Cancelling…' : 'Cancel'}
                                                </button>
                                            </div>
                                        ) : installed ? (
                                            <div className="model-item-status installed">
                                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                                                    <polyline points="20 6 9 17 4 12" />
                                                </svg>
                                                <span>{isSelected ? 'Active' : 'Installed'}</span>
                                            </div>
                                        ) : (
                                            <div className="model-item-status download">
                                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                                                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                                                    <polyline points="7 10 12 15 17 10" />
                                                    <line x1="12" y1="15" x2="12" y2="3" />
                                                </svg>
                                                <span>Download</span>
                                            </div>
                                        )}
                                    </div>
                                </button>
                            );
                        })}
                    </div>

                    {otherModels.length > 0 && (
                        <>
                            <div className="dash-card-header" style={{ marginTop: '1.5rem', opacity: 0.8 }}>
                                <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                                    <path d="M4 22h14a2 2 0 0 0 2-2V7.5L14.5 2H6a2 2 0 0 0-2 2v4" />
                                    <polyline points="14 2 14 8 20 8" />
                                    <path d="M2 15h10" />
                                    <path d="m9 18 3-3-3-3" />
                                </svg>
                                <span>Other Installed Models</span>
                            </div>
                            <div className="model-list" style={{ marginTop: '0.5rem' }}>
                                {otherModels.map(modelName => {
                                    const isSelected = settings.llm_model === modelName;
                                    return (
                                        <button
                                            key={modelName}
                                            className={`model-item ${isSelected ? 'model-selected' : ''} model-installed`}
                                            onClick={() => updateSetting('llm_model', modelName)}
                                        >
                                            <div className="model-item-left">
                                                <div className="model-item-name">{modelName}</div>
                                                <div className="model-item-desc">Locally detected Ollama model</div>
                                            </div>
                                            <div className="model-item-right">
                                                <div className="model-item-status installed">
                                                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                                                        <polyline points="20 6 9 17 4 12" />
                                                    </svg>
                                                    <span>{isSelected ? 'Active' : 'Installed'}</span>
                                                </div>
                                            </div>
                                        </button>
                                    );
                                })}
                            </div>
                        </>
                    )}

                    {pullError && (
                        <p className="pull-error mt-2">{pullError}</p>
                    )}
                </div>
            )}

            {ollamaStatus === 'offline' && (
                <div className="dash-card">
                    <p className="dash-hint" style={{ marginBottom: '0.5rem' }}>
                        Install <a href="https://ollama.com/download" target="_blank" rel="noopener noreferrer" className="ollama-install-link">Ollama</a> for text polishing. Without it, raw transcriptions are injected.
                    </p>
                    <button className="ghost-btn" onClick={checkOllama}>Retry Connection</button>
                </div>
            )}
        </div>
    );
}
