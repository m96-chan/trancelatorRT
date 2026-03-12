use crate::audio::state::PipelineState;
use crate::audio::AudioPipeline;
use crate::model::downloader::{Downloader, HttpDownloader};
use crate::model::storage::FileStorage;
use crate::model::{ModelManager, ModelStatusInfo, StorageInfo};
use crate::pipeline::stt::Language;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};

#[derive(Clone, serde::Serialize)]
pub struct TranscriptionResult {
    pub recognized: String,
    pub translated: String,
}

#[derive(Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub id: String,
    pub downloaded: u64,
    pub total: u64,
}

#[derive(Clone, serde::Serialize)]
pub struct DownloadComplete {
    pub id: String,
    pub success: bool,
    pub error: Option<String>,
}

pub type AppModelManager = Arc<Mutex<ModelManager<HttpDownloader, FileStorage>>>;

#[derive(Debug, serde::Serialize)]
pub struct LanguageInfo {
    pub code: String,
    pub name: String,
}

#[derive(Debug, serde::Serialize)]
pub struct LanguageSettings {
    pub source: String,
    pub target: String,
}

pub struct LanguageState {
    pub source: Language,
    pub target: Language,
}

impl LanguageState {
    pub fn new() -> Self {
        Self {
            source: Language::English,
            target: Language::Japanese,
        }
    }
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to trancelatorRT.", name)
}

#[tauri::command]
pub fn start_recording(pipeline: State<'_, Mutex<AudioPipeline>>) -> Result<(), String> {
    pipeline
        .lock()
        .map_err(|e| e.to_string())?
        .start_recording()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_recording(pipeline: State<'_, Mutex<AudioPipeline>>) -> Result<(), String> {
    pipeline
        .lock()
        .map_err(|e| e.to_string())?
        .stop_recording()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pause_recording(pipeline: State<'_, Mutex<AudioPipeline>>) -> Result<(), String> {
    pipeline
        .lock()
        .map_err(|e| e.to_string())?
        .pause()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resume_recording(pipeline: State<'_, Mutex<AudioPipeline>>) -> Result<(), String> {
    pipeline
        .lock()
        .map_err(|e| e.to_string())?
        .resume()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_audio_state(pipeline: State<'_, Mutex<AudioPipeline>>) -> Result<PipelineState, String> {
    Ok(pipeline.lock().map_err(|e| e.to_string())?.state())
}

#[tauri::command]
pub fn get_languages() -> Vec<LanguageInfo> {
    Language::all()
        .iter()
        .map(|lang| LanguageInfo {
            code: lang.whisper_code().to_string(),
            name: format!("{:?}", lang),
        })
        .collect()
}

#[tauri::command]
pub fn get_language_settings(
    lang_state: State<'_, Arc<Mutex<LanguageState>>>,
) -> Result<LanguageSettings, String> {
    let state = lang_state.lock().map_err(|e| e.to_string())?;
    Ok(LanguageSettings {
        source: state.source.whisper_code().to_string(),
        target: state.target.whisper_code().to_string(),
    })
}

#[tauri::command]
pub fn set_source_language(
    code: &str,
    lang_state: State<'_, Arc<Mutex<LanguageState>>>,
) -> Result<(), String> {
    let lang = Language::from_whisper_code(code)
        .ok_or_else(|| format!("Unknown language code: {}", code))?;
    lang_state.lock().map_err(|e| e.to_string())?.source = lang;
    Ok(())
}

#[tauri::command]
pub fn set_target_language(
    code: &str,
    lang_state: State<'_, Arc<Mutex<LanguageState>>>,
) -> Result<(), String> {
    let lang = Language::from_whisper_code(code)
        .ok_or_else(|| format!("Unknown language code: {}", code))?;
    lang_state.lock().map_err(|e| e.to_string())?.target = lang;
    Ok(())
}

// --- Model management commands ---

#[tauri::command]
pub fn get_model_list(
    manager: State<'_, AppModelManager>,
) -> Result<Vec<ModelStatusInfo>, String> {
    Ok(manager.lock().map_err(|e| e.to_string())?.list_models())
}

#[tauri::command]
pub fn get_storage_info(
    manager: State<'_, AppModelManager>,
) -> Result<StorageInfo, String> {
    manager
        .lock()
        .map_err(|e| e.to_string())?
        .storage_info()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn download_model(
    id: String,
    manager: State<'_, AppModelManager>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // Briefly lock to prepare download (validate, set status, get URL)
    let (url, dest) = manager
        .lock()
        .map_err(|e| e.to_string())?
        .prepare_download(&id)
        .map_err(|e| e.to_string())?;

    // Clone Arc for the background thread
    let manager_arc = Arc::clone(&manager);
    let app_handle = app.clone();
    let model_id = id.clone();

    // Download in background thread — mutex is NOT held during download
    std::thread::spawn(move || {
        let downloader = HttpDownloader::new();
        let result = downloader.download(&url, &dest, &|downloaded, total| {
            let _ = app_handle.emit(
                "download-progress",
                DownloadProgress {
                    id: model_id.clone(),
                    downloaded,
                    total,
                },
            );
        });

        // Briefly lock to update status
        let success;
        let error;
        if let Ok(mut mgr) = manager_arc.lock() {
            match &result {
                Ok(()) => {
                    mgr.finish_download(&id);
                    success = true;
                    error = None;
                }
                Err(e) => {
                    mgr.fail_download(&id);
                    success = false;
                    error = Some(e.to_string());
                }
            }
        } else {
            success = result.is_ok();
            error = result.err().map(|e| e.to_string());
        }

        let _ = app_handle.emit(
            "download-complete",
            DownloadComplete {
                id: id.clone(),
                success,
                error,
            },
        );
    });

    Ok(())
}

#[tauri::command]
pub fn delete_model(
    id: String,
    manager: State<'_, AppModelManager>,
) -> Result<(), String> {
    manager
        .lock()
        .map_err(|e| e.to_string())?
        .delete_model(&id)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        let result = greet("World");
        assert_eq!(result, "Hello, World! Welcome to trancelatorRT.");
    }

    #[test]
    fn test_get_languages_returns_all() {
        let langs = get_languages();
        assert_eq!(langs.len(), 8);
        assert!(langs.iter().any(|l| l.code == "en"));
        assert!(langs.iter().any(|l| l.code == "ja"));
    }

    #[test]
    fn test_language_state_defaults() {
        let state = LanguageState::new();
        assert_eq!(state.source, Language::English);
        assert_eq!(state.target, Language::Japanese);
    }
}
