/// Model management: registry, download, storage, and lifecycle

pub mod downloader;
pub mod registry;
pub mod storage;

use downloader::Downloader;
use registry::ModelRegistry;
use storage::Storage;
use thiserror::Error;

use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ModelType {
    Whisper,
    Nllb,
    Piper,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ModelStatus {
    NotDownloaded,
    Downloading { progress_percent: u8 },
    Downloaded,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelInfo {
    pub model_type: ModelType,
    pub id: String,
    pub display_name: String,
    pub version: String,
    pub url: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub filename: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelStatusInfo {
    pub info: ModelInfo,
    pub status: ModelStatus,
    pub local_path: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageInfo {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub models_bytes: u64,
}

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Model not found: {0}")]
    NotFound(String),
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
    #[error("Insufficient storage: need {needed} bytes, have {available} bytes")]
    InsufficientStorage { needed: u64, available: u64 },
    #[error("IO error: {0}")]
    Io(String),
    #[error("Model already downloading: {0}")]
    AlreadyDownloading(String),
}

pub type ModelResult<T> = Result<T, ModelError>;

pub struct ModelManager<D: Downloader, S: Storage> {
    registry: ModelRegistry,
    downloader: D,
    storage: S,
    statuses: HashMap<String, ModelStatus>,
}

impl<D: Downloader, S: Storage> ModelManager<D, S> {
    pub fn new(registry: ModelRegistry, downloader: D, storage: S) -> Self {
        let mut statuses = HashMap::new();

        // Initialize statuses based on what's already on disk
        for model in registry.list() {
            let status = if storage.exists(&model.filename) {
                ModelStatus::Downloaded
            } else {
                ModelStatus::NotDownloaded
            };
            statuses.insert(model.id.clone(), status);
        }

        Self {
            registry,
            downloader,
            storage,
            statuses,
        }
    }

    pub fn list_models(&self) -> Vec<ModelStatusInfo> {
        self.registry
            .list()
            .iter()
            .map(|info| {
                let status = self
                    .statuses
                    .get(&info.id)
                    .cloned()
                    .unwrap_or(ModelStatus::NotDownloaded);
                let local_path = if status == ModelStatus::Downloaded {
                    Some(self.storage.model_path(&info.filename).to_string_lossy().to_string())
                } else {
                    None
                };
                ModelStatusInfo {
                    info: info.clone(),
                    status,
                    local_path,
                }
            })
            .collect()
    }

    pub fn get_model_status(&self, id: &str) -> ModelResult<ModelStatusInfo> {
        let info = self
            .registry
            .get(id)
            .ok_or_else(|| ModelError::NotFound(id.to_string()))?;
        let status = self
            .statuses
            .get(id)
            .cloned()
            .unwrap_or(ModelStatus::NotDownloaded);
        let local_path = if status == ModelStatus::Downloaded {
            Some(self.storage.model_path(&info.filename).to_string_lossy().to_string())
        } else {
            None
        };
        Ok(ModelStatusInfo {
            info: info.clone(),
            status,
            local_path,
        })
    }

    pub fn download_model(
        &mut self,
        id: &str,
        progress_cb: &dyn Fn(u64, u64),
    ) -> ModelResult<PathBuf> {
        let info = self
            .registry
            .get(id)
            .ok_or_else(|| ModelError::NotFound(id.to_string()))?
            .clone();

        // Check if already downloading
        if let Some(ModelStatus::Downloading { .. }) = self.statuses.get(id) {
            return Err(ModelError::AlreadyDownloading(id.to_string()));
        }

        // Check storage capacity
        let available = self.storage.available_space()?;
        if available < info.size_bytes {
            return Err(ModelError::InsufficientStorage {
                needed: info.size_bytes,
                available,
            });
        }

        self.storage.ensure_dir()?;

        // Set downloading status
        self.statuses
            .insert(id.to_string(), ModelStatus::Downloading { progress_percent: 0 });

        let dest = self.storage.model_path(&info.filename);

        // Download
        match self.downloader.download(&info.url, &dest, progress_cb) {
            Ok(()) => {
                self.statuses
                    .insert(id.to_string(), ModelStatus::Downloaded);
                Ok(dest)
            }
            Err(e) => {
                self.statuses
                    .insert(id.to_string(), ModelStatus::NotDownloaded);
                // Clean up partial file
                let _ = self.storage.delete(&info.filename);
                Err(e)
            }
        }
    }

    pub fn delete_model(&mut self, id: &str) -> ModelResult<()> {
        let info = self
            .registry
            .get(id)
            .ok_or_else(|| ModelError::NotFound(id.to_string()))?;

        self.storage.delete(&info.filename)?;
        self.statuses
            .insert(id.to_string(), ModelStatus::NotDownloaded);
        Ok(())
    }

    pub fn model_path(&self, id: &str) -> ModelResult<Option<PathBuf>> {
        let info = self
            .registry
            .get(id)
            .ok_or_else(|| ModelError::NotFound(id.to_string()))?;
        let status = self.statuses.get(id);
        if matches!(status, Some(ModelStatus::Downloaded)) {
            Ok(Some(self.storage.model_path(&info.filename)))
        } else {
            Ok(None)
        }
    }

    pub fn is_downloaded(&self, id: &str) -> ModelResult<bool> {
        let _ = self
            .registry
            .get(id)
            .ok_or_else(|| ModelError::NotFound(id.to_string()))?;
        Ok(matches!(
            self.statuses.get(id),
            Some(ModelStatus::Downloaded)
        ))
    }

    pub fn storage_info(&self) -> ModelResult<StorageInfo> {
        let total_bytes = self.storage.total_space()?;
        let available_bytes = self.storage.available_space()?;
        let models_bytes: u64 = self
            .registry
            .list()
            .iter()
            .filter(|m| self.storage.exists(&m.filename))
            .map(|m| self.storage.file_size(&m.filename))
            .sum();

        Ok(StorageInfo {
            total_bytes,
            available_bytes,
            models_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use downloader::MockDownloader;
    use storage::MockStorage;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn test_manager(
        downloader: MockDownloader,
        storage: MockStorage,
    ) -> ModelManager<MockDownloader, MockStorage> {
        ModelManager::new(ModelRegistry::default(), downloader, storage)
    }

    #[test]
    fn test_manager_list_models_all_not_downloaded() {
        let manager = test_manager(MockDownloader::success(vec![]), MockStorage::new());
        let models = manager.list_models();
        assert!(!models.is_empty());
        assert!(models
            .iter()
            .all(|m| m.status == ModelStatus::NotDownloaded));
        assert!(models.iter().all(|m| m.local_path.is_none()));
    }

    #[test]
    fn test_manager_download_model_success() {
        let progress_count = AtomicU32::new(0);
        let mut manager = test_manager(
            MockDownloader::success(vec![0u8; 100]),
            MockStorage::new(),
        );

        let result = manager.download_model("whisper-tiny", &|_downloaded, _total| {
            progress_count.fetch_add(1, Ordering::Relaxed);
        });

        assert!(result.is_ok());
        assert!(progress_count.load(Ordering::Relaxed) >= 1);
        assert!(manager.is_downloaded("whisper-tiny").unwrap());
        assert!(manager.model_path("whisper-tiny").unwrap().is_some());
    }

    #[test]
    fn test_manager_download_insufficient_storage() {
        let mut manager = test_manager(
            MockDownloader::success(vec![]),
            MockStorage::new().with_available_space(1),
        );

        let result = manager.download_model("whisper-tiny", &|_, _| {});
        assert!(matches!(
            result,
            Err(ModelError::InsufficientStorage { .. })
        ));
    }

    #[test]
    fn test_manager_download_failure_reverts_status() {
        let mut manager = test_manager(MockDownloader::failure(), MockStorage::new());

        let result = manager.download_model("whisper-tiny", &|_, _| {});
        assert!(matches!(result, Err(ModelError::DownloadFailed(_))));
        assert!(!manager.is_downloaded("whisper-tiny").unwrap());
    }

    #[test]
    fn test_manager_delete_model() {
        let mut manager = test_manager(
            MockDownloader::success(vec![0u8; 100]),
            MockStorage::new(),
        );

        manager.download_model("whisper-tiny", &|_, _| {}).unwrap();
        assert!(manager.is_downloaded("whisper-tiny").unwrap());

        manager.delete_model("whisper-tiny").unwrap();
        assert!(!manager.is_downloaded("whisper-tiny").unwrap());
        assert!(manager.model_path("whisper-tiny").unwrap().is_none());
    }

    #[test]
    fn test_manager_get_model_status_not_found() {
        let manager = test_manager(MockDownloader::success(vec![]), MockStorage::new());
        assert!(matches!(
            manager.get_model_status("nonexistent"),
            Err(ModelError::NotFound(_))
        ));
    }

    #[test]
    fn test_manager_storage_info() {
        let manager = test_manager(MockDownloader::success(vec![]), MockStorage::new());
        let info = manager.storage_info().unwrap();
        assert!(info.total_bytes > 0);
        assert!(info.available_bytes > 0);
        assert_eq!(info.models_bytes, 0);
    }

    #[test]
    fn test_manager_already_downloaded_on_disk() {
        let mut storage = MockStorage::new();
        storage.add_file("ggml-tiny.bin", vec![0u8; 100]);
        let manager = ModelManager::new(
            ModelRegistry::default(),
            MockDownloader::success(vec![]),
            storage,
        );
        assert!(manager.is_downloaded("whisper-tiny").unwrap());
    }
}
