pub struct FileHasher {

}

impl FileHasher {
    pub fn new() -> Self {
        FileHasher {}
    }

    pub fn hash_file(&self, file_path: &str) -> Option<String> {
        use std::fs::File;
        use std::io::{BufReader, Read};
        use sha2::{Sha256, Digest};

        let file = File::open(file_path).ok()?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0; 1024];

        loop {
            let bytes_read = reader.read(&mut buffer).ok()?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Some(hex::encode(hasher.finalize()))
    }
}