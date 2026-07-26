pub mod audio;
pub mod context;
pub mod corrections;
pub mod dictionary;
pub mod history;
pub mod hotkey;
pub mod injector;
pub mod llm;
pub mod settings;
pub mod whisper;

use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};

/// Shared cancel flag for any in-progress model download.
/// Set to `true` via the `cancel_download` command; reset to `false` at the
/// start of each download so a previous cancel doesn't block future downloads.
pub type DownloadCancelFlag = Arc<AtomicBool>;
pub type RecordingStateStore = Arc<Mutex<String>>;
pub type DictationResultStore = Arc<Mutex<Option<hotkey::DictationResult>>>;

#[derive(Debug, Serialize)]
pub struct ModelHealth {
    whisper_model: String,
    whisper_installed: bool,
    llm_model: String,
    ollama_connected: bool,
    ollama_version: Option<String>,
    ollama_models: Vec<String>,
    injection_mode: String,
    dictation_enabled: bool,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn check_ollama_status(ollama: tauri::State<'_, Arc<Mutex<llm::OllamaClient>>>) -> bool {
    if let Ok(client) = ollama.lock() {
        client.check_health()
    } else {
        false
    }
}

#[tauri::command]
fn list_ollama_models(
    ollama: tauri::State<'_, Arc<Mutex<llm::OllamaClient>>>,
) -> Result<Vec<String>, String> {
    if let Ok(client) = ollama.lock() {
        client.list_models()
    } else {
        Err("Failed to access Ollama client".to_string())
    }
}

#[tauri::command]
async fn pull_ollama_model(
    app: tauri::AppHandle,
    model_name: String,
    cancel_flag: tauri::State<'_, DownloadCancelFlag>,
) -> Result<(), String> {
    let flag = cancel_flag.inner().clone();
    flag.store(false, Ordering::SeqCst);
    tauri::async_runtime::spawn_blocking(move || {
        let client = crate::llm::OllamaClient::new();
        client.pull_model(&model_name, &app, &flag)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Signal any active model download (Whisper or Ollama) to abort.
#[tauri::command]
fn cancel_download(cancel_flag: tauri::State<'_, DownloadCancelFlag>) {
    cancel_flag.store(true, Ordering::SeqCst);
    println!("[Download] Cancel signal sent.");
}

#[tauri::command]
fn get_app_settings(
    settings_manager: tauri::State<'_, Arc<settings::SettingsManager>>,
) -> settings::AppSettings {
    settings_manager.get()
}

#[tauri::command]
fn update_app_settings(
    settings_manager: tauri::State<'_, Arc<settings::SettingsManager>>,
    whisper_state: tauri::State<'_, Arc<Mutex<whisper::WhisperTranscriber>>>,
    active_id: tauri::State<'_, ActiveHotkeyId>,
    app: tauri::AppHandle,
    new_settings: settings::AppSettings,
) -> Result<(), String> {
    let current_settings = settings_manager.get();
    let whisper_changed = current_settings.whisper_model != new_settings.whisper_model;
    let hotkey_changed = current_settings.dictation_hotkey != new_settings.dictation_hotkey;

    settings_manager.update(new_settings.clone())?;

    if hotkey_changed {
        let hotkey_str = new_settings.dictation_hotkey.clone();
        let active_id_clone = active_id.0.clone();
        let _ = app.run_on_main_thread(move || {
            hotkey::HotkeyListener::update_hotkey_on_main_thread(&hotkey_str, &active_id_clone);
        });
    }

    if whisper_changed {
        if let Ok(path) = whisper::get_model_path(&app, &new_settings.whisper_model) {
            if let Ok(mut transcriber) = whisper_state.lock() {
                println!(
                    "Whisper model changed to: {}, unloading old model.",
                    new_settings.whisper_model
                );
                transcriber.set_model_path(path);
            }
        }
    }

    Ok(())
}

#[tauri::command]
fn get_audio_devices() -> Vec<String> {
    crate::audio::list_input_devices()
}

#[tauri::command]
fn get_recording_state(
    recording_state: tauri::State<'_, RecordingStateStore>,
) -> Result<String, String> {
    recording_state
        .lock()
        .map(|state| state.clone())
        .map_err(|_| "Failed to access recording state".to_string())
}

#[tauri::command]
fn get_active_dictation_result(
    active_result: tauri::State<'_, DictationResultStore>,
) -> Option<hotkey::DictationResult> {
    active_result.lock().ok().and_then(|result| result.clone())
}

#[tauri::command]
fn set_overlay_interactive(app: tauri::AppHandle, interactive: bool) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.set_ignore_cursor_events(!interactive);
    }
}

#[tauri::command]
fn dismiss_overlay(
    app: tauri::AppHandle,
    active_result: tauri::State<'_, DictationResultStore>,
) {
    if let Ok(mut result) = active_result.lock() {
        *result = None;
    }
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.set_ignore_cursor_events(true);
        let _ = overlay.hide();
    }
}

#[tauri::command]
fn list_transcription_history(
    history: tauri::State<'_, Arc<history::HistoryManager>>,
    limit: Option<usize>,
) -> Vec<history::TranscriptionRecord> {
    history.list(limit.unwrap_or(20).min(100))
}

#[tauri::command]
fn clear_transcription_history(
    history: tauri::State<'_, Arc<history::HistoryManager>>,
) -> Result<(), String> {
    history.clear()
}

#[tauri::command]
fn copy_last_transcription(
    history: tauri::State<'_, Arc<history::HistoryManager>>,
) -> Result<(), String> {
    let record = history
        .last()
        .ok_or_else(|| "No transcription history yet".to_string())?;
    injector::copy_text_to_clipboard(&record.final_text)
}

#[tauri::command]
fn copy_transcription(
    id: u64,
    history: tauri::State<'_, Arc<history::HistoryManager>>,
) -> Result<(), String> {
    let record = history
        .get(id)
        .ok_or_else(|| "That transcription is no longer available.".to_string())?;
    injector::copy_text_to_clipboard(&record.final_text)
}

#[tauri::command]
fn list_corrections(
    corrections: tauri::State<'_, Arc<corrections::CorrectionManager>>,
) -> Vec<corrections::CorrectionEntry> {
    corrections.list()
}

#[tauri::command]
fn add_correction(
    corrections: tauri::State<'_, Arc<corrections::CorrectionManager>>,
    spoken: String,
    replacement: String,
) -> Result<corrections::CorrectionEntry, String> {
    corrections.add(spoken, replacement)
}

#[tauri::command]
fn remove_correction(
    corrections: tauri::State<'_, Arc<corrections::CorrectionManager>>,
    id: u64,
) -> Result<(), String> {
    corrections.remove(id)
}

#[tauri::command]
fn set_correction_enabled(
    corrections: tauri::State<'_, Arc<corrections::CorrectionManager>>,
    id: u64,
    enabled: bool,
) -> Result<(), String> {
    corrections.set_enabled(id, enabled)
}

#[tauri::command]
fn get_model_health(
    app: tauri::AppHandle,
    settings_manager: tauri::State<'_, Arc<settings::SettingsManager>>,
    ollama: tauri::State<'_, Arc<Mutex<llm::OllamaClient>>>,
) -> ModelHealth {
    let settings = settings_manager.get();
    let whisper_installed = whisper::get_model_path(&app, &settings.whisper_model)
        .map(|path| path.exists())
        .unwrap_or(false);

    let (ollama_connected, ollama_version, ollama_models) = if let Ok(client) = ollama.lock() {
        let connected = client.check_health();
        let version = if connected {
            client.version().ok()
        } else {
            None
        };
        let models = if connected {
            client.list_models().unwrap_or_default()
        } else {
            Vec::new()
        };
        (connected, version, models)
    } else {
        (false, None, Vec::new())
    };

    ModelHealth {
        whisper_model: settings.whisper_model,
        whisper_installed,
        llm_model: settings.llm_model,
        ollama_connected,
        ollama_version,
        ollama_models,
        injection_mode: settings.injection_mode,
        dictation_enabled: settings.dictation_enabled,
    }
}

pub struct ActiveHotkeyId(pub Arc<std::sync::atomic::AtomicU32>);

fn setup_tray(app: &mut tauri::App) -> tauri::Result<()> {
    use tauri::menu::MenuBuilder;
    use tauri::tray::TrayIconBuilder;

    let menu = MenuBuilder::new(app)
        .text("open", "Open Tinkflow")
        .text("toggle_enabled", "Toggle Dictation")
        .text("copy_last", "Copy Last Transcription")
        .separator()
        .text("quit", "Quit")
        .build()?;

    let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/icon.png"))?;

    TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .icon(icon)
        .tooltip("Tinkflow")
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().0.as_str() {
            "open" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "toggle_enabled" => {
                let settings_manager = app.state::<Arc<settings::SettingsManager>>();
                let mut settings = settings_manager.get();
                settings.dictation_enabled = !settings.dictation_enabled;
                let _ = settings_manager.update(settings.clone());
                let _ = app.emit("dictation-enabled", settings.dictation_enabled);
            }
            "copy_last" => {
                let history = app.state::<Arc<history::HistoryManager>>();
                if let Some(record) = history.last() {
                    let _ = injector::copy_text_to_clipboard(&record.final_text);
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let cancel_flag: DownloadCancelFlag = Arc::new(AtomicBool::new(false));
            app.manage(cancel_flag);

            let recording_state: RecordingStateStore = Arc::new(Mutex::new("idle".to_string()));
            app.manage(recording_state.clone());

            let active_result: DictationResultStore = Arc::new(Mutex::new(None));
            app.manage(active_result);

            if let Some(main_window) = app.get_webview_window("main") {
                let icon_bytes = include_bytes!("../icons/icon.png");
                let icon = tauri::image::Image::from_bytes(icon_bytes)
                    .expect("failed to load embedded icon");
                let _ = main_window.set_icon(icon);
            }

            #[cfg(desktop)]
            {
                use tauri_plugin_autostart::MacosLauncher;
                app.handle().plugin(tauri_plugin_autostart::init(
                    MacosLauncher::LaunchAgent,
                    None,
                ))?;
            }

            let settings_manager = Arc::new(settings::SettingsManager::new(app.handle()));
            app.manage(settings_manager.clone());

            let history_manager = Arc::new(history::HistoryManager::new(app.handle()));
            app.manage(history_manager.clone());

            let correction_manager = Arc::new(corrections::CorrectionManager::new(app.handle()));
            app.manage(correction_manager.clone());

            #[cfg(desktop)]
            {
                use tauri_plugin_autostart::ManagerExt;
                let autolaunch = app.autolaunch();
                let should_autostart = settings_manager.get().launch_at_startup;
                let is_enabled = autolaunch.is_enabled().unwrap_or(false);
                if should_autostart && !is_enabled {
                    let _ = autolaunch.enable();
                } else if !should_autostart && is_enabled {
                    let _ = autolaunch.disable();
                }
            }

            let audio_capturer = Arc::new(Mutex::new(crate::audio::AudioCapturer::new(
                0.002,
                settings_manager.clone(),
            )));
            app.manage(audio_capturer.clone());

            let whisper_model = settings_manager.get().whisper_model;
            let whisper_model_path = whisper::get_model_path(&app.handle(), &whisper_model)
                .unwrap_or_else(|_| {
                    app.path()
                        .app_data_dir()
                        .unwrap_or_default()
                        .join(format!("ggml-{}.bin", whisper_model))
                });
            let mut whisper_transcriber = whisper::WhisperTranscriber::new(whisper_model_path);
            let _ = whisper_transcriber.load_model();

            let whisper_state = Arc::new(Mutex::new(whisper_transcriber));
            app.manage(whisper_state.clone());

            let ollama_client = Arc::new(Mutex::new(llm::OllamaClient::new()));
            app.manage(ollama_client.clone());

            setup_tray(app)?;

            let active_id = hotkey::HotkeyListener::init_on_main_thread(
                app.handle().clone(),
                audio_capturer,
                whisper_state,
                ollama_client,
                settings_manager.clone(),
                correction_manager,
                history_manager,
                recording_state,
            );
            app.manage(ActiveHotkeyId(active_id));

            let overlay_width = 680.0_f64;
            let overlay_height = 240.0_f64;

            let mut builder = tauri::WebviewWindowBuilder::new(
                app,
                "overlay",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("Tinkflow Overlay")
            .inner_size(overlay_width, overlay_height)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false)
            .resizable(false);

            #[cfg(target_os = "windows")]
            {
                builder = builder.transparent(true);
            }

            if let Some(monitor) = app.primary_monitor().ok().flatten() {
                let screen_size = monitor.size();
                let x = ((screen_size.width as f64 - overlay_width) / 2.0) as i32;
                let y = (screen_size.height as f64 - overlay_height - 60.0) as i32;
                builder = builder.position(x as f64, y as f64);
            } else {
                builder = builder.center();
            }

            match builder.build() {
                Ok(overlay_win) => {
                    let _ = overlay_win.set_ignore_cursor_events(true);
                }
                Err(e) => {
                    eprintln!("Failed to create overlay window: {:?}", e);
                }
            }

            Ok(())
        })
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            whisper::check_whisper_model,
            whisper::list_installed_whisper_models,
            whisper::download_whisper_model,
            whisper::load_whisper_model,
            check_ollama_status,
            list_ollama_models,
            pull_ollama_model,
            cancel_download,
            get_app_settings,
            update_app_settings,
            get_audio_devices,
            get_recording_state,
            get_active_dictation_result,
            set_overlay_interactive,
            dismiss_overlay,
            list_transcription_history,
            clear_transcription_history,
            copy_last_transcription,
            copy_transcription,
            list_corrections,
            add_correction,
            remove_correction,
            set_correction_enabled,
            get_model_health
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
