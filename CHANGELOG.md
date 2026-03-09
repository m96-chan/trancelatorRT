# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-09

### Added
- Audio pipeline: capture, VAD, ring buffer, speech segmentation, playback
- STT module with Whisper backend (whisper-rs) and SpeechRecognizer trait
- Translation module with NLLB backend stub and Translator trait
- TTS module with Piper backend stub and TtsSynthesizer trait
- E2E pipeline orchestrator: STT -> Translation -> TTS
- Frontend UI: language selection, recording control, text display, pipeline status
- Model management: registry, download, storage lifecycle, capacity check
- Tauri IPC commands for audio control, language settings, model management
- GitHub Actions CI/CD: test pipeline and release automation
- Support for 8 languages: JA, KO, EN, FR, DE, PT, RU, AR
