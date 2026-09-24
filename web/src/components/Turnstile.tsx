import { useCallback, useEffect, useRef, useState } from "react";

interface TurnstileApi {
  render: (
    container: HTMLElement,
    options: {
      sitekey: string;
      appearance?: "always" | "execute" | "interaction-only";
      theme?: "auto" | "light" | "dark";
      callback?: (token: string) => void;
      "expired-callback"?: () => void;
      "error-callback"?: () => void;
    },
  ) => string;
  reset: (widgetId?: string) => void;
}

declare global {
  interface Window {
    turnstile?: TurnstileApi;
  }
}

const SCRIPT_URL = "https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit";
const TOKEN_WAIT_MS = 6000;

function loadScript(): Promise<void> {
  if (window.turnstile) return Promise.resolve();
  const existing = document.querySelector<HTMLScriptElement>(`script[src="${SCRIPT_URL}"]`);
  if (existing) {
    return new Promise((resolve, reject) => {
      existing.addEventListener("load", () => resolve());
      existing.addEventListener("error", () => reject(new Error("turnstile failed to load")));
    });
  }
  return new Promise((resolve, reject) => {
    const script = document.createElement("script");
    script.src = SCRIPT_URL;
    script.async = true;
    script.defer = true;
    script.onload = () => resolve();
    script.onerror = () => reject(new Error("turnstile failed to load"));
    document.head.appendChild(script);
  });
}

/// Turnstile only raises the rate limit; every failure path resolves to `undefined` so the
/// lookup still goes through as an anonymous request.
export function useTurnstile(sitekey: string) {
  const containerRef = useRef<HTMLDivElement>(null);
  const widgetRef = useRef<string | null>(null);
  const tokenRef = useRef<string | null>(null);
  const waitersRef = useRef<Array<(token: string | undefined) => void>>([]);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    if (!sitekey || !containerRef.current) return;
    let cancelled = false;

    const settle = (token: string | undefined) => {
      const waiters = waitersRef.current;
      waitersRef.current = [];
      waiters.forEach((resolve) => resolve(token));
    };

    loadScript()
      .then(() => {
        if (cancelled || !containerRef.current || !window.turnstile) return;
        widgetRef.current = window.turnstile.render(containerRef.current, {
          sitekey,
          appearance: "interaction-only",
          theme: "dark",
          callback: (token) => {
            tokenRef.current = token;
            settle(token);
          },
          "expired-callback": () => {
            tokenRef.current = null;
          },
          "error-callback": () => {
            tokenRef.current = null;
            settle(undefined);
          },
        });
        setReady(true);
      })
      .catch(() => setReady(false));

    return () => {
      cancelled = true;
      settle(undefined);
    };
  }, [sitekey]);

  const getToken = useCallback(async (): Promise<string | undefined> => {
    if (!sitekey || !ready) return undefined;

    if (tokenRef.current) {
      const token = tokenRef.current;
      tokenRef.current = null;
      window.turnstile?.reset(widgetRef.current ?? undefined);
      return token;
    }

    return new Promise<string | undefined>((resolve) => {
      const timer = window.setTimeout(() => {
        waitersRef.current = waitersRef.current.filter((waiter) => waiter !== waiter_);
        resolve(undefined);
      }, TOKEN_WAIT_MS);

      const waiter_ = (token: string | undefined) => {
        window.clearTimeout(timer);
        tokenRef.current = null;
        resolve(token);
      };
      waitersRef.current.push(waiter_);
    });
  }, [ready, sitekey]);

  const widget = <div ref={containerRef} className="empty:hidden" />;

  return { widget, getToken };
}
