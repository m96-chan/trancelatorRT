use crate::audio::state::PipelineState;
use crate::audio::AudioPipeline;
use std::sync::Mutex;
use tauri::State;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        let result = greet("World");
        assert_eq!(result, "Hello, World! Welcome to trancelatorRT.");
    }
}
