import type { AppConfig, LookupResponse, Problem } from "./types";

export class ApiError extends Error {
  readonly problem: Problem;

  constructor(problem: Problem) {
    super(problem.detail);
    this.problem = problem;
  }
}

async function parseProblem(response: Response): Promise<Problem> {
  try {
    const body = (await response.json()) as Partial<Problem>;
    return {
      type: body.type ?? "about:blank",
      title: body.title ?? "Request failed",
      status: body.status ?? response.status,
      detail: body.detail ?? response.statusText,
    };
  } catch {
    return {
      type: "about:blank",
      title: "Request failed",
      status: response.status,
      detail: `The API returned HTTP ${response.status}.`,
    };
  }
}

export async function lookup(
  query: string,
  options: { turnstileToken?: string; signal?: AbortSignal } = {},
): Promise<LookupResponse> {
  const headers: HeadersInit = {};
  if (options.turnstileToken) {
    headers["cf-turnstile-response"] = options.turnstileToken;
  }

  const response = await fetch(`/api/v1/lookup/${encodeURIComponent(query)}`, {
    headers,
    signal: options.signal,
  });

  if (!response.ok) {
    throw new ApiError(await parseProblem(response));
  }
  return (await response.json()) as LookupResponse;
}

export async function fetchConfig(): Promise<AppConfig> {
  const response = await fetch("/api/v1/config");
  if (!response.ok) {
    return { turnstileSitekey: "", turnstileEnabled: false, cacheTtlSeconds: 0 };
  }
  return (await response.json()) as AppConfig;
}
