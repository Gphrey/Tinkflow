use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CorrectionEntry {
    pub id: u64,
    pub spoken: String,
    pub replacement: String,
    pub enabled: bool,
    pub created_at_ms: u64,
}

pub struct CorrectionManager {
    corrections_path: PathBuf,
    entries: Mutex<Vec<CorrectionEntry>>,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

fn replace_ascii_case_insensitive(input: &str, needle: &str, replacement: &str) -> String {
    if needle.is_empty() {
        return input.to_string();
    }

    let lower_input = input.to_ascii_lowercase();
    let lower_needle = needle.to_ascii_lowercase();
    let mut result = String::new();
    let mut cursor = 0;

    while let Some(pos) = lower_input[cursor..].find(&lower_needle) {
        let start = cursor + pos;
        let end = start + needle.len();
        result.push_str(&input[cursor..start]);
        result.push_str(replacement);
        cursor = end;
    }

    result.push_str(&input[cursor..]);
    result
}

impl CorrectionManager {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        let mut path = app_handle
            .path()
            .app_data_dir()
            .expect("Failed to get app data dir");
        let _ = fs::create_dir_all(&path);
        path.push("corrections.json");

        let entries = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Self {
            corrections_path: path,
            entries: Mutex::new(entries),
        }
    }

    pub fn list(&self) -> Vec<CorrectionEntry> {
        self.entries
            .lock()
            .map(|entries| entries.clone())
            .unwrap_or_default()
    }

    pub fn add(&self, spoken: String, replacement: String) -> Result<CorrectionEntry, String> {
        let spoken = spoken.trim().to_string();
        let replacement = replacement.trim().to_string();
        if spoken.is_empty() || replacement.is_empty() {
            return Err("Correction phrases cannot be empty".to_string());
        }

        let mut entries = self
            .entries
            .lock()
            .map_err(|_| "Failed to lock corrections".to_string())?;
        let entry = CorrectionEntry {
            id: now_ms(),
            spoken,
            replacement,
            enabled: true,
            created_at_ms: now_ms(),
        };
        entries.insert(0, entry.clone());
        self.save_locked(&entries)?;
        Ok(entry)
    }

    pub fn remove(&self, id: u64) -> Result<(), String> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| "Failed to lock corrections".to_string())?;
        entries.retain(|entry| entry.id != id);
        self.save_locked(&entries)
    }

    pub fn set_enabled(&self, id: u64, enabled: bool) -> Result<(), String> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| "Failed to lock corrections".to_string())?;
        if let Some(entry) = entries.iter_mut().find(|entry| entry.id == id) {
            entry.enabled = enabled;
        }
        self.save_locked(&entries)
    }

    pub fn apply(&self, text: &str) -> String {
        let mut result = text.to_string();
        let mut entries = self.list();
        entries.sort_by(|a, b| b.spoken.len().cmp(&a.spoken.len()));
        for entry in entries.into_iter().filter(|entry| entry.enabled) {
            result = replace_ascii_case_insensitive(&result, &entry.spoken, &entry.replacement);
        }
        result
    }

    fn save_locked(&self, entries: &[CorrectionEntry]) -> Result<(), String> {
        let json = serde_json::to_string_pretty(entries)
            .map_err(|e| format!("Failed to serialize corrections: {}", e))?;
        fs::write(&self.corrections_path, json)
            .map_err(|e| format!("Failed to save corrections: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::replace_ascii_case_insensitive;

    #[test]
    fn replacement_is_case_insensitive() {
        assert_eq!(
            replace_ascii_case_insensitive("Use tink flow here", "Tink Flow", "Tinkflow"),
            "Use Tinkflow here"
        );
    }
}
