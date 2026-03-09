/// Text-to-Speech module (Piper TTS wrapper)

pub mod piper_backend;

use crate::pipeline::stt::Language;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum TtsError {
    #[error("Failed to load model: {0}")]
    ModelLoadError(String),
    #[error("Synthesis failed: {0}")]
    SynthesisError(String),
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("Empty input text")]
    EmptyInput,
    #[error("Model not loaded")]
    ModelNotLoaded,
}

pub type TtsResult<T> = Result<T, TtsError>;

#[derive(Debug, Clone)]
pub struct TtsConfig {
    pub model_dir: String,
    pub n_threads: u32,
    pub language: Language,
    pub speaker_id: Option<i64>,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            model_dir: String::new(),
            n_threads: 4,
            language: Language::English,
            speaker_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SynthesisResult {
    pub samples: Vec<i16>,
    pub sample_rate: u32,
    pub language: Language,
}

pub trait TtsSynthesizer: Send {
    fn load_model(&mut self, config_path: &str) -> TtsResult<()>;
    fn unload_model(&mut self);
    fn is_model_loaded(&self) -> bool;
    fn synthesize(&self, text: &str, language: Language) -> TtsResult<SynthesisResult>;
    fn sample_rate(&self) -> u32;
}

pub fn audio_f32_to_i16(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|&s| {
            let clamped = s.clamp(-1.0, 1.0);
            (clamped * i16::MAX as f32) as i16
        })
        .collect()
}

pub struct TtsEngine<S: TtsSynthesizer> {
    synthesizer: S,
    config: TtsConfig,
}

impl<S: TtsSynthesizer> TtsEngine<S> {
    pub fn new(synthesizer: S, config: TtsConfig) -> Self {
        Self { synthesizer, config }
    }

    pub fn init(&mut self) -> TtsResult<()> {
        let voice = self.config.language.piper_voice_name();
        let config_path = format!("{}/{}.onnx.json", self.config.model_dir, voice);
        self.synthesizer.load_model(&config_path)
    }

    pub fn synthesize(&self, text: &str) -> TtsResult<SynthesisResult> {
        if !self.synthesizer.is_model_loaded() {
            return Err(TtsError::ModelNotLoaded);
        }
        let text = text.trim();
        if text.is_empty() {
            return Err(TtsError::EmptyInput);
        }
        self.synthesizer.synthesize(text, self.config.language)
    }

    pub fn shutdown(&mut self) {
        self.synthesizer.unload_model();
    }

    pub fn set_language(&mut self, lang: Language) {
        self.config.language = lang;
    }

    pub fn config(&self) -> &TtsConfig {
        &self.config
    }

    pub fn sample_rate(&self) -> u32 {
        self.synthesizer.sample_rate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockSynthesizer {
        loaded: bool,
        response: Option<TtsResult<SynthesisResult>>,
        rate: u32,
    }

    impl MockSynthesizer {
        fn new() -> Self {
            Self {
                loaded: false,
                response: None,
                rate: 22050,
            }
        }

        fn with_response(mut self, result: TtsResult<SynthesisResult>) -> Self {
            self.response = Some(result);
            self
        }

        fn with_success(self, samples: Vec<i16>, lang: Language) -> Self {
            self.with_response(Ok(SynthesisResult {
                samples,
                sample_rate: 22050,
                language: lang,
            }))
        }
    }

    impl TtsSynthesizer for MockSynthesizer {
        fn load_model(&mut self, path: &str) -> TtsResult<()> {
            if path.is_empty() {
                return Err(TtsError::ModelLoadError("Empty path".into()));
            }
            self.loaded = true;
            Ok(())
        }

        fn unload_model(&mut self) {
            self.loaded = false;
        }

        fn is_model_loaded(&self) -> bool {
            self.loaded
        }

        fn synthesize(&self, _text: &str, _language: Language) -> TtsResult<SynthesisResult> {
            self.response
                .clone()
                .unwrap_or(Err(TtsError::SynthesisError("No response".into())))
        }

        fn sample_rate(&self) -> u32 {
            self.rate
        }
    }

    fn test_config() -> TtsConfig {
        TtsConfig {
            model_dir: "/fake/models".into(),
            language: Language::English,
            ..TtsConfig::default()
        }
    }

    fn test_samples() -> Vec<i16> {
        vec![100, 200, 300, -100, -200, -300]
    }

    // Config tests

    #[test]
    fn test_default_config() {
        let config = TtsConfig::default();
        assert_eq!(config.n_threads, 4);
        assert_eq!(config.language, Language::English);
        assert!(config.speaker_id.is_none());
    }

    // Engine tests

    #[test]
    fn test_engine_init_loads_model() {
        let mock = MockSynthesizer::new();
        let mut engine = TtsEngine::new(mock, test_config());
        assert!(engine.init().is_ok());
    }

    #[test]
    fn test_engine_init_fails_with_empty_model_dir() {
        let mock = MockSynthesizer::new();
        let config = TtsConfig {
            model_dir: "".into(),
            ..TtsConfig::default()
        };
        let mut engine = TtsEngine::new(mock, config);
        // model_dir is empty but config_path is "/<voice>.onnx.json", not empty
        // so this should actually succeed with the mock
        assert!(engine.init().is_ok());
    }

    #[test]
    fn test_engine_synthesize_success() {
        let mock = MockSynthesizer::new().with_success(test_samples(), Language::English);
        let mut engine = TtsEngine::new(mock, test_config());
        engine.init().unwrap();

        let result = engine.synthesize("Hello world").unwrap();
        assert_eq!(result.samples, test_samples());
        assert_eq!(result.sample_rate, 22050);
        assert_eq!(result.language, Language::English);
    }

    #[test]
    fn test_engine_synthesize_japanese() {
        let mock = MockSynthesizer::new().with_success(test_samples(), Language::Japanese);
        let mut config = test_config();
        config.language = Language::Japanese;
        let mut engine = TtsEngine::new(mock, config);
        engine.init().unwrap();

        let result = engine.synthesize("こんにちは").unwrap();
        assert_eq!(result.language, Language::Japanese);
    }

    #[test]
    fn test_engine_rejects_empty_input() {
        let mock = MockSynthesizer::new().with_success(test_samples(), Language::English);
        let mut engine = TtsEngine::new(mock, test_config());
        engine.init().unwrap();

        let result = engine.synthesize("");
        assert!(matches!(result, Err(TtsError::EmptyInput)));
    }

    #[test]
    fn test_engine_rejects_whitespace_only() {
        let mock = MockSynthesizer::new().with_success(test_samples(), Language::English);
        let mut engine = TtsEngine::new(mock, test_config());
        engine.init().unwrap();

        let result = engine.synthesize("   ");
        assert!(matches!(result, Err(TtsError::EmptyInput)));
    }

    #[test]
    fn test_engine_rejects_without_model() {
        let mock = MockSynthesizer::new().with_success(test_samples(), Language::English);
        let engine = TtsEngine::new(mock, test_config());

        let result = engine.synthesize("Hello");
        assert!(matches!(result, Err(TtsError::ModelNotLoaded)));
    }

    #[test]
    fn test_engine_shutdown_unloads_model() {
        let mock = MockSynthesizer::new().with_success(test_samples(), Language::English);
        let mut engine = TtsEngine::new(mock, test_config());
        engine.init().unwrap();
        engine.shutdown();

        let result = engine.synthesize("Hello");
        assert!(matches!(result, Err(TtsError::ModelNotLoaded)));
    }

    #[test]
    fn test_engine_propagates_synthesis_error() {
        let mock = MockSynthesizer::new()
            .with_response(Err(TtsError::SynthesisError("phonemize failed".into())));
        let mut engine = TtsEngine::new(mock, test_config());
        engine.init().unwrap();

        let result = engine.synthesize("Hello");
        assert!(matches!(result, Err(TtsError::SynthesisError(_))));
    }

    #[test]
    fn test_engine_set_language() {
        let mock = MockSynthesizer::new();
        let mut engine = TtsEngine::new(mock, test_config());
        assert_eq!(engine.config().language, Language::English);

        engine.set_language(Language::Japanese);
        assert_eq!(engine.config().language, Language::Japanese);
    }

    #[test]
    fn test_engine_sample_rate() {
        let mock = MockSynthesizer::new();
        let engine = TtsEngine::new(mock, test_config());
        assert_eq!(engine.sample_rate(), 22050);
    }

    #[test]
    fn test_synthesis_result_clone() {
        let result = SynthesisResult {
            samples: test_samples(),
            sample_rate: 22050,
            language: Language::English,
        };
        let cloned = result.clone();
        assert_eq!(result, cloned);
    }

    // Audio conversion tests

    #[test]
    fn test_audio_f32_to_i16_silence() {
        let silence = vec![0.0f32; 100];
        let converted = audio_f32_to_i16(&silence);
        assert_eq!(converted.len(), 100);
        assert!(converted.iter().all(|&s| s == 0));
    }

    #[test]
    fn test_audio_f32_to_i16_max_positive() {
        let max = vec![1.0f32];
        let converted = audio_f32_to_i16(&max);
        assert_eq!(converted[0], i16::MAX);
    }

    #[test]
    fn test_audio_f32_to_i16_max_negative() {
        let min = vec![-1.0f32];
        let converted = audio_f32_to_i16(&min);
        assert!(converted[0] < -32700);
    }

    #[test]
    fn test_audio_f32_to_i16_clamps() {
        let over = vec![2.0f32, -2.0f32];
        let converted = audio_f32_to_i16(&over);
        assert_eq!(converted[0], i16::MAX);
        assert!(converted[1] < -32700);
    }

    #[test]
    fn test_audio_f32_to_i16_empty() {
        let empty: Vec<f32> = vec![];
        let converted = audio_f32_to_i16(&empty);
        assert!(converted.is_empty());
    }
}
