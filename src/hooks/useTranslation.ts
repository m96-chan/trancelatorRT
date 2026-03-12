import { useState, useCallback, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import type { PipelineState } from "../types";

export interface LogEntry {
  stage: string;
  message: string;
  timestamp: number;
}

export interface TranslationState {
  sourceLanguage: string;
  targetLanguage: string;
  pipelineState: PipelineState;
  transcriptionText: string;
  translationText: string;
  error: string | null;
  pipelineLogs: LogEntry[];
}

export interface TranslationActions {
  setSourceLanguage: (code: string) => void;
  setTargetLanguage: (code: string) => void;
  swapLanguages: () => void;
  startRecording: () => Promise<void>;
  stopRecording: () => Promise<void>;
  clearTexts: () => void;
}

interface TauriInvoke {
  (cmd: string, args?: Record<string, unknown>): Promise<unknown>;
}

interface TranscriptionResult {
  recognized: string;
  translated: string;
}

interface PipelineLogEvent {
  stage: string;
  message: string;
}

const MAX_LOG_ENTRIES = 100;

export function useTranslation(
  invoke: TauriInvoke,
): [TranslationState, TranslationActions] {
  const [sourceLanguage, setSourceLang] = useState("en");
  const [targetLanguage, setTargetLang] = useState("ja");
  const [pipelineState, setPipelineState] = useState<PipelineState>("Idle");
  const [transcriptionText, setTranscriptionText] = useState("");
  const [translationText, setTranslationText] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [pipelineLogs, setPipelineLogs] = useState<LogEntry[]>([]);
  const logsRef = useRef<LogEntry[]>([]);

  useEffect(() => {
    const unlistenLog = listen<PipelineLogEvent>("pipeline-log", (event) => {
      const entry: LogEntry = {
        stage: event.payload.stage,
        message: event.payload.message,
        timestamp: Date.now(),
      };
      const updated = [...logsRef.current, entry].slice(-MAX_LOG_ENTRIES);
      logsRef.current = updated;
      setPipelineLogs(updated);
    });
    return () => {
      unlistenLog.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    const unlisten = listen<TranscriptionResult>(
      "transcription-result",
      (event) => {
        const { recognized, translated } = event.payload;
        setTranscriptionText((prev) =>
          prev ? prev + "\n" + recognized : recognized,
        );
        setTranslationText((prev) =>
          prev ? prev + "\n" + translated : translated,
        );
      },
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const setSourceLanguage = useCallback(
    (code: string) => {
      setSourceLang(code);
      invoke("set_source_language", { code }).catch((e) =>
        setError(String(e)),
      );
    },
    [invoke],
  );

  const setTargetLanguage = useCallback(
    (code: string) => {
      setTargetLang(code);
      invoke("set_target_language", { code }).catch((e) =>
        setError(String(e)),
      );
    },
    [invoke],
  );

  const swapLanguages = useCallback(() => {
    setSourceLang((prev) => {
      setTargetLang((prevTarget) => {
        invoke("set_source_language", { code: prevTarget }).catch((e) =>
          setError(String(e)),
        );
        invoke("set_target_language", { code: prev }).catch((e) =>
          setError(String(e)),
        );
        return prev;
      });
      return targetLanguage;
    });
  }, [invoke, targetLanguage]);

  const startRecording = useCallback(async () => {
    try {
      setError(null);
      setTranscriptionText("");
      setTranslationText("");
      logsRef.current = [];
      setPipelineLogs([]);
      await invoke("start_recording");
      setPipelineState("Recording");
    } catch (e) {
      setError(String(e));
    }
  }, [invoke]);

  const stopRecording = useCallback(async () => {
    try {
      await invoke("stop_recording");
      setPipelineState("Idle");
    } catch (e) {
      setError(String(e));
    }
  }, [invoke]);

  const clearTexts = useCallback(() => {
    setTranscriptionText("");
    setTranslationText("");
    setError(null);
  }, []);

  const state: TranslationState = {
    sourceLanguage,
    targetLanguage,
    pipelineState,
    transcriptionText,
    translationText,
    error,
    pipelineLogs,
  };

  const actions: TranslationActions = {
    setSourceLanguage,
    setTargetLanguage,
    swapLanguages,
    startRecording,
    stopRecording,
    clearTexts,
  };

  return [state, actions];
}
