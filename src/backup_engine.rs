pub struct BackupEngine {
    
}

impl BackupEngine {
    pub fn new() -> Self {
        BackupEngine {}
    }

    pub fn backup(&self, source: &str, destination: &str) {
        if source.is_empty() || destination.is_empty() {
            println!("Source and destination cannot be empty.");
            return;
        }

        if source == destination {
            println!("Source and destination cannot be the same.");
            return;
        }

        match std::fs::metadata(source) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => {
                println!("Source must be a directory.");
                return;
            }
            Err(err) => {
                println!("Source path is invalid: {err}");
                return;
            }
        }

        if let Ok(metadata) = std::fs::metadata(destination) {
            if !metadata.is_dir() {
                println!("Destination must be a directory.");
                return;
            }
        }

        if let Err(err) = std::fs::create_dir_all(destination) {
            println!("Failed to create destination directory: {err}");
            return;
        }

        let source_folder_name = std::path::Path::new(source)
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
            return;
        }

        self.copy_dir_recursive(source, destination_path.to_str().unwrap());
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

    fn copy_dir_recursive(&self, source: &str, destination: &str) {
        for entry in std::fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let file_name = entry.file_name();
            let dest_path = std::path::Path::new(destination).join(file_name);

            if path.is_file() {
                std::fs::copy(&path, &dest_path).unwrap();
            } else if path.is_dir() {
                if let Err(err) = std::fs::create_dir_all(&dest_path) {
                    println!("Failed to create directory: {err}");
                    continue;
                }
                self.copy_dir_recursive(path.to_str().unwrap(), dest_path.to_str().unwrap());
            }
        }
    }
}