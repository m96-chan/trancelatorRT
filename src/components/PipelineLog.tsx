import { useEffect, useRef } from "react";

interface LogEntry {
  stage: string;
  message: string;
  timestamp: number;
}

interface PipelineLogProps {
  entries: LogEntry[];
}

const STAGE_COLORS: Record<string, string> = {
  init: "#1976d2",
  vad: "#7b1fa2",
  stt: "#2e7d32",
  translate: "#e65100",
  emit: "#546e7a",
  error: "#d32f2f",
};

export type { LogEntry };

export function PipelineLog({ entries }: PipelineLogProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [entries.length]);

  if (entries.length === 0) {
    return (
      <div className="pipeline-log">
        <div className="pipeline-log-header">Pipeline Log</div>
        <div className="pipeline-log-empty">Waiting for activity...</div>
      </div>
    );
  }

  return (
    <div className="pipeline-log">
      <div className="pipeline-log-header">Pipeline Log</div>
      <div className="pipeline-log-entries" ref={scrollRef}>
        {entries.map((entry, i) => {
          const elapsed =
            i === 0
              ? 0
              : ((entry.timestamp - entries[0].timestamp) / 1000).toFixed(1);
          return (
            <div key={i} className="pipeline-log-entry">
              <span className="log-time">+{elapsed}s</span>
              <span
                className="log-stage"
                style={{ color: STAGE_COLORS[entry.stage] || "#666" }}
              >
                [{entry.stage}]
              </span>
              <span className="log-message">{entry.message}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
