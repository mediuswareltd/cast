use chrono::{DateTime, Utc};
use log::{info, warn};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct FileEntry {
    path: PathBuf,
    created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CleanupService {
    files: HashMap<String, FileEntry>,
}

impl CleanupService {
    pub fn new() -> Self {
        CleanupService {
            files: HashMap::new(),
        }
    }

    /// Register a file for cleanup tracking
    pub fn register_file(&mut self, path: PathBuf) {
        let file_id = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let entry = FileEntry {
            path: path.clone(),
            created_at: Utc::now(),
        };

        self.files.insert(file_id.clone(), entry);
        info!("Registered file for cleanup: {} ({})", file_id, path.display());
    }

    /// Remove a specific file from tracking
    pub fn unregister_file(&mut self, file_id: &str) {
        if let Some(entry) = self.files.remove(file_id) {
            info!("Unregistered file: {} ({})", file_id, entry.path.display());
        }
    }

    /// Clean up files older than the specified retention period (in minutes)
    pub fn cleanup_old_files(&mut self, retention_minutes: u64) {
        let now = Utc::now();
        let retention_duration = chrono::Duration::minutes(retention_minutes as i64);
        let mut files_to_remove = Vec::new();

        for (file_id, entry) in &self.files {
            let age = now - entry.created_at;

            if age > retention_duration {
                files_to_remove.push(file_id.clone());
            }
        }

        for file_id in files_to_remove {
            if let Some(entry) = self.files.remove(&file_id) {
                match fs::remove_file(&entry.path) {
                    Ok(_) => {
                        info!(
                            "Cleaned up old file: {} (age: {} minutes)",
                            entry.path.display(),
                            (now - entry.created_at).num_minutes()
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Failed to delete file {}: {}",
                            entry.path.display(),
                            e
                        );
                    }
                }
            }
        }

        if !self.files.is_empty() {
            info!("Active files being tracked: {}", self.files.len());
        }
    }

    /// Get the number of files currently being tracked
    pub fn tracked_files_count(&self) -> usize {
        self.files.len()
    }
}

impl Default for CleanupService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_register_file() {
        let mut service = CleanupService::new();
        let path = PathBuf::from("/tmp/test.mp4");
        service.register_file(path);
        assert_eq!(service.tracked_files_count(), 1);
    }

    #[test]
    fn test_unregister_file() {
        let mut service = CleanupService::new();
        let path = PathBuf::from("/tmp/test.mp4");
        service.register_file(path);
        service.unregister_file("test.mp4");
        assert_eq!(service.tracked_files_count(), 0);
    }
}
