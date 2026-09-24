import type { LookupResponse, UpstreamCall } from "../types";
import { JsonTree } from "./JsonTree";
import { useState } from "react";

const RAW_KEYS = [
  "odc_federationprovider",
  "userrealm_v1",
  "userrealm_v2",
  "userrealm_old",
  "credential_type",
  "autodiscover_federation_information",
  "openid_configuration",
  "tenant_information",
] as const;

/// Maps a `raw` key to the diagnostics recorded for the call that produced it.
function callFor(calls: UpstreamCall[], key: string): UpstreamCall | undefined {
  return calls.find((call) => call.name === key);
}

export function RawPane({ data }: { data: LookupResponse }) {
  const [active, setActive] = useState<string>(RAW_KEYS[0]);
  const payload = data.raw[active];
  const call = callFor(data.meta.calls, active);

  return (
    <div className="grid gap-3 lg:grid-cols-[16rem_1fr]">
      <nav className="panel h-fit" aria-label="Upstream endpoints">
        <ul className="divide-y divide-line/60">
          {RAW_KEYS.map((key) => {
            const entry = callFor(data.meta.calls, key);
            const selected = key === active;
            return (
              <li key={key}>
                <button
                  type="button"
                  onClick={() => setActive(key)}
                  aria-current={selected}
                  className={`flex w-full items-center gap-2 px-3 py-1.5 text-left ${
                    selected ? "bg-surface-2 text-ink-bright" : "text-muted hover:text-ink"
                  }`}
                >
                  <span
                    aria-hidden
                    className={
                      entry === undefined
                        ? "text-muted/40"
                        : entry.ok
                          ? "text-ok"
                          : "text-danger"
                    }
                  >
                    ●
                  </span>
                  <span className="min-w-0 flex-1 truncate text-[12px]">{key}</span>
                  {entry && (
                    <span className="shrink-0 text-[10px] text-muted">{entry.durationMs}ms</span>
                  )}
                </button>
              </li>
            );
          })}
        </ul>
      </nav>

      <div className="panel min-w-0">
        <header className="panel-header">
          <span className="text-accent">{call?.method ?? "—"}</span>
          <span className="min-w-0 flex-1 truncate normal-case tracking-normal text-muted">
            {call?.url ?? "not called"}
          </span>
          {call?.status != null && (
            <span className={call.ok ? "text-ok" : "text-danger"}>{call.status}</span>
          )}
        </header>
        {call?.error && (
          <p role="status" className="border-b border-line px-3 py-1.5 text-danger">
            {call.error}
          </p>
        )}
        {payload === undefined || payload === null ? (
          <p className="px-3 py-6 text-center text-muted">
            This endpoint was not queried for this tenant.
          </p>
        ) : (
          <JsonTree data={payload} />
        )}
      </div>
    </div>
  );
}
