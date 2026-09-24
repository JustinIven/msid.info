import type { ReactNode } from "react";

export type Tone = "neutral" | "accent" | "ok" | "warn" | "danger" | "magenta";

const TONES: Record<Tone, string> = {
  neutral: "border-line-bright text-muted",
  accent: "border-accent/40 text-accent",
  ok: "border-ok/40 text-ok",
  warn: "border-warn/40 text-warn",
  danger: "border-danger/40 text-danger",
  magenta: "border-magenta/40 text-magenta",
};

interface Props {
  tone?: Tone;
  children: ReactNode;
  title?: string;
}

export function Badge({ tone = "neutral", children, title }: Props) {
  return (
    <span
      title={title}
      className={`inline-flex items-center border px-1.5 py-px text-[10px] uppercase tracking-[0.1em] ${TONES[tone]}`}
    >
      {children}
    </span>
  );
}
