import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';

export type RecordingState = 'idle' | 'listening' | 'processing' | 'transcribing' | 'polishing' | 'error' | 'done';

export function useRecording() {
    const [status, setStatus] = useState<RecordingState>('idle');

    useEffect(() => {
        // Listen to "recording-state" events emitted from Rust backend (via hotkey)
        const unlisten = listen<unknown>('recording-state', (event) => {
            console.log('Received recording-state event from backend:', event.payload);

            // Clean up the string just in case there are quotes around it (due to serialization)
            const payloadStr = String(event.payload).replace(/^["']|["']$/g, '');

            if (['idle', 'listening', 'processing', 'transcribing', 'polishing', 'error', 'done'].includes(payloadStr)) {
                setStatus(payloadStr as RecordingState);
            } else {
                console.warn('Unknown recording-state received:', payloadStr);
            }
        });

        // Cleanup listener on unmount
        return () => {
            unlisten.then((f) => f());
        };
    }, []);

    return { status, setStatus };
}
