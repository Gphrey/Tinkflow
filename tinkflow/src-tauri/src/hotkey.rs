use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use tauri::{AppHandle, Emitter, Manager};
use std::sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc, Mutex};
use crate::whisper::WhisperTranscriber;
use crate::llm::OllamaClient;
use crate::context::ContextDetector;

/// Show or hide the overlay window
fn set_overlay_visible(app: &AppHandle, visible: bool) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        if visible {
            let _ = overlay.show();
        } else {
            let _ = overlay.hide();
        }
    }
}

pub struct HotkeyManager {
    _manager: GlobalHotKeyManager,
}

impl HotkeyManager {
    pub fn new(
        app: AppHandle,
        audio_capturer: Arc<Mutex<crate::audio::AudioCapturer>>,
        whisper: Arc<Mutex<WhisperTranscriber>>,
        ollama: Arc<Mutex<OllamaClient>>,
        settings_manager: Arc<crate::settings::SettingsManager>,
    ) -> Self {
        let manager = GlobalHotKeyManager::new().expect("Failed to initialize GlobalHotKeyManager");
        
        // Ctrl + Shift + Space
        let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
        if let Err(e) = manager.register(hotkey) {
            eprintln!("Warning: Failed to register hotkey Ctrl+Shift+Space. It may already be in use by another application or an old instance of Tinkflow. Error: {:?}", e);
        }

        let is_recording = Arc::new(AtomicBool::new(false));
        // Monotonically increasing counter — incremented on every new hotkey press.
        // Background pipeline threads capture their session ID at spawn time and
        // bail out before emitting any state if a newer session has begun.
        let session_id = Arc::new(AtomicUsize::new(0));
        let context_detector = ContextDetector::new();

        // Spawn a background thread to listen for the hotkey events
        std::thread::spawn(move || {
            let receiver = GlobalHotKeyEvent::receiver();
            loop {
                if let Ok(event) = receiver.recv() {
                    if event.id == hotkey.id() {
                        let current_state = is_recording.load(Ordering::SeqCst);
                        
                        if event.state == global_hotkey::HotKeyState::Pressed && !current_state {
                            is_recording.store(true, Ordering::SeqCst);
                            // Bump the session counter so any old pipeline thread stops emitting
                            let my_session = session_id.fetch_add(1, Ordering::SeqCst) + 1;
                            println!("Hotkey pressed - Start recording (session {})", my_session);
                            
                            if let Ok(mut capturer) = audio_capturer.lock() {
                                if let Err(e) = capturer.start_recording() {
                                    eprintln!("Failed to start recording: {}", e);
                                }
                            }
                            
                            let _ = app.emit("recording-state", "listening");
                            set_overlay_visible(&app, true);
                        } else if event.state == global_hotkey::HotKeyState::Released && current_state {
                            is_recording.store(false, Ordering::SeqCst);
                            // Read the current session so the pipeline thread can detect preemption
                            let my_session = session_id.load(Ordering::SeqCst);
                            println!("Hotkey released - Stop recording (session {})", my_session);
                            let _ = app.emit("recording-state", "processing");
                            
                            let audio_data_opt = if let Ok(mut capturer) = audio_capturer.lock() {
                                match capturer.stop_recording() {
                                    Ok(data) => Some(data),
                                    Err(e) => {
                                        eprintln!("Error stopping recording: {}", e);
                                        let _ = app.emit("recording-state", "error");
                                        None
                                    }
                                }
                            } else {
                                None
                            };

                            if let Some(audio_data) = audio_data_opt {
                                println!("Successfully captured {} samples of audio data", audio_data.len());
                                if audio_data.len() > 16000 / 2 { // At least half a second
                                    let app_bg = app.clone();
                                    let whisper_bg = whisper.clone();
                                    let context_detector_bg = context_detector.clone();
                                    let ollama_bg = ollama.clone();
                                    let settings_manager_bg = settings_manager.clone();
                                    let session_id_bg = session_id.clone();

                                    std::thread::spawn(move || {
                                        // Macro to bail out silently if a newer session has started
                                        macro_rules! guard {
                                            () => {
                                                if session_id_bg.load(Ordering::SeqCst) != my_session {
                                                    println!("Session {} preempted, pipeline thread exiting", my_session);
                                                    return;
                                                }
                                            };
                                        }

                                        let mut success = false;
                                        if let Ok(mut whisper_guard) = whisper_bg.lock() {
                                            // Lazy load the model if it was changed/unloaded
                                            guard!();
                                            if !whisper_guard.is_model_loaded() {
                                                println!("Whisper model not loaded, attempting to load now...");
                                                let _ = app_bg.emit("recording-state", "loading-model");
                                                if let Err(e) = whisper_guard.load_model() {
                                                    eprintln!("Failed to load whisper model: {}", e);
                                                }
                                            }

                                            if whisper_guard.is_model_loaded() {
                                                guard!();
                                                let _ = app_bg.emit("recording-state", "transcribing");
                                                match whisper_guard.transcribe(&audio_data) {
                                                    Ok(raw_text) => {
                                                        println!("Transcribed: {}", raw_text);

                                                        // --- Dictionary Corrections ---
                                                        let dict = crate::dictionary::DeveloperDictionary::new();
                                                        let corrected_text = dict.apply(&raw_text);
                                                        if corrected_text != raw_text {
                                                            println!("Dictionary corrected: {}", corrected_text);
                                                        }

                                                        // --- Phase 4: LLM Polish ---
                                                        guard!();
                                                        let _ = app_bg.emit("recording-state", "polishing");
                                                        let context = context_detector_bg.detect_current_context();
                                                        println!("Detected context: {}", context);

                                                        let model_name = settings_manager_bg.get().llm_model;
                                                        let final_text = if let Ok(ollama_guard) = ollama_bg.lock() {
                                                            if ollama_guard.check_health() && !model_name.is_empty() {
                                                                println!("Using LLM Model: {}", model_name);
                                                                ollama_guard.polish_text(&corrected_text, &context, &model_name)
                                                            } else {
                                                                println!("Ollama unhealthy or no model selected, using raw text");
                                                                corrected_text.clone()
                                                            }
                                                        } else {
                                                            corrected_text.clone()
                                                        };

                                                        // Text Injection
                                                        guard!();
                                                        if let Ok(mut injector) = crate::injector::TextInjector::new() {
                                                            if let Err(e) = injector.inject(&final_text) {
                                                                eprintln!("Injection error: {}", e);
                                                            } else {
                                                                guard!();
                                                                let _ = app_bg.emit("recording-state", "done");
                                                                success = true;
                                                            }
                                                        } else {
                                                            eprintln!("Failed to initialize TextInjector");
                                                        }
                                                    }
                                                    Err(e) => {
                                                        eprintln!("Whisper Error: {}", e);
                                                        let _ = app_bg.emit("recording-state", "error");
                                                    }
                                                }
                                            } else {
                                                eprintln!("Whisper model is not loaded yet.");
                                                let _ = app_bg.emit("recording-state", "error");
                                            }
                                        }

                                        // Universal cleanup for this background thread
                                        // Only hide if we are still the active session
                                        if session_id_bg.load(Ordering::SeqCst) == my_session {
                                            std::thread::sleep(std::time::Duration::from_millis(if success { 800 } else { 1500 }));
                                            // Final guard before hiding — a new session may have started during sleep
                                            if session_id_bg.load(Ordering::SeqCst) == my_session {
                                                let _ = app_bg.emit("recording-state", "idle");
                                                std::thread::sleep(std::time::Duration::from_millis(800));
                                                set_overlay_visible(&app_bg, false);
                                            }
                                        }
                                    });
                                } else {
                                    println!("Audio too short, discarded.");
                                    let _ = app.emit("recording-state", "idle");
                                    set_overlay_visible(&app, false);
                                }
                            } else {
                                // If recording couldn't be stopped properly, cleanup the error state
                                let app_clone = app.clone();
                                std::thread::spawn(move || {
                                    std::thread::sleep(std::time::Duration::from_millis(1500));
                                    let _ = app_clone.emit("recording-state", "idle");
                                    std::thread::sleep(std::time::Duration::from_millis(800));
                                    set_overlay_visible(&app_clone, false);
                                });
                            }
                        }
                    }
                }
            }
        });

        Self { _manager: manager }
    }
}
