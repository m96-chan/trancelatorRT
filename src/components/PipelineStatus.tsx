import type { PipelineState } from "../types";

interface PipelineStatusProps {
  state: PipelineState;
  error: string | null;
}

const STATUS_DISPLAY: Record<PipelineState, { label: string; className: string }> = {
  Idle: { label: "Ready", className: "status-idle" },
  Recording: { label: "Listening...", className: "status-recording" },
  Paused: { label: "Paused", className: "status-paused" },
  Processing: { label: "Processing...", className: "status-processing" },
};

export function PipelineStatus({ state, error }: PipelineStatusProps) {
  const { label, className } = STATUS_DISPLAY[state];

  return (
    <div className="pipeline-status">
      <span className={`status-indicator ${className}`} data-testid="pipeline-status">
        {label}
      </span>
      {error && (
        <div className="status-error" role="alert">
          {error}
        </div>
      )}
    </div>
  );
}
