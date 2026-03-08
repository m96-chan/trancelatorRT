# trancelatorRT

On-device real-time voice translation app for Android.
すべての推論をクライアントサイドで実行し、ネットワーク不要で動作する。

All inference runs entirely on-device — no network connection required.

## Overview / 概要

Captures voice input, converts it to text with Whisper, translates with NLLB, and synthesizes speech with Piper — all on-device for low-latency, privacy-preserving real-time translation.

音声入力をWhisperでテキスト化し、NLLBで翻訳、Piperで音声合成するパイプラインを端末内で完結させる。

```
🎤 Voice Input / 音声入力
    ↓
[whisper.cpp] STT (Speech → Text)
    ↓
[NLLB-200]   Translation / 翻訳 (Text → Text)
    ↓
[Piper TTS]  Speech Synthesis / 音声合成 (Text → Speech)
    ↓
🔊 Translated Voice Output / 翻訳音声出力
```

## Tech Stack / 技術スタック

| Layer | Technology | Notes / 備考 |
|-------|-----------|--------------|
| UI | **Tauri Mobile** | Rust backend + WebView (HTML/CSS/JS) |
| STT | **whisper.cpp** | Via Rust FFI bindings. NNAPI/GPU support |
| Translation / 翻訳 | **NLLB-200** (Meta) | Via CTranslate2. distilled-600M model. 200+ languages |
| TTS | **Piper TTS** | C++ impl + Rust bindings. Multilingual voice models |
| Core Language | **Rust** | Core logic, inference pipeline, all bindings |
| Frontend | **TypeScript** | UI on Tauri WebView |
| Build | **Cargo + Gradle** | Tauri Mobile build chain |
| Target | **Android** (API 26+) | Native libraries built via NDK |

## Architecture / アーキテクチャ

```
┌─────────────────────────────────────────┐
│              Android APK                │
│  ┌───────────────────────────────────┐  │
│  │         Tauri WebView (UI)        │  │
│  │    HTML / CSS / TypeScript        │  │
│  └──────────────┬────────────────────┘  │
│                 │ Tauri Commands (IPC)   │
│  ┌──────────────▼────────────────────┐  │
│  │        Rust Core Engine           │  │
│  │  ┌──────────────────────────┐     │  │
│  │  │   Audio Pipeline Manager │     │  │
│  │  │   - Audio Capture        │     │  │
│  │  │   - VAD (Voice Activity  │     │  │
│  │  │         Detection)       │     │  │
│  │  │   - Streaming Control    │     │  │
│  │  └──────────────────────────┘     │  │
│  │  ┌──────────┐ ┌────────┐ ┌─────┐ │  │
│  │  │whisper.cpp│→│ NLLB   │→│Piper│ │  │
│  │  │  (STT)   │ │(Trans.)│ │(TTS)│ │  │
│  │  └──────────┘ └────────┘ └─────┘ │  │
│  └───────────────────────────────────┘  │
│  ┌───────────────────────────────────┐  │
│  │    Native Libraries (NDK)         │  │
│  │    libwhisper / libctranslate2    │  │
│  │    libpiper / libonnxruntime      │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

## Supported Languages / 対応言語

- **Whisper (STT):** 99 languages for speech recognition / 99言語の音声認識
- **NLLB-200 (Translation):** 200+ language pairs / 200+言語間の翻訳
- **Piper (TTS):** Speech synthesis for supported languages (separate model per language) / 対応言語の音声合成（言語ごとにモデルが必要）

## Development Setup / 開発環境セットアップ

### Prerequisites / 前提条件

- Rust (stable, latest)
- Android SDK / NDK
- Node.js (20+)
- Tauri CLI (`cargo install tauri-cli`)
- Android Studio (for emulator / on-device debugging)

### Build / ビルド

```bash
# Install dependencies / 依存関係のインストール
npm install

# Development build (Android) / 開発ビルド
cargo tauri android dev

# Release build / リリースビルド
cargo tauri android build
```

## Project Structure (Planned) / プロジェクト構成（予定）

```
trancelatorRT/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── pipeline/    # Inference pipeline / 推論パイプライン
│   │   │   ├── stt.rs       # whisper.cpp wrapper
│   │   │   ├── translate.rs # NLLB wrapper
│   │   │   └── tts.rs       # Piper wrapper
│   │   ├── audio/       # Audio capture / playback
│   │   └── commands.rs  # Tauri IPC commands
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                 # Frontend (TypeScript)
│   ├── App.tsx
│   ├── components/
│   └── styles/
├── models/              # AI models (gitignored)
│   ├── whisper/
│   ├── nllb/
│   └── piper/
├── package.json
└── README.md
```

## Model Sizes / モデルサイズ目安

| Model | Size | Notes / 備考 |
|-------|------|--------------|
| Whisper tiny | ~75MB | Low accuracy, fast / 低精度だが高速 |
| Whisper base | ~150MB | Balanced / バランス型 |
| Whisper small | ~500MB | High accuracy / 高精度 |
| NLLB distilled-600M | ~1.2GB | Can be reduced with CTranslate2 quantization / 量子化で縮小可能 |
| Piper (per language) | ~60MB | Additional model per language / 言語ごとに追加 |

## License / ライセンス

TBD

## Notes / 注意事項

- iOS is not supported (no plans to support) / iOS非対応（対応予定なし）
- Requires sufficient storage and RAM to run all models on-device / 全モデルをオンデバイスで実行するため、十分なストレージとRAMが必要
- Models must be downloaded on first launch / 初回起動時にモデルのダウンロードが必要
