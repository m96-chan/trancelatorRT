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

/// Known Whisper hallucination patterns (especially with tiny model)
fn is_hallucination(text: &str) -> bool {
    let lower = text.to_lowercase();
    let hallucinations = [
        "ご視聴ありがとうございました",
        "ありがとうございました",
        "チャンネル登録",
        "お願いします",
        "thank you for watching",
        "thanks for watching",
        "please subscribe",
        "like and subscribe",
        "see you next time",
        "subtitles by",
        "translated by",
        "blank audio",
        "music playing",
        "music",
        "[music]",
        "(music)",
        "you",
        "www.",
        "http",
    ];
    for h in &hallucinations {
        if lower.contains(&h.to_lowercase()) {
            return true;
        }
    }
    // Detect repetitive patterns (e.g., same word repeated many times)
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() >= 4 {
        let first = words[0];
        if words.iter().all(|w| *w == first) {
            return true;
        }
    }
    false
}

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

            // Pick the best available Whisper model (prefer larger)
            let whisper_models = ["whisper-large-v3-turbo", "whisper-medium", "whisper-small", "whisper-base", "whisper-tiny"];
            let whisper_model_path = whisper_models
                .iter()
                .find_map(|id| {
                    if model_manager.is_downloaded(id).unwrap_or(false) {
                        model_manager.model_path(id).ok().flatten()
                    } else {
                        None
                    }
                });

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

                    // Drain queued segments - only process the latest one
                    let mut latest_segment = segment;
                    let mut skipped = 0u32;
                    while let Ok(newer) = segment_rx.try_recv() {
                        latest_segment = newer;
                        skipped += 1;
                        segment_count += 1;
                    }
                    if skipped > 0 {
                        emit_log(
                            "vad",
                            &format!("Skipped {} queued segments (too slow), using latest", skipped),
                        );
                    }

                    let segment = latest_segment;
                    let duration_ms = segment.len() as f64 / 16.0;
                    emit_log(
                        "vad",
                        &format!(
                            "Speech segment #{} ({:.0}ms, {} samples)",
                            segment_count, duration_ms, segment.len()
                        ),
                    );

                    // Try to load model if not ready (user may have downloaded it after startup)
                    if !stt_ready {
                        emit_log("stt", "Whisper not loaded, checking for model...");
                        if let Ok(mgr) = model_manager_for_thread.lock() {
                            // Try best model first
                            let models = ["whisper-large-v3-turbo", "whisper-medium", "whisper-small", "whisper-base", "whisper-tiny"];
                            for id in &models {
                                if let Ok(Some(path)) = mgr.model_path(id) {
                                    emit_log("stt", &format!("Found {} at {}, loading...", id, path.display()));
                                    if let Ok(()) = recognizer.load_model(path.to_str().unwrap_or("")) {
                                        stt_ready = true;
                                        emit_log("stt", &format!("{} loaded", id));
                                        break;
                                    }
                                }
                            }
                            if !stt_ready {
                                emit_log("stt", "No Whisper model found on disk");
                            }
                        }
                    }

                    let (recognized, detected_lang) = if stt_ready {
                        // Get current source language setting
                        let source_lang = lang_state_for_thread
                            .lock()
                            .ok()
                            .map(|s| s.source.clone());

                        let audio_duration_sec = segment.len() as f64 / 16000.0;
                        emit_log(
                            "stt",
                            &format!(
                                "Transcribing {:.1}s audio (lang: {})...",
                                audio_duration_sec,
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

                        let start = std::time::Instant::now();
                        let transcribe_result = recognizer.transcribe(&segment, &stt_config);
                        let elapsed = start.elapsed();

                        match transcribe_result {
                            Ok(result) => {
                                let speed_ratio = audio_duration_sec / elapsed.as_secs_f64();
                                if result.no_speech_probability > 0.6 || result.text.trim().is_empty() {
                                    emit_log(
                                        "stt",
                                        &format!(
                                            "No speech ({:.1}s, {:.1}x realtime, prob={:.2})",
                                            elapsed.as_secs_f64(),
                                            speed_ratio,
                                            result.no_speech_probability
                                        ),
                                    );
                                    continue;
                                }
                                // Filter known Whisper hallucinations
                                let text = result.text.trim();
                                if is_hallucination(text) {
                                    emit_log(
                                        "stt",
                                        &format!(
                                            "Hallucination filtered ({:.1}s): \"{}\"",
                                            elapsed.as_secs_f64(),
                                            text
                                        ),
                                    );
                                    continue;
                                }
                                emit_log(
                                    "stt",
                                    &format!(
                                        "OK ({:.1}s, {:.1}x realtime, {}): \"{}\"",
                                        elapsed.as_secs_f64(),
                                        speed_ratio,
                                        result.language.whisper_code(),
                                        text
                                    ),
                                );
                                (text.to_string(), Some(result.language))
                            }
                            Err(e) => {
                                emit_log("error", &format!("STT failed ({:.1}s): {}", elapsed.as_secs_f64(), e));
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

                    // Translation: not yet implemented - show empty for translated panel
                    let translated = {
                        let target_lang = lang_state_for_thread
                            .lock()
                            .ok()
                            .map(|s| s.target.clone());

                        if detected_lang.is_some() {
                            emit_log(
                                "translate",
                                &format!(
                                    "Translation not available (NLLB not integrated). Target: {}",
                                    target_lang
                                        .as_ref()
                                        .map(|l| l.whisper_code())
                                        .unwrap_or("?")
                                ),
                            );
                            // Don't duplicate recognized text - show nothing for translation
                            String::new()
                        } else {
                            String::new()
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
