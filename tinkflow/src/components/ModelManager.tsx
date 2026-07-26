import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Check, Download, LoaderCircle, X } from 'lucide-react';

interface ModelManagerProps {
    modelName?: string;
    onReadyChange?: (ready: boolean) => void;
}

const MODEL_SIZES: Record<string, string> = {
    'tiny.en': 'about 75 MB',
    'base.en': 'about 142 MB',
    'small.en': 'about 466 MB',
    'medium.en': 'about 1.5 GB',
};

export function ModelManager({ modelName = 'tiny.en', onReadyChange }: ModelManagerProps) {
    const [modelExists, setModelExists] = useState<boolean | null>(null);
    const [downloading, setDownloading] = useState(false);
    const [cancelling, setCancelling] = useState(false);
    const [progress, setProgress] = useState(0);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        setModelExists(null);
        setError(null);
        invoke<boolean>('check_whisper_model', { modelName })
            .then(setModelExists)
            .catch(reason => {
                console.error('Failed to check model:', reason);
                setError(String(reason));
                setModelExists(false);
            });
    }, [modelName]);

    useEffect(() => {
        const unlisten = listen<number>('model-download-progress', event => {
            if (event.payload === -1) {
                setDownloading(false);
                setCancelling(false);
                setProgress(0);
                setError('Download cancelled.');
                return;
            }
            setProgress(event.payload);
        });

        return () => { unlisten.then(dispose => dispose()); };
    }, []);

    useEffect(() => {
        onReadyChange?.(modelExists === true);
    }, [modelExists, onReadyChange]);

    const handleDownload = async () => {
        try {
            setDownloading(true);
            setCancelling(false);
            setError(null);
            setProgress(0);
            await invoke('download_whisper_model', { modelName });
            await invoke('load_whisper_model');
            setModelExists(true);
        } catch (reason) {
            const message = String(reason);
            if (!message.includes('cancelled')) setError(message);
        } finally {
            setDownloading(false);
            setCancelling(false);
        }
    };

    const handleCancel = async () => {
        setCancelling(true);
        await invoke('cancel_download');
    };

    if (modelExists === null) {
        return (
            <div className="model-manager-state">
                <LoaderCircle className="model-manager-spinner" size={17} />
                <span>Checking {modelName} on this device...</span>
            </div>
        );
    }

    if (modelExists) {
        return (
            <div className="model-manager-state is-ready">
                <span className="model-ready-icon"><Check size={15} /></span>
                <div>
                    <strong>Whisper {modelName}</strong>
                    <span>Installed and ready for dictation</span>
                </div>
            </div>
        );
    }

    return (
        <div className="model-manager">
            <div className="model-manager-meta">
                <div>
                    <strong>Whisper {modelName}</strong>
                    <span>{MODEL_SIZES[modelName] || 'local model download'}</span>
                </div>
                <span>Not installed</span>
            </div>

            {error && <p className="model-manager-error">{error}</p>}

            {downloading ? (
                <div className="model-download-state">
                    <div className="model-download-meta">
                        <span>{cancelling ? 'Stopping download...' : 'Downloading securely'}</span>
                        <span>{progress.toFixed(0)}%</span>
                    </div>
                    <div className="model-progress-track" aria-label={`Download ${progress.toFixed(0)} percent complete`}>
                        <span style={{ width: `${progress}%` }} />
                    </div>
                    <button className="model-cancel-action" disabled={cancelling} onClick={handleCancel}>
                        <X size={15} />
                        {cancelling ? 'Stopping...' : 'Cancel'}
                    </button>
                </div>
            ) : (
                <button className="model-download-action" onClick={handleDownload}>
                    <Download size={17} />
                    Download {modelName}
                </button>
            )}
        </div>
    );
}