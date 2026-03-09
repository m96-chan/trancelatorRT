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
- **Piper (TTS):** Speech synthesis for major languages / 主要言語の音声合成（言語ごとにモデルが必要）

### Target Languages / 対応予定言語

| Language / 言語 | STT | Translation / 翻訳 | TTS |
|----------------|-----|-------------------|-----|
| Japanese / 日本語 | ✅ | ✅ | ✅ |
| Korean / 韓国語 | ✅ | ✅ | ✅ |
| English / 英語 | ✅ | ✅ | ✅ |
| French / フランス語 | ✅ | ✅ | ✅ |
| German / ドイツ語 | ✅ | ✅ | ✅ |
| Portuguese / ポルトガル語 | ✅ | ✅ | ✅ |
| Russian / ロシア語 | ✅ | ✅ | ✅ |
| Arabic / アラビア語 | ✅ | ✅ | ✅ |

## Development Setup / 開発環境セットアップ

### Prerequisites / 前提条件

- Rust (stable, latest)
- Node.js (20+)
- Android Studio
- JDK 17

### 1. Install Rust / Rustのインストール

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add aarch64-linux-android    # ARM64 (実機用)
rustup target add x86_64-linux-android     # エミュレータ用 (必要に応じて)
```

### 2. Install Android Studio / Android Studioのインストール

**Arch Linux:**
```bash
yay -S android-studio
```

Android Studio初回起動時に Standard セットアップを選択。
Settings → Languages & Frameworks → Android SDK → SDK Tools で以下にチェック:
- Android SDK Build-Tools
- Android SDK Command-line Tools
- **NDK (Side by side)**
- Android SDK Platform-Tools

### 3. Install JDK 17

**Arch Linux:**
```bash
sudo pacman -S jdk17-openjdk
sudo archlinux-java set java-17-openjdk
```

### 4. Environment Variables / 環境変数の設定

`~/.bashrc` or `~/.zshrc` に追加:

```bash
export JAVA_HOME=/usr/lib/jvm/java-17-openjdk
export ANDROID_HOME=$HOME/Android/Sdk
export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/<version>  # ls ~/Android/Sdk/ndk/ で確認
export PATH=$PATH:$ANDROID_HOME/platform-tools
```

```bash
source ~/.bashrc
```

### 5. Project Setup / プロジェクトセットアップ

```bash
git clone https://github.com/m96-chan/trancelatorRT.git
cd trancelatorRT

# Install dependencies / 依存関係のインストール
npm install

# Initialize Android target / Android初期化
npx tauri android init
```

### 6. Run / 実行

Android Studio でエミュレータを起動するか、実機をUSBデバッグで接続してから:

```bash
# Development / 開発ビルド
npx tauri android dev

# Release / リリースビルド
npx tauri android build
```

> **Note:** `npx tauri android dev` はエミュレータまたは実機が接続されていないと起動しません。
> `adb devices` でデバイスが表示されることを確認してください。

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
