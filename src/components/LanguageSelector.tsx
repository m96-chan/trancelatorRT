import { LANGUAGES, LANGUAGE_LABELS } from "../types";

interface LanguageSelectorProps {
  label: string;
  value: string;
  onChange: (code: string) => void;
  disabled?: boolean;
}

export function LanguageSelector({
  label,
  value,
  onChange,
  disabled = false,
}: LanguageSelectorProps) {
  return (
    <div className="language-selector">
      <label className="language-label">{label}</label>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        disabled={disabled}
        aria-label={label}
      >
        {LANGUAGES.map((lang) => (
          <option key={lang.code} value={lang.code}>
            {LANGUAGE_LABELS[lang.code]} ({lang.code.toUpperCase()})
          </option>
        ))}
      </select>
    </div>
  );
}
