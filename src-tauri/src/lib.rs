pub mod audio;
mod commands;
mod model;
mod pipeline;

use audio::capture::CaptureConfig;
use audio::AudioPipeline;
use commands::{LanguageState, PipelineLog, TranscriptionResult};
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
                let emit_log = |stage: &str, message: &str| {
                    let _ = app_handle.emit(
                        "pipeline-log",
                        PipelineLog {
                            stage: stage.to_string(),
                            message: message.to_string(),
                        },
                    );
                };

                // Initialize Whisper recognizer
                let mut recognizer = WhisperRecognizer::new();
                let mut stt_ready = false;

                if let Some(path) = &whisper_model_path {
                    emit_log("init", &format!("Loading Whisper model: {}", path.display()));
                    match recognizer.load_model(path.to_str().unwrap_or("")) {
                        Ok(()) => {
                            stt_ready = true;
                            emit_log("init", "Whisper model loaded successfully");
                        }
                        Err(e) => {
                            emit_log("error", &format!("Failed to load Whisper: {}", e));
                        }
                    }
                } else {
                    emit_log("init", "Whisper model not downloaded. Download from Models panel.");
                }

                let mut segment_count = 0u32;

                while let Ok(segment) = segment_rx.recv() {
                    segment_count += 1;
                    let duration_ms = segment.len() as f64 / 16.0;
                    emit_log(
                        "vad",
                        &format!(
                            "Speech segment #{} received ({:.0}ms, {} samples)",
                            segment_count, duration_ms, segment.len()
                        ),
                    );

                    // Try to load model if not ready (user may have downloaded it after startup)
                    if !stt_ready {
                        emit_log("stt", "Whisper not loaded, checking for model...");
                        if let Ok(mgr) = model_manager_for_thread.lock() {
                            if let Ok(Some(path)) = mgr.model_path("whisper-tiny") {
                                emit_log("stt", &format!("Found model at {}, loading...", path.display()));
                                if let Ok(()) = recognizer.load_model(path.to_str().unwrap_or("")) {
                                    stt_ready = true;
                                    emit_log("stt", "Whisper model loaded (lazy)");
                                }
                            } else {
                                emit_log("stt", "Whisper model not found on disk");
                            }
                        }
                    }

                    let (recognized, detected_lang) = if stt_ready {
                        // Get current source language setting
                        let source_lang = lang_state_for_thread
                            .lock()
                            .ok()
                            .map(|s| s.source.clone());

                        emit_log(
                            "stt",
                            &format!(
                                "Transcribing with Whisper (lang: {})...",
                                source_lang
                                    .as_ref()
                                    .map(|l| l.whisper_code())
                                    .unwrap_or("auto")
                            ),
                        );

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
                                    emit_log(
                                        "stt",
                                        &format!(
                                            "Skipped: no speech (prob={:.2})",
                                            result.no_speech_probability
                                        ),
                                    );
                                    continue;
                                }
                                emit_log(
                                    "stt",
                                    &format!(
                                        "Recognized ({}): \"{}\"",
                                        result.language.whisper_code(),
                                        &result.text
                                    ),
                                );
                                (result.text, Some(result.language))
                            }
                            Err(e) => {
                                emit_log("error", &format!("STT failed: {}", e));
                                continue;
                            }
                        }
                    } else {
                        emit_log("stt", "No Whisper model - cannot transcribe");
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
                        let target_lang = lang_state_for_thread
                            .lock()
                            .ok()
                            .map(|s| s.target.clone());

                        if detected_lang.is_some() {
                            emit_log(
                                "translate",
                                &format!(
                                    "Translation skipped (NLLB model not integrated). Target: {}",
                                    target_lang
                                        .as_ref()
                                        .map(|l| l.whisper_code())
                                        .unwrap_or("?")
                                ),
                            );
                            recognized.clone()
                        } else {
                            recognized.clone()
                        }
                    };

                    emit_log("emit", "Sending result to UI");

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
