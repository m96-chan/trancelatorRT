import type { PipelineState } from "../types";

interface RecordButtonProps {
  pipelineState: PipelineState;
  onStart: () => void;
  onStop: () => void;
}

const STATE_LABELS: Record<PipelineState, string> = {
  Idle: "Start",
  Recording: "Stop",
  Paused: "Resume",
  Processing: "Processing...",
};

export function RecordButton({
  pipelineState,
  onStart,
  onStop,
}: RecordButtonProps) {
  const isRecording = pipelineState === "Recording";
  const isProcessing = pipelineState === "Processing";

  const handleClick = () => {
    if (isRecording) {
      onStop();
    } else if (!isProcessing) {
      onStart();
    }
  };

  return (
    <button
      className={`record-button ${isRecording ? "recording" : ""} ${isProcessing ? "processing" : ""}`}
      onClick={handleClick}
      disabled={isProcessing}
      aria-label={isRecording ? "Stop recording" : "Start recording"}
    >
      <span className="record-icon">{isRecording ? "\u25A0" : "\u25CF"}</span>
      <span className="record-label">{STATE_LABELS[pipelineState]}</span>
    </button>
  );
}
