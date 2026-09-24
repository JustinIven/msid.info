import type { LookupResponse } from "../types";

export function StatusBar({ data }: { data: LookupResponse }) {
  const { calls, durationMs, cache, degraded } = data.meta;
  const failed = calls.filter((call) => !call.ok).length;

  return (
    <div className="flex flex-wrap items-center gap-x-4 gap-y-1 border border-line bg-surface px-3 py-1.5 text-[11px] text-muted">
      <span>
        <span className="text-muted/60">time</span>{" "}
        <span className="text-ink-bright">{durationMs}ms</span>
      </span>
      <span>
        <span className="text-muted/60">cache</span>{" "}
        <span className={cache.status === "MISS" ? "text-warn" : "text-ok"}>
          {cache.status.toLowerCase()}
        </span>
      </span>
      <span>
        <span className="text-muted/60">calls</span>{" "}
        <span className={failed ? "text-danger" : "text-ok"}>
          {calls.length - failed}/{calls.length}
        </span>
      </span>
      {degraded.length > 0 && (
        <span title={degraded.join(", ")} className="truncate text-warn">
          degraded: {degraded.join(" · ")}
        </span>
      )}
    </div>
  );
}
