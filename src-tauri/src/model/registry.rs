use super::ModelInfo;
use super::ModelType;

pub struct ModelRegistry {
    models: Vec<ModelInfo>,
}

impl ModelRegistry {
    pub fn get(&self, id: &str) -> Option<&ModelInfo> {
        self.models.iter().find(|m| m.id == id)
    }

    pub fn list(&self) -> &[ModelInfo] {
        &self.models
    }

    pub fn by_type(&self, model_type: ModelType) -> Vec<&ModelInfo> {
        self.models
            .iter()
            .filter(|m| m.model_type == model_type)
            .collect()
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self {
            models: vec![
                ModelInfo {
                    model_type: ModelType::Whisper,
                    id: "whisper-tiny".into(),
                    display_name: "Whisper Tiny (75MB)".into(),
                    version: "1.0.0".into(),
                    url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin".into(),
                    size_bytes: 75_000_000,
                    sha256: "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21".into(),
                    filename: "ggml-tiny.bin".into(),
                },
                ModelInfo {
                    model_type: ModelType::Whisper,
                    id: "whisper-base".into(),
                    display_name: "Whisper Base (142MB, recommended)".into(),
                    version: "1.0.0".into(),
                    url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin".into(),
                    size_bytes: 142_000_000,
                    sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b7f0f4f2f7e626f477".into(),
                    filename: "ggml-base.bin".into(),
                },
                ModelInfo {
                    model_type: ModelType::Whisper,
                    id: "whisper-small".into(),
                    display_name: "Whisper Small (466MB, best quality)".into(),
                    version: "1.0.0".into(),
                    url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin".into(),
                    size_bytes: 466_000_000,
                    sha256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1571c230d4".into(),
                    filename: "ggml-small.bin".into(),
                },
                ModelInfo {
                    model_type: ModelType::Nllb,
                    id: "nllb-distilled-600m".into(),
                    display_name: "NLLB-200 Distilled 600M".into(),
                    version: "1.0.0".into(),
                    url: "https://huggingface.co/JustFrederik/nllb-200-distilled-600M-ct2-int8/resolve/main/model.bin".into(),
                    size_bytes: 600_000_000,
                    sha256: "placeholder-nllb-checksum".into(),
                    filename: "nllb-200-distilled-600M.bin".into(),
                },
                ModelInfo {
                    model_type: ModelType::Piper,
                    id: "piper-en-us-lessac".into(),
                    display_name: "Piper English (US)".into(),
                    version: "1.0.0".into(),
                    url: "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium/en_US-lessac-medium.onnx".into(),
                    size_bytes: 63_000_000,
                    sha256: "placeholder-piper-en-checksum".into(),
                    filename: "en_US-lessac-medium.onnx".into(),
                },
                // Note: Japanese Piper voice not available in rhasspy/piper-voices
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_default_has_all_model_types() {
        let registry = ModelRegistry::default();
        let types: Vec<ModelType> = registry.list().iter().map(|m| m.model_type).collect();
        assert!(types.contains(&ModelType::Whisper));
        assert!(types.contains(&ModelType::Nllb));
        assert!(types.contains(&ModelType::Piper));
    }

    #[test]
    fn test_registry_get_by_id() {
        let registry = ModelRegistry::default();
        let model = registry.get("whisper-tiny").unwrap();
        assert_eq!(model.model_type, ModelType::Whisper);
        assert_eq!(model.filename, "ggml-tiny.bin");
        assert!(model.size_bytes > 0);
    }

    #[test]
    fn test_registry_get_unknown_returns_none() {
        let registry = ModelRegistry::default();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_by_type_filters_correctly() {
        let registry = ModelRegistry::default();
        let whisper_models = registry.by_type(ModelType::Whisper);
        assert_eq!(whisper_models.len(), 3);
        assert!(whisper_models.iter().all(|m| m.model_type == ModelType::Whisper));
        let piper_models = registry.by_type(ModelType::Piper);
        assert!(piper_models.len() >= 1);
        assert!(piper_models.iter().all(|m| m.model_type == ModelType::Piper));
    }

    #[test]
    fn test_registry_all_entries_have_valid_urls() {
        let registry = ModelRegistry::default();
        for model in registry.list() {
            assert!(!model.url.is_empty(), "URL empty for {}", model.id);
            assert!(model.url.starts_with("https://"), "URL not HTTPS for {}", model.id);
        }
    }

    #[test]
    fn test_registry_all_entries_have_checksums() {
        let registry = ModelRegistry::default();
        for model in registry.list() {
            assert!(!model.sha256.is_empty(), "Checksum empty for {}", model.id);
        }
    }
}
