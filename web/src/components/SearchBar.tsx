import { useEffect, useRef, useState } from "react";
import { normalize, validationError } from "../validate";

interface Props {
  value: string;
  onChange: (value: string) => void;
  onSubmit: (value: string) => void;
  loading: boolean;
  recent: string[];
}

export function SearchBar({ value, onChange, onSubmit, loading, recent }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [touched, setTouched] = useState(false);
  const error = touched ? validationError(value) : null;

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const typingElsewhere =
        target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement;
      if (event.key === "/" && !typingElsewhere) {
        event.preventDefault();
        inputRef.current?.focus();
      }
      if (event.key === "Escape") {
        inputRef.current?.blur();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  return (
    <div>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          setTouched(true);
          const normalized = normalize(value);
          if (!normalized || validationError(normalized)) return;
          onSubmit(normalized);
        }}
        className="group flex items-center gap-2 border border-line bg-surface px-3 py-2.5 focus-within:border-accent/60"
      >
        <label htmlFor="query" className="sr-only">
          Domain or user principal name
        </label>
        <span aria-hidden className="select-none text-ok">
          $
        </span>
        <span aria-hidden className="select-none text-muted">
          msid lookup
        </span>
        <input
          id="query"
          ref={inputRef}
          value={value}
          onChange={(event) => onChange(event.target.value)}
          onBlur={() => setTouched(true)}
          placeholder="contoso.com or user@contoso.com"
          autoComplete="off"
          autoCapitalize="off"
          autoCorrect="off"
          spellCheck={false}
          enterKeyHint="search"
          aria-invalid={Boolean(error)}
          aria-describedby={error ? "query-error" : undefined}
          className="min-w-0 flex-1 bg-transparent text-ink-bright caret-accent outline-none placeholder:text-muted/50"
        />
        {!value && (
          <kbd className="hidden shrink-0 border border-line px-1 text-[10px] text-muted sm:block">
            /
          </kbd>
        )}
        <button
          type="submit"
          disabled={loading}
          className="shrink-0 border border-accent/40 px-2.5 py-0.5 text-[11px] uppercase tracking-[0.12em] text-accent transition-colors hover:bg-accent/10 disabled:opacity-40"
        >
          {loading ? "…" : "run"}
        </button>
      </form>

      <div className="mt-1.5 flex min-h-5 flex-wrap items-center gap-x-3 gap-y-1 text-[11px]">
        {error ? (
          <span id="query-error" role="alert" className="text-danger">
            {error}
          </span>
        ) : recent.length > 0 ? (
          <>
            <span className="text-muted">recent:</span>
            {recent.map((item) => (
              <button
                key={item}
                type="button"
                onClick={() => {
                  onChange(item);
                  onSubmit(item);
                }}
                className="text-muted underline decoration-dotted underline-offset-2 hover:text-accent"
              >
                {item}
              </button>
            ))}
          </>
        ) : null}
      </div>
    </div>
  );
}
