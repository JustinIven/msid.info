import { useMemo, useState } from "react";
import { CopyButton } from "./CopyButton";
import { Panel } from "./Panel";

interface Props {
  domains: string[];
  onSelect: (domain: string) => void;
}

export function DomainList({ domains, onSelect }: Props) {
  const [filter, setFilter] = useState("");

  const visible = useMemo(() => {
    const needle = filter.trim().toLowerCase();
    return needle ? domains.filter((domain) => domain.includes(needle)) : domains;
  }, [domains, filter]);

  return (
    <Panel
      title={`domains · ${domains.length}`}
      actions={<CopyButton value={domains.join("\n")} label="Copy all domains" />}
    >
      <div className="border-b border-line px-3 py-1.5">
        <label htmlFor="domain-filter" className="sr-only">
          Filter domains
        </label>
        <input
          id="domain-filter"
          value={filter}
          onChange={(event) => setFilter(event.target.value)}
          placeholder="filter…"
          className="w-full bg-transparent text-ink-bright outline-none placeholder:text-muted/50"
        />
      </div>

      {visible.length === 0 ? (
        <p className="px-3 py-6 text-center text-muted">
          {domains.length === 0
            ? "Autodiscover returned no domains for this tenant."
            : "No domain matches the filter."}
        </p>
      ) : (
        <ul className="max-h-[28rem] divide-y divide-line/60 overflow-y-auto">
          {visible.map((domain) => (
            <li key={domain} className="group flex items-center gap-2 px-3 py-1">
              <button
                type="button"
                onClick={() => onSelect(domain)}
                title={`Look up ${domain}`}
                className="min-w-0 flex-1 truncate text-left text-ink-bright hover:text-accent"
              >
                {domain}
              </button>
              <div className="opacity-0 transition-opacity group-hover:opacity-100 focus-within:opacity-100">
                <CopyButton value={domain} />
              </div>
            </li>
          ))}
        </ul>
      )}
    </Panel>
  );
}
