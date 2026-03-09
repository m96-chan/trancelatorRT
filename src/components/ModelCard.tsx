import type { ModelStatusInfo } from "../types";
import { formatBytes, getModelStatusLabel, isDownloading } from "../types";

interface ModelCardProps {
  model: ModelStatusInfo;
  onDownload: (id: string) => void;
  onDelete: (id: string) => void;
}

export function ModelCard({ model, onDownload, onDelete }: ModelCardProps) {
  const { info, status } = model;
  const downloading = isDownloading(status);
  const downloaded = status === "Downloaded";
  const progressPercent =
    typeof status === "object" && "Downloading" in status
      ? status.Downloading.progress_percent
      : 0;

  return (
    <div className="model-card" data-testid={`model-card-${info.id}`}>
      <div className="model-card-header">
        <span className="model-name">{info.display_name}</span>
        <span className="model-type-badge">{info.model_type}</span>
      </div>
      <div className="model-card-details">
        <span className="model-size">{formatBytes(info.size_bytes)}</span>
        <span className="model-version">v{info.version}</span>
      </div>
      <div className="model-card-status">
        <span
          className={`model-status ${downloaded ? "status-ok" : ""}`}
          data-testid={`model-status-${info.id}`}
        >
          {getModelStatusLabel(status)}
        </span>
      </div>
      {downloading && (
        <div className="model-progress-bar" role="progressbar" aria-valuenow={progressPercent}>
          <div
            className="model-progress-fill"
            style={{ width: `${progressPercent}%` }}
          />
        </div>
      )}
      <div className="model-card-actions">
        {!downloaded && !downloading && (
          <button
            className="model-btn download-btn"
            onClick={() => onDownload(info.id)}
            aria-label={`Download ${info.display_name}`}
          >
            Download
          </button>
        )}
        {downloaded && (
          <button
            className="model-btn delete-btn"
            onClick={() => onDelete(info.id)}
            aria-label={`Delete ${info.display_name}`}
          >
            Delete
          </button>
        )}
      </div>
    </div>
  );
}
