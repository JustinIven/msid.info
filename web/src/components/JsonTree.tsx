import { useState } from "react";
import { CopyButton } from "./CopyButton";

type Json = unknown;

function typeOf(value: Json): string {
  if (value === null) return "null";
  if (Array.isArray(value)) return "array";
  return typeof value;
}

function Scalar({ value }: { value: Json }) {
  const kind = typeOf(value);
  if (kind === "null") return <span className="text-muted/70">null</span>;
  if (kind === "boolean")
    return <span className={value ? "text-ok" : "text-danger"}>{String(value)}</span>;
  if (kind === "number") return <span className="text-magenta">{String(value)}</span>;
  return <span className="break-all text-accent">"{String(value)}"</span>;
}

interface NodeProps {
  name: string | null;
  value: Json;
  depth: number;
  defaultOpen: boolean;
}

function Node({ name, value, depth, defaultOpen }: NodeProps) {
  const [open, setOpen] = useState(defaultOpen);
  const kind = typeOf(value);
  const isBranch = kind === "object" || kind === "array";

  if (!isBranch) {
    return (
      <div className="flex gap-2" style={{ paddingLeft: depth * 14 }}>
        {name !== null && <span className="text-ink">{name}:</span>}
        <Scalar value={value} />
      </div>
    );
  }

  const entries = Array.isArray(value)
    ? value.map((item, index) => [String(index), item] as const)
    : Object.entries(value as Record<string, Json>);
  const open_ = kind === "array" ? "[" : "{";
  const close = kind === "array" ? "]" : "}";

  return (
    <div style={{ paddingLeft: depth * 14 }}>
      <button
        type="button"
        onClick={() => setOpen((previous) => !previous)}
        aria-expanded={open}
        className="group flex items-center gap-2 text-left hover:text-ink-bright"
      >
        <span className="w-3 shrink-0 text-muted">{open ? "▾" : "▸"}</span>
        {name !== null && <span className="text-ink">{name}:</span>}
        <span className="text-muted">
          {open_}
          {open ? "" : ` ${entries.length} ${close}`}
        </span>
      </button>
      {open && (
        <>
          {entries.map(([key, child]) => (
            <Node
              key={key}
              name={key}
              value={child}
              depth={depth + 1}
              defaultOpen={depth < 1}
            />
          ))}
          <div className="text-muted" style={{ paddingLeft: 14 }}>
            {close}
          </div>
        </>
      )}
    </div>
  );
}

export function JsonTree({ data }: { data: Json }) {
  return (
    <div className="relative">
      <div className="absolute right-2 top-2 z-10">
        <CopyButton value={JSON.stringify(data, null, 2)} label="Copy JSON" />
      </div>
      <pre className="overflow-x-auto p-3 text-[12px] leading-[1.7]">
        <Node name={null} value={data} depth={0} defaultOpen />
      </pre>
    </div>
  );
}
