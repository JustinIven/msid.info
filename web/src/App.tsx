import { useCallback, useEffect, useRef, useState } from "react";
import { ApiError, fetchConfig, lookup } from "./api";
import { CurlSnippet } from "./components/CurlSnippet";
import { EmptyState } from "./components/EmptyState";
import { ResultView } from "./components/ResultView";
import { SearchBar } from "./components/SearchBar";
import { StatusBar } from "./components/StatusBar";
import { useTurnstile } from "./components/Turnstile";
import { pushHistory, readHistory } from "./history";
import type { AppConfig, LookupResponse, Problem } from "./types";

function queryFromLocation(): string {
  return new URLSearchParams(window.location.search).get("q") ?? "";
}

export default function App() {
  const [input, setInput] = useState(queryFromLocation);
  const [result, setResult] = useState<LookupResponse | null>(null);
  const [problem, setProblem] = useState<Problem | null>(null);
  const [loading, setLoading] = useState(false);
  const [recent, setRecent] = useState<string[]>(readHistory);
  const [config, setConfig] = useState<AppConfig>({
    turnstileSitekey: "",
    turnstileEnabled: false,
    cacheTtlSeconds: 0,
  });

  const inFlight = useRef<AbortController | null>(null);
  const { widget, getToken } = useTurnstile(config.turnstileSitekey);

  useEffect(() => {
    void fetchConfig().then(setConfig);
  }, []);

  const run = useCallback(
    async (query: string, options: { pushState?: boolean } = {}) => {
      inFlight.current?.abort();
      const controller = new AbortController();
      inFlight.current = controller;

      setLoading(true);
      setProblem(null);
      setInput(query);

      if (options.pushState !== false) {
        const url = `${window.location.pathname}?q=${encodeURIComponent(query)}`;
        window.history.pushState({ q: query }, "", url);
      }

      try {
        const turnstileToken = await getToken();
        const data = await lookup(query, { turnstileToken, signal: controller.signal });
        if (controller.signal.aborted) return;
        setResult(data);
        setRecent(pushHistory(query));
      } catch (error) {
        if (controller.signal.aborted) return;
        setResult(null);
        setProblem(
          error instanceof ApiError
            ? error.problem
            : {
                type: "about:blank",
                title: "Network error",
                status: 0,
                detail: "The lookup could not be sent. Check your connection and try again.",
              },
        );
      } finally {
        if (!controller.signal.aborted) setLoading(false);
      }
    },
    [getToken],
  );

  useEffect(() => {
    const initial = queryFromLocation();
    if (initial) void run(initial, { pushState: false });

    const onPopState = () => {
      const query = queryFromLocation();
      if (query) {
        void run(query, { pushState: false });
      } else {
        setResult(null);
        setProblem(null);
        setInput("");
      }
    };
    window.addEventListener("popstate", onPopState);
    return () => window.removeEventListener("popstate", onPopState);
    // `run` is stable enough here; rerunning on every identity change would re-fire the lookup.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="mx-auto flex min-h-dvh max-w-6xl flex-col gap-4 px-4 py-6 sm:px-6">
      <header className="flex flex-wrap items-baseline justify-between gap-2">
        <a href="/" className="group flex items-baseline gap-2">
          <span className="text-accent">▚</span>
          <span className="text-base text-ink-bright">msid.info</span>
          <span className="text-[11px] text-muted">entra id tenant lookup</span>
        </a>
        <nav className="flex items-center gap-4 text-[11px] text-muted">
          <a href="/api/v1/openapi.json" className="hover:text-accent">
            api
          </a>
          <a
            href="https://github.com/msid-info/msid.info-v2"
            target="_blank"
            rel="noreferrer noopener"
            className="hover:text-accent"
          >
            source
          </a>
        </nav>
      </header>

      <SearchBar
        value={input}
        onChange={setInput}
        onSubmit={(query) => void run(query)}
        loading={loading}
        recent={recent}
      />

      {widget}

      <main className="flex-1 space-y-3">
        {loading && (
          <p role="status" className="border border-line bg-surface px-3 py-2 text-muted">
            <span className="text-ok">$ </span>
            resolving {input}…
          </p>
        )}

        {!loading && problem && (
          <div
            role="alert"
            className="border border-danger/40 bg-danger/5 px-3 py-3 text-ink-bright"
          >
            <p className="text-danger">
              {problem.status || "ERR"} · {problem.title}
            </p>
            <p className="mt-1 text-muted">{problem.detail}</p>
          </div>
        )}

        {!loading && !problem && result && (
          <>
            <StatusBar data={result} />
            <CurlSnippet query={result.query.input} />
            <ResultView data={result} onLookup={(query) => void run(query)} />
          </>
        )}

        {!loading && !problem && !result && (
          <EmptyState
            onPick={(query) => {
              setInput(query);
              void run(query);
            }}
          />
        )}
      </main>

      <footer className="border-t border-line pt-3 text-[11px] text-muted">
        Passive lookups against public Microsoft endpoints. No credentials are submitted and no
        sign-in attempt is made. Use it on tenants you are authorised to assess.
      </footer>
    </div>
  );
}
