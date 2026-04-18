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

fn emit_recording_state(
    app: &AppHandle,
    recording_state: &crate::RecordingStateStore,
    state: &'static str,
) {
    if let Ok(mut current_state) = recording_state.lock() {
        *current_state = state.to_string();
    }

    let _ = app.emit("recording-state", state);

    if let Some(main) = app.get_webview_window("main") {
        let _ = main.emit("recording-state", state);
    }

    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.emit("recording-state", state);
    }
}

pub fn parse_hotkey(s: &str) -> HotKey {
    match s {
        "Alt+Space" => HotKey::new(Some(Modifiers::ALT), Code::Space),
        "Shift+Space" => HotKey::new(Some(Modifiers::SHIFT), Code::Space),
        "Super+Space" => HotKey::new(Some(Modifiers::SUPER), Code::Space),
        _ => HotKey::new(Some(Modifiers::CONTROL), Code::Space),
    }
}

use std::cell::RefCell;

thread_local! {
    static MANAGER: RefCell<Option<GlobalHotKeyManager>> = RefCell::new(None);
    static CURRENT_HOTKEY: RefCell<Option<HotKey>> = RefCell::new(None);
}

pub struct HotkeyListener;

impl HotkeyListener {
    pub fn init_on_main_thread(
        app: AppHandle,
        audio_capturer: Arc<Mutex<crate::audio::AudioCapturer>>,
        whisper: Arc<Mutex<WhisperTranscriber>>,
        ollama: Arc<Mutex<OllamaClient>>,
        settings_manager: Arc<crate::settings::SettingsManager>,
        recording_state: crate::RecordingStateStore,
    ) -> Arc<std::sync::atomic::AtomicU32> {
        let manager = GlobalHotKeyManager::new().expect("Failed to initialize GlobalHotKeyManager");
        
        let hotkey_str = settings_manager.get().dictation_hotkey;
        let hotkey = parse_hotkey(&hotkey_str);
        if let Err(e) = manager.register(hotkey) {
            eprintln!("Warning: Failed to register hotkey {}. It may already be in use by another application or an old instance of Tinkflow. Error: {:?}", hotkey_str, e);
        }

        let active_id = Arc::new(std::sync::atomic::AtomicU32::new(hotkey.id()));
        
        MANAGER.with(|m| *m.borrow_mut() = Some(manager));
        CURRENT_HOTKEY.with(|h| *h.borrow_mut() = Some(hotkey));

        let is_recording = Arc::new(AtomicBool::new(false));
        let session_id = Arc::new(AtomicUsize::new(0));
        let context_detector = ContextDetector::new();
        
        let active_id_bg = active_id.clone();

        // Spawn a background thread to listen for the hotkey events
        std::thread::spawn(move || {
            let receiver = GlobalHotKeyEvent::receiver();
            loop {
                if let Ok(event) = receiver.recv() {
                    if event.id == active_id_bg.load(std::sync::atomic::Ordering::SeqCst) {
                        let current_state = is_recording.load(Ordering::SeqCst);
                        
                        if event.state == global_hotkey::HotKeyState::Pressed && !current_state {
                            is_recording.store(true, Ordering::SeqCst);
                            let my_session = session_id.fetch_add(1, Ordering::SeqCst) + 1;
                            println!("Hotkey pressed - Start recording (session {})", my_session);
                            
                            if let Ok(mut capturer) = audio_capturer.lock() {
                                if let Err(e) = capturer.start_recording() {
                                    eprintln!("Failed to start recording: {}", e);
                                }
                            }
                            
                            emit_recording_state(&app, &recording_state, "listening");
                            set_overlay_visible(&app, true);
                        } else if event.state == global_hotkey::HotKeyState::Released && current_state {
                            is_recording.store(false, Ordering::SeqCst);
                            let my_session = session_id.load(Ordering::SeqCst);
                            println!("Hotkey released - Stop recording (session {})", my_session);
                            emit_recording_state(&app, &recording_state, "processing");
                            
                            let audio_data_opt = if let Ok(mut capturer) = audio_capturer.lock() {
                                match capturer.stop_recording() {
                                    Ok(data) => Some(data),
                                    Err(e) => {
                                        eprintln!("Error stopping recording: {}", e);
                                        emit_recording_state(&app, &recording_state, "error");
                                        None
                                    }
                                }
                            } else {
                                None
                            };

                            if let Some(audio_data) = audio_data_opt {
                                println!("Successfully captured {} samples of audio data", audio_data.len());
                                if audio_data.len() > 16000 / 2 { 
                                    let app_bg = app.clone();
                                    let whisper_bg = whisper.clone();
                                    let context_detector_bg = context_detector.clone();
                                    let ollama_bg = ollama.clone();
                                    let settings_manager_bg = settings_manager.clone();
                                    let session_id_bg = session_id.clone();
                                    let recording_state_bg = recording_state.clone();

                                    std::thread::spawn(move || {
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
                                            guard!();
                                            if !whisper_guard.is_model_loaded() {
                                                println!("Whisper model not loaded, attempting to load now...");
                                                emit_recording_state(&app_bg, &recording_state_bg, "loading-model");
                                                if let Err(e) = whisper_guard.load_model() {
                                                    eprintln!("Failed to load whisper model: {}", e);
                                                }
                                            }

                                            if whisper_guard.is_model_loaded() {
                                                guard!();
                                                emit_recording_state(&app_bg, &recording_state_bg, "transcribing");
                                                match whisper_guard.transcribe(&audio_data) {
                                                    Ok(raw_text) => {
                                                        println!("Transcribed: {}", raw_text);

                                                        let dict = crate::dictionary::DeveloperDictionary::new();
                                                        let corrected_text = dict.apply(&raw_text);
                                                        if corrected_text != raw_text {
                                                            println!("Dictionary corrected: {}", corrected_text);
                                                        }

                                                        guard!();
                                                        emit_recording_state(&app_bg, &recording_state_bg, "polishing");
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

                                                        guard!();
                                                        if let Ok(mut injector) = crate::injector::TextInjector::new() {
                                                            if let Err(e) = injector.inject(&final_text) {
                                                                eprintln!("Injection error: {}", e);
                                                            } else {
                                                                guard!();
                                                                emit_recording_state(&app_bg, &recording_state_bg, "done");
                                                                success = true;
                                                            }
                                                        } else {
                                                            eprintln!("Failed to initialize TextInjector");
                                                        }
                                                    }
                                                    Err(e) => {
                                                        eprintln!("Whisper Error: {}", e);
                                                        emit_recording_state(&app_bg, &recording_state_bg, "error");
                                                    }
                                                }
                                            } else {
                                                eprintln!("Whisper model is not loaded yet.");
                                                emit_recording_state(&app_bg, &recording_state_bg, "error");
                                            }
                                        }

                                        if session_id_bg.load(Ordering::SeqCst) == my_session {
                                            std::thread::sleep(std::time::Duration::from_millis(if success { 800 } else { 1500 }));
                                            if session_id_bg.load(Ordering::SeqCst) == my_session {
                                                emit_recording_state(&app_bg, &recording_state_bg, "idle");
                                                std::thread::sleep(std::time::Duration::from_millis(800));
                                                set_overlay_visible(&app_bg, false);
                                            }
                                        }
                                    });
                                } else {
                                    println!("Audio too short, discarded.");
                                    emit_recording_state(&app, &recording_state, "idle");
                                    set_overlay_visible(&app, false);
                                }
                            } else {
                                let app_clone = app.clone();
                                let recording_state_clone = recording_state.clone();
                                std::thread::spawn(move || {
                                    std::thread::sleep(std::time::Duration::from_millis(1500));
                                    emit_recording_state(&app_clone, &recording_state_clone, "idle");
                                    std::thread::sleep(std::time::Duration::from_millis(800));
                                    set_overlay_visible(&app_clone, false);
                                });
                            }
                        }
                    }
                }
            }
        });

        active_id
    }

    pub fn update_hotkey_on_main_thread(new_hotkey_str: &str, active_id: &Arc<std::sync::atomic::AtomicU32>) {
        let new_hotkey = parse_hotkey(new_hotkey_str);
        
        CURRENT_HOTKEY.with(|curr| {
            if let Some(mut current_hotkey_ref) = curr.borrow_mut().as_mut() {
                if *current_hotkey_ref == new_hotkey { return; }
                
                MANAGER.with(|m| {
                    if let Some(manager) = m.borrow().as_ref() {
                        println!("Updating hotkey to {}", new_hotkey_str);
                        let _ = manager.unregister(*current_hotkey_ref);
                        if let Err(e) = manager.register(new_hotkey) {
                            eprintln!("Failed to register new hotkey {}: {:?}", new_hotkey_str, e);
                        } else {
                            *current_hotkey_ref = new_hotkey;
                            active_id.store(new_hotkey.id(), std::sync::atomic::Ordering::SeqCst);
                        }
                    }
                });
            }
        });
    }
}
