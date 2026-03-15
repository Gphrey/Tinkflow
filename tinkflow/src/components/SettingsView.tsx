import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import '../styles/settings.css';

interface AppSettings {
    whisper_model: string;
    llm_model: string;
    audio_device_name: string;
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
    });
    const [audioDevices, setAudioDevices] = useState<string[]>(['default']);
    const [installedWhisperModels, setInstalledWhisperModels] = useState<string[]>([]);
    const [pullingWhisper, setPullingWhisper] = useState<boolean>(false);
    const [whisperProgress, setWhisperProgress] = useState<number>(0);

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

    const refreshAudioDevices = async () => {
        try {
            const devices = await invoke<string[]>('get_audio_devices');
            setAudioDevices(devices);
            if (settings.audio_device_name !== 'default' && !devices.includes(settings.audio_device_name)) {
                console.warn(`[Audio] Previously selected device '${settings.audio_device_name}' no longer available — resetting to default`);
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

            {/* Dictation Section */}
            <div className="settings-section">
                <h3 className="settings-section-title">Dictation</h3>

                <div className="settings-card">
                    <div className="setting-row">
                        <div className="setting-info">
                            <span className="setting-label">Global Hotkey</span>
                            <span className="setting-desc">Hold to record, release to transcribe</span>
                        </div>
                        <div className="setting-value">
                            <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Space</kbd>
                        </div>
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
                        <div className="setting-control-group">
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
                                        {w.label} {installedWhisperModels.includes(w.name) ? '✓' : '⬇'}
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

            {/* About Section */}
            <div className="settings-section">
                <h3 className="settings-section-title">About</h3>

                <div className="settings-card">
                    <div className="setting-row">
                        <div className="setting-info">
                            <span className="setting-label">Tinkflow</span>
                            <span className="setting-desc">Voice-to-text for developers — local, private, fast</span>
                        </div>
                        <div className="setting-value">
                            <span className="setting-version-badge">v0.1.0</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
