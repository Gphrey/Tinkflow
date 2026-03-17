use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub whisper_model: String,
    pub llm_model: String,
    pub audio_device_name: String,
    #[serde(default)]
    pub launch_at_startup: bool,
    #[serde(default)]
    pub onboarding_completed: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            whisper_model: "tiny.en".to_string(),
            llm_model: "".to_string(),
            audio_device_name: "default".to_string(),
            launch_at_startup: false,
            onboarding_completed: false,
        }
    }
}

pub struct SettingsManager {
    settings_path: PathBuf,
    pub current_settings: Mutex<AppSettings>,
}

impl SettingsManager {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        // In Tauri v2, use the PathResolver
        let mut path = app_handle.path().app_data_dir().expect("Failed to get app data dir");
        // Ensure the directory exists
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

        // Save immediately to ensure file exists
        if !path.exists() {
            if let Ok(json) = serde_json::to_string_pretty(&settings) {
                let _ = fs::write(&path, json);
            }
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
