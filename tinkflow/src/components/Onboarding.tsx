import { useEffect, useState, type CSSProperties } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
    ArrowLeft,
    ArrowRight,
    Check,
    CheckCircle2,
    Copy,
    Cpu,
    FileText,
    LockKeyhole,
    Mic2,
    RefreshCw,
    ShieldCheck,
    Sparkles,
} from 'lucide-react';
import { ModelManager } from './ModelManager';
import '../styles/onboarding.css';

interface OnboardingProps {
    onComplete?: () => void;
}

interface AppSettings {
    whisper_model: string;
    llm_model: string;
    [key: string]: unknown;
}

const STEPS = [
    { label: 'Welcome', detail: 'Meet Tinkflow' },
    { label: 'Voice setup', detail: 'Prepare local models' },
    { label: 'Ready', detail: 'Start dictating' },
];

const WAVEFORM_BARS = Array.from({ length: 18 }, (_, index) => index);

function VoiceProductPreview() {
    return (
        <div className="onboarding-product-preview" aria-label="Preview of Tinkflow dictating into an editor">
            <div className="preview-app-bar">
                <div className="preview-app-title">
                    <FileText size={15} aria-hidden="true" />
                    <span>Project brief</span>
                </div>
                <div className="preview-window-actions" aria-hidden="true">
                    <span />
                    <span />
                    <span />
                </div>
            </div>

            <div className="preview-editor">
                <div className="preview-gutter" aria-hidden="true">
                    <span>01</span>
                    <span>02</span>
                    <span>03</span>
                    <span>04</span>
                    <span>05</span>
                </div>
                <div className="preview-document">
                    <span className="preview-kicker">Launch notes</span>
                    <strong>Ship ideas at the speed you speak.</strong>
                    <p>
                        Tinkflow turns natural speech into clean, useful text
                        right where the cursor is waiting.
                    </p>
                    <div className="preview-caret-line">
                        <span>Every thought stays recoverable in local history.</span>
                        <i />
                    </div>
                </div>
            </div>

            <div className="preview-dictation-overlay">
                <div className="preview-listening-state">
                    <span className="preview-mic">
                        <Mic2 size={18} aria-hidden="true" />
                    </span>
                    <div>
                        <strong>Listening</strong>
                        <span>Release Ctrl + Space when finished</span>
                    </div>
                </div>
                <div className="preview-waveform" aria-hidden="true">
                    {WAVEFORM_BARS.map(bar => (
                        <i key={bar} style={{ '--bar-index': bar } as CSSProperties} />
                    ))}
                </div>
            </div>

            <div className="preview-result">
                <div>
                    <Check size={14} aria-hidden="true" />
                    <span>Inserted and saved</span>
                </div>
                <span className="preview-copy-action">
                    <Copy size={13} aria-hidden="true" />
                    Copy
                </span>
            </div>
        </div>
    );
}

function SetupSummary() {
    return (
        <div className="onboarding-setup-summary" aria-hidden="true">
            <div className="setup-summary-node is-active">
                <span><Mic2 size={18} /></span>
                <div>
                    <strong>Your voice</strong>
                    <small>Captured with the global hotkey</small>
                </div>
            </div>
            <i className="setup-summary-line" />
            <div className="setup-summary-node">
                <span><Cpu size={18} /></span>
                <div>
                    <strong>Whisper</strong>
                    <small>Transcribed privately on this device</small>
                </div>
            </div>
            <i className="setup-summary-line" />
            <div className="setup-summary-node">
                <span><Sparkles size={18} /></span>
                <div>
                    <strong>Text polish</strong>
                    <small>Optional cleanup with local Ollama</small>
                </div>
            </div>
        </div>
    );
}

function ReadyProductPreview({ hasPolishingModel }: { hasPolishingModel: boolean }) {
    return (
        <div className="onboarding-ready-preview">
            <div className="ready-preview-status">
                <span className="ready-status-mark"><CheckCircle2 size={18} /></span>
                <div>
                    <strong>Tinkflow is ready</strong>
                    <span>Waiting quietly in your tray</span>
                </div>
                <span className="ready-status-live">Active</span>
            </div>

            <div className="ready-preview-transcript">
                <span className="ready-preview-label">Your next dictation</span>
                <p>Speak naturally. Your finished text will appear here and wherever your cursor is active.</p>
                <span className="ready-preview-caret" />
            </div>

            <div className="ready-preview-flow">
                <span><Mic2 size={15} /> Listen</span>
                <i />
                <span><Cpu size={15} /> Transcribe</span>
                <i />
                <span className={hasPolishingModel ? '' : 'is-muted'}>
                    <Sparkles size={15} /> {hasPolishingModel ? 'Polish' : 'Raw text'}
                </span>
                <i />
                <span><Check size={15} /> Insert</span>
            </div>
        </div>
    );
}

export function Onboarding({ onComplete }: OnboardingProps) {
    const [step, setStep] = useState(1);
    const [visible, setVisible] = useState(false);
    const [whisperReady, setWhisperReady] = useState(false);
    const [whisperModel, setWhisperModel] = useState('tiny.en');
    const [ollamaStatus, setOllamaStatus] = useState<'checking' | 'connected' | 'not_found'>('checking');
    const [availableModels, setAvailableModels] = useState<string[]>([]);
    const [selectedModel, setSelectedModel] = useState('');
    const [savingModel, setSavingModel] = useState(false);
    const [modelSaveError, setModelSaveError] = useState<string | null>(null);

    useEffect(() => {
        const timer = window.setTimeout(() => setVisible(true), 80);
        invoke<AppSettings>('get_app_settings')
            .then(settings => {
                setWhisperModel(settings.whisper_model || 'tiny.en');
                setSelectedModel(settings.llm_model || '');
            })
            .catch(error => console.error('Failed to load onboarding settings:', error));
        return () => window.clearTimeout(timer);
    }, []);

    useEffect(() => {
        if (step === 2) checkOllama();
    }, [step]);

    const checkOllama = async () => {
        setOllamaStatus('checking');
        try {
            const connected = await invoke<boolean>('check_ollama_status');
            if (!connected) {
                setOllamaStatus('not_found');
                return;
            }

            const models = await invoke<string[]>('list_ollama_models');
            setAvailableModels(models);
            setSelectedModel(current => models.includes(current) ? current : models[0] || '');
            setOllamaStatus('connected');
        } catch {
            setOllamaStatus('not_found');
        }
    };

    const moveTo = (nextStep: number) => {
        setVisible(false);
        window.setTimeout(() => {
            setStep(nextStep);
            setVisible(true);
        }, 170);
    };

    const persistSelectedModel = async () => {
        if (ollamaStatus !== 'connected' || !selectedModel) return;

        setSavingModel(true);
        setModelSaveError(null);
        try {
            const settings = await invoke<AppSettings>('get_app_settings');
            await invoke('update_app_settings', {
                newSettings: { ...settings, llm_model: selectedModel },
            });
        } catch (error) {
            setModelSaveError('We could not save this model. You can choose it again from Settings.');
            console.error('Failed to save Ollama model:', error);
        } finally {
            setSavingModel(false);
        }
    };

    const handleSetupContinue = async () => {
        await persistSelectedModel();
        moveTo(3);
    };

    return (
        <main className="onboarding-shell">
            <header className="onboarding-topbar">
                <div className="onboarding-brand">
                    <img src="/logo.png" alt="" width="30" height="30" />
                    <span>Tinkflow</span>
                    <small>Private voice workspace</small>
                </div>

                <nav className="onboarding-progress" aria-label={`Setup step ${step} of ${STEPS.length}`}>
                    {STEPS.map((item, index) => {
                        const number = index + 1;
                        const state = step === number ? 'is-active' : step > number ? 'is-complete' : '';
                        return (
                            <div className={`onboarding-progress-item ${state}`} key={item.label}>
                                <span className="onboarding-progress-marker">
                                    {step > number ? <Check size={13} aria-hidden="true" /> : number}
                                </span>
                                <span>
                                    <strong>{item.label}</strong>
                                    <small>{item.detail}</small>
                                </span>
                            </div>
                        );
                    })}
                </nav>
            </header>

            <section className={`onboarding-page ${visible ? 'is-visible' : 'is-leaving'}`}>
                {step === 1 && (
                    <div className="onboarding-hero">
                        <div className="onboarding-hero-copy">
                            <p className="onboarding-eyebrow">
                                <ShieldCheck size={15} aria-hidden="true" />
                                Local by design
                            </p>
                            <h1>Speak. Tinkflow writes.</h1>
                            <p className="onboarding-lead">
                                Turn natural speech into clean, ready-to-use text in any app.
                                Fast, private, and always recoverable.
                            </p>
                            <button className="onboarding-primary-action" onClick={() => moveTo(2)}>
                                Set up my voice
                                <ArrowRight size={18} aria-hidden="true" />
                            </button>
                            <div className="onboarding-trust-row">
                                <span><LockKeyhole size={15} /> Stays on this device</span>
                                <span><FileText size={15} /> Works wherever you type</span>
                                <span><CheckCircle2 size={15} /> Saved if insertion misses</span>
                            </div>
                        </div>

                        <div className="onboarding-hero-visual">
                            <VoiceProductPreview />
                            <p>Hold one shortcut, speak naturally, and keep moving.</p>
                        </div>
                    </div>
                )}

                {step === 2 && (
                    <div className="onboarding-setup-layout">
                        <div className="onboarding-setup-intro">
                            <p className="onboarding-eyebrow">
                                <Cpu size={15} aria-hidden="true" />
                                Voice setup
                            </p>
                            <h1>Prepare your private voice engine.</h1>
                            <p className="onboarding-lead">
                                Tinkflow needs one local speech model. Text polishing is optional
                                and can be changed later.
                            </p>
                            <SetupSummary />
                        </div>

                        <div className="onboarding-setup-pane">
                            <section className="onboarding-setup-block">
                                <div className="setup-block-heading">
                                    <span className="setup-block-icon"><Mic2 size={20} /></span>
                                    <div>
                                        <span className="setup-block-kicker">Required</span>
                                        <h2>Local transcription</h2>
                                    </div>
                                    {whisperReady && <span className="setup-ready-label"><Check size={13} /> Ready</span>}
                                </div>
                                <p>Whisper listens and transcribes entirely on this computer.</p>
                                <ModelManager modelName={whisperModel} onReadyChange={setWhisperReady} />
                            </section>

                            <section className="onboarding-setup-block">
                                <div className="setup-block-heading">
                                    <span className="setup-block-icon is-warm"><Sparkles size={20} /></span>
                                    <div>
                                        <span className="setup-block-kicker is-optional">Optional</span>
                                        <h2>Local text polish</h2>
                                    </div>
                                </div>
                                <p>Ollama can remove filler words and tidy punctuation after transcription.</p>

                                <div className="onboarding-service-state">
                                    {ollamaStatus === 'checking' && (
                                        <span className="service-state checking"><i />Checking this computer for Ollama...</span>
                                    )}
                                    {ollamaStatus === 'connected' && (
                                        <>
                                            <span className="service-state connected"><i />Ollama is connected locally</span>
                                            {availableModels.length > 0 ? (
                                                <label className="onboarding-model-select">
                                                    <span>Polishing model</span>
                                                    <select value={selectedModel} onChange={event => setSelectedModel(event.target.value)}>
                                                        {availableModels.map(model => <option key={model} value={model}>{model}</option>)}
                                                    </select>
                                                </label>
                                            ) : (
                                                <p className="onboarding-inline-note">No local models found. You can add one later in Settings.</p>
                                            )}
                                        </>
                                    )}
                                    {ollamaStatus === 'not_found' && (
                                        <div className="onboarding-optional-state">
                                            <div>
                                                <span className="service-state unavailable"><i />Ollama is not running</span>
                                                <p className="onboarding-inline-note">No problem. Tinkflow works fully with raw transcription.</p>
                                            </div>
                                            <button className="onboarding-icon-button" onClick={checkOllama} title="Check again" aria-label="Check for Ollama again">
                                                <RefreshCw size={16} />
                                            </button>
                                        </div>
                                    )}
                                </div>
                                {modelSaveError && <p className="onboarding-error">{modelSaveError}</p>}
                            </section>
                        </div>
                    </div>
                )}

                {step === 3 && (
                    <div className="onboarding-ready-layout">
                        <div className="onboarding-ready-copy">
                            <p className="onboarding-eyebrow">
                                <CheckCircle2 size={15} aria-hidden="true" />
                                Setup complete
                            </p>
                            <h1>Your voice is ready.</h1>
                            <p className="onboarding-lead">
                                Place your cursor anywhere, hold the shortcut, and speak.
                                Tinkflow handles the rest.
                            </p>

                            <div className="onboarding-hotkey">
                                <span>Hold to dictate</span>
                                <div>
                                    <kbd>Ctrl</kbd>
                                    <i>+</i>
                                    <kbd>Space</kbd>
                                </div>
                            </div>

                            <button className="onboarding-primary-action" onClick={onComplete}>
                                Open Tinkflow
                                <ArrowRight size={18} aria-hidden="true" />
                            </button>
                        </div>

                        <div className="onboarding-ready-visual">
                            <ReadyProductPreview hasPolishingModel={Boolean(selectedModel)} />
                            <p>Your transcript also stays available in History, even when no text field is active.</p>
                        </div>
                    </div>
                )}
            </section>

            <footer className="onboarding-footer">
                <div>
                    {step > 1 && (
                        <button className="onboarding-back-action" onClick={() => moveTo(step - 1)}>
                            <ArrowLeft size={17} aria-hidden="true" />
                            Back
                        </button>
                    )}
                </div>
                <span className="onboarding-step-count">Step {step} of {STEPS.length}</span>
                <div>
                    {step === 2 && (
                        <button className="onboarding-primary-action is-compact" disabled={!whisperReady || savingModel} onClick={handleSetupContinue}>
                            {savingModel ? 'Saving...' : whisperReady ? 'Continue' : 'Waiting for Whisper'}
                            {whisperReady && !savingModel && <ArrowRight size={17} aria-hidden="true" />}
                        </button>
                    )}
                </div>
            </footer>
        </main>
    );
}