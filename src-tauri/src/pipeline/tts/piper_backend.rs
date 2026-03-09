/// Piper TTS backend
///
/// TODO: Implement with piper-rs when espeak-ng cross-compilation is resolved.
/// The TtsSynthesizer trait abstraction allows swapping backends.
///
/// Candidate backends:
/// - piper-rs (ONNX + espeak-ng phonemizer)
/// - Direct ort (ONNX Runtime) with Piper ONNX models

use super::{SynthesisResult, TtsError, TtsResult, TtsSynthesizer};
use crate::pipeline::stt::Language;

pub struct PiperSynthesizer {
    loaded: bool,
    current_sample_rate: u32,
}

unsafe impl Send for PiperSynthesizer {}

impl PiperSynthesizer {
    pub fn new() -> Self {
        Self {
            loaded: false,
            current_sample_rate: 22050,
        }
    }
}

impl TtsSynthesizer for PiperSynthesizer {
    fn load_model(&mut self, config_path: &str) -> TtsResult<()> {
        if config_path.is_empty() {
            return Err(TtsError::ModelLoadError("Empty path".into()));
        }
        // TODO: Load actual Piper model
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
        Err(TtsError::SynthesisError(
            "Piper backend not yet implemented".into(),
        ))
    }

    fn sample_rate(&self) -> u32 {
        self.current_sample_rate
    }
}
