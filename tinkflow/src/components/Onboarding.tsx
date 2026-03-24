import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ModelManager } from './ModelManager';

export function Onboarding({ onComplete }: { onComplete?: () => void }) {
    const [step, setStep] = useState(1);
    const [visible, setVisible] = useState(false);

    // LLM step state
    const [ollamaStatus, setOllamaStatus] = useState<'checking' | 'connected' | 'not_found'>('checking');
    const [availableModels, setAvailableModels] = useState<string[]>([]);
    const [selectedModel, setSelectedModel] = useState<string>('');

    // Trigger initial fade-in
    useEffect(() => {
        const timer = setTimeout(() => setVisible(true), 100);
        return () => clearTimeout(timer);
    }, []);

    // Check Ollama when LLM step is reached
    useEffect(() => {
        if (step === 3) {
            checkOllama();
        }
    }, [step]);

    const checkOllama = async () => {
        setOllamaStatus('checking');
        try {
            const isHealthy = await invoke<boolean>('check_ollama_status');
            if (isHealthy) {
                setOllamaStatus('connected');
                const models = await invoke<string[]>('list_ollama_models');
                setAvailableModels(models);
                if (models.length > 0 && !selectedModel) {
                    setSelectedModel(models[0]);
                }
            } else {
                setOllamaStatus('not_found');
            }
        } catch {
            setOllamaStatus('not_found');
        }
    };

    // Handle step transitions
    const handleNext = (nextStep: number) => {
        setVisible(false);
        setTimeout(() => {
            setStep(nextStep);
            setVisible(true);
        }, 300);
    };

    const totalSteps = 4;

    return (
        <div className="onboarding-overlay">
            <div className={`onboarding-card ${visible ? 'fade-in' : 'fade-out'}`}>

                {/* Step Progress Indicators */}
                <div className="step-indicator-container">
                    {Array.from({ length: totalSteps }, (_, i) => i + 1).map(i => (
                        <div key={i} className={`step-dot ${step === i ? 'active' : step > i ? 'completed' : ''}`} />
                    ))}
                </div>

                <div className="onboarding-content">
                    {step === 1 && (
                        <div className="step-panel">
                            <div className="step-icon" style={{ padding: '4px' }}>
                                <img
                                    src="/logo.png"
                                    alt="Tinkflow Logo"
                                    width="32" height="32"
                                    style={{ objectFit: 'contain' }}
                                />
                            </div>
                            <h2>Welcome to Tinkflow</h2>
                            <p>An open-source, local-first voice-to-text engine for developers. Dictate code, chat messages, or emails completely offline.</p>
                            <button className="primary-btn mt-6" onClick={() => handleNext(2)}>
                                Get Started
                            </button>
                        </div>
                    )}

                    {step === 2 && (
                        <div className="step-panel">
                            <div className="step-icon">
                                <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><polyline points="7 10 12 15 17 10" /><line x1="12" y1="15" x2="12" y2="3" /></svg>
                            </div>
                            <h2>Download AI Model</h2>
                            <p>Tinkflow runs locally on your machine. We need to download a small Whisper transcription model to process your voice.</p>

                            <div className="model-manager-wrapper mt-4">
                                <ModelManager />
                            </div>

                            <button className="secondary-btn mt-6" onClick={() => handleNext(3)}>
                                Continue
                            </button>
                        </div>
                    )}

                    {step === 3 && (
                        <div className="step-panel">
                            <div className="step-icon">
                                {/* Sparkle/AI icon */}
                                <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                                    <path d="M12 3l1.912 5.813a2 2 0 0 0 1.275 1.275L21 12l-5.813 1.912a2 2 0 0 0-1.275 1.275L12 21l-1.912-5.813a2 2 0 0 0-1.275-1.275L3 12l5.813-1.912a2 2 0 0 0 1.275-1.275L12 3z" />
                                </svg>
                            </div>
                            <h2>LLM Text Polishing</h2>
                            <p>Tinkflow uses a local LLM via <strong>Ollama</strong> to clean up transcriptions — removing filler words, fixing grammar, and adding punctuation.</p>

                            <div className="ollama-status-card mt-4">
                                {ollamaStatus === 'checking' && (
                                    <div className="ollama-status checking">
                                        <div className="status-dot checking-dot" />
                                        <span>Checking for Ollama...</span>
                                    </div>
                                )}

                                {ollamaStatus === 'connected' && (
                                    <>
                                        <div className="ollama-status connected">
                                            <div className="status-dot connected-dot" />
                                            <span>Ollama connected</span>
                                        </div>

                                        {availableModels.length > 0 ? (
                                            <div className="model-selector mt-4">
                                                <label className="model-selector-label">Select a model for text polishing:</label>
                                                <select
                                                    className="model-dropdown"
                                                    value={selectedModel}
                                                    onChange={(e) => setSelectedModel(e.target.value)}
                                                >
                                                    {availableModels.map(model => (
                                                        <option key={model} value={model}>{model}</option>
                                                    ))}
                                                </select>
                                            </div>
                                        ) : (
                                            <p className="text-muted mt-2" style={{ fontSize: '0.85rem' }}>
                                                No models found. Run <code>ollama pull phi3</code> in your terminal to download one.
                                            </p>
                                        )}
                                    </>
                                )}

                                {ollamaStatus === 'not_found' && (
                                    <>
                                        <div className="ollama-status not-found">
                                            <div className="status-dot not-found-dot" />
                                            <span>Ollama not detected</span>
                                        </div>
                                        <p className="text-muted mt-2" style={{ fontSize: '0.85rem' }}>
                                            Text polishing is optional. Without it, Tinkflow will inject raw transcriptions.
                                        </p>
                                        <a
                                            href="https://ollama.com/download"
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            className="ollama-install-link mt-2"
                                        >
                                            Download Ollama →
                                        </a>
                                        <button className="ghost-btn mt-2" onClick={checkOllama}>
                                            Retry Detection
                                        </button>
                                    </>
                                )}
                            </div>

                            <button className="secondary-btn mt-6" onClick={() => handleNext(4)}>
                                {ollamaStatus === 'connected' ? 'Continue' : 'Skip for now'}
                            </button>
                        </div>
                    )}

                    {step === 4 && (
                        <div className="step-panel">
                            <div className="step-icon success-glow">
                                <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" /><polyline points="22 4 12 14.01 9 11.01" /></svg>
                            </div>
                            <h2>You're all set!</h2>
                            <p className="mb-4">Tinkflow is now running in the background.</p>

                            <div className="hotkey-display">
                                <span>Press</span>
                                <kbd>Ctrl</kbd> + <kbd>Space</kbd>
                                <span>to dictate</span>
                            </div>

                            <button className="primary-btn mt-6" onClick={() => {
                                if (onComplete) onComplete();
                                else console.log('Done');
                            }}>
                                Finish Setup
                            </button>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
