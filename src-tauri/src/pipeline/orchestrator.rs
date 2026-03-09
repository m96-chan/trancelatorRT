/// End-to-end pipeline orchestrator: STT → Translation → TTS

use super::stt::{Language, SpeechRecognizer, SttConfig, SttEngine, TranscriptionResult};
use super::translate::{TranslateConfig, TranslateEngine, TranslationResult, Translator};
use super::tts::{SynthesisResult, TtsConfig, TtsEngine, TtsSynthesizer};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    Idle,
    Transcribing,
    Translating,
    Synthesizing,
}

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("STT error: {0}")]
    Stt(#[from] super::stt::SttError),
    #[error("Translation error: {0}")]
    Translate(#[from] super::translate::TranslateError),
    #[error("TTS error: {0}")]
    Tts(#[from] super::tts::TtsError),
    #[error("Pipeline not initialized")]
    NotInitialized,
}

pub type PipelineResult<T> = Result<T, PipelineError>;

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub source_language: Language,
    pub target_language: Language,
    pub stt_config: SttConfig,
    pub translate_config: TranslateConfig,
    pub tts_config: TtsConfig,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            source_language: Language::English,
            target_language: Language::Japanese,
            stt_config: SttConfig::default(),
            translate_config: TranslateConfig::default(),
            tts_config: TtsConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PipelineSegmentResult {
    pub transcription: TranscriptionResult,
    pub translation: TranslationResult,
    pub synthesis: SynthesisResult,
}

pub struct TranslationPipeline<R: SpeechRecognizer, T: Translator, S: TtsSynthesizer> {
    stt_engine: SttEngine<R>,
    translate_engine: TranslateEngine<T>,
    tts_engine: TtsEngine<S>,
    source_language: Language,
    target_language: Language,
    initialized: bool,
    stage: PipelineStage,
}

impl<R: SpeechRecognizer, T: Translator, S: TtsSynthesizer> TranslationPipeline<R, T, S> {
    pub fn new(
        recognizer: R,
        translator: T,
        synthesizer: S,
        config: PipelineConfig,
    ) -> Self {
        let stt_engine = SttEngine::new(recognizer, config.stt_config);
        let translate_engine = TranslateEngine::new(translator, config.translate_config);
        let tts_engine = TtsEngine::new(synthesizer, config.tts_config);

        Self {
            stt_engine,
            translate_engine,
            tts_engine,
            source_language: config.source_language,
            target_language: config.target_language,
            initialized: false,
            stage: PipelineStage::Idle,
        }
    }

    pub fn init(&mut self) -> PipelineResult<()> {
        self.stt_engine.init()?;
        self.translate_engine.init()?;
        self.tts_engine.init()?;
        self.initialized = true;
        Ok(())
    }

    pub fn process_segment(&mut self, audio_samples: &[i16]) -> PipelineResult<PipelineSegmentResult> {
        if !self.initialized {
            return Err(PipelineError::NotInitialized);
        }

        // STT
        self.stage = PipelineStage::Transcribing;
        let transcription = self.stt_engine.process_segment(audio_samples)?;

        // Translation
        self.stage = PipelineStage::Translating;
        let translation = self.translate_engine.translate(
            &transcription.text,
            self.source_language,
            self.target_language,
        )?;

        // TTS
        self.stage = PipelineStage::Synthesizing;
        let synthesis = self.tts_engine.synthesize(&translation.text)?;

        self.stage = PipelineStage::Idle;

        Ok(PipelineSegmentResult {
            transcription,
            translation,
            synthesis,
        })
    }

    pub fn shutdown(&mut self) {
        self.stt_engine.shutdown();
        self.translate_engine.shutdown();
        self.tts_engine.shutdown();
        self.initialized = false;
        self.stage = PipelineStage::Idle;
    }

    pub fn stage(&self) -> PipelineStage {
        self.stage
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn set_source_language(&mut self, lang: Language) {
        self.source_language = lang;
        self.stt_engine.set_language(Some(lang));
    }

    pub fn set_target_language(&mut self, lang: Language) {
        self.target_language = lang;
        self.tts_engine.set_language(lang);
    }

    pub fn source_language(&self) -> Language {
        self.source_language
    }

    pub fn target_language(&self) -> Language {
        self.target_language
    }

    pub fn sample_rate(&self) -> u32 {
        self.tts_engine.sample_rate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::stt::{SttConfig, SttError, SttResult, TranscriptionResult};
    use crate::pipeline::translate::{
        TranslateError, TranslateResult, TranslationRequest, TranslationResult, Translator,
    };
    use crate::pipeline::tts::{SynthesisResult, TtsError, TtsResult, TtsSynthesizer};

    // --- Mock STT ---

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

        fn with_success(mut self, text: &str, lang: Language) -> Self {
            self.response = Some(Ok(TranscriptionResult {
                text: text.to_string(),
                language: lang,
                no_speech_probability: 0.1,
            }));
            self
        }

        fn with_error(mut self, err: SttError) -> Self {
            self.response = Some(Err(err));
            self
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
                .unwrap_or(Err(SttError::TranscriptionError("No response".into())))
        }
    }

    // --- Mock Translator ---

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

        fn with_success(mut self, text: &str, src: Language, tgt: Language) -> Self {
            self.response = Some(Ok(TranslationResult {
                text: text.to_string(),
                source_language: src,
                target_language: tgt,
                score: Some(-1.5),
            }));
            self
        }

        fn with_error(mut self, err: TranslateError) -> Self {
            self.response = Some(Err(err));
            self
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

        fn translate(
            &self,
            _request: &TranslationRequest,
        ) -> TranslateResult<TranslationResult> {
            self.response
                .clone()
                .unwrap_or(Err(TranslateError::TranslationError("No response".into())))
        }
    }

    // --- Mock TTS ---

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

        fn with_success(mut self, samples: Vec<i16>, lang: Language) -> Self {
            self.response = Some(Ok(SynthesisResult {
                samples,
                sample_rate: 22050,
                language: lang,
            }));
            self
        }

        fn with_error(mut self, err: TtsError) -> Self {
            self.response = Some(Err(err));
            self
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

    // --- Helpers ---

    fn test_config() -> PipelineConfig {
        PipelineConfig {
            source_language: Language::English,
            target_language: Language::Japanese,
            stt_config: SttConfig {
                model_path: "/fake/stt/model.bin".into(),
                ..SttConfig::default()
            },
            translate_config: TranslateConfig {
                model_path: "/fake/translate/model".into(),
                ..TranslateConfig::default()
            },
            tts_config: TtsConfig {
                model_dir: "/fake/tts/models".into(),
                language: Language::Japanese,
                ..TtsConfig::default()
            },
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

    fn create_pipeline(
        recognizer: MockRecognizer,
        translator: MockTranslator,
        synthesizer: MockSynthesizer,
    ) -> TranslationPipeline<MockRecognizer, MockTranslator, MockSynthesizer> {
        TranslationPipeline::new(recognizer, translator, synthesizer, test_config())
    }

    fn success_pipeline() -> TranslationPipeline<MockRecognizer, MockTranslator, MockSynthesizer> {
        create_pipeline(
            MockRecognizer::new().with_success("Hello", Language::English),
            MockTranslator::new().with_success("こんにちは", Language::English, Language::Japanese),
            MockSynthesizer::new().with_success(vec![100, 200, 300], Language::Japanese),
        )
    }

    // --- Tests ---

    #[test]
    fn test_pipeline_init_success() {
        let mut pipeline = success_pipeline();
        assert!(pipeline.init().is_ok());
        assert!(pipeline.is_initialized());
    }

    #[test]
    fn test_pipeline_not_initialized_by_default() {
        let pipeline = success_pipeline();
        assert!(!pipeline.is_initialized());
        assert_eq!(pipeline.stage(), PipelineStage::Idle);
    }

    #[test]
    fn test_pipeline_process_segment_full() {
        let mut pipeline = success_pipeline();
        pipeline.init().unwrap();

        let result = pipeline.process_segment(&speech_samples(16000)).unwrap();
        assert_eq!(result.transcription.text, "Hello");
        assert_eq!(result.transcription.language, Language::English);
        assert_eq!(result.translation.text, "こんにちは");
        assert_eq!(result.synthesis.samples, vec![100, 200, 300]);
        assert_eq!(result.synthesis.sample_rate, 22050);
    }

    #[test]
    fn test_pipeline_rejects_without_init() {
        let mut pipeline = success_pipeline();
        let result = pipeline.process_segment(&speech_samples(16000));
        assert!(matches!(result, Err(PipelineError::NotInitialized)));
    }

    #[test]
    fn test_pipeline_stt_error_propagates() {
        let mut pipeline = create_pipeline(
            MockRecognizer::new().with_error(SttError::TranscriptionError("decode fail".into())),
            MockTranslator::new().with_success("x", Language::English, Language::Japanese),
            MockSynthesizer::new().with_success(vec![1], Language::Japanese),
        );
        pipeline.init().unwrap();

        let result = pipeline.process_segment(&speech_samples(16000));
        assert!(matches!(result, Err(PipelineError::Stt(_))));
    }

    #[test]
    fn test_pipeline_translate_error_propagates() {
        let mut pipeline = create_pipeline(
            MockRecognizer::new().with_success("Hello", Language::English),
            MockTranslator::new()
                .with_error(TranslateError::TranslationError("decode fail".into())),
            MockSynthesizer::new().with_success(vec![1], Language::Japanese),
        );
        pipeline.init().unwrap();

        let result = pipeline.process_segment(&speech_samples(16000));
        assert!(matches!(result, Err(PipelineError::Translate(_))));
    }

    #[test]
    fn test_pipeline_tts_error_propagates() {
        let mut pipeline = create_pipeline(
            MockRecognizer::new().with_success("Hello", Language::English),
            MockTranslator::new().with_success("こんにちは", Language::English, Language::Japanese),
            MockSynthesizer::new().with_error(TtsError::SynthesisError("synth fail".into())),
        );
        pipeline.init().unwrap();

        let result = pipeline.process_segment(&speech_samples(16000));
        assert!(matches!(result, Err(PipelineError::Tts(_))));
    }

    #[test]
    fn test_pipeline_shutdown() {
        let mut pipeline = success_pipeline();
        pipeline.init().unwrap();
        assert!(pipeline.is_initialized());

        pipeline.shutdown();
        assert!(!pipeline.is_initialized());
        assert_eq!(pipeline.stage(), PipelineStage::Idle);
    }

    #[test]
    fn test_pipeline_shutdown_then_process_fails() {
        let mut pipeline = success_pipeline();
        pipeline.init().unwrap();
        pipeline.shutdown();

        let result = pipeline.process_segment(&speech_samples(16000));
        assert!(matches!(result, Err(PipelineError::NotInitialized)));
    }

    #[test]
    fn test_pipeline_set_source_language() {
        let mut pipeline = success_pipeline();
        assert_eq!(pipeline.source_language(), Language::English);

        pipeline.set_source_language(Language::French);
        assert_eq!(pipeline.source_language(), Language::French);
    }

    #[test]
    fn test_pipeline_set_target_language() {
        let mut pipeline = success_pipeline();
        assert_eq!(pipeline.target_language(), Language::Japanese);

        pipeline.set_target_language(Language::German);
        assert_eq!(pipeline.target_language(), Language::German);
    }

    #[test]
    fn test_pipeline_sample_rate() {
        let pipeline = success_pipeline();
        assert_eq!(pipeline.sample_rate(), 22050);
    }

    #[test]
    fn test_pipeline_stage_idle_after_process() {
        let mut pipeline = success_pipeline();
        pipeline.init().unwrap();
        pipeline.process_segment(&speech_samples(16000)).unwrap();
        assert_eq!(pipeline.stage(), PipelineStage::Idle);
    }

    #[test]
    fn test_pipeline_default_config() {
        let config = PipelineConfig::default();
        assert_eq!(config.source_language, Language::English);
        assert_eq!(config.target_language, Language::Japanese);
    }
}
