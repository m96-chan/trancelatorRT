/// NLLB-200 translation backend
///
/// TODO: Implement with ct2rs or ort (ONNX Runtime) when build issues are resolved.
/// The Translator trait abstraction allows swapping backends without changing
/// the rest of the codebase.
///
/// Candidate backends:
/// - ct2rs (CTranslate2) -- sentencepiece-sys build currently broken
/// - ort (ONNX Runtime) + NLLB ONNX model
/// - rust-bert with ONNX feature

use super::{TranslateError, TranslateResult, TranslationRequest, TranslationResult, Translator};

pub struct NllbTranslator {
    loaded: bool,
}

unsafe impl Send for NllbTranslator {}

impl NllbTranslator {
    pub fn new() -> Self {
        Self { loaded: false }
    }
}

impl Translator for NllbTranslator {
    fn load_model(&mut self, path: &str) -> TranslateResult<()> {
        if path.is_empty() {
            return Err(TranslateError::ModelLoadError("Empty path".into()));
        }
        // TODO: Load actual NLLB model
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
        Err(TranslateError::TranslationError(
            "NLLB backend not yet implemented".into(),
        ))
    }
}
