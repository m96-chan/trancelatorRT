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
use pipeline::stt::whisper_backend::WhisperRecognizer;
use pipeline::stt::{SpeechRecognizer, SttConfig};
use tauri::{Emitter, Manager};

type AppModelManager = std::sync::Arc<std::sync::Mutex<ModelManager<HttpDownloader, FileStorage>>>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (segment_tx, segment_rx) = std::sync::mpsc::channel();
    let pipeline =
        AudioPipeline::new(CaptureConfig::default(), segment_tx).expect("Failed to create audio pipeline");

    // Create shared LanguageState wrapped in Arc for thread sharing
    let lang_state = std::sync::Arc::new(std::sync::Mutex::new(LanguageState::new()));
    let lang_state_for_thread = lang_state.clone();

    tauri::Builder::default()
        .manage(std::sync::Mutex::new(pipeline))
        .manage(lang_state)
        .setup(|app| {
            let models_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir")
                .join("models");

            let registry = ModelRegistry::default();
            let downloader = HttpDownloader::new();
            let storage = FileStorage::new(models_dir.clone());
            let model_manager = ModelManager::new(registry, downloader, storage);

            // Check if Whisper model is already downloaded
            let whisper_model_path = if model_manager.is_downloaded("whisper-tiny").unwrap_or(false) {
                model_manager.model_path("whisper-tiny").ok().flatten()
            } else {
                None
            };

            let model_manager_arc: AppModelManager =
                std::sync::Arc::new(std::sync::Mutex::new(model_manager));
            let model_manager_for_thread = model_manager_arc.clone();
            app.manage(model_manager_arc);

            // Spawn segment processing thread with real Whisper STT
            let app_handle = app.handle().clone();

            std::thread::spawn(move || {
                // Initialize Whisper recognizer
                let mut recognizer = WhisperRecognizer::new();
                let mut stt_ready = false;

                if let Some(path) = whisper_model_path {
                    match recognizer.load_model(path.to_str().unwrap_or("")) {
                        Ok(()) => {
                            stt_ready = true;
                            eprintln!("[trancelatorRT] Whisper model loaded successfully");
                        }
                        Err(e) => {
                            eprintln!("[trancelatorRT] Failed to load Whisper model: {}", e);
                        }
                    }
                } else {
                    eprintln!("[trancelatorRT] Whisper model not downloaded yet, using stub STT");
                }

                while let Ok(segment) = segment_rx.recv() {
                    // Try to load model if not ready (user may have downloaded it after startup)
                    if !stt_ready {
                        if let Ok(mgr) = model_manager_for_thread.lock() {
                            if let Ok(Some(path)) = mgr.model_path("whisper-tiny") {
                                if let Ok(()) = recognizer.load_model(path.to_str().unwrap_or("")) {
                                    stt_ready = true;
                                    eprintln!("[trancelatorRT] Whisper model loaded (lazy)");
                                }
                            }
                        }
                    }

                    let (recognized, detected_lang) = if stt_ready {
                        // Get current source language setting
                        let source_lang = lang_state_for_thread
                            .lock()
                            .ok()
                            .map(|s| s.source.clone());

                        let stt_config = SttConfig {
                            n_threads: 4,
                            language: source_lang,
                            sample_rate: 16000,
                            no_speech_threshold: 0.6,
                            ..SttConfig::default()
                        };

                        match recognizer.transcribe(&segment, &stt_config) {
                            Ok(result) => {
                                if result.no_speech_probability > 0.6 || result.text.trim().is_empty() {
                                    continue; // Skip silence/noise
                                }
                                (result.text, Some(result.language))
                            }
                            Err(e) => {
                                eprintln!("[trancelatorRT] STT error: {}", e);
                                continue;
                            }
                        }
                    } else {
                        let duration_ms = segment.len() as f64 / 16.0;
                        (
                            format!(
                                "[Whisper model not loaded - speech {:.0}ms detected. Download model in Settings.]",
                                duration_ms
                            ),
                            None,
                        )
                    };

                    // Translation: pass through for now (NLLB integration pending)
                    let translated = {
                        let _target_lang = lang_state_for_thread
                            .lock()
                            .ok()
                            .map(|s| s.target.clone());

                        // TODO: Wire NllbTranslator when ort + tokenizer integration is complete
                        // For now, indicate that translation is pending
                        if detected_lang.is_some() {
                            format!("[Translation pending NLLB model] {}", &recognized)
                        } else {
                            recognized.clone()
                        }
                    };

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
