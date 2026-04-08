use indicatif::{ProgressBar, ProgressStyle};

pub struct BackupEngine {
    
}

impl BackupEngine {
    pub fn new() -> Self {
        BackupEngine {}
    }

    pub fn backup(&self, source: &str, destination: &str) -> Option<String> {
        if source.is_empty() || destination.is_empty() {
            println!("Source and destination cannot be empty.");
            return None;
        }

        if source == destination {
            println!("Source and destination cannot be the same.");
            return None;
        }

        let source_metadata = match std::fs::metadata(source) {
            Ok(metadata) => metadata,
            Err(err) => {
                println!("Source path is invalid: {err}");
                return None;
            }
        };

        if let Ok(metadata) = std::fs::metadata(destination) {
            if !metadata.is_dir() {
                println!("Destination must be a directory.");
                return None;
            }
        }

        if let Err(err) = std::fs::create_dir_all(destination) {
            println!("Failed to create destination directory: {err}");
            return None;
        }

        let source_path = std::path::Path::new(source);

        if source_metadata.is_dir() {
            let source_folder_name = source_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let backup_folder_name = format!("{}_backup_{}", source_folder_name, timestamp);
            let destination_path = std::path::Path::new(destination).join(backup_folder_name);

            if let Err(err) = std::fs::create_dir_all(&destination_path) {
                println!("Failed to create backup directory: {err}");
                return None;
            }

            let total_files = self.count_files_recursive(source_path);
            let progress_bar = ProgressBar::new(total_files);
            progress_bar.set_style(
                ProgressStyle::with_template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("=>-"),
            );

            if let Err(err) =
                self.copy_dir_recursive(source_path, &destination_path, source_path, &progress_bar)
            {
                progress_bar.abandon_with_message(format!("Backup failed: {err}"));
                return None;
            }

            progress_bar.finish_with_message("Backup copy complete".to_string());
            Some(destination_path.to_string_lossy().to_string())
        } else {
            let source_file_name = source_path.file_name().unwrap().to_str().unwrap();
            let destination_path = std::path::Path::new(destination).join(source_file_name);

            if let Err(err) = std::fs::copy(source, &destination_path) {
                println!("Failed to backup file: {err}");
                return None;
            }

            Some(destination_path.to_string_lossy().to_string())
        }
    }

    pub fn replace_file(&self, source: &str, destination: &str) {
        if source.is_empty() || destination.is_empty() {
            println!("Source and destination cannot be empty.");
            return;
        }

        match std::fs::metadata(source) {
            Ok(metadata) if metadata.is_file() => {}
            Ok(_) => {
                println!("Source must be a file.");
                return;
            }
            Err(err) => {
                println!("Source path is invalid: {err}");
                return;
            }
        }

        if let Ok(metadata) = std::fs::metadata(destination) {
            if metadata.is_dir() {
                println!("Destination must be a file, not a directory.");
                return;
            }
        }

        if let Err(err) = std::fs::copy(source, destination) {
            println!("Failed to replace file: {err}");
        }
    }

    fn count_files_recursive(&self, source: &std::path::Path) -> u64 {
        walkdir::WalkDir::new(source)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_file())
            .count() as u64
    }

    fn copy_dir_recursive(
        &self,
        source: &std::path::Path,
        destination: &std::path::Path,
        root_source: &std::path::Path,
        progress_bar: &ProgressBar,
    ) -> std::io::Result<()> {
        for entry_result in std::fs::read_dir(source)? {
            let entry = entry_result?;
            let path = entry.path();
            let file_name = entry.file_name();
            let dest_path = destination.join(file_name);

            if path.is_file() {
                let display_path = path
                    .strip_prefix(root_source)
                    .unwrap_or(path.as_path())
                    .to_string_lossy()
                    .to_string();
                progress_bar.set_message(format!("Copying {display_path}"));
                std::fs::copy(&path, &dest_path)?;
                progress_bar.inc(1);
            } else if path.is_dir() {
                std::fs::create_dir_all(&dest_path)?;
                self.copy_dir_recursive(&path, &dest_path, root_source, progress_bar)?;
            }
        }

        Ok(())
    }
}