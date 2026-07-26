use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextProfile {
    pub context: String,
    pub enabled: bool,
    pub tone: String,
    pub preserve_symbols: bool,
    pub remove_fillers: bool,
    pub punctuation: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub whisper_model: String,
    pub llm_model: String,
    pub audio_device_name: String,
    #[serde(default)]
    pub launch_at_startup: bool,
    #[serde(default)]
    pub onboarding_completed: bool,
    #[serde(default = "default_hotkey")]
    pub dictation_hotkey: String,
    #[serde(default = "default_injection_mode")]
    pub injection_mode: String,
    #[serde(default = "default_transcription_quality")]
    pub transcription_quality: String,
    #[serde(default = "default_dictation_enabled")]
    pub dictation_enabled: bool,
    #[serde(default = "default_context_profiles")]
    pub context_profiles: Vec<ContextProfile>,
}

fn default_hotkey() -> String {
    "Ctrl+Space".to_string()
}

fn default_injection_mode() -> String {
    "auto".to_string()
}

fn default_transcription_quality() -> String {
    "balanced".to_string()
}

fn default_dictation_enabled() -> bool {
    true
}

fn profile(context: &str, tone: &str, preserve_symbols: bool) -> ContextProfile {
    ContextProfile {
        context: context.to_string(),
        enabled: true,
        tone: tone.to_string(),
        preserve_symbols,
        remove_fillers: true,
        punctuation: true,
    }
}

pub fn default_context_profiles() -> Vec<ContextProfile> {
    vec![
        profile("code", "precise developer dictation", true),
        profile("chat", "casual and concise", false),
        profile("email", "professional and polished", false),
        profile("terminal", "short command-like text", true),
        profile("general", "natural clear English", false),
    ]
}

impl AppSettings {
    pub fn context_profile(&self, context: &str) -> Option<ContextProfile> {
        self.context_profiles
            .iter()
            .find(|profile| profile.context == context)
            .cloned()
            .or_else(|| {
                self.context_profiles
                    .iter()
                    .find(|profile| profile.context == "general")
                    .cloned()
            })
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            whisper_model: "tiny.en".to_string(),
            llm_model: "".to_string(),
            audio_device_name: "default".to_string(),
            launch_at_startup: false,
            onboarding_completed: false,
            dictation_hotkey: default_hotkey(),
            injection_mode: default_injection_mode(),
            transcription_quality: default_transcription_quality(),
            dictation_enabled: true,
            context_profiles: default_context_profiles(),
        }
    }
}

pub struct SettingsManager {
    settings_path: PathBuf,
    pub current_settings: Mutex<AppSettings>,
}

impl SettingsManager {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        let mut path = app_handle
            .path()
            .app_data_dir()
            .expect("Failed to get app data dir");
        let _ = fs::create_dir_all(&path);

        path.push("settings.json");

        let settings = if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                serde_json::from_str(&content).unwrap_or_default()
            } else {
                AppSettings::default()
            }
        } else {
            AppSettings::default()
        };

        if let Ok(json) = serde_json::to_string_pretty(&settings) {
            let _ = fs::write(&path, json);
        }

        Self {
            settings_path: path,
            current_settings: Mutex::new(settings),
        }
    }

    pub fn get(&self) -> AppSettings {
        self.current_settings.lock().unwrap().clone()
    }

    pub fn update(&self, new_settings: AppSettings) -> Result<(), String> {
        let mut current = self.current_settings.lock().unwrap();
        *current = new_settings.clone();

        let json = serde_json::to_string_pretty(&new_settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(&self.settings_path, json)
            .map_err(|e| format!("Failed to save settings: {}", e))?;

        Ok(())
    }
}
