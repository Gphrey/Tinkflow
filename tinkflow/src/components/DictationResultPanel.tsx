import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useRef, useState } from 'react';

interface DictationResult {
    record_id: number;
    text: string;
    insertion_succeeded: boolean;
    injection_mode: string;
    injection_error: string | null;
}

type PanelPhase = 'expanded' | 'hint' | 'fading' | null;

const INSERTED_VISIBLE_MS = 4_500;
const SAVED_VISIBLE_MS = 9_000;
const HINT_VISIBLE_MS = 3_500;

export function DictationResultPanel() {
    const [result, setResult] = useState<DictationResult | null>(null);
    const [phase, setPhase] = useState<PanelPhase>(null);
    const [copyState, setCopyState] = useState<'idle' | 'copied' | 'error'>('idle');
    const lastRecordId = useRef<number | null>(null);
    const dismissTimer = useRef<number | null>(null);
    const hintTimer = useRef<number | null>(null);
    const hideTimer = useRef<number | null>(null);

    const clearTimers = () => {
        if (dismissTimer.current !== null) {
            window.clearTimeout(dismissTimer.current);
            dismissTimer.current = null;
        }
        if (hintTimer.current !== null) {
            window.clearTimeout(hintTimer.current);
            hintTimer.current = null;
        }
        if (hideTimer.current !== null) {
            window.clearTimeout(hideTimer.current);
            hideTimer.current = null;
        }
    };

    const setOverlayInteractive = (interactive: boolean) => {
        invoke('set_overlay_interactive', { interactive }).catch(() => {});
    };

    useEffect(() => {
        const applyResult = (payload: DictationResult) => {
            if (!payload.text || lastRecordId.current === payload.record_id) {
                return;
            }

            lastRecordId.current = payload.record_id;
            clearTimers();
            setResult(payload);
            setCopyState('idle');
            setPhase('expanded');
            setOverlayInteractive(true);

            dismissTimer.current = window.setTimeout(() => {
                setPhase('hint');
                setOverlayInteractive(false);
                hintTimer.current = window.setTimeout(() => {
                    setPhase('fading');
                    hideTimer.current = window.setTimeout(() => {
                        setPhase(null);
                        setResult(null);
                        invoke('dismiss_overlay').catch(() => {});
                    }, 220);
                }, HINT_VISIBLE_MS);
            }, payload.insertion_succeeded ? INSERTED_VISIBLE_MS : SAVED_VISIBLE_MS);
        };

        const unlisten = listen<DictationResult>('dictation-result', ({ payload }) => {
            applyResult(payload);
        }, { target: { kind: 'Any' } });

        const pollActiveResult = () => {
            invoke<DictationResult | null>('get_active_dictation_result')
                .then((payload) => {
                    if (payload) applyResult(payload);
                })
                .catch(() => {});
        };

        pollActiveResult();
        const pollInterval = window.setInterval(pollActiveResult, 200);

        return () => {
            clearTimers();
            setOverlayInteractive(false);
            window.clearInterval(pollInterval);
            unlisten.then((dispose) => dispose());
        };
    }, []);

    const copyResult = async () => {
        if (!result) return;

        try {
            await invoke('copy_transcription', { id: result.record_id });
            setCopyState('copied');
        } catch {
            setCopyState('error');
        }
    };

    if (!result || !phase) {
        return null;
    }

    const isSaved = !result.insertion_succeeded;
    const label = isSaved ? 'Saved' : 'Inserted';

    return (
        <section className={`dictation-result-panel dictation-result-panel--${phase}`} aria-live="polite">
            {phase === 'expanded' ? (
                <>
                    <div className="dictation-result-panel__header">
                        <span className={`dictation-result-panel__status ${isSaved ? 'is-saved' : 'is-inserted'}`}>
                            {label}
                        </span>
                        <button
                            className="dictation-result-panel__copy"
                            type="button"
                            onClick={copyResult}
                            title="Copy transcript"
                        >
                            <svg aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                                <rect x="9" y="9" width="11" height="11" rx="2" />
                                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                            </svg>
                            <span>{copyState === 'copied' ? 'Copied' : copyState === 'error' ? 'Retry' : 'Copy'}</span>
                        </button>
                    </div>
                    <p className="dictation-result-panel__text">{result.text}</p>
                </>
            ) : (
                <p className="dictation-result-panel__hint">Transcript saved in Tinkflow</p>
            )}
        </section>
    );
}
