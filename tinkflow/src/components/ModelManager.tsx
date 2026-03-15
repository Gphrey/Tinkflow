import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, Event } from '@tauri-apps/api/event';

export function ModelManager() {
    const [modelExists, setModelExists] = useState<boolean | null>(null);
    const [downloading, setDownloading] = useState(false);
    const [progress, setProgress] = useState(0);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        // Check if the whispered model exists on mount
        invoke<boolean>('check_whisper_model', { modelName: 'tiny.en' })
            .then((exists) => setModelExists(exists))
            .catch((err) => {
                console.error("Failed to check model:", err);
                setError(err.toString());
            });

        const unlisten = listen<number>('model-download-progress', (event: Event<number>) => {
            setProgress(event.payload);
        });

        return () => {
            unlisten.then((f) => f());
        };
    }, []);

    const handleDownload = async () => {
        try {
            setDownloading(true);
            setError(null);
            await invoke('download_whisper_model', { modelName: 'tiny.en' });
            await invoke('load_whisper_model');
            setDownloading(false);
            setModelExists(true);
        } catch (err: any) {
            setDownloading(false);
            setError(err.toString());
        }
    };

    return (
        <div className="model-manager">
            {modelExists === null ? (
                <div className="flex items-center gap-2 text-secondary">
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="spinner-icon"><path d="M21 12a9 9 0 1 1-6.219-8.56" /></svg>
                    <span>Checking model status...</span>
                </div>
            ) : modelExists ? (
                <div className="flex items-center gap-2" style={{ color: 'var(--accent-cyan)' }}>
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="20 6 9 17 4 12" /></svg>
                    <span className="font-medium">Whisper model is ready.</span>
                </div>
            ) : (
                <div className="download-prompt">
                    <div className="model-info-box mb-4">
                        <p className="text-sm text-secondary m-0">
                            <strong>ggml-tiny.en.bin</strong> (~75MB)<br />
                            Required for offline transcription
                        </p>
                    </div>

                    {error && (
                        <div className="error-box mb-4" style={{ color: 'var(--accent-red)', fontSize: '0.85rem', padding: '0.5rem', background: 'rgba(239, 68, 68, 0.1)', borderRadius: '6px' }}>
                            {error}
                        </div>
                    )}

                    {downloading ? (
                        <div className="progress-container mt-4">
                            <div className="flex justify-between text-sm mb-2 text-secondary">
                                <span>Downloading model...</span>
                                <span className="font-mono">{progress.toFixed(1)}%</span>
                            </div>
                            <div className="progress-bar-bg" style={{ width: '100%', height: '6px', background: 'rgba(255, 255, 255, 0.1)', borderRadius: '3px', overflow: 'hidden' }}>
                                <div className="progress-bar-fg" style={{ width: `${progress}%`, height: '100%', background: 'var(--accent-cyan)', transition: 'width 0.2s cubic-bezier(0.16, 1, 0.3, 1)', boxShadow: '0 0 10px var(--accent-cyan-glow)' }}></div>
                            </div>
                        </div>
                    ) : (
                        <button className="primary-btn w-full mt-4" onClick={handleDownload}>
                            Download Model
                        </button>
                    )}
                </div>
            )}
        </div>
    );
}
