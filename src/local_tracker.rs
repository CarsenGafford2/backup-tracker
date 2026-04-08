use serde_json;
use crate::file_hasher;
use std::path::Path;

pub struct LocalTracker {
    json_file: String,
}

impl LocalTracker {
    pub fn new() -> Self {
        LocalTracker {
            json_file: Self::get_json_path().to_str().unwrap().to_string(),
        }
    }

    fn get_json_path() -> std::path::PathBuf {
        let home_dir: std::path::PathBuf = dirs::home_dir().expect("Could not find home directory");
        let app_dir: std::path::PathBuf = home_dir.join(".backup_tracker");
        if !app_dir.exists() {
            std::fs::create_dir_all(&app_dir).expect("Failed to create application directory");
        }
        app_dir.join("backup_tracker_data.json")
    }

    pub fn track_folder(&self, folder_path: &str) {
        let folder = std::path::Path::new(folder_path);
        if folder.exists() && folder.is_dir() {
            for entry in walkdir::WalkDir::new(folder) {
                if let Ok(entry) = entry {
                    if entry.path().is_file() {
                        self.track_file(entry.path().to_str().unwrap());
                    }
                }
            }
        } else {
            println!("Invalid folder path: {}", folder_path);
            return;
        }
    }

    fn normalize_location(path: &Path, fallback: &str) -> String {
        let location = path
            .canonicalize()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| fallback.to_string());

        Self::strip_windows_extended_prefix(&location)
    }

    fn strip_windows_extended_prefix(path: &str) -> String {
        if let Some(trimmed) = path.strip_prefix("\\\\?\\UNC\\") {
            return format!("\\\\{}", trimmed);
        }

        if let Some(trimmed) = path.strip_prefix("\\\\?\\") {
            return trimmed.to_string();
        }

        path.to_string()
    }

    fn normalize_locations_in_entries(entries: &mut [serde_json::Value]) {
        for entry in entries.iter_mut() {
            if let Some(location) = entry.get("location").and_then(|value| value.as_str()) {
                let normalized = Self::strip_windows_extended_prefix(location);
                if normalized != location {
                    entry["location"] = serde_json::Value::String(normalized);
                }
            }
        }
    }

    pub fn track_file(&self, file_path: &str) {
        let file = std::path::Path::new(file_path);
        if file.exists() && file.is_file() {
            let name = file.file_name().unwrap().to_str().unwrap();
            let hash = file_hasher::FileHasher::new().hash_file(file_path);
            let location = Self::normalize_location(file, file_path);

            let file_info = serde_json::json!({
                "name": name,
                "hash": hash,
                "location": location
            });

            let mut tracked_files: Vec<serde_json::Value> = if std::path::Path::new(&self.json_file).exists() {
                match std::fs::read_to_string(&self.json_file) {
                    Ok(content) => {
                        let content = content.trim();
                        if content.is_empty() {
                            Vec::new()
                        } else {
                            match serde_json::from_str::<serde_json::Value>(content) {
                                Ok(serde_json::Value::Array(arr)) => arr,
                                Ok(_) => Vec::new(),
                                Err(_) => Vec::new(),
                            }
                        }
                    }
                    Err(_) => Vec::new(),
                }
            } else {
                Vec::new()
            };

            if !tracked_files.is_empty() && !tracked_files[0].is_object() {
                tracked_files = Vec::new();
            }

            Self::normalize_locations_in_entries(&mut tracked_files);

            for tracked_file in &tracked_files {
                if let Some(tracked_location) = tracked_file
                    .get("location")
                    .and_then(|value| value.as_str())
                {
                    let normalized_tracked_location = Self::strip_windows_extended_prefix(tracked_location);
                    if normalized_tracked_location == location {
                        println!("File is already being tracked.");
                        return;
                    }
                }
            }

            if tracked_files
                .last()
                .map(|value| value.is_object())
                .unwrap_or(false)
            {
                tracked_files.push(file_info);
            } else {
                tracked_files = vec![file_info];
            }

            match serde_json::to_string_pretty(&tracked_files) {
                Ok(serialized) => {
                    if let Err(err) = std::fs::write(&self.json_file, serialized) {
                        eprintln!("Failed to write to JSON file: {}", err);
                    }
                }
                Err(err) => eprintln!("Failed to serialize JSON: {}", err),
            }

        } else {
            println!("Invalid file path: {}", file_path);
            return;
        }
    }

    pub fn untrack_file(&self, file_path: &str) {
        let file = std::path::Path::new(file_path);
        if file.exists() && file.is_file() {
            let location = Self::normalize_location(file, file_path);

            let mut tracked_files: Vec<serde_json::Value> = if std::path::Path::new(&self.json_file).exists() {
                match std::fs::read_to_string(&self.json_file) {
                    Ok(content) => {
                        let content = content.trim();
                        if content.is_empty() {
                            Vec::new()
                        } else {
                            match serde_json::from_str::<serde_json::Value>(content) {
                                Ok(serde_json::Value::Array(arr)) => arr,
                                Ok(_) => Vec::new(),
                                Err(_) => Vec::new(),
                            }
                        }
                    }
                    Err(_) => Vec::new(),
                }
            } else {
                Vec::new()
            };

            Self::normalize_locations_in_entries(&mut tracked_files);

            let updated_files: Vec<serde_json::Value> = tracked_files
                .into_iter()
                .filter(|tracked_file| {
                    tracked_file
                        .get("location")
                        .and_then(|value| value.as_str())
                        .map_or(true, |tracked_location| {
                            let normalized_tracked_location = Self::strip_windows_extended_prefix(tracked_location);
                            normalized_tracked_location != location
                        })
                })
                .collect();

            match serde_json::to_string_pretty(&updated_files) {
                Ok(serialized) => {
                    if let Err(err) = std::fs::write(&self.json_file, serialized) {
                        eprintln!("Failed to write to JSON file: {}", err);
                    }
                }
                Err(err) => eprintln!("Failed to serialize JSON: {}", err),
            }

        } else {
            println!("Invalid file path: {}", file_path);
            return;
        }
    }

    pub fn update_file(&self, file_path: &str) {
        let file = std::path::Path::new(file_path);
        if file.exists() && file.is_file() {
            self.untrack_file(file_path);
            self.track_file(file_path);
        } else {
            println!("Invalid file path: {}", file_path);
            return;
        }
    }
}