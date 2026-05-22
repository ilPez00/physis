use crate::ai::AiResult;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub text: String,
    pub kind: String,
    pub mode: String,
    pub timestamp: i64,
    pub extra: HashMap<String, String>,
}

pub struct EpisodicMemory {
    db: Db,
}

impl EpisodicMemory {
    pub fn new(db: Db) -> Self {
        log::info!("episodic memory initialized");
        Self { db }
    }

    pub fn store(&self, text: &str, kind: &str, mode: &str, extra: HashMap<String, String>) -> AiResult<String> {
        let id = Uuid::new_v4().to_string();
        let entry = MemoryEntry {
            id: id.clone(),
            text: text.chars().take(2000).collect(),
            kind: kind.to_string(),
            mode: mode.to_string(),
            timestamp: Utc::now().timestamp(),
            extra,
        };
        let key = format!("mem:{id}");
        let value = serde_json::to_vec(&entry)?;
        self.db.insert(key.as_bytes(), value)?;
        self.db.flush()?;
        Ok(id)
    }

    pub fn query(&self, limit: usize) -> AiResult<Vec<MemoryEntry>> {
        let mut results = Vec::new();
        for item in self.db.scan_prefix("mem:".as_bytes()).rev() {
            let (_, value) = item?;
            if let Ok(entry) = serde_json::from_slice::<MemoryEntry>(&value) {
                results.push(entry);
                if results.len() >= limit {
                    break;
                }
            }
        }
        Ok(results)
    }

    pub fn query_by_kind(&self, kind: &str, limit: usize) -> AiResult<Vec<MemoryEntry>> {
        let mut results = Vec::new();
        for item in self.db.scan_prefix("mem:".as_bytes()).rev() {
            let (_, value) = item?;
            if let Ok(entry) = serde_json::from_slice::<MemoryEntry>(&value) {
                if entry.kind == kind {
                    results.push(entry);
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }
        Ok(results)
    }

    pub fn search(&self, text: &str, limit: usize) -> AiResult<Vec<MemoryEntry>> {
        let query = text.to_lowercase();
        let mut results = Vec::new();
        for item in self.db.scan_prefix("mem:".as_bytes()).rev() {
            let (_, value) = item?;
            if let Ok(entry) = serde_json::from_slice::<MemoryEntry>(&value) {
                if entry.text.to_lowercase().contains(&query) {
                    results.push(entry);
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }
        Ok(results)
    }

    pub fn count(&self) -> AiResult<u64> {
        let mut count = 0u64;
        for item in self.db.scan_prefix("mem:".as_bytes()) {
            let _ = item?;
            count += 1;
        }
        Ok(count)
    }

    pub fn delete_before(&self, days: i64) -> AiResult<u64> {
        let cutoff = Utc::now().timestamp() - (days * 86400);
        let mut deleted = 0u64;
        let to_remove: Vec<Vec<u8>> = self
            .db
            .scan_prefix("mem:".as_bytes())
            .filter_map(|item| {
                item.ok().and_then(|(key, value)| {
                    serde_json::from_slice::<MemoryEntry>(&value)
                        .ok()
                        .filter(|e| e.timestamp < cutoff)
                        .map(|_| key.to_vec())
                })
            })
            .collect();

        for key in &to_remove {
            self.db.remove(key)?;
            deleted += 1;
        }
        self.db.flush()?;
        Ok(deleted)
    }
}
