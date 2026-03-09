import type { StorageInfo } from "../types";
import { formatBytes } from "../types";

interface StorageBarProps {
  storageInfo: StorageInfo | null;
}

export function StorageBar({ storageInfo }: StorageBarProps) {
  if (!storageInfo) {
    return <div className="storage-bar" data-testid="storage-bar">Loading...</div>;
  }

  const usedPercent =
    storageInfo.total_bytes > 0
      ? (storageInfo.models_bytes / storageInfo.total_bytes) * 100
      : 0;

  return (
    <div className="storage-bar" data-testid="storage-bar">
      <div className="storage-label">
        Models: {formatBytes(storageInfo.models_bytes)} | Free:{" "}
        {formatBytes(storageInfo.available_bytes)}
      </div>
      <div className="storage-track">
        <div
          className="storage-fill"
          style={{ width: `${Math.min(usedPercent, 100)}%` }}
          data-testid="storage-fill"
        />
      </div>
    </div>
  );
}
