import type { ModelManagerState, ModelManagerActions } from "../hooks/useModelManager";
import { ModelCard } from "./ModelCard";
import { StorageBar } from "./StorageBar";

interface ModelManagerProps {
  state: ModelManagerState;
  actions: ModelManagerActions;
  onClose: () => void;
}

export function ModelManagerPanel({ state, actions, onClose }: ModelManagerProps) {
  const { models, storageInfo, loading, error } = state;

  const whisperModels = models.filter((m) => m.info.model_type === "Whisper");
  const nllbModels = models.filter((m) => m.info.model_type === "Nllb");
  const piperModels = models.filter((m) => m.info.model_type === "Piper");

  return (
    <div className="model-manager-panel" data-testid="model-manager">
      <div className="model-manager-header">
        <h2>Model Manager</h2>
        <button className="close-btn" onClick={onClose} aria-label="Close model manager">
          X
        </button>
      </div>

      <StorageBar storageInfo={storageInfo} />

      {error && (
        <div className="model-error" role="alert">
          {error}
        </div>
      )}

      {loading && <div className="model-loading">Loading models...</div>}

      {!loading && (
        <>
          <ModelSection title="Speech Recognition (Whisper)" models={whisperModels} actions={actions} />
          <ModelSection title="Translation (NLLB)" models={nllbModels} actions={actions} />
          <ModelSection title="Text-to-Speech (Piper)" models={piperModels} actions={actions} />
        </>
      )}
    </div>
  );
}

function ModelSection({
  title,
  models,
  actions,
}: {
  title: string;
  models: ModelManagerState["models"];
  actions: ModelManagerActions;
}) {
  if (models.length === 0) return null;

  return (
    <div className="model-section">
      <h3>{title}</h3>
      {models.map((model) => (
        <ModelCard
          key={model.info.id}
          model={model}
          onDownload={actions.downloadModel}
          onDelete={actions.deleteModel}
        />
      ))}
    </div>
  );
}
