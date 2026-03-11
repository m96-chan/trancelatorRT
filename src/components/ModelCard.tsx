import type { ModelStatusInfo } from "../types";
import { formatBytes } from "../types";

interface ModelCardProps {
  model: ModelStatusInfo;
  downloadPercent: number | undefined;
  onDownload: (id: string) => void;
  onDelete: (id: string) => void;
}

export function ModelCard({
  model,
  downloadPercent,
  onDownload,
  onDelete,
}: ModelCardProps) {
  const { info, status } = model;
  const downloading = downloadPercent !== undefined;
  const downloaded = status === "Downloaded";

  const statusLabel = downloading
    ? `Downloading ${downloadPercent}%`
    : downloaded
      ? "Downloaded"
      : "Not Downloaded";

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
          {statusLabel}
        </span>
      </div>
      {downloading && (
        <div
          className="model-progress-bar"
          role="progressbar"
          aria-valuenow={downloadPercent}
        >
          <div
            className="model-progress-fill"
            style={{ width: `${downloadPercent}%` }}
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
