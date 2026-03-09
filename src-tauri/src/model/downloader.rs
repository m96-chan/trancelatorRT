use super::{ModelError, ModelResult};
use std::io::Write;
use std::path::Path;

pub trait Downloader: Send {
    fn download(
        &self,
        url: &str,
        dest_path: &Path,
        progress_cb: &dyn Fn(u64, u64),
    ) -> ModelResult<()>;
}

pub struct HttpDownloader;

impl HttpDownloader {
    pub fn new() -> Self {
        Self
    }
}

impl Downloader for HttpDownloader {
    fn download(
        &self,
        url: &str,
        dest_path: &Path,
        progress_cb: &dyn Fn(u64, u64),
    ) -> ModelResult<()> {
        let response = reqwest::blocking::get(url)
            .map_err(|e| ModelError::DownloadFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ModelError::DownloadFailed(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let total = response.content_length().unwrap_or(0);

        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ModelError::Io(e.to_string()))?;
        }

        let mut file = std::fs::File::create(dest_path)
            .map_err(|e| ModelError::Io(e.to_string()))?;

        let bytes = response
            .bytes()
            .map_err(|e| ModelError::DownloadFailed(e.to_string()))?;

        let chunk_size = 8192;
        let mut downloaded: u64 = 0;
        for chunk in bytes.chunks(chunk_size) {
            file.write_all(chunk)
                .map_err(|e| ModelError::Io(e.to_string()))?;
            downloaded += chunk.len() as u64;
            progress_cb(downloaded, total);
        }

        Ok(())
    }
}

// --- Mock Downloader for tests ---

pub struct MockDownloader {
    should_succeed: bool,
    fake_data: Vec<u8>,
}

impl MockDownloader {
    pub fn success(data: Vec<u8>) -> Self {
        Self {
            should_succeed: true,
            fake_data: data,
        }
    }

    pub fn failure() -> Self {
        Self {
            should_succeed: false,
            fake_data: vec![],
        }
    }
}

impl Downloader for MockDownloader {
    fn download(
        &self,
        _url: &str,
        _dest_path: &Path,
        progress_cb: &dyn Fn(u64, u64),
    ) -> ModelResult<()> {
        if !self.should_succeed {
            return Err(ModelError::DownloadFailed("Mock download failure".into()));
        }

        let total = self.fake_data.len() as u64;
        // Simulate progress in 2 steps
        let half = total / 2;
        progress_cb(half, total);
        progress_cb(total, total);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_mock_downloader_success_calls_progress() {
        let downloader = MockDownloader::success(vec![0u8; 1000]);
        let progress_count = AtomicU32::new(0);

        let result = downloader.download(
            "https://example.com/model.bin",
            &PathBuf::from("/tmp/test.bin"),
            &|_downloaded, _total| {
                progress_count.fetch_add(1, Ordering::Relaxed);
            },
        );

        assert!(result.is_ok());
        assert!(progress_count.load(Ordering::Relaxed) >= 2);
    }

    #[test]
    fn test_mock_downloader_failure_returns_error() {
        let downloader = MockDownloader::failure();
        let result = downloader.download(
            "https://example.com/model.bin",
            &PathBuf::from("/tmp/test.bin"),
            &|_, _| {},
        );

        assert!(matches!(result, Err(ModelError::DownloadFailed(_))));
    }
}
