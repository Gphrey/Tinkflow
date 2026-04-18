import { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export type RecordingState = 'idle' | 'listening' | 'processing' | 'transcribing' | 'polishing' | 'loading-model' | 'error' | 'done';

const VALID_STATES: readonly string[] = ['idle', 'listening', 'processing', 'transcribing', 'polishing', 'loading-model', 'error', 'done'];

export function useRecording() {
    const [status, setStatus] = useState<RecordingState>('idle');
    // Ref tracks latest status to avoid stale closures in the poll interval
    const statusRef = useRef<RecordingState>('idle');

    useEffect(() => {
        let isMounted = true;

        const applyStatus = (nextStatus: unknown) => {
            const payloadStr = String(nextStatus).replace(/^["']|["']$/g, '');

            if (VALID_STATES.includes(payloadStr)) {
                const next = payloadStr as RecordingState;
                // Only update if the value actually changed (prevents unnecessary re-renders)
                if (isMounted && statusRef.current !== next) {
                    statusRef.current = next;
                    setStatus(next);
                }
            }
        };

        // Fetch initial state synchronously on mount
        invoke<string>('get_recording_state')
            .then(applyStatus)
            .catch(() => {});

        // Primary: event-driven updates (works reliably on the main window)
        const unlisten = listen<unknown>('recording-state', (event) => {
            applyStatus(event.payload);
        }, { target: { kind: 'Any' } });

        // Fallback: poll the Rust state store every 120ms.
        // This guarantees overlay windows on WebView2 (Windows) stay in sync
        // even when Tauri events aren't delivered to transparent/always-on-top
        // windows with set_ignore_cursor_events(true).
        const pollInterval = setInterval(() => {
            invoke<string>('get_recording_state')
                .then(applyStatus)
                .catch(() => {});
        }, 120);

        return () => {
            isMounted = false;
            clearInterval(pollInterval);
            unlisten.then((f) => f());
        };
    }, []);

    return { status, setStatus };
}
