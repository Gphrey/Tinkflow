use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranscriptionRecord {
    pub id: u64,
    pub created_at_ms: u64,
    pub raw_text: String,
    pub corrected_text: String,
    pub final_text: String,
    pub context: String,
    pub injection_mode: String,
    pub injection_succeeded: bool,
    pub injection_error: Option<String>,
}

pub struct HistoryManager {
    history_path: PathBuf,
    records: Mutex<Vec<TranscriptionRecord>>,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

impl HistoryManager {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        let mut path = app_handle
            .path()
            .app_data_dir()
            .expect("Failed to get app data dir");
        let _ = fs::create_dir_all(&path);
        path.push("transcription_history.json");

        let records = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Self {
            history_path: path,
            records: Mutex::new(records),
        }
    }

    pub fn add(
        &self,
        raw_text: String,
        corrected_text: String,
        final_text: String,
        context: String,
        injection_mode: String,
        injection_succeeded: bool,
        injection_error: Option<String>,
    ) -> Result<TranscriptionRecord, String> {
        let mut records = self
            .records
            .lock()
            .map_err(|_| "Failed to lock history".to_string())?;
        let record = TranscriptionRecord {
            id: now_ms(),
            created_at_ms: now_ms(),
            raw_text,
            corrected_text,
            final_text,
            context,
            injection_mode,
            injection_succeeded,
            injection_error,
        };

        records.insert(0, record.clone());
        records.truncate(100);
        self.save_locked(&records)?;
        Ok(record)
    }

    pub fn list(&self, limit: usize) -> Vec<TranscriptionRecord> {
        self.records
            .lock()
            .map(|records| records.iter().take(limit).cloned().collect())
            .unwrap_or_default()
    }

    pub fn last(&self) -> Option<TranscriptionRecord> {
        self.records
            .lock()
            .ok()
            .and_then(|records| records.first().cloned())
    }

    pub fn get(&self, id: u64) -> Option<TranscriptionRecord> {
        self.records
            .lock()
            .ok()?
            .iter()
            .find(|record| record.id == id)
            .cloned()
    }

    pub fn clear(&self) -> Result<(), String> {
        let mut records = self
            .records
            .lock()
            .map_err(|_| "Failed to lock history".to_string())?;
        records.clear();
        self.save_locked(&records)
    }

    fn save_locked(&self, records: &[TranscriptionRecord]) -> Result<(), String> {
        let json = serde_json::to_string_pretty(records)
            .map_err(|e| format!("Failed to serialize history: {}", e))?;
        fs::write(&self.history_path, json)
            .map_err(|e| format!("Failed to save transcription history: {}", e))
    }
}
