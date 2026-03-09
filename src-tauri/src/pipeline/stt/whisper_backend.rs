use super::{
    audio_i16_to_f32, Language, SpeechRecognizer, SttConfig, SttError, SttResult,
    TranscriptionResult,
};
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

pub struct WhisperRecognizer {
    context: Option<WhisperContext>,
    state: Option<WhisperState>,
}

// Safety: WhisperContext/WhisperState contain raw pointers to C++ objects,
// but we only access them through &mut self behind a Mutex.
unsafe impl Send for WhisperRecognizer {}

impl WhisperRecognizer {
    pub fn new() -> Self {
        Self {
            context: None,
            state: None,
        }
    }
}

impl SpeechRecognizer for WhisperRecognizer {
    fn load_model(&mut self, path: &str) -> SttResult<()> {
        let params = WhisperContextParameters::default();

        let ctx = WhisperContext::new_with_params(path, params)
            .map_err(|e| SttError::ModelLoadError(format!("{e}")))?;

        let state = ctx
            .create_state()
            .map_err(|e| SttError::ModelLoadError(format!("Failed to create state: {e}")))?;

        self.context = Some(ctx);
        self.state = Some(state);
        Ok(())
    }

    fn unload_model(&mut self) {
        self.state = None;
        self.context = None;
    }

    fn is_model_loaded(&self) -> bool {
        self.context.is_some() && self.state.is_some()
    }

    fn transcribe(
        &mut self,
        samples: &[i16],
        config: &SttConfig,
    ) -> SttResult<TranscriptionResult> {
        let state = self.state.as_mut().ok_or(SttError::ModelNotLoaded)?;

        let audio_f32 = audio_i16_to_f32(samples);

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        match &config.language {
            Some(lang) => params.set_language(Some(lang.whisper_code())),
            None => params.set_language(None),
        }

        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_single_segment(true);
        params.set_n_threads(config.n_threads as i32);

        state
            .full(params, &audio_f32)
            .map_err(|e| SttError::TranscriptionError(format!("{e}")))?;

        let n_segments = state.full_n_segments();
        if n_segments == 0 {
            return Err(SttError::NoSpeechDetected);
        }

        let lang_id = state.full_lang_id_from_state();
        let lang_str = whisper_rs::get_lang_str(lang_id).unwrap_or("en");
        let language = Language::from_whisper_code(lang_str)
            .ok_or_else(|| SttError::UnsupportedLanguage(lang_str.to_string()))?;

        let mut text = String::new();
        let mut max_no_speech_prob: f32 = 0.0;

        for i in 0..n_segments {
            if let Some(segment) = state.get_segment(i) {
                match segment.to_str() {
                    Ok(s) => text.push_str(s),
                    Err(_) => {
                        if let Ok(s) = segment.to_str_lossy() {
                            text.push_str(&s);
                        }
                    }
                }
                let prob = segment.no_speech_probability();
                if prob > max_no_speech_prob {
                    max_no_speech_prob = prob;
                }
            }
        }

        Ok(TranscriptionResult {
            text: text.trim().to_string(),
            language,
            no_speech_probability: max_no_speech_prob,
        })
    }
}
