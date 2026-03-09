interface TextDisplayProps {
  label: string;
  text: string;
  language: string;
}

export function TextDisplay({ label, text, language }: TextDisplayProps) {
  return (
    <div className="text-display" lang={language}>
      <div className="text-display-label">{label}</div>
      <div className="text-display-content" data-testid={`text-${label.toLowerCase().replace(/\s+/g, "-")}`}>
        {text || <span className="text-placeholder">---</span>}
      </div>
    </div>
  );
}
