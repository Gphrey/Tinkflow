import { useRecording } from '../hooks/useRecording';
import { useEffect, useState } from 'react';

interface StatusIndicatorProps {
    isOverlay?: boolean;
}

export function StatusIndicator({ isOverlay = false }: StatusIndicatorProps) {
    const { status } = useRecording();
    const [isVisible, setIsVisible] = useState(isOverlay); // Overlay starts visible

    useEffect(() => {
        if (isOverlay) {
            // In overlay mode, always visible — Rust controls window show/hide
            setIsVisible(true);
            return;
        }
        if (status === 'listening' || status === 'processing' || status === 'transcribing' || status === 'polishing' || status === 'done') {
            setIsVisible(true);
        } else if (status === 'idle') {
            const timer = setTimeout(() => setIsVisible(false), 800);
            return () => clearTimeout(timer);
        }
    }, [status, isOverlay]);

    // In overlay mode, show "listening" as default when status is idle.
    // This handles the race condition where the overlay window opens and React
    // mounts before the first "recording-state: listening" event arrives from Rust.
    // Real states (processing, transcribing, polishing, done) are NOT idle, so they
    // pass through this line unchanged and render correctly.
    const displayStatus = isOverlay && status === 'idle' ? 'listening' : status;

    return (
        <div className={`status-indicator-container ${isVisible ? 'visible' : 'hidden'}`}>
            <div className={`status-pill status-${displayStatus}`}>
                {/* Listening State: Cyan Microphone Icon that pulses */}
                {displayStatus === 'listening' && (
                    <>
                        <div className="icon-container listening-glow">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="mic-icon">
                                <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
                                <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
                                <line x1="12" x2="12" y1="19" y2="22" />
                            </svg>
                        </div>
                        <span className="status-text listening-text">Listening...</span>
                    </>
                )}

                {/* Processing State: Yellow Spinner Icon */}
                {displayStatus === 'processing' && (
                    <>
                        <div className="icon-container processing-glow">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="spinner-icon">
                                <path d="M21 12a9 9 0 1 1-6.219-8.56" />
                            </svg>
                        </div>
                        <span className="status-text processing-text">Processing...</span>
                    </>
                )}

                {/* Transcribing State: Orange Waveform Icon */}
                {displayStatus === 'transcribing' && (
                    <>
                        <div className="icon-container transcribing-glow">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="waveform-icon">
                                <path d="M12 2v20M17 7v10M22 10v4M7 5v14M2 9v6" />
                            </svg>
                        </div>
                        <span className="status-text transcribing-text">Transcribing...</span>
                    </>
                )}

                {/* Polishing State: Purple Sparkle Icon */}
                {displayStatus === 'polishing' && (
                    <>
                        <div className="icon-container polishing-glow">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="sparkle-icon">
                                <path d="M12 3l1.912 5.813a2 2 0 0 0 1.275 1.275L21 12l-5.813 1.912a2 2 0 0 0-1.275 1.275L12 21l-1.912-5.813a2 2 0 0 0-1.275-1.275L3 12l5.813-1.912a2 2 0 0 0 1.275-1.275L12 3z" />
                            </svg>
                        </div>
                        <span className="status-text polishing-text">Polishing...</span>
                    </>
                )}

                {/* Done State: Green Checkmark Icon */}
                {displayStatus === 'done' && (
                    <>
                        <div className="icon-container done-glow">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="check-icon">
                                <polyline points="20 6 9 17 4 12" />
                            </svg>
                        </div>
                        <span className="status-text done-text">Done!</span>
                    </>
                )}

                {/* Error State: Red X Icon */}
                {displayStatus === 'error' && (
                    <>
                        <div className="icon-container error-glow">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="error-icon">
                                <line x1="18" y1="6" x2="6" y2="18" />
                                <line x1="6" y1="6" x2="18" y2="18" />
                            </svg>
                        </div>
                        <span className="status-text error-text">Error capturing audio</span>
                    </>
                )}
            </div>
        </div>
    );
}
