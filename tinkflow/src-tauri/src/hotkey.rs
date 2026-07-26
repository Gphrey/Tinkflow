use crate::context::ContextDetector;
use crate::corrections::CorrectionManager;
use crate::history::HistoryManager;
use crate::llm::OllamaClient;
use crate::whisper::WhisperTranscriber;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use tauri::{AppHandle, Emitter, Manager};

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

fn set_overlay_interactive(app: &AppHandle, interactive: bool) {
    let app = app.clone();
    if let Err(error) = app.clone().run_on_main_thread(move || {
        if let Some(overlay) = app.get_webview_window("overlay") {
            if let Err(error) = overlay.set_ignore_cursor_events(!interactive) {
                eprintln!("Failed to update overlay cursor events: {}", error);
            }
        }
    }) {
        eprintln!("Failed to schedule overlay cursor update: {}", error);
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

#[derive(Clone, serde::Serialize)]
pub struct DictationResult {
    record_id: u64,
    text: String,
    insertion_succeeded: bool,
    injection_mode: String,
    injection_error: Option<String>,
}

fn clear_active_dictation_result(app: &AppHandle) {
    let active_result = app.state::<crate::DictationResultStore>();
    if let Ok(mut result) = active_result.lock() {
        *result = None;
    };
}

fn emit_dictation_result(app: &AppHandle, record: &crate::history::TranscriptionRecord) {
    let result = DictationResult {
        record_id: record.id,
        text: record.final_text.clone(),
        insertion_succeeded: record.injection_succeeded,
        injection_mode: record.injection_mode.clone(),
        injection_error: record.injection_error.clone(),
    };

    let active_result = app.state::<crate::DictationResultStore>();
    if let Ok(mut active) = active_result.lock() {
        *active = Some(result.clone());
    }

    // The overlay may miss its event, but it must be click-ready before either
    // event delivery or the polling fallback displays the Copy control.
    set_overlay_interactive(app, true);
    let _ = app.emit("dictation-result", &result);

    if let Some(main) = app.get_webview_window("main") {
        let _ = main.emit("dictation-result", &result);
    }

    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.emit("dictation-result", &result);
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

fn context_with_profile(context: &str, settings: &crate::settings::AppSettings) -> String {
    if let Some(profile) = settings.context_profile(context) {
        if !profile.enabled {
            return context.to_string();
        }
        format!(
            "{}|profile:tone={}, preserve_symbols={}, remove_fillers={}, punctuation={}",
            context,
            profile.tone,
            profile.preserve_symbols,
            profile.remove_fillers,
            profile.punctuation
        )
    } else {
        context.to_string()
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
        correction_manager: Arc<CorrectionManager>,
        history_manager: Arc<HistoryManager>,
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

        std::thread::spawn(move || {
            let receiver = GlobalHotKeyEvent::receiver();
            loop {
                if let Ok(event) = receiver.recv() {
                    if event.id == active_id_bg.load(std::sync::atomic::Ordering::SeqCst) {
                        if !settings_manager.get().dictation_enabled {
                            continue;
                        }

                        let current_state = is_recording.load(Ordering::SeqCst);

                        if event.state == global_hotkey::HotKeyState::Pressed && !current_state {
                            is_recording.store(true, Ordering::SeqCst);
                            clear_active_dictation_result(&app);
                            let my_session = session_id.fetch_add(1, Ordering::SeqCst) + 1;
                            println!("Hotkey pressed - Start recording (session {})", my_session);

                            if let Ok(mut capturer) = audio_capturer.lock() {
                                if let Err(e) = capturer.start_recording() {
                                    eprintln!("Failed to start recording: {}", e);
                                }
                            }

                            emit_recording_state(&app, &recording_state, "listening");
                            set_overlay_visible(&app, true);
                        } else if event.state == global_hotkey::HotKeyState::Released
                            && current_state
                        {
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
                                println!(
                                    "Successfully captured {} samples of audio data",
                                    audio_data.len()
                                );
                                if audio_data.len() > 16000 / 2 {
                                    let app_bg = app.clone();
                                    let whisper_bg = whisper.clone();
                                    let context_detector_bg = context_detector.clone();
                                    let ollama_bg = ollama.clone();
                                    let settings_manager_bg = settings_manager.clone();
                                    let correction_manager_bg = correction_manager.clone();
                                    let history_manager_bg = history_manager.clone();
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
                                        let mut result_emitted = false;
                                        if let Ok(mut whisper_guard) = whisper_bg.lock() {
                                            guard!();
                                            if !whisper_guard.is_model_loaded() {
                                                println!("Whisper model not loaded, attempting to load now...");
                                                emit_recording_state(
                                                    &app_bg,
                                                    &recording_state_bg,
                                                    "loading-model",
                                                );
                                                if let Err(e) = whisper_guard.load_model() {
                                                    eprintln!(
                                                        "Failed to load whisper model: {}",
                                                        e
                                                    );
                                                }
                                            }

                                            if whisper_guard.is_model_loaded() {
                                                guard!();
                                                emit_recording_state(
                                                    &app_bg,
                                                    &recording_state_bg,
                                                    "transcribing",
                                                );
                                                let transcription_quality = settings_manager_bg
                                                    .get()
                                                    .transcription_quality;
                                                match whisper_guard.transcribe(
                                                    &audio_data,
                                                    &transcription_quality,
                                                ) {
                                                    Ok(raw_text) => {
                                                        println!("Transcribed: {}", raw_text);

                                                        let dict = crate::dictionary::DeveloperDictionary::new();
                                                        let static_corrected_text =
                                                            dict.apply(&raw_text);
                                                        let corrected_text = correction_manager_bg
                                                            .apply(&static_corrected_text);
                                                        if corrected_text != raw_text {
                                                            println!(
                                                                "Corrections applied: {}",
                                                                corrected_text
                                                            );
                                                        }

                                                        guard!();
                                                        emit_recording_state(
                                                            &app_bg,
                                                            &recording_state_bg,
                                                            "polishing",
                                                        );
                                                        let context = context_detector_bg
                                                            .detect_current_context();
                                                        println!("Detected context: {}", context);

                                                        let settings = settings_manager_bg.get();
                                                        let model_name = settings.llm_model.clone();
                                                        let profiled_context = context_with_profile(
                                                            &context, &settings,
                                                        );
                                                        let final_text = if let Ok(ollama_guard) =
                                                            ollama_bg.lock()
                                                        {
                                                            if ollama_guard.check_health()
                                                                && !model_name.is_empty()
                                                            {
                                                                println!(
                                                                    "Using LLM Model: {}",
                                                                    model_name
                                                                );
                                                                ollama_guard.polish_text(
                                                                    &corrected_text,
                                                                    &profiled_context,
                                                                    &model_name,
                                                                )
                                                            } else {
                                                                println!("Ollama unhealthy or no model selected, using corrected text");
                                                                corrected_text.clone()
                                                            }
                                                        } else {
                                                            corrected_text.clone()
                                                        };

                                                        guard!();
                                                        let injection_mode =
                                                            settings.injection_mode.clone();
                                                        let injection_report = match crate::injector::TextInjector::new() {
                                                            Ok(mut injector) => injector.inject_with_mode(&final_text, &injection_mode),
                                                            Err(e) => crate::injector::InjectionReport {
                                                                mode_used: injection_mode.clone(),
                                                                succeeded: false,
                                                                error: Some(format!("Failed to initialize TextInjector: {}", e)),
                                                            },
                                                        };

                                                        match history_manager_bg.add(
                                                            raw_text,
                                                            corrected_text,
                                                            final_text.clone(),
                                                            context,
                                                            injection_report.mode_used.clone(),
                                                            injection_report.succeeded,
                                                            injection_report.error.clone(),
                                                        ) {
                                                            Ok(record) => {
                                                                emit_dictation_result(&app_bg, &record);
                                                                result_emitted = true;
                                                            },
                                                            Err(e) => eprintln!(
                                                                "Failed to save transcription history: {}",
                                                                e
                                                            ),
                                                        }

                                                        if injection_report.succeeded {
                                                            guard!();
                                                            emit_recording_state(
                                                                &app_bg,
                                                                &recording_state_bg,
                                                                "done",
                                                            );
                                                            success = true;
                                                        } else {
                                                            if let Some(error) =
                                                                injection_report.error
                                                            {
                                                                eprintln!(
                                                                    "Injection error: {}",
                                                                    error
                                                                );
                                                            }
                                                            let _ = crate::injector::copy_text_to_clipboard(&final_text);
                                                            emit_recording_state(
                                                                &app_bg,
                                                                &recording_state_bg,
                                                                "error",
                                                            );
                                                        }
                                                    }
                                                    Err(e) => {
                                                        eprintln!("Whisper Error: {}", e);
                                                        emit_recording_state(
                                                            &app_bg,
                                                            &recording_state_bg,
                                                            "error",
                                                        );
                                                    }
                                                }
                                            } else {
                                                eprintln!("Whisper model is not loaded yet.");
                                                emit_recording_state(
                                                    &app_bg,
                                                    &recording_state_bg,
                                                    "error",
                                                );
                                            }
                                        }

                                        if session_id_bg.load(Ordering::SeqCst) == my_session {
                                            std::thread::sleep(std::time::Duration::from_millis(
                                                if success { 800 } else { 1500 },
                                            ));
                                            if session_id_bg.load(Ordering::SeqCst) == my_session {
                                                emit_recording_state(
                                                    &app_bg,
                                                    &recording_state_bg,
                                                    "idle",
                                                );
                                                if !result_emitted {
                                                    std::thread::sleep(
                                                        std::time::Duration::from_millis(800),
                                                    );
                                                    set_overlay_visible(&app_bg, false);
                                                }
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
                                    emit_recording_state(
                                        &app_clone,
                                        &recording_state_clone,
                                        "idle",
                                    );
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

    pub fn update_hotkey_on_main_thread(
        new_hotkey_str: &str,
        active_id: &Arc<std::sync::atomic::AtomicU32>,
    ) {
        let new_hotkey = parse_hotkey(new_hotkey_str);

        CURRENT_HOTKEY.with(|curr| {
            if let Some(mut current_hotkey_ref) = curr.borrow_mut().as_mut() {
                if *current_hotkey_ref == new_hotkey {
                    return;
                }

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
