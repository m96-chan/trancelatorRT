pub mod audio;
mod commands;
mod model;
mod pipeline;

use audio::capture::CaptureConfig;
use audio::AudioPipeline;
use commands::LanguageState;
use model::downloader::HttpDownloader;
use model::registry::ModelRegistry;
use model::storage::FileStorage;
use model::ModelManager;

use std::path::PathBuf;

fn create_model_manager() -> ModelManager<HttpDownloader, FileStorage> {
    // On Android: /data/data/com.m96chan.trancelatorrt/models/
    // On desktop: ~/.local/share/trancelatorrt/models/ (or current dir fallback)
    let models_dir = std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".local/share/trancelatorrt/models"))
        .unwrap_or_else(|_| PathBuf::from("models"));

    let registry = ModelRegistry::default();
    let downloader = HttpDownloader::new();
    let storage = FileStorage::new(models_dir);

    ModelManager::new(registry, downloader, storage)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (segment_tx, _segment_rx) = std::sync::mpsc::channel();
    let pipeline =
        AudioPipeline::new(CaptureConfig::default(), segment_tx).expect("Failed to create audio pipeline");

    let model_manager = create_model_manager();

    tauri::Builder::default()
        .manage(std::sync::Mutex::new(pipeline))
        .manage(std::sync::Mutex::new(LanguageState::new()))
        .manage(std::sync::Mutex::new(model_manager))
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::start_recording,
            commands::stop_recording,
            commands::pause_recording,
            commands::resume_recording,
            commands::get_audio_state,
            commands::get_languages,
            commands::get_language_settings,
            commands::set_source_language,
            commands::set_target_language,
            commands::get_model_list,
            commands::get_storage_info,
            commands::download_model,
            commands::delete_model,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
