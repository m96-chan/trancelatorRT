pub mod audio;
mod commands;
mod model;
mod pipeline;

use audio::capture::CaptureConfig;
use audio::AudioPipeline;
use commands::{LanguageState, TranscriptionResult};
use model::downloader::HttpDownloader;
use model::registry::ModelRegistry;
use model::storage::FileStorage;
use model::ModelManager;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (segment_tx, segment_rx) = std::sync::mpsc::channel();
    let pipeline =
        AudioPipeline::new(CaptureConfig::default(), segment_tx).expect("Failed to create audio pipeline");

    tauri::Builder::default()
        .manage(std::sync::Mutex::new(pipeline))
        .manage(std::sync::Mutex::new(LanguageState::new()))
        .setup(|app| {
            let models_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir")
                .join("models");

            let registry = ModelRegistry::default();
            let downloader = HttpDownloader::new();
            let storage = FileStorage::new(models_dir);
            let model_manager = ModelManager::new(registry, downloader, storage);

            app.manage(std::sync::Arc::new(std::sync::Mutex::new(model_manager)));

            // Spawn segment processing thread
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut segment_count = 0u32;
                while let Ok(segment) = segment_rx.recv() {
                    segment_count += 1;
                    let duration_ms = segment.len() as f64 / 16.0; // ~16 samples/ms at 16kHz

                    // Stub STT: report that speech was detected with duration
                    let recognized = format!(
                        "[Speech segment #{} detected: {:.0}ms, {} samples]",
                        segment_count,
                        duration_ms,
                        segment.len()
                    );

                    // Stub translation
                    let translated = format!(
                        "[Translation stub: segment #{} pending model integration]",
                        segment_count
                    );

                    let _ = app_handle.emit(
                        "transcription-result",
                        TranscriptionResult {
                            recognized,
                            translated,
                        },
                    );
                }
            });

            Ok(())
        })
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
