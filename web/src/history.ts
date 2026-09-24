const KEY = "msid.recent";
const LIMIT = 8;

export function readHistory(): string[] {
  try {
    const stored = window.localStorage.getItem(KEY);
    if (!stored) return [];
    const parsed: unknown = JSON.parse(stored);
    return Array.isArray(parsed) ? parsed.filter((x) => typeof x === "string") : [];
  } catch {
    return [];
  }
}

export function pushHistory(query: string): string[] {
  const next = [query, ...readHistory().filter((item) => item !== query)].slice(0, LIMIT);
  try {
    window.localStorage.setItem(KEY, JSON.stringify(next));
  } catch {
    // Storage disabled (private mode); history is a nice-to-have.
  }
  return next;
}

export function clearHistory(): void {
  try {
    window.localStorage.removeItem(KEY);
  } catch {
    // ignore
  }
}
