export interface LanguageInfo {
  code: string;
  name: string;
}

export interface LanguageSettings {
  source: string;
  target: string;
}

export type PipelineState = "Idle" | "Recording" | "Paused" | "Processing";

export const LANGUAGES: LanguageInfo[] = [
  { code: "ja", name: "Japanese" },
  { code: "ko", name: "Korean" },
  { code: "en", name: "English" },
  { code: "fr", name: "French" },
  { code: "de", name: "German" },
  { code: "pt", name: "Portuguese" },
  { code: "ru", name: "Russian" },
  { code: "ar", name: "Arabic" },
];

export const LANGUAGE_LABELS: Record<string, string> = {
  ja: "日本語",
  ko: "한국어",
  en: "English",
  fr: "Français",
  de: "Deutsch",
  pt: "Português",
  ru: "Русский",
  ar: "العربية",
};

// Model management types

export type ModelType = "Whisper" | "Nllb" | "Piper";

export interface ModelInfo {
  model_type: ModelType;
  id: string;
  display_name: string;
  version: string;
  url: string;
  size_bytes: number;
  sha256: string;
  filename: string;
}

export type ModelStatus =
  | "NotDownloaded"
  | { Downloading: { progress_percent: number } }
  | "Downloaded";

export interface ModelStatusInfo {
  info: ModelInfo;
  status: ModelStatus;
  local_path: string | null;
}

export interface StorageInfo {
  total_bytes: number;
  available_bytes: number;
  models_bytes: number;
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

export function getModelStatusLabel(status: ModelStatus): string {
  if (status === "NotDownloaded") return "Not Downloaded";
  if (status === "Downloaded") return "Downloaded";
  if (typeof status === "object" && "Downloading" in status)
    return `Downloading ${status.Downloading.progress_percent}%`;
  return "Unknown";
}

export function isDownloading(status: ModelStatus): boolean {
  return typeof status === "object" && "Downloading" in status;
}
