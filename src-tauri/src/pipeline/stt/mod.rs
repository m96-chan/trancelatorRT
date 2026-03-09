/// Speech-to-Text module (whisper.cpp wrapper)

pub mod whisper_backend;

use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Language {
    Japanese,
    Korean,
    English,
    French,
    German,
    Portuguese,
    Russian,
    Arabic,
}

impl Language {
    pub fn whisper_code(&self) -> &'static str {
        match self {
            Language::Japanese => "ja",
            Language::Korean => "ko",
            Language::English => "en",
            Language::French => "fr",
            Language::German => "de",
            Language::Portuguese => "pt",
            Language::Russian => "ru",
            Language::Arabic => "ar",
        }
    }

    pub fn from_whisper_code(code: &str) -> Option<Self> {
        match code {
            "ja" | "japanese" => Some(Language::Japanese),
            "ko" | "korean" => Some(Language::Korean),
            "en" | "english" => Some(Language::English),
            "fr" | "french" => Some(Language::French),
            "de" | "german" => Some(Language::German),
            "pt" | "portuguese" => Some(Language::Portuguese),
            "ru" | "russian" => Some(Language::Russian),
            "ar" | "arabic" => Some(Language::Arabic),
            _ => None,
        }
    }

    pub fn all() -> &'static [Language] {
        &[
            Language::Japanese,
            Language::Korean,
            Language::English,
            Language::French,
            Language::German,
            Language::Portuguese,
            Language::Russian,
            Language::Arabic,
        ]
    }

    pub fn nllb_code(&self) -> &'static str {
        match self {
            Language::Japanese => "jpn_Jpan",
            Language::Korean => "kor_Hang",
            Language::English => "eng_Latn",
            Language::French => "fra_Latn",
            Language::German => "deu_Latn",
            Language::Portuguese => "por_Latn",
            Language::Russian => "rus_Cyrl",
            Language::Arabic => "arb_Arab",
        }
    }

    pub fn from_nllb_code(code: &str) -> Option<Self> {
        match code {
            "jpn_Jpan" => Some(Language::Japanese),
            "kor_Hang" => Some(Language::Korean),
            "eng_Latn" => Some(Language::English),
            "fra_Latn" => Some(Language::French),
            "deu_Latn" => Some(Language::German),
            "por_Latn" => Some(Language::Portuguese),
            "rus_Cyrl" => Some(Language::Russian),
            "arb_Arab" => Some(Language::Arabic),
            _ => None,
        }
    }

    pub fn piper_voice_name(&self) -> &'static str {
        match self {
            Language::Japanese => "ja_JP-kokoro-medium",
            Language::Korean => "ko_KR-kss-medium",
            Language::English => "en_US-lessac-medium",
            Language::French => "fr_FR-siwis-medium",
            Language::German => "de_DE-thorsten-medium",
            Language::Portuguese => "pt_BR-faber-medium",
            Language::Russian => "ru_RU-denis-medium",
            Language::Arabic => "ar_JO-kareem-medium",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.whisper_code())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: Language,
    pub no_speech_probability: f32,
}

#[derive(Debug, Error)]
pub enum SttError {
    #[error("Failed to load model: {0}")]
    ModelLoadError(String),
    #[error("Transcription failed: {0}")]
    TranscriptionError(String),
    #[error("Unsupported language detected: {0}")]
    UnsupportedLanguage(String),
    #[error("No speech detected in audio segment")]
    NoSpeechDetected,
    #[error("Invalid audio data: {0}")]
    InvalidAudio(String),
    #[error("Model not loaded")]
    ModelNotLoaded,
}

pub type SttResult<T> = Result<T, SttError>;

#[derive(Debug, Clone)]
pub struct SttConfig {
    pub model_path: String,
    pub n_threads: u32,
    pub language: Option<Language>,
    pub sample_rate: u32,
    pub no_speech_threshold: f32,
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            n_threads: 4,
            language: None,
            sample_rate: 16000,
            no_speech_threshold: 0.6,
        }
    }
}

pub trait SpeechRecognizer: Send {
    fn load_model(&mut self, path: &str) -> SttResult<()>;
    fn unload_model(&mut self);
    fn is_model_loaded(&self) -> bool;
    fn transcribe(&mut self, samples: &[i16], config: &SttConfig) -> SttResult<TranscriptionResult>;
}

pub fn audio_i16_to_f32(samples: &[i16]) -> Vec<f32> {
    samples
        .iter()
        .map(|&s| s as f32 / i16::MAX as f32)
        .collect()
}

pub struct SttEngine<R: SpeechRecognizer> {
    recognizer: R,
    config: SttConfig,
}

impl<R: SpeechRecognizer> SttEngine<R> {
    pub fn new(recognizer: R, config: SttConfig) -> Self {
        Self { recognizer, config }
    }

    pub fn init(&mut self) -> SttResult<()> {
        self.recognizer.load_model(&self.config.model_path)
    }

    pub fn process_segment(&mut self, samples: &[i16]) -> SttResult<TranscriptionResult> {
        if !self.recognizer.is_model_loaded() {
            return Err(SttError::ModelNotLoaded);
        }
        if samples.is_empty() {
            return Err(SttError::InvalidAudio("Empty audio segment".into()));
        }
        if samples.len() < 1600 {
            return Err(SttError::InvalidAudio(format!(
                "Segment too short: {} samples (minimum 1600)",
                samples.len()
            )));
        }

        let result = self.recognizer.transcribe(samples, &self.config)?;

        if result.no_speech_probability > self.config.no_speech_threshold {
            return Err(SttError::NoSpeechDetected);
        }
        if result.text.trim().is_empty() {
            return Err(SttError::NoSpeechDetected);
        }

        Ok(result)
    }

    pub fn shutdown(&mut self) {
        self.recognizer.unload_model();
    }

    pub fn config(&self) -> &SttConfig {
        &self.config
    }

    pub fn set_language(&mut self, lang: Option<Language>) {
        self.config.language = lang;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRecognizer {
        loaded: bool,
        response: Option<SttResult<TranscriptionResult>>,
    }

    impl MockRecognizer {
        fn new() -> Self {
            Self {
                loaded: false,
                response: None,
            }
        }

        fn with_response(mut self, result: SttResult<TranscriptionResult>) -> Self {
            self.response = Some(result);
            self
        }

        fn with_success(self, text: &str, lang: Language) -> Self {
            self.with_response(Ok(TranscriptionResult {
                text: text.to_string(),
                language: lang,
                no_speech_probability: 0.1,
            }))
        }
    }

    impl SpeechRecognizer for MockRecognizer {
        fn load_model(&mut self, path: &str) -> SttResult<()> {
            if path.is_empty() {
                return Err(SttError::ModelLoadError("Empty path".into()));
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

        fn transcribe(
            &mut self,
            _samples: &[i16],
            _config: &SttConfig,
        ) -> SttResult<TranscriptionResult> {
            self.response
                .take()
                .unwrap_or(Err(SttError::TranscriptionError(
                    "No response configured".into(),
                )))
        }
    }

    fn test_config() -> SttConfig {
        SttConfig {
            model_path: "/fake/model.bin".into(),
            ..SttConfig::default()
        }
    }

    fn speech_samples(len: usize) -> Vec<i16> {
        (0..len)
            .map(|i| {
                let t = i as f32 / 16000.0;
                (f32::sin(2.0 * std::f32::consts::PI * 300.0 * t) * 20000.0) as i16
            })
            .collect()
    }

    // Language tests

    #[test]
    fn test_language_nllb_codes() {
        assert_eq!(Language::Japanese.nllb_code(), "jpn_Jpan");
        assert_eq!(Language::Korean.nllb_code(), "kor_Hang");
        assert_eq!(Language::English.nllb_code(), "eng_Latn");
        assert_eq!(Language::French.nllb_code(), "fra_Latn");
        assert_eq!(Language::German.nllb_code(), "deu_Latn");
        assert_eq!(Language::Portuguese.nllb_code(), "por_Latn");
        assert_eq!(Language::Russian.nllb_code(), "rus_Cyrl");
        assert_eq!(Language::Arabic.nllb_code(), "arb_Arab");
    }

    #[test]
    fn test_language_from_nllb_code() {
        assert_eq!(Language::from_nllb_code("jpn_Jpan"), Some(Language::Japanese));
        assert_eq!(Language::from_nllb_code("eng_Latn"), Some(Language::English));
        assert_eq!(Language::from_nllb_code("unknown"), None);
    }

    #[test]
    fn test_language_nllb_roundtrip() {
        for lang in Language::all() {
            let code = lang.nllb_code();
            assert_eq!(Language::from_nllb_code(code), Some(*lang));
        }
    }

    #[test]
    fn test_language_piper_voice_names() {
        assert!(!Language::Japanese.piper_voice_name().is_empty());
        assert!(!Language::English.piper_voice_name().is_empty());
        for lang in Language::all() {
            assert!(lang.piper_voice_name().contains("-medium"));
        }
    }

    #[test]
    fn test_language_whisper_codes() {
        assert_eq!(Language::Japanese.whisper_code(), "ja");
        assert_eq!(Language::Korean.whisper_code(), "ko");
        assert_eq!(Language::English.whisper_code(), "en");
        assert_eq!(Language::French.whisper_code(), "fr");
        assert_eq!(Language::German.whisper_code(), "de");
        assert_eq!(Language::Portuguese.whisper_code(), "pt");
        assert_eq!(Language::Russian.whisper_code(), "ru");
        assert_eq!(Language::Arabic.whisper_code(), "ar");
    }

    #[test]
    fn test_language_from_whisper_code_short() {
        assert_eq!(Language::from_whisper_code("ja"), Some(Language::Japanese));
        assert_eq!(Language::from_whisper_code("en"), Some(Language::English));
        assert_eq!(Language::from_whisper_code("xx"), None);
    }

    #[test]
    fn test_language_from_whisper_code_full() {
        assert_eq!(
            Language::from_whisper_code("japanese"),
            Some(Language::Japanese)
        );
        assert_eq!(
            Language::from_whisper_code("german"),
            Some(Language::German)
        );
    }

    #[test]
    fn test_language_all_returns_8_languages() {
        assert_eq!(Language::all().len(), 8);
    }

    #[test]
    fn test_language_display() {
        assert_eq!(format!("{}", Language::Japanese), "ja");
    }

    #[test]
    fn test_language_roundtrip() {
        for lang in Language::all() {
            let code = lang.whisper_code();
            assert_eq!(Language::from_whisper_code(code), Some(*lang));
        }
    }

    // Audio conversion tests

    #[test]
    fn test_audio_conversion_silence() {
        let silence = vec![0i16; 100];
        let f32_samples = audio_i16_to_f32(&silence);
        assert_eq!(f32_samples.len(), 100);
        assert!(f32_samples.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_audio_conversion_max_positive() {
        let max = vec![i16::MAX];
        let converted = audio_i16_to_f32(&max);
        assert!((converted[0] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_audio_conversion_max_negative() {
        let min = vec![i16::MIN];
        let converted = audio_i16_to_f32(&min);
        assert!(converted[0] < -0.99);
        assert!(converted[0] > -1.01);
    }

    #[test]
    fn test_audio_conversion_empty() {
        let empty: Vec<i16> = vec![];
        let converted = audio_i16_to_f32(&empty);
        assert!(converted.is_empty());
    }

    // Config tests

    #[test]
    fn test_default_config() {
        let config = SttConfig::default();
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.n_threads, 4);
        assert!(config.language.is_none());
        assert!((config.no_speech_threshold - 0.6).abs() < f32::EPSILON);
    }

    // Engine tests

    #[test]
    fn test_engine_init_loads_model() {
        let mock = MockRecognizer::new();
        let mut engine = SttEngine::new(mock, test_config());
        assert!(engine.init().is_ok());
    }

    #[test]
    fn test_engine_init_fails_with_empty_path() {
        let mock = MockRecognizer::new();
        let config = SttConfig {
            model_path: "".into(),
            ..SttConfig::default()
        };
        let mut engine = SttEngine::new(mock, config);
        assert!(engine.init().is_err());
    }

    #[test]
    fn test_engine_process_segment_success() {
        let mock = MockRecognizer::new().with_success("Hello world", Language::English);
        let mut engine = SttEngine::new(mock, test_config());
        engine.init().unwrap();

        let result = engine.process_segment(&speech_samples(16000)).unwrap();
        assert_eq!(result.text, "Hello world");
        assert_eq!(result.language, Language::English);
    }

    #[test]
    fn test_engine_process_segment_japanese() {
        let mock = MockRecognizer::new().with_success("こんにちは", Language::Japanese);
        let mut engine = SttEngine::new(mock, test_config());
        engine.init().unwrap();

        let result = engine.process_segment(&speech_samples(16000)).unwrap();
        assert_eq!(result.text, "こんにちは");
        assert_eq!(result.language, Language::Japanese);
    }

    #[test]
    fn test_engine_rejects_empty_segment() {
        let mock = MockRecognizer::new().with_success("text", Language::English);
        let mut engine = SttEngine::new(mock, test_config());
        engine.init().unwrap();

        let result = engine.process_segment(&[]);
        assert!(matches!(result, Err(SttError::InvalidAudio(_))));
    }

    #[test]
    fn test_engine_rejects_too_short_segment() {
        let mock = MockRecognizer::new().with_success("text", Language::English);
        let mut engine = SttEngine::new(mock, test_config());
        engine.init().unwrap();

        let result = engine.process_segment(&speech_samples(100));
        assert!(matches!(result, Err(SttError::InvalidAudio(_))));
    }

    #[test]
    fn test_engine_rejects_without_model_loaded() {
        let mock = MockRecognizer::new().with_success("text", Language::English);
        let mut engine = SttEngine::new(mock, test_config());

        let result = engine.process_segment(&speech_samples(16000));
        assert!(matches!(result, Err(SttError::ModelNotLoaded)));
    }

    #[test]
    fn test_engine_filters_high_no_speech_probability() {
        let mock = MockRecognizer::new().with_response(Ok(TranscriptionResult {
            text: "[silence]".into(),
            language: Language::English,
            no_speech_probability: 0.9,
        }));
        let mut engine = SttEngine::new(mock, test_config());
        engine.init().unwrap();

        let result = engine.process_segment(&speech_samples(16000));
        assert!(matches!(result, Err(SttError::NoSpeechDetected)));
    }

    #[test]
    fn test_engine_filters_empty_transcription() {
        let mock = MockRecognizer::new().with_response(Ok(TranscriptionResult {
            text: "   ".into(),
            language: Language::English,
            no_speech_probability: 0.1,
        }));
        let mut engine = SttEngine::new(mock, test_config());
        engine.init().unwrap();

        let result = engine.process_segment(&speech_samples(16000));
        assert!(matches!(result, Err(SttError::NoSpeechDetected)));
    }

    #[test]
    fn test_engine_shutdown_unloads_model() {
        let mock = MockRecognizer::new().with_success("text", Language::English);
        let mut engine = SttEngine::new(mock, test_config());
        engine.init().unwrap();
        engine.shutdown();

        let result = engine.process_segment(&speech_samples(16000));
        assert!(matches!(result, Err(SttError::ModelNotLoaded)));
    }

    #[test]
    fn test_engine_set_language() {
        let mock = MockRecognizer::new();
        let mut engine = SttEngine::new(mock, test_config());
        assert!(engine.config().language.is_none());

        engine.set_language(Some(Language::Japanese));
        assert_eq!(engine.config().language, Some(Language::Japanese));

        engine.set_language(None);
        assert!(engine.config().language.is_none());
    }

    #[test]
    fn test_engine_propagates_transcription_error() {
        let mock = MockRecognizer::new()
            .with_response(Err(SttError::TranscriptionError("decode failed".into())));
        let mut engine = SttEngine::new(mock, test_config());
        engine.init().unwrap();

        let result = engine.process_segment(&speech_samples(16000));
        assert!(matches!(result, Err(SttError::TranscriptionError(_))));
    }

    #[test]
    fn test_transcription_result_clone() {
        let result = TranscriptionResult {
            text: "test".into(),
            language: Language::English,
            no_speech_probability: 0.1,
        };
        let cloned = result.clone();
        assert_eq!(result, cloned);
    }
}
