import { CopyButton } from "./CopyButton";

export function CurlSnippet({ query }: { query: string }) {
  const origin = typeof window === "undefined" ? "https://msid.info" : window.location.origin;
  const command = `curl -s ${origin}/api/v1/lookup/${encodeURIComponent(query)} | jq`;

  return (
    <div className="flex items-center gap-2 border border-line bg-surface px-3 py-1.5">
      <code className="min-w-0 flex-1 overflow-x-auto whitespace-nowrap text-[11px] text-muted">
        <span className="text-ok">$ </span>
        {command}
      </code>
      <CopyButton value={command} label="Copy curl command" />
    </div>
  );
}
