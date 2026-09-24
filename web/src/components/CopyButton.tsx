import { useCallback, useEffect, useState } from "react";

interface Props {
  value: string;
  label?: string;
  className?: string;
}

export function CopyButton({ value, label = "Copy", className = "" }: Props) {
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!copied) return;
    const timer = window.setTimeout(() => setCopied(false), 1200);
    return () => window.clearTimeout(timer);
  }, [copied]);

  const copy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
    } catch {
      setCopied(false);
    }
  }, [value]);

  return (
    <button
      type="button"
      onClick={copy}
      aria-label={copied ? "Copied" : label}
      title={copied ? "Copied" : label}
      className={`shrink-0 border border-line px-1.5 py-0.5 text-[10px] uppercase tracking-wider transition-colors ${
        copied
          ? "border-ok/40 text-ok"
          : "text-muted hover:border-line-bright hover:text-ink-bright"
      } ${className}`}
    >
      {copied ? "ok" : "cp"}
    </button>
  );
}
