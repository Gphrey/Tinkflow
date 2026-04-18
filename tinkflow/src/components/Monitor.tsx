import { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { StatusIndicator } from './StatusIndicator';
import { useRecording } from '../hooks/useRecording';
import '../styles/monitor.css';

interface EventLogEntry {
    id: number;
    timestamp: string;
    state: string;
}

export function Monitor() {
    const { status } = useRecording();
    const [eventLog, setEventLog] = useState<EventLogEntry[]>([]);
    const logRef = useRef<HTMLDivElement>(null);
    const idCounter = useRef(0);
    // Track the last logged state to deduplicate the 3x emissions from Rust
    const lastLoggedState = useRef<string>('');
    const lastLoggedTime = useRef<number>(0);

    // Listen to recording-state events and log them (deduplicated)
    useEffect(() => {
        const unlisten = listen<unknown>('recording-state', (event) => {
            const payloadStr = String(event.payload).replace(/^["']|["']$/g, '');
            const now = Date.now();

            // Deduplicate: Rust emits 3x per state change (app + main + overlay).
            // Skip if same state arrived within 100ms.
            if (payloadStr === lastLoggedState.current && now - lastLoggedTime.current < 100) {
                return;
            }
            lastLoggedState.current = payloadStr;
            lastLoggedTime.current = now;

            const entry: EventLogEntry = {
                id: idCounter.current++,
                timestamp: (() => {
                    const d = new Date();
                    const hms = d.toLocaleTimeString('en-GB', { hour12: false });
                    const ms = String(d.getMilliseconds()).padStart(3, '0');
                    return `${hms}.${ms}`;
                })(),
                state: payloadStr,
            };
            setEventLog(prev => [entry, ...prev].slice(0, 50));
        }, { target: { kind: 'Any' as const } });

        return () => { unlisten.then(f => f()); };
    }, []);

    // Auto-scroll log to top (newest first)
    useEffect(() => {
        if (logRef.current) {
            logRef.current.scrollTop = 0;
        }
    }, [eventLog]);

    const clearLog = () => setEventLog([]);

    const stateColor = (state: string): string => {
        switch (state) {
            case 'listening': return '#24c8db';
            case 'processing': return '#facc15';
            case 'transcribing': return '#f97316';
            case 'polishing': return '#8b5cf6';
            case 'done': return '#10b981';
            case 'error': return '#ef4444';
            case 'loading-model': return '#facc15';
            default: return '#71717a';
        }
    };

    return (
        <div className="monitor">
            {/* Header */}
            <div className="monitor-header">
                <div className="monitor-logo-row">
                    <div className="monitor-icon-badge">
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M12 2v20M17 7v10M22 10v4M7 5v14M2 9v6" />
                        </svg>
                    </div>
                    <div>
                        <h1 className="monitor-title">Pipeline Monitor</h1>
                        <p className="monitor-subtitle">Live view of the dictation pipeline state</p>
                    </div>
                </div>
            </div>

            {/* Live Status */}
            <div className="monitor-card monitor-status-card">
                <div className="monitor-card-header">
                    <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                        <circle cx="12" cy="12" r="10" />
                        <line x1="12" y1="16" x2="12" y2="12" />
                        <line x1="12" y1="8" x2="12.01" y2="8" />
                    </svg>
                    <span>Current State</span>
                </div>

                <div className="monitor-live-status">
                    <div className="monitor-state-display">
                        <div className="monitor-state-dot-large" style={{ background: stateColor(status), boxShadow: `0 0 12px ${stateColor(status)}40` }} />
                        <span className="monitor-state-label" style={{ color: stateColor(status) }}>
                            {status}
                        </span>
                    </div>
                    <div className="monitor-pipeline-stages">
                        {(['idle', 'listening', 'processing', 'transcribing', 'polishing', 'done'] as const).map((stage, i) => {
                            const stages = ['idle', 'listening', 'processing', 'transcribing', 'polishing', 'done'];
                            const currentIdx = stages.indexOf(status === 'loading-model' ? 'processing' : status);
                            const stageIdx = stages.indexOf(stage);
                            const isActive = stage === status || (status === 'loading-model' && stage === 'processing');
                            const isPast = stageIdx < currentIdx;

                            return (
                                <div key={stage} className="monitor-stage-item">
                                    <div
                                        className={`monitor-stage-dot ${isActive ? 'active' : ''} ${isPast ? 'past' : ''}`}
                                        style={isActive ? { background: stateColor(status), boxShadow: `0 0 8px ${stateColor(status)}` } : {}}
                                    />
                                    <span className={`monitor-stage-label ${isActive ? 'active-label' : ''}`}>
                                        {stage}
                                    </span>
                                    {i < 5 && <div className={`monitor-stage-connector ${isPast ? 'past' : ''}`} />}
                                </div>
                            );
                        })}
                    </div>
                </div>

                {/* Inline StatusIndicator pill for comparison */}
                <div className="monitor-pill-wrapper">
                    <StatusIndicator isOverlay={false} />
                </div>
            </div>

            {/* Event Log */}
            <div className="monitor-card">
                <div className="monitor-card-header" style={{ justifyContent: 'space-between' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '0.45rem' }}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                            <polyline points="14,2 14,8 20,8" />
                            <line x1="16" y1="13" x2="8" y2="13" />
                            <line x1="16" y1="17" x2="8" y2="17" />
                        </svg>
                        <span>Event Log</span>
                        <span className="monitor-event-count">{eventLog.length}</span>
                    </div>
                    <button className="monitor-clear-btn" onClick={clearLog}>Clear</button>
                </div>

                <div className="monitor-event-log" ref={logRef}>
                    {eventLog.length === 0 ? (
                        <div className="monitor-empty-log">
                            <p>No events yet. Press <kbd>Ctrl</kbd>+<kbd>Space</kbd> to start dictating.</p>
                        </div>
                    ) : (
                        eventLog.map(entry => (
                            <div key={entry.id} className="monitor-event-row">
                                <span className="monitor-event-time">{entry.timestamp}</span>
                                <div
                                    className="monitor-event-dot"
                                    style={{ background: stateColor(entry.state), boxShadow: `0 0 6px ${stateColor(entry.state)}50` }}
                                />
                                <span
                                    className="monitor-event-state"
                                    style={{ color: stateColor(entry.state) }}
                                >
                                    {entry.state}
                                </span>
                            </div>
                        ))
                    )}
                </div>
            </div>

            {/* Diagnostics */}
            <div className="monitor-card monitor-diag-card">
                <div className="monitor-card-header">
                    <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                        <path d="M12 20h9" />
                        <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
                    </svg>
                    <span>Diagnostics</span>
                </div>
                <p className="monitor-diag-text">
                    This page listens to the same <code>recording-state</code> events as the overlay.
                    If states transition correctly here but the overlay stays stuck on "listening",
                    the issue is specific to the overlay window's event delivery.
                </p>
                <div className="monitor-diag-row">
                    <span className="monitor-diag-label">Window</span>
                    <span className="monitor-diag-value">main</span>
                </div>
                <div className="monitor-diag-row">
                    <span className="monitor-diag-label">Hook status</span>
                    <span className="monitor-diag-value" style={{ color: stateColor(status) }}>{status}</span>
                </div>
            </div>
        </div>
    );
}
