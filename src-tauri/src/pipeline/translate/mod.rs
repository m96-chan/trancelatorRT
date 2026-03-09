/// Translation module (NLLB-200 via CTranslate2)

pub mod nllb_backend;

use crate::pipeline::stt::Language;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct TranslationRequest {
    pub text: String,
    pub source_language: Language,
    pub target_language: Language,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranslationResult {
    pub text: String,
    pub source_language: Language,
    pub target_language: Language,
    pub score: Option<f32>,
}

#[derive(Debug, Error, Clone)]
pub enum TranslateError {
    #[error("Failed to load model: {0}")]
    ModelLoadError(String),
    #[error("Translation failed: {0}")]
    TranslationError(String),
    #[error("Unsupported language pair")]
    UnsupportedLanguagePair,
    #[error("Empty input text")]
    EmptyInput,
    #[error("Model not loaded")]
    ModelNotLoaded,
}

pub type TranslateResult<T> = Result<T, TranslateError>;

#[derive(Debug, Clone)]
pub struct TranslateConfig {
    pub model_path: String,
    pub n_threads: u32,
    pub beam_size: usize,
    pub max_length: usize,
}

impl Default for TranslateConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            n_threads: 4,
            beam_size: 4,
            max_length: 256,
        }
    }
}

pub trait Translator: Send {
    fn load_model(&mut self, path: &str) -> TranslateResult<()>;
    fn unload_model(&mut self);
    fn is_model_loaded(&self) -> bool;
    fn translate(&self, request: &TranslationRequest) -> TranslateResult<TranslationResult>;
}

pub struct TranslateEngine<T: Translator> {
    translator: T,
    config: TranslateConfig,
}

impl<T: Translator> TranslateEngine<T> {
    pub fn new(translator: T, config: TranslateConfig) -> Self {
        Self { translator, config }
    }

    pub fn init(&mut self) -> TranslateResult<()> {
        self.translator.load_model(&self.config.model_path)
    }

    pub fn translate(
        &self,
        text: &str,
        source: Language,
        target: Language,
    ) -> TranslateResult<TranslationResult> {
        if !self.translator.is_model_loaded() {
            return Err(TranslateError::ModelNotLoaded);
        }

        let text = text.trim();
        if text.is_empty() {
            return Err(TranslateError::EmptyInput);
        }

        if source == target {
            return Ok(TranslationResult {
                text: text.to_string(),
                source_language: source,
                target_language: target,
                score: None,
            });
        }

        self.translator.translate(&TranslationRequest {
            text: text.to_string(),
            source_language: source,
            target_language: target,
        })
    }

    pub fn shutdown(&mut self) {
        self.translator.unload_model();
    }

    pub fn config(&self) -> &TranslateConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTranslator {
        loaded: bool,
        response: Option<TranslateResult<TranslationResult>>,
    }

    impl MockTranslator {
        fn new() -> Self {
            Self {
                loaded: false,
                response: None,
            }
        }

        fn with_response(mut self, result: TranslateResult<TranslationResult>) -> Self {
            self.response = Some(result);
            self
        }

        fn with_success(self, text: &str, src: Language, tgt: Language) -> Self {
            self.with_response(Ok(TranslationResult {
                text: text.to_string(),
                source_language: src,
                target_language: tgt,
                score: Some(-1.5),
            }))
        }
    }

    impl Translator for MockTranslator {
        fn load_model(&mut self, path: &str) -> TranslateResult<()> {
            if path.is_empty() {
                return Err(TranslateError::ModelLoadError("Empty path".into()));
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

        fn translate(&self, _request: &TranslationRequest) -> TranslateResult<TranslationResult> {
            self.response
                .clone()
                .unwrap_or(Err(TranslateError::TranslationError(
                    "No response configured".into(),
                )))
        }
    }

    fn test_config() -> TranslateConfig {
        TranslateConfig {
            model_path: "/fake/model".into(),
            ..TranslateConfig::default()
        }
    }

    // Config tests

    #[test]
    fn test_default_config() {
        let config = TranslateConfig::default();
        assert_eq!(config.n_threads, 4);
        assert_eq!(config.beam_size, 4);
        assert_eq!(config.max_length, 256);
    }

    // Engine tests

    #[test]
    fn test_engine_init_loads_model() {
        let mock = MockTranslator::new();
        let mut engine = TranslateEngine::new(mock, test_config());
        assert!(engine.init().is_ok());
    }

    #[test]
    fn test_engine_init_fails_with_empty_path() {
        let mock = MockTranslator::new();
        let config = TranslateConfig {
            model_path: "".into(),
            ..TranslateConfig::default()
        };
        let mut engine = TranslateEngine::new(mock, config);
        assert!(engine.init().is_err());
    }

    #[test]
    fn test_engine_translate_en_to_ja() {
        let mock =
            MockTranslator::new().with_success("こんにちは", Language::English, Language::Japanese);
        let mut engine = TranslateEngine::new(mock, test_config());
        engine.init().unwrap();

        let result = engine
            .translate("Hello", Language::English, Language::Japanese)
            .unwrap();
        assert_eq!(result.text, "こんにちは");
        assert_eq!(result.source_language, Language::English);
        assert_eq!(result.target_language, Language::Japanese);
    }

    #[test]
    fn test_engine_translate_ja_to_en() {
        let mock =
            MockTranslator::new().with_success("Hello", Language::Japanese, Language::English);
        let mut engine = TranslateEngine::new(mock, test_config());
        engine.init().unwrap();

        let result = engine
            .translate("こんにちは", Language::Japanese, Language::English)
            .unwrap();
        assert_eq!(result.text, "Hello");
    }

    #[test]
    fn test_engine_translate_fr_to_de() {
        let mock =
            MockTranslator::new().with_success("Hallo Welt", Language::French, Language::German);
        let mut engine = TranslateEngine::new(mock, test_config());
        engine.init().unwrap();

        let result = engine
            .translate("Bonjour le monde", Language::French, Language::German)
            .unwrap();
        assert_eq!(result.text, "Hallo Welt");
    }

    #[test]
    fn test_engine_rejects_empty_input() {
        let mock =
            MockTranslator::new().with_success("text", Language::English, Language::Japanese);
        let mut engine = TranslateEngine::new(mock, test_config());
        engine.init().unwrap();

        let result = engine.translate("", Language::English, Language::Japanese);
        assert!(matches!(result, Err(TranslateError::EmptyInput)));
    }

    #[test]
    fn test_engine_rejects_whitespace_only() {
        let mock =
            MockTranslator::new().with_success("text", Language::English, Language::Japanese);
        let mut engine = TranslateEngine::new(mock, test_config());
        engine.init().unwrap();

        let result = engine.translate("   ", Language::English, Language::Japanese);
        assert!(matches!(result, Err(TranslateError::EmptyInput)));
    }

    #[test]
    fn test_engine_rejects_without_model() {
        let mock =
            MockTranslator::new().with_success("text", Language::English, Language::Japanese);
        let engine = TranslateEngine::new(mock, test_config());

        let result = engine.translate("Hello", Language::English, Language::Japanese);
        assert!(matches!(result, Err(TranslateError::ModelNotLoaded)));
    }

    #[test]
    fn test_engine_same_language_passthrough() {
        let mock = MockTranslator::new();
        let mut engine = TranslateEngine::new(mock, test_config());
        engine.init().unwrap();

        let result = engine
            .translate("Hello", Language::English, Language::English)
            .unwrap();
        assert_eq!(result.text, "Hello");
        assert_eq!(result.source_language, Language::English);
        assert_eq!(result.target_language, Language::English);
        assert!(result.score.is_none());
    }

    #[test]
    fn test_engine_shutdown_unloads_model() {
        let mock =
            MockTranslator::new().with_success("text", Language::English, Language::Japanese);
        let mut engine = TranslateEngine::new(mock, test_config());
        engine.init().unwrap();
        engine.shutdown();

        let result = engine.translate("Hello", Language::English, Language::Japanese);
        assert!(matches!(result, Err(TranslateError::ModelNotLoaded)));
    }

    #[test]
    fn test_engine_propagates_error() {
        let mock = MockTranslator::new().with_response(Err(TranslateError::TranslationError(
            "decode failed".into(),
        )));
        let mut engine = TranslateEngine::new(mock, test_config());
        engine.init().unwrap();

        let result = engine.translate("Hello", Language::English, Language::Japanese);
        assert!(matches!(result, Err(TranslateError::TranslationError(_))));
    }

    #[test]
    fn test_translation_result_clone() {
        let result = TranslationResult {
            text: "test".into(),
            source_language: Language::English,
            target_language: Language::Japanese,
            score: Some(-1.5),
        };
        let cloned = result.clone();
        assert_eq!(result, cloned);
    }

    #[test]
    fn test_request_creation() {
        let req = TranslationRequest {
            text: "Hello".into(),
            source_language: Language::English,
            target_language: Language::Japanese,
        };
        assert_eq!(req.text, "Hello");
        assert_eq!(req.source_language, Language::English);
        assert_eq!(req.target_language, Language::Japanese);
    }
}
