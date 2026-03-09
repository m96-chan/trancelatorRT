import { invoke } from "@tauri-apps/api/core";
import { LanguageSelector } from "./components/LanguageSelector";
import { RecordButton } from "./components/RecordButton";
import { TextDisplay } from "./components/TextDisplay";
import { PipelineStatus } from "./components/PipelineStatus";
import { useTranslation } from "./hooks/useTranslation";
import { LANGUAGE_LABELS } from "./types";

function App() {
  const [state, actions] = useTranslation(invoke);

  return (
    <main>
      <h1>trancelatorRT</h1>

      <div className="language-bar">
        <LanguageSelector
          label="Source"
          value={state.sourceLanguage}
          onChange={actions.setSourceLanguage}
          disabled={state.pipelineState === "Recording"}
        />
        <button
          className="swap-button"
          onClick={actions.swapLanguages}
          disabled={state.pipelineState === "Recording"}
          aria-label="Swap languages"
        >
          &#8646;
        </button>
        <LanguageSelector
          label="Target"
          value={state.targetLanguage}
          onChange={actions.setTargetLanguage}
          disabled={state.pipelineState === "Recording"}
        />
      </div>

      <PipelineStatus state={state.pipelineState} error={state.error} />

      <RecordButton
        pipelineState={state.pipelineState}
        onStart={actions.startRecording}
        onStop={actions.stopRecording}
      />

      <div className="text-panels">
        <TextDisplay
          label="Recognized"
          text={state.transcriptionText}
          language={state.sourceLanguage}
        />
        <TextDisplay
          label="Translated"
          text={state.translationText}
          language={state.targetLanguage}
        />
      </div>

      <div className="language-hint">
        {LANGUAGE_LABELS[state.sourceLanguage]} → {LANGUAGE_LABELS[state.targetLanguage]}
      </div>
    </main>
  );
}

export default App;
