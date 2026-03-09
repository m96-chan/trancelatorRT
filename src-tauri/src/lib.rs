pub mod audio;
mod commands;
mod pipeline;

use audio::capture::CaptureConfig;
use audio::AudioPipeline;
use commands::LanguageState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (segment_tx, _segment_rx) = std::sync::mpsc::channel();
    let pipeline =
        AudioPipeline::new(CaptureConfig::default(), segment_tx).expect("Failed to create audio pipeline");

    tauri::Builder::default()
        .manage(std::sync::Mutex::new(pipeline))
        .manage(std::sync::Mutex::new(LanguageState::new()))
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
