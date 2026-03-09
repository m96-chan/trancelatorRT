use crate::audio::state::PipelineState;
use crate::audio::AudioPipeline;
use crate::pipeline::stt::Language;
use std::sync::Mutex;
use tauri::State;

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
    lang_state: State<'_, Mutex<LanguageState>>,
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
    lang_state: State<'_, Mutex<LanguageState>>,
) -> Result<(), String> {
    let lang = Language::from_whisper_code(code)
        .ok_or_else(|| format!("Unknown language code: {}", code))?;
    lang_state.lock().map_err(|e| e.to_string())?.source = lang;
    Ok(())
}

#[tauri::command]
pub fn set_target_language(
    code: &str,
    lang_state: State<'_, Mutex<LanguageState>>,
) -> Result<(), String> {
    let lang = Language::from_whisper_code(code)
        .ok_or_else(|| format!("Unknown language code: {}", code))?;
    lang_state.lock().map_err(|e| e.to_string())?.target = lang;
    Ok(())
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
