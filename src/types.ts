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
