use super::{ModelError, ModelResult};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Returns (total_bytes, available_bytes) for the filesystem containing `path`.
fn get_fs_space(path: &Path) -> ModelResult<(u64, u64)> {
    use std::ffi::CString;

    // Use parent or fallback to "/" if path doesn't exist yet
    let check_path = if path.exists() {
        path.to_path_buf()
    } else if let Some(parent) = path.parent() {
        if parent.exists() {
            parent.to_path_buf()
        } else {
            PathBuf::from("/")
        }
    } else {
        PathBuf::from("/")
    };

    let c_path = CString::new(check_path.to_string_lossy().as_bytes())
        .map_err(|e| ModelError::Io(e.to_string()))?;

    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c_path.as_ptr(), &mut stat) != 0 {
            return Err(ModelError::Io(format!(
                "statvfs failed: {}",
                std::io::Error::last_os_error()
            )));
        }
        let total = stat.f_blocks as u64 * stat.f_frsize as u64;
        let available = stat.f_bavail as u64 * stat.f_frsize as u64;
        Ok((total, available))
    }
}

pub trait Storage: Send {
    fn models_dir(&self) -> &Path;
    fn model_path(&self, filename: &str) -> PathBuf;
    fn exists(&self, filename: &str) -> bool;
    fn delete(&self, filename: &str) -> ModelResult<()>;
    fn available_space(&self) -> ModelResult<u64>;
    fn total_space(&self) -> ModelResult<u64>;
    fn file_size(&self, filename: &str) -> u64;
    fn verify_checksum(&self, filename: &str, expected_sha256: &str) -> ModelResult<bool>;
    fn ensure_dir(&self) -> ModelResult<()>;
}

pub struct FileStorage {
    base_dir: PathBuf,
}

impl FileStorage {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }
}

impl Storage for FileStorage {
    fn models_dir(&self) -> &Path {
        &self.base_dir
    }

    fn model_path(&self, filename: &str) -> PathBuf {
        self.base_dir.join(filename)
    }

    fn exists(&self, filename: &str) -> bool {
        self.model_path(filename).exists()
    }

    fn delete(&self, filename: &str) -> ModelResult<()> {
        let path = self.model_path(filename);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| ModelError::Io(e.to_string()))?;
        }
        Ok(())
    }

    fn available_space(&self) -> ModelResult<u64> {
        get_fs_space(&self.base_dir).map(|(_, available)| available)
    }

    fn total_space(&self) -> ModelResult<u64> {
        get_fs_space(&self.base_dir).map(|(total, _)| total)
    }

    fn file_size(&self, filename: &str) -> u64 {
        std::fs::metadata(self.model_path(filename))
            .map(|m| m.len())
            .unwrap_or(0)
    }

    fn verify_checksum(&self, filename: &str, expected_sha256: &str) -> ModelResult<bool> {
        let path = self.model_path(filename);
        let mut file =
            std::fs::File::open(&path).map_err(|e| ModelError::Io(e.to_string()))?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];
        loop {
            let n = file.read(&mut buffer).map_err(|e| ModelError::Io(e.to_string()))?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        let hash = format!("{:x}", hasher.finalize());
        Ok(hash == expected_sha256)
    }

    fn ensure_dir(&self) -> ModelResult<()> {
        std::fs::create_dir_all(&self.base_dir).map_err(|e| ModelError::Io(e.to_string()))
    }
}

// --- Mock Storage for tests ---

pub struct MockStorage {
    base_dir: PathBuf,
    files: HashMap<String, Vec<u8>>,
    available: u64,
    total: u64,
}

impl MockStorage {
    pub fn new() -> Self {
        Self {
            base_dir: PathBuf::from("/mock/models"),
            files: HashMap::new(),
            available: 10_000_000_000,
            total: 64_000_000_000,
        }
    }

    pub fn with_available_space(mut self, bytes: u64) -> Self {
        self.available = bytes;
        self
    }

    pub fn add_file(&mut self, filename: &str, data: Vec<u8>) {
        self.files.insert(filename.to_string(), data);
    }
}

impl Storage for MockStorage {
    fn models_dir(&self) -> &Path {
        &self.base_dir
    }

    fn model_path(&self, filename: &str) -> PathBuf {
        self.base_dir.join(filename)
    }

    fn exists(&self, filename: &str) -> bool {
        self.files.contains_key(filename)
    }

    fn delete(&self, _filename: &str) -> ModelResult<()> {
        // Note: can't mutate through &self, but for test verification we return Ok
        Ok(())
    }

    fn available_space(&self) -> ModelResult<u64> {
        Ok(self.available)
    }

    fn total_space(&self) -> ModelResult<u64> {
        Ok(self.total)
    }

    fn file_size(&self, filename: &str) -> u64 {
        self.files.get(filename).map(|f| f.len() as u64).unwrap_or(0)
    }

    fn verify_checksum(&self, filename: &str, expected_sha256: &str) -> ModelResult<bool> {
        match self.files.get(filename) {
            Some(data) => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                let hash = format!("{:x}", hasher.finalize());
                Ok(hash == expected_sha256)
            }
            None => Err(ModelError::Io("File not found".into())),
        }
    }

    fn ensure_dir(&self) -> ModelResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_storage_exists_false_initially() {
        let storage = MockStorage::new();
        assert!(!storage.exists("any_file.bin"));
    }

    #[test]
    fn test_mock_storage_model_path_construction() {
        let storage = MockStorage::new();
        let path = storage.model_path("ggml-tiny.bin");
        assert_eq!(path, PathBuf::from("/mock/models/ggml-tiny.bin"));
    }

    #[test]
    fn test_mock_storage_available_space() {
        let storage = MockStorage::new().with_available_space(2_000_000_000);
        assert_eq!(storage.available_space().unwrap(), 2_000_000_000);
    }

    #[test]
    fn test_mock_storage_delete_returns_ok() {
        let mut storage = MockStorage::new();
        storage.add_file("test.bin", vec![1, 2, 3]);
        assert!(storage.delete("test.bin").is_ok());
    }

    #[test]
    fn test_mock_storage_verify_checksum_match() {
        let mut storage = MockStorage::new();
        let data = b"hello world".to_vec();
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let expected = format!("{:x}", hasher.finalize());

        storage.add_file("test.bin", data);
        assert!(storage.verify_checksum("test.bin", &expected).unwrap());
    }

    #[test]
    fn test_mock_storage_verify_checksum_mismatch() {
        let mut storage = MockStorage::new();
        storage.add_file("test.bin", b"hello world".to_vec());
        assert!(!storage.verify_checksum("test.bin", "wrong-hash").unwrap());
    }

    #[test]
    fn test_mock_storage_file_size() {
        let mut storage = MockStorage::new();
        storage.add_file("test.bin", vec![0u8; 1024]);
        assert_eq!(storage.file_size("test.bin"), 1024);
        assert_eq!(storage.file_size("nonexistent.bin"), 0);
    }

    #[test]
    fn test_mock_storage_exists_after_add() {
        let mut storage = MockStorage::new();
        storage.add_file("model.bin", vec![1]);
        assert!(storage.exists("model.bin"));
    }
}
