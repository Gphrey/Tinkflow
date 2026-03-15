//! Local Whisper transcription engine.
//!
//! Manages GGML model lifecycle (download, load, unload) and provides
//! low-latency speech-to-text via [`whisper-rs`]. Designed for short-form
//! developer dictation with a greedy, single-pass inference strategy.
//!
//! ```text
//! Audio capture → WhisperTranscriber::transcribe() → DeveloperDictionary → LLM polish
//! ```

use std::path::PathBuf;
use tauri::{Emitter, Manager};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

// ─── Supported Models ────────────────────────────────────────────────────────

/// Canonical model identifiers and their HuggingFace download URLs.
///
/// Adding a new model is a single-line change here — all other functions
/// derive behaviour from this table.
const MODELS: &[(&str, &str)] = &[
    (
        "tiny.en",
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin",
    ),
    (
        "base.en",
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin",
    ),
    (
        "small.en",
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin",
    ),
    (
        "medium.en",
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.en.bin",
    ),
];

/// Resolve a user-supplied model name to a canonical short name.
///
/// Accepts both `"base.en"` and `"ggml-base.en.bin"` formats and returns the
/// canonical short form (e.g. `"base.en"`), or `None` if unrecognised.
fn canonical_model_name(input: &str) -> Option<&'static str> {
    // Strip the `ggml-` prefix and `.bin` suffix if present
    let normalised = input
        .strip_prefix("ggml-")
        .unwrap_or(input)
        .strip_suffix(".bin")
        .unwrap_or(input);

    MODELS
        .iter()
        .find(|(name, _)| *name == normalised)
        .map(|(name, _)| *name)
}

/// Look up the download URL for a canonical model name.
fn model_url(canonical: &str) -> Option<&'static str> {
    MODELS
        .iter()
        .find(|(name, _)| *name == canonical)
        .map(|(_, url)| *url)
}

// ─── Transcriber ─────────────────────────────────────────────────────────────

/// Encapsulates a loaded Whisper GGML context for speech-to-text.
///
/// Holds the model path and a lazily-initialised [`WhisperContext`].
/// Thread-safety is provided externally via `Arc<Mutex<WhisperTranscriber>>`
/// in the Tauri managed state.
pub struct WhisperTranscriber {
    model_path: PathBuf,
    context: Option<WhisperContext>,
}

impl WhisperTranscriber {
    /// Create a new transcriber pointing at `model_path`.
    ///
    /// The model is **not** loaded until [`load_model`](Self::load_model) is
    /// called explicitly.
    pub fn new(model_path: PathBuf) -> Self {
        Self {
            model_path,
            context: None,
        }
    }

    /// Update the model path and unload any currently loaded model.
    ///
    /// The next call to [`load_model`](Self::load_model) will load the new
    /// binary.
    pub fn set_model_path(&mut self, new_path: PathBuf) {
        self.model_path = new_path;
        self.context = None;
    }

    /// Load the GGML binary into a [`WhisperContext`].
    ///
    /// This can take several seconds for larger models. Returns an error if the
    /// file does not exist or the binary is corrupt.
    pub fn load_model(&mut self) -> Result<(), String> {
        if !self.model_path.exists() {
            return Err(format!(
                "Model file not found: {}",
                self.model_path.display()
            ));
        }

        let params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(&self.model_path.to_string_lossy(), params)
            .map_err(|e| format!("Failed to load Whisper model: {}", e))?;

        self.context = Some(ctx);
        Ok(())
    }

    /// Returns `true` if a model is currently loaded and ready for inference.
    pub fn is_model_loaded(&self) -> bool {
        self.context.is_some()
    }

    /// Run inference on `audio_data` (16 kHz, f32, mono).
    ///
    /// Uses a **greedy** sampling strategy with up to 4 threads for minimal
    /// latency on short-form dictation clips.
    ///
    /// # Errors
    ///
    /// Returns an error if the model has not been loaded or if inference fails.
    pub fn transcribe(&self, audio_data: &[f32]) -> Result<String, String> {
        let ctx = self
            .context
            .as_ref()
            .ok_or("WhisperContext not initialized. Call load_model() first.")?;

        let mut state = ctx
            .create_state()
            .map_err(|e| format!("Failed to create Whisper state: {}", e))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_progress(false);
        params.set_print_special(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        // Cap at 4 threads to avoid over-subscription on high-core machines
        let threads = i32::try_from(num_cpus::get().min(4)).unwrap_or(4);
        params.set_n_threads(threads);

        state
            .full(params, audio_data)
            .map_err(|e| format!("Failed to run Whisper model: {}", e))?;

        let num_segments = state.full_n_segments();
        let mut full_text = String::new();
        for i in 0..num_segments {
            if let Some(segment) = state.get_segment(i) {
                if let Ok(text) = segment.to_str() {
                    full_text.push_str(text);
                }
            }
        }

        Ok(full_text.trim().to_string())
    }
}

// ─── Path Helpers ────────────────────────────────────────────────────────────

/// Resolve the on-disk path for a given model name.
///
/// The file is stored as `<app_data_dir>/ggml-<canonical>.bin`.
///
/// # Errors
///
/// Returns an error if the app data directory cannot be resolved or if
/// `model_name` is not a recognised model.
pub fn get_model_path(
    app: &tauri::AppHandle,
    model_name: &str,
) -> Result<PathBuf, String> {
    let canonical = canonical_model_name(model_name).ok_or_else(|| {
        format!(
            "Unknown model: '{}'. Supported: {}",
            model_name,
            MODELS
                .iter()
                .map(|(n, _)| *n)
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    let app_dir = app.path().app_data_dir().map_err(|e: tauri::Error| e.to_string())?;
    std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    Ok(app_dir.join(format!("ggml-{}.bin", canonical)))
}

// ─── Tauri Commands ──────────────────────────────────────────────────────────

/// Check whether a Whisper model binary exists on disk.
#[tauri::command]
pub fn check_whisper_model(
    app: tauri::AppHandle,
    model_name: String,
) -> Result<bool, String> {
    let path = get_model_path(&app, &model_name)?;
    Ok(path.exists())
}

/// List all installed GGML model binaries in the app data directory.
///
/// Returns canonical short names (e.g. `["tiny.en", "base.en"]`).
#[tauri::command]
pub fn list_installed_whisper_models(
    app: tauri::AppHandle,
) -> Result<Vec<String>, String> {
    let app_dir = app.path().app_data_dir().map_err(|e: tauri::Error| e.to_string())?;
    let mut installed = Vec::new();

    if let Ok(entries) = std::fs::read_dir(app_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if let Some(canonical) = canonical_model_name(file_name) {
                        installed.push(canonical.to_string());
                    }
                }
            }
        }
    }

    Ok(installed)
}

/// Download a Whisper GGML model from HuggingFace.
///
/// Writes to a `.tmp` file first, then atomically renames on success to
/// prevent partial/corrupt binaries from being used.
///
/// Emits `"model-download-progress"` events (0.0–100.0) to the frontend.
///
/// # Errors
///
/// Returns an error if the model name is unrecognised, the download fails,
/// or the file cannot be written.
#[tauri::command]
pub async fn download_whisper_model(
    app: tauri::AppHandle,
    model_name: String,
    cancel_flag: tauri::State<'_, crate::DownloadCancelFlag>,
) -> Result<(), String> {
    let path = get_model_path(&app, &model_name)?;
    if path.exists() {
        return Ok(());
    }

    let canonical = canonical_model_name(&model_name).ok_or_else(|| {
        format!("Unknown model: '{}'", model_name)
    })?;
    let url = model_url(canonical)
        .ok_or_else(|| format!("No URL for model: '{}'", canonical))?
        .to_string();

    let tmp_path = path.with_extension("bin.tmp");

    // Reset the cancel flag before starting
    let flag = cancel_flag.inner().clone();
    flag.store(false, std::sync::atomic::Ordering::SeqCst);

    tauri::async_runtime::spawn_blocking(move || {
        use std::io::Write;
        use std::sync::atomic::Ordering;

        let client = reqwest::blocking::Client::new();
        let mut response = client
            .get(&url)
            .send()
            .map_err(|e: reqwest::Error| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!(
                "Download failed with status: {}",
                response.status()
            ));
        }

        let total_size = response.content_length().unwrap_or(75_000_000);
        let mut file =
            std::fs::File::create(&tmp_path).map_err(|e| e.to_string())?;

        let mut downloaded: u64 = 0;
        let mut buffer = [0u8; 8192];
        let mut last_emit = 0.0_f64;

        loop {
            // Check for cancellation before every chunk
            if flag.load(Ordering::SeqCst) {
                drop(file);
                let _ = std::fs::remove_file(&tmp_path); // clean up partial file
                let _ = app.emit("model-download-progress", -1.0_f64); // sentinel for cancelled
                println!("[Whisper] Download cancelled by user.");
                return Err("cancelled".to_string());
            }

            let bytes_read = std::io::Read::read(&mut response, &mut buffer)
                .map_err(|e| e.to_string())?;
            if bytes_read == 0 {
                break;
            }

            file.write_all(&buffer[..bytes_read])
                .map_err(|e| e.to_string())?;
            downloaded += bytes_read as u64;

            let progress = (downloaded as f64 / total_size as f64) * 100.0;
            if progress - last_emit > 1.0 || downloaded >= total_size {
                let _ = app.emit("model-download-progress", progress);
                last_emit = progress;
            }
        }

        // Flush and atomically move into place
        file.flush().map_err(|e| e.to_string())?;
        drop(file);
        std::fs::rename(&tmp_path, &path).map_err(|e| {
            format!(
                "Downloaded OK but failed to rename {:?} → {:?}: {}",
                tmp_path, path, e
            )
        })?;

        Ok::<(), String>(())
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(())
}

/// Load a model into the managed `WhisperTranscriber` state.
///
/// Called by the frontend after a successful download to prepare the engine
/// for transcription.
#[tauri::command]
pub fn load_whisper_model(
    state: tauri::State<'_, std::sync::Arc<std::sync::Mutex<WhisperTranscriber>>>,
) -> Result<(), String> {
    state
        .lock()
        .map_err(|_| "Failed to lock whisper state".to_string())?
        .load_model()
}
