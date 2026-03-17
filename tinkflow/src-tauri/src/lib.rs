pub mod audio;
pub mod context;
pub mod dictionary;
pub mod hotkey;
pub mod injector;
pub mod llm;
pub mod settings;
pub mod whisper;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;

/// Shared cancel flag for any in-progress model download.
/// Set to `true` via the `cancel_download` command; reset to `false` at the
/// start of each download so a previous cancel doesn't block future downloads.
pub type DownloadCancelFlag = Arc<AtomicBool>;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
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
fn list_ollama_models(ollama: tauri::State<'_, Arc<Mutex<llm::OllamaClient>>>) -> Result<Vec<String>, String> {
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
    flag.store(false, Ordering::SeqCst); // reset any previous cancel
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
fn get_app_settings(settings_manager: tauri::State<'_, Arc<settings::SettingsManager>>) -> settings::AppSettings {
    settings_manager.get()
}

#[tauri::command]
fn update_app_settings(
    settings_manager: tauri::State<'_, Arc<settings::SettingsManager>>, 
    whisper_state: tauri::State<'_, Arc<Mutex<whisper::WhisperTranscriber>>>,
    app: tauri::AppHandle,
    new_settings: settings::AppSettings
) -> Result<(), String> {
    let current_settings = settings_manager.get();
    let whisper_changed = current_settings.whisper_model != new_settings.whisper_model;

    settings_manager.update(new_settings.clone())?;

    if whisper_changed {
        if let Ok(path) = whisper::get_model_path(&app, &new_settings.whisper_model) {
            if let Ok(mut transcriber) = whisper_state.lock() {
                println!("Whisper model changed to: {}, unloading old model.", new_settings.whisper_model);
                transcriber.set_model_path(path); // This correctly resets the context
            }
        }
    }

    Ok(())
}

#[tauri::command]
fn get_audio_devices() -> Vec<String> {
    crate::audio::list_input_devices()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
    // Register the shared download-cancel flag
            let cancel_flag: DownloadCancelFlag = Arc::new(AtomicBool::new(false));
            app.manage(cancel_flag);

            // Set custom window icon (embedded at compile time)
            if let Some(main_window) = app.get_webview_window("main") {
                let icon_bytes = include_bytes!("../icons/icon.png");
                let icon = tauri::image::Image::from_bytes(icon_bytes)
                    .expect("failed to load embedded icon");
                let _ = main_window.set_icon(icon);
            }

            // Register the autostart plugin (desktop only)
            #[cfg(desktop)]
            {
                use tauri_plugin_autostart::MacosLauncher;
                app.handle().plugin(tauri_plugin_autostart::init(
                    MacosLauncher::LaunchAgent,
                    None,
                ))?;
            }

            // Load persistent settings
            let settings_manager = Arc::new(settings::SettingsManager::new(app.handle()));
            app.manage(settings_manager.clone());

            // Sync OS autostart state with persisted setting
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

            // Instantiate Audio Capturer with a default VAD RMS threshold of 0.002
            let audio_capturer = Arc::new(Mutex::new(crate::audio::AudioCapturer::new(0.002, settings_manager.clone())));
            app.manage(audio_capturer.clone());

            // Instantiate Whisper Transcriber state based on saved settings
            let whisper_model = settings_manager.get().whisper_model;
            let whisper_model_path = whisper::get_model_path(&app.handle(), &whisper_model)
                .unwrap_or_else(|_| app.path().app_data_dir().unwrap_or_default().join(format!("ggml-{}.bin", whisper_model)));
            let mut whisper_transcriber = whisper::WhisperTranscriber::new(whisper_model_path);
            let _ = whisper_transcriber.load_model(); // Ignore error if not downloaded yet
            
            let whisper_state = Arc::new(Mutex::new(whisper_transcriber));
            app.manage(whisper_state.clone());

            // Initialize Ollama LLM client
            let ollama_client = Arc::new(Mutex::new(llm::OllamaClient::new()));
            app.manage(ollama_client.clone());

            // Initialize the global hotkey state and leak it to keep it alive indefinitely
            let hotkey_manager = hotkey::HotkeyManager::new(
                app.handle().clone(),
                audio_capturer,
                whisper_state,
                ollama_client,
                settings_manager.clone(),
            );
            Box::leak(Box::new(hotkey_manager));

            // Create the overlay window programmatically for proper WebView2 transparency on Windows
            let overlay_width = 300.0_f64;
            let overlay_height = 80.0_f64;

            let mut builder = tauri::WebviewWindowBuilder::new(
                app,
                "overlay",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("Tinkflow Overlay")
            .inner_size(overlay_width, overlay_height)
            .transparent(true)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false) // Hidden by default, shown when recording
            .resizable(false);

            // Position at the bottom center of the primary monitor
            if let Some(monitor) = app.primary_monitor().ok().flatten() {
                let screen_size = monitor.size();
                let x = ((screen_size.width as f64 - overlay_width) / 2.0) as i32;
                let y = (screen_size.height as f64 - overlay_height - 60.0) as i32; // 60px from bottom
                builder = builder.position(x as f64, y as f64);
            } else {
                builder = builder.center();
            }

            let overlay = builder.build();

            if let Ok(overlay_win) = overlay {
                let _ = overlay_win.set_ignore_cursor_events(true);
            } else {
                eprintln!("Failed to create overlay window: {:?}", overlay.err());
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
            get_audio_devices
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
