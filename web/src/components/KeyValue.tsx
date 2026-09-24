import type { ReactNode } from "react";
import { CopyButton } from "./CopyButton";

export interface Row {
  key: string;
  value: unknown;
  hint?: string;
  render?: ReactNode;
}

function isUrl(value: string): boolean {
  return value.startsWith("https://") || value.startsWith("http://");
}

function ValueCell({ value }: { value: unknown }) {
  if (value === null || value === undefined || value === "") {
    return <span className="text-muted/60">null</span>;
  }
  if (typeof value === "boolean") {
    return (
      <span className={value ? "text-ok" : "text-danger"}>{String(value)}</span>
    );
  }
  if (typeof value === "number") {
    return <span className="text-magenta">{value}</span>;
  }
  const text = String(value);
  if (isUrl(text)) {
    return (
      <a
        href={text}
        target="_blank"
        rel="noreferrer noopener"
        className="break-all text-accent underline decoration-accent/30 underline-offset-2 hover:decoration-accent"
      >
        {text}
      </a>
    );
  }
  return <span className="break-all text-ink-bright">{text}</span>;
}

/// Dense two-column grid. Every non-empty value gets its own copy affordance.
export function KeyValue({ rows }: { rows: Row[] }) {
  return (
    <dl className="divide-y divide-line/60">
      {rows.map((row) => {
        const copyable =
          row.value !== null && row.value !== undefined && row.value !== "";
        return (
          <div
            key={row.key}
            className="group grid grid-cols-[minmax(0,13rem)_1fr_auto] items-start gap-3 px-3 py-1.5 hover:bg-surface-2/60"
          >
            <dt className="truncate text-muted" title={row.hint ?? row.key}>
              {row.key}
            </dt>
            <dd className="min-w-0">
              {row.render ?? <ValueCell value={row.value} />}
            </dd>
            <div className="opacity-0 transition-opacity group-hover:opacity-100 focus-within:opacity-100">
              {copyable ? <CopyButton value={String(row.value)} /> : null}
            </div>
          </div>
        );
      })}
    </dl>
  );
}
