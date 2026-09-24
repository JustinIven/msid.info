# msid.info

Passive OSINT lookups for Microsoft Entra ID tenants. Give it a domain or a UPN and it asks the
public Microsoft identity endpoints who owns that namespace, then folds every answer into one JSON
document.

A Rust Cloudflare Worker serves the API; a static React SPA on Workers Static Assets serves the UI.

> [!NOTE]
> This App is version 2 of msid.info. The original implementation can be found under the [msid-info](https://github.com/msid-info) GitHub organization.

## What it reports

| | |
|---|---|
| Tenant | tenant ID, display name, default domain, sovereign cloud, region scope |
| Namespace | managed vs. federated, federation protocol, metadata and active-auth URLs |
| Endpoints | OIDC issuer, authorization/token endpoints, Graph host |
| Branding | sign-in page logos, illustration, background colour, boilerplate text |
| Identity | user existence, preferred credential, password / FIDO2 / Authenticator / TAP / certificate / seamless SSO |
| Domains | every domain verified on the tenant, via Autodiscover |

Everything is unauthenticated and passive — the same requests a browser makes on its way to a
Microsoft sign-in page. No credential is ever submitted.

## API

```console
$ curl -s https://msid.info/api/v1/lookup/contoso.com | jq
$ curl -s https://msid.info/api/v1/lookup/user@contoso.com | jq
$ curl -s "https://msid.info/api/v1/lookup?q=contoso.com&refresh=1" | jq
```

| Route | Purpose |
|---|---|
| `GET /api/v1/lookup/{domain\|upn}` | The lookup. `?refresh=1` bypasses the cache. |
| `GET /api/v1/lookup?q=…` | Same, via query parameter. |
| `GET /api/v1/health` | Liveness probe. |
| `GET /api/v1/config` | Public frontend configuration. |
| `GET /api/v1/openapi.json` | OpenAPI 3.1 document. |

No key and no signup. GET only, CORS open, responses cached at the edge. Errors are
[RFC 9457](https://www.rfc-editor.org/rfc/rfc9457) problem details:

```json
{
  "type": "https://msid.info/errors/tenant-not-found",
  "title": "Tenant not found",
  "status": 404,
  "detail": "example.invalid is not backed by a Microsoft Entra ID tenant."
}
```

Every response carries a `meta` block with per-endpoint latency, upstream status codes, cache state
and a `degraded` list naming anything that was skipped or failed.

### Rate limits

Requests are limited per client IP. The web UI attaches a Cloudflare Turnstile token, which moves
the request into a higher-limit bucket. Turnstile is never required — a tokenless request is served
under the stricter anonymous limit, so `curl` and CI keep working. Exceeding a limit returns `429`
with `Retry-After`.

## Layout

```
src/                Rust Worker
  lib.rs            router, CORS, cache/rate-limit wiring
  lookup.rs         fan-out across every Microsoft endpoint, response assembly
  validate.rs       strict input allowlist (runs before any URL interpolation)
  enums.rs          cloud environments and Microsoft's coded enums
  error.rs          RFC 9457 problem responses
  cache.rs          Cache API + KV response cache
  ratelimit.rs      Workers rate limiting binding
  turnstile.rs      Turnstile verification (tier upgrade only)
  model/            typed upstream payloads and the public response shape
  msft/             one module per Microsoft endpoint
web/                Vite + React + TypeScript + Tailwind SPA
openapi.json        served at /api/v1/openapi.json
_old/               the original Python/FastAPI implementation, kept for reference
```

## Development

Requires Rust with the `wasm32-unknown-unknown` target and Node 24. The dev container provides both.

```console
$ npm ci && npm --prefix web ci
$ npm run dev            # builds the SPA, then runs wrangler dev on :8787
```

To iterate on the UI with hot reload, run the Worker and Vite side by side — Vite proxies `/api` to
`wrangler dev`:

```console
$ npx wrangler dev       # :8787
$ npm run dev:web        # :5173
```

Other scripts:

```console
$ npm test               # cargo test + tsc --noEmit
$ npm run lint           # cargo fmt --check + clippy -D warnings
$ npm run build          # SPA + wasm worker
```

## Deployment

1. Create a KV namespace and put its ID in `wrangler.jsonc` under `kv_namespaces`:

   ```console
   $ npx wrangler kv namespace create CACHE
   ```

2. Optionally configure Turnstile. Put the sitekey in `vars.TURNSTILE_SITEKEY` and the secret in
   the Worker:

   ```console
   $ npx wrangler secret put TURNSTILE_SECRET
   ```

   Leave `TURNSTILE_SITEKEY` empty to disable Turnstile; the anonymous rate limit then applies to
   everyone.

3. Optionally configure Microsoft Graph. `defaultDomain` comes from
   `tenantRelationships/findTenantInformationByTenantId`, which needs an app registration with the
   `CrossTenantInformation.ReadBasic.All` application permission:

   ```console
   $ npx wrangler secret put GRAPH_TENANT_ID
   $ npx wrangler secret put GRAPH_CLIENT_ID
   $ npx wrangler secret put GRAPH_CLIENT_SECRET
   ```

   Without these the lookup still succeeds; `defaultDomain` is `null` and `meta.degraded` contains
   `tenant_information:not-configured`.

4. Deploy:

   ```console
   $ npm run deploy
   ```

CI runs fmt, clippy, tests and a bundle dry-run on every push. Pushes to `main` deploy via
`CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` repository secrets.

For local development copy `.dev.vars.example` to `.dev.vars`; all of its values are optional.

## Notes on the upstream endpoints

These are undocumented and change without notice. Two consequences shape the code:

- Every upstream field is optional and unknown fields are ignored, so a Microsoft-side change
  degrades one field instead of failing the request. Unrecognised enum codes surface as
  `Unknown(<code>)`.
- The verbatim upstream payloads are always echoed under `raw`, so nothing is lost to the typed
  mapping.

`GetCredentialType` is the most fragile call. It historically required an `sCtx` value scraped from
the login page; it now generally accepts a bare request, and the scrape is kept only as a fallback.
It also throttles aggressively, which is the main reason responses are cached.

## Responsible use

These lookups are passive and unauthenticated, but they still reveal an organisation's identity
posture. Use them on tenants you are authorised to assess.

## License

MIT
