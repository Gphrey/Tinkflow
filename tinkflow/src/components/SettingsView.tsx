import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import '../styles/settings.css';

interface AppSettings {
    whisper_model: string;
    llm_model: string;
    audio_device_name: string;
    launch_at_startup: boolean;
    onboarding_completed: boolean;
    dictation_hotkey: string;
    injection_mode: string;
    transcription_quality: string;
    dictation_enabled: boolean;
    context_profiles: ContextProfile[];
}

interface ContextProfile {
    context: string;
    enabled: boolean;
    tone: string;
    preserve_symbols: boolean;
    remove_fillers: boolean;
    punctuation: boolean;
}

interface CorrectionEntry {
    id: number;
    spoken: string;
    replacement: string;
    enabled: boolean;
    created_at_ms: number;
}

const WHISPER_MODELS = [
    { name: 'tiny.en', label: 'Tiny (~75MB)' },
    { name: 'base.en', label: 'Base (~150MB)' },
    { name: 'small.en', label: 'Small (~500MB)' },
    { name: 'medium.en', label: 'Medium (~1.5GB)' },
];

export function SettingsView() {
    const [settings, setSettings] = useState<AppSettings>({
        whisper_model: 'tiny.en',
        llm_model: '',
        audio_device_name: 'default',
        launch_at_startup: false,
        onboarding_completed: false,
        dictation_hotkey: 'Ctrl+Space',
        injection_mode: 'auto',
        transcription_quality: 'balanced',
        dictation_enabled: true,
        context_profiles: [],
    });
    const [corrections, setCorrections] = useState<CorrectionEntry[]>([]);
    const [newSpoken, setNewSpoken] = useState('');
    const [newReplacement, setNewReplacement] = useState('');
    const [audioDevices, setAudioDevices] = useState<string[]>(['default']);
    const [installedWhisperModels, setInstalledWhisperModels] = useState<string[]>([]);
    const [pullingWhisper, setPullingWhisper] = useState<boolean>(false);
    const [whisperProgress, setWhisperProgress] = useState<number>(0);
    const [autoStartEnabled, setAutoStartEnabled] = useState<boolean>(false);

    useEffect(() => {
        loadSettingsAndDevices();

        const unlistenWhisper = listen<number>('model-download-progress', (event) => {
            setWhisperProgress(event.payload);
        });

        return () => {
            unlistenWhisper.then(f => f());
        };
    }, []);

    const loadSettingsAndDevices = async () => {
        try {
            const currentSettings = await invoke<AppSettings>('get_app_settings');
            setSettings(currentSettings);
            const devices = await invoke<string[]>('get_audio_devices');
            setAudioDevices(devices);
            const whisperList = await invoke<string[]>('list_installed_whisper_models');
            setInstalledWhisperModels(whisperList);
            setCorrections(await invoke<CorrectionEntry[]>('list_corrections'));

            // Sync autostart toggle from the OS-level plugin (source of truth)
            try {
                const enabled = await isEnabled();
                setAutoStartEnabled(enabled);
            } catch {
                setAutoStartEnabled(currentSettings.launch_at_startup);
            }
        } catch (e) {
            console.error("Failed to load settings:", e);
        }
    };

    const updateSetting = async (key: keyof AppSettings, value: string | boolean | ContextProfile[]) => {
        const newSettings = { ...settings, [key]: value };
        setSettings(newSettings);
        try {
            await invoke('update_app_settings', { newSettings });
        } catch (e) {
            console.error("Failed to save settings:", e);
        }
    };

    const handleAutoStartToggle = async (checked: boolean) => {
        setAutoStartEnabled(checked);
        try {
            if (checked) {
                await enable();
            } else {
                await disable();
            }
            await updateSetting('launch_at_startup', checked);
        } catch (e) {
            console.error('Failed to toggle autostart:', e);
            // Revert on failure
            setAutoStartEnabled(!checked);
        }
    };

    const addCorrection = async () => {
        try {
            await invoke('add_correction', { spoken: newSpoken, replacement: newReplacement });
            setNewSpoken('');
            setNewReplacement('');
            setCorrections(await invoke<CorrectionEntry[]>('list_corrections'));
        } catch (e) {
            console.error('Failed to add correction:', e);
        }
    };

    const removeCorrection = async (id: number) => {
        await invoke('remove_correction', { id });
        setCorrections(await invoke<CorrectionEntry[]>('list_corrections'));
    };

    const toggleCorrection = async (entry: CorrectionEntry) => {
        await invoke('set_correction_enabled', { id: entry.id, enabled: !entry.enabled });
        setCorrections(await invoke<CorrectionEntry[]>('list_corrections'));
    };

    const updateProfile = (index: number, patch: Partial<ContextProfile>) => {
        const context_profiles = settings.context_profiles.map((profile, i) =>
            i === index ? { ...profile, ...patch } : profile
        );
        updateSetting('context_profiles', context_profiles);
    };
    const refreshAudioDevices = async () => {
        try {
            const devices = await invoke<string[]>('get_audio_devices');
            setAudioDevices(devices);
            if (settings.audio_device_name !== 'default' && !devices.includes(settings.audio_device_name)) {
                console.warn(`[Audio] Previously selected device '${settings.audio_device_name}' no longer available - resetting to default`);
                updateSetting('audio_device_name', 'default');
            }
        } catch (e) {
            console.error('Failed to refresh audio devices:', e);
        }
    };

    return (
        <div className="settings-view">
            <div className="settings-header">
                <h1 className="settings-title">Settings</h1>
                <p className="settings-subtitle">Configure your Tinkflow experience</p>
            </div>

            {/* General Section */}
            <div className="settings-section">
                <h3 className="settings-section-title">General</h3>

                <div className="settings-card">
                    <div className="setting-row">
                        <div className="setting-info">
                            <span className="setting-label">Launch at Startup</span>
                            <span className="setting-desc">Automatically start Tinkflow when your computer boots</span>
                        </div>
                        <label className="toggle-switch">
                            <input
                                type="checkbox"
                                checked={autoStartEnabled}
                                onChange={(e) => handleAutoStartToggle(e.target.checked)}
                            />
                            <span className="toggle-slider" />
                        </label>
                    </div>
                </div>
            </div>

            {/* Dictation Section */}
            <div className="settings-section">
                <h3 className="settings-section-title">Dictation</h3>

                <div className="settings-card">
                    <div className="setting-row">
                        <div className="setting-info">
                            <span className="setting-label">Global Hotkey</span>
                            <span className="setting-desc">Hold to record, release to transcribe</span>
                        </div>
                        <select
                            className="settings-select"
                            value={settings.dictation_hotkey || 'Ctrl+Space'}
                            onChange={(e) => updateSetting('dictation_hotkey', e.target.value)}
                        >
                            <option value="Ctrl+Space">Ctrl + Space</option>
                            <option value="Alt+Space">Alt + Space</option>
                            <option value="Shift+Space">Shift + Space</option>
                            <option value="Super+Space">Super + Space (Cmd/Win)</option>
                        </select>
                    </div>
                </div>

                <div className="settings-card">
                    <div className="setting-row">
                        <div className="setting-info">
                            <span className="setting-label">Dictation Enabled</span>
                            <span className="setting-desc">Turn global hotkey capture on or off without quitting Tinkflow</span>
                        </div>
                        <label className="toggle-switch">
                            <input
                                type="checkbox"
                                checked={settings.dictation_enabled}
                                onChange={(e) => updateSetting('dictation_enabled', e.target.checked)}
                            />
                            <span className="toggle-slider" />
                        </label>
                    </div>
                </div>

                <div className="settings-card">
                    <div className="setting-row">
                        <div className="setting-info">
                            <span className="setting-label">Text Insertion</span>
                            <span className="setting-desc">Choose direct typing, clipboard paste, or automatic fallback</span>
                        </div>
                        <select
                            className="settings-select"
                            value={settings.injection_mode || 'auto'}
                            onChange={(e) => updateSetting('injection_mode', e.target.value)}
                        >
                            <option value="auto">Auto fallback</option>
                            <option value="direct">Direct typing</option>
                            <option value="clipboard">Clipboard paste</option>
                        </select>
                    </div>
                </div>

                <div className="settings-card">
                    <div className="setting-row">
                        <div className="setting-info">
                            <span className="setting-label">Recognition Quality</span>
                            <span className="setting-desc">Accurate checks more candidates for technical terms; balanced stays faster.</span>
                        </div>
                        <select
                            className="settings-select"
                            value={settings.transcription_quality || 'balanced'}
                            onChange={(e) => updateSetting('transcription_quality', e.target.value)}
                        >
                            <option value="balanced">Balanced</option>
                            <option value="accurate">Accurate</option>
                        </select>
                    </div>
                </div>

                <div className="settings-card">
                    <div className="setting-row">
                        <div className="setting-info">
                            <span className="setting-label">Audio Input</span>
                            <span className="setting-desc">Select which microphone to record from</span>
                        </div>
                        <select
                            className="settings-select"
                            value={settings.audio_device_name}
                            onFocus={refreshAudioDevices}
                            onChange={(e) => updateSetting('audio_device_name', e.target.value)}
                        >
                            {audioDevices.map(d => (
                                <option key={d} value={d}>
                                    {d === 'default' ? 'System Default' : d.split('(')[0].trim()}
                                </option>
                            ))}
                        </select>
                    </div>
                </div>
            </div>

            {/* Transcription Section */}
            <div className="settings-section">
                <h3 className="settings-section-title">Transcription</h3>

                <div className="settings-card">
                    <div className="setting-row">
                        <div className="setting-info">
                            <span className="setting-label">Whisper Model</span>
                            <span className="setting-desc">Smaller = faster, larger = more accurate</span>
                        </div>
                        <div className="setting-control-group correction-controls">
                            <select
                                className="settings-select"
                                value={settings.whisper_model}
                                disabled={pullingWhisper}
                                onChange={async (e) => {
                                    const newModel = e.target.value;
                                    if (installedWhisperModels.includes(newModel)) {
                                        updateSetting('whisper_model', newModel);
                                    } else {
                                        setPullingWhisper(true);
                                        try {
                                            await invoke('download_whisper_model', { modelName: newModel });
                                            setInstalledWhisperModels([...installedWhisperModels, newModel]);
                                            updateSetting('whisper_model', newModel);
                                        } catch (err) {
                                            console.error("Failed to download whisper model", err);
                                        } finally {
                                            setPullingWhisper(false);
                                        }
                                    }
                                }}
                            >
                                {WHISPER_MODELS.map(w => (
                                    <option key={w.name} value={w.name}>
                                        {w.label} {installedWhisperModels.includes(w.name) ? '(installed)' : '(download)'}
                                    </option>
                                ))}
                            </select>
                            {pullingWhisper && (
                                <div className="setting-download-status">
                                    <div className="mini-spinner" />
                                    <span className="font-mono text-xs text-secondary">{whisperProgress.toFixed(1)}%</span>
                                </div>
                            )}
                        </div>
                    </div>
                </div>
            </div>

            {/* Personal Corrections Section */}
            <div className="settings-section">
                <h3 className="settings-section-title">Personal Corrections</h3>
                <div className="settings-card corrections-card">
                    <div className="setting-row">
                        <div className="setting-info">
                            <span className="setting-label">Add Correction</span>
                            <span className="setting-desc">Teach Tinkflow your project names, APIs, and repeated speech fixes</span>
                        </div>
                        <div className="setting-control-group">
                            <input className="settings-select" value={newSpoken} onChange={(e) => setNewSpoken(e.target.value)} placeholder="spoken phrase" />
                            <input className="settings-select" value={newReplacement} onChange={(e) => setNewReplacement(e.target.value)} placeholder="replacement" />
                            <button className="secondary-btn" onClick={addCorrection}>Add</button>
                        </div>
                    </div>
                    {corrections.map(entry => (
                        <div className="setting-row" key={entry.id}>
                            <div className="setting-info">
                                <span className="setting-label">{entry.spoken} {'->'} {entry.replacement}</span>
                                <span className="setting-desc">{entry.enabled ? 'Enabled' : 'Disabled'}</span>
                            </div>
                            <div className="setting-control-group">
                                <button className="secondary-btn" onClick={() => toggleCorrection(entry)}>{entry.enabled ? 'Disable' : 'Enable'}</button>
                                <button className="secondary-btn" onClick={() => removeCorrection(entry.id)}>Remove</button>
                            </div>
                        </div>
                    ))}
                </div>
            </div>

            {/* Context Profiles Section */}
            <div className="settings-section">
                <h3 className="settings-section-title">Context Profiles</h3>
                {settings.context_profiles.map((profile, index) => (
                    <div className="settings-card profile-card" key={profile.context}>
                        <div className="setting-row">
                            <div className="setting-info">
                                <span className="setting-label">{profile.context}</span>
                                <span className="setting-desc">Controls how local LLM polishing behaves in this context</span>
                            </div>
                            <label className="toggle-switch">
                                <input type="checkbox" checked={profile.enabled} onChange={(e) => updateProfile(index, { enabled: e.target.checked })} />
                                <span className="toggle-slider" />
                            </label>
                        </div>
                        <div className="setting-row">
                            <div className="setting-info">
                                <span className="setting-label">Tone</span>
                                <span className="setting-desc">Short instruction added to the polishing prompt</span>
                            </div>
                            <input className="settings-select" value={profile.tone} onChange={(e) => updateProfile(index, { tone: e.target.value })} />
                        </div>
                        <div className="setting-row">
                            <div className="setting-info">
                                <span className="setting-label">Preserve Symbols</span>
                                <span className="setting-desc">Protect code symbols and technical notation</span>
                            </div>
                            <label className="toggle-switch">
                                <input type="checkbox" checked={profile.preserve_symbols} onChange={(e) => updateProfile(index, { preserve_symbols: e.target.checked })} />
                                <span className="toggle-slider" />
                            </label>
                        </div>                        <div className="setting-row">
                            <div className="setting-info">
                                <span className="setting-label">Remove Fillers</span>
                                <span className="setting-desc">Ask the polishing model to remove spoken filler words</span>
                            </div>
                            <label className="toggle-switch">
                                <input type="checkbox" checked={profile.remove_fillers} onChange={(e) => updateProfile(index, { remove_fillers: e.target.checked })} />
                                <span className="toggle-slider" />
                            </label>
                        </div>

                        <div className="setting-row">
                            <div className="setting-info">
                                <span className="setting-label">Punctuation</span>
                                <span className="setting-desc">Ask the polishing model to add natural punctuation</span>
                            </div>
                            <label className="toggle-switch">
                                <input type="checkbox" checked={profile.punctuation} onChange={(e) => updateProfile(index, { punctuation: e.target.checked })} />
                                <span className="toggle-slider" />
                            </label>
                        </div>
                    </div>
                ))}
            </div>
            {/* About Section */}
            <div className="settings-section">
                <h3 className="settings-section-title">About</h3>

                <div className="settings-card">
                    <div className="setting-row">
                        <div className="setting-info">
                            <span className="setting-label">Tinkflow</span>
                            <span className="setting-desc">Voice-to-text for developers - local, private, fast</span>
                        </div>
                        <div className="setting-value">
                            <span className="setting-version-badge">v1.5.0</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
