import { Panel } from "./Panel";

const EXAMPLES: Array<{ query: string; note: string }> = [
  { query: "microsoft.com", note: "Worldwide" },
  { query: "visa.com", note: "Worldwide, federated" },
  { query: "gd.com", note: "US Gov GCC High" },
  { query: "spaceforce.mil", note: "US Gov DoD" },
  { query: "21vianet.com", note: "21Vianet (China)" },
];

export function EmptyState({ onPick }: { onPick: (query: string) => void }) {
  return (
    <div className="grid gap-3 lg:grid-cols-2">
      <Panel title="what this does">
        <div className="space-y-3 px-3 py-3 text-muted">
          <p>
            Give it a domain or a user principal name. It asks the public Microsoft identity
            endpoints who owns that namespace and folds every answer into one JSON document:
            tenant ID, sovereign cloud, federation setup, sign-in branding, supported credential
            types and every domain verified on the tenant.
          </p>
          <p>
            Every request is passive and unauthenticated — the same calls your browser makes on
            the way to a Microsoft sign-in page. Nothing is logged against the target tenant and
            no credential is ever submitted.
          </p>
        </div>
      </Panel>

      <Panel title="try one">
        <ul className="divide-y divide-line/60">
          {EXAMPLES.map((example) => (
            <li key={example.query}>
              <button
                type="button"
                onClick={() => onPick(example.query)}
                className="flex w-full items-center gap-3 px-3 py-1.5 text-left hover:bg-surface-2/60"
              >
                <span className="text-ok">$</span>
                <span className="flex-1 text-ink-bright">{example.query}</span>
                <span className="text-[11px] text-muted">{example.note}</span>
              </button>
            </li>
          ))}
        </ul>
      </Panel>

      <Panel title="api">
        <div className="space-y-2 px-3 py-3">
          <pre className="overflow-x-auto text-muted">
            <span className="text-ok">$ </span>curl -s https://msid.info/api/v1/lookup/contoso.com
          </pre>
          <p className="text-muted">
            No key, no signup. GET only, CORS open, responses cached at the edge. Errors use RFC
            9457 problem details. Rate limits apply per IP; the web UI solves a Turnstile challenge
            to get a higher limit.
          </p>
          <a
            href="/api/v1/openapi.json"
            className="inline-block text-accent underline decoration-accent/30 underline-offset-2 hover:decoration-accent"
          >
            openapi.json →
          </a>
        </div>
      </Panel>

      <Panel title="keys">
        <dl className="divide-y divide-line/60">
          {[
            ["/", "focus the query field"],
            ["Enter", "run the lookup"],
            ["Esc", "leave the query field"],
          ].map(([key, description]) => (
            <div key={key} className="flex items-center gap-3 px-3 py-1.5">
              <dt>
                <kbd className="border border-line px-1.5 py-px text-[11px] text-ink-bright">
                  {key}
                </kbd>
              </dt>
              <dd className="text-muted">{description}</dd>
            </div>
          ))}
        </dl>
      </Panel>
    </div>
  );
}
