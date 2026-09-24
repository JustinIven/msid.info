mod cache;
mod enums;
mod error;
mod lookup;
mod model;
mod msft;
mod ratelimit;
mod turnstile;
mod validate;

use percent_encoding::percent_decode_str;
use worker::{event, Context, Env, Method, Request, Response, Result};

use crate::error::ApiError;

const OPENAPI: &str = include_str!("../openapi.json");

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let response = route(&req, &env).await.unwrap_or_else(|err| {
        err.to_response()
            .unwrap_or_else(|_| Response::error("Internal error", 500).unwrap())
    });
    Ok(with_standard_headers(response))
}

async fn route(req: &Request, env: &Env) -> std::result::Result<Response, ApiError> {
    let url = req.url().map_err(ApiError::from)?;
    let path = url.path().to_string();

    if req.method() == Method::Options {
        return Ok(Response::empty().map_err(ApiError::from)?.with_status(204));
    }

    if req.method() != Method::Get {
        return Err(ApiError::invalid_input(
            "Only GET and OPTIONS are supported.",
        ));
    }

    match path.as_str() {
        "/api/v1/health" => json(&serde_json::json!({
            "status": "ok",
            "version": env!("CARGO_PKG_VERSION"),
        })),
        "/api/v1/config" => json(&serde_json::json!({
            "turnstileSitekey": turnstile::sitekey(env),
            "turnstileEnabled": turnstile::is_enabled(env),
            "cacheTtlSeconds": cache::ttl_seconds(env),
        })),
        "/api/v1/openapi.json" => Response::ok(OPENAPI)
            .map(|res| res.with_headers(json_headers()))
            .map_err(ApiError::from),
        _ => {
            let Some(raw_query) = lookup_query(&path, req) else {
                return Err(ApiError::TenantNotFound(format!(
                    "No API route matches {path}."
                )));
            };
            handle_lookup(req, env, &raw_query).await
        }
    }
}

fn lookup_query(path: &str, req: &Request) -> Option<String> {
    if let Some(rest) = path.strip_prefix("/api/v1/lookup/") {
        let decoded = percent_decode_str(rest).decode_utf8_lossy().into_owned();
        if !decoded.is_empty() {
            return Some(decoded);
        }
    }
    if path == "/api/v1/lookup" {
        let url = req.url().ok()?;
        return url
            .query_pairs()
            .find(|(key, _)| key == "q")
            .map(|(_, value)| value.into_owned());
    }
    None
}

async fn handle_lookup(
    req: &Request,
    env: &Env,
    raw_query: &str,
) -> std::result::Result<Response, ApiError> {
    let target = validate::parse(raw_query)?;

    let token = req
        .headers()
        .get("cf-turnstile-response")
        .ok()
        .flatten()
        .filter(|value| !value.is_empty());
    let client_ip = req.headers().get("cf-connecting-ip").ok().flatten();
    let tier = turnstile::classify(env, token, client_ip.clone()).await;

    let rate_limit_key = client_ip.unwrap_or_else(|| "unknown".to_string());
    ratelimit::enforce(env, tier, &rate_limit_key).await?;

    let ttl = cache::ttl_seconds(env);
    let cache_key = target.cache_key();
    let url = req.url().ok();
    let origin = url
        .as_ref()
        .map(|url| url.origin().ascii_serialization())
        .filter(|origin| origin != "null");
    let refresh = url
        .as_ref()
        .map(|url| {
            url.query_pairs()
                .any(|(key, value)| key == "refresh" && value != "0")
        })
        .unwrap_or(false);

    if !refresh {
        if let Some((body, source)) = cache::get(env, origin.as_deref(), &cache_key).await {
            return cached_response(&body, source.label(), ttl);
        }
    }

    let mut result = lookup::run(env, &target).await?;
    result.meta.cache.ttl_seconds = ttl;
    let body = serde_json::to_string(&result)
        .map_err(|err| ApiError::internal(format!("Failed to serialize response: {err}")))?;

    cache::put(env, origin.as_deref(), &cache_key, &body, ttl).await;

    Response::ok(body)
        .map(|res| res.with_headers(json_headers()))
        .map_err(ApiError::from)
}

/// Rewrites `meta.cache.status` so callers can tell a cached answer from a live one.
fn cached_response(body: &str, status: &str, ttl: u64) -> std::result::Result<Response, ApiError> {
    let body = match serde_json::from_str::<serde_json::Value>(body) {
        Ok(mut value) => {
            if let Some(cache) = value.pointer_mut("/meta/cache") {
                cache["status"] = serde_json::Value::String(status.to_string());
                cache["ttlSeconds"] = serde_json::Value::from(ttl);
            }
            value.to_string()
        }
        Err(_) => body.to_string(),
    };

    Response::ok(body)
        .map(|res| res.with_headers(json_headers()))
        .map_err(ApiError::from)
}

fn json(value: &serde_json::Value) -> std::result::Result<Response, ApiError> {
    Response::ok(value.to_string())
        .map(|res| res.with_headers(json_headers()))
        .map_err(ApiError::from)
}

fn json_headers() -> worker::Headers {
    let headers = worker::Headers::new();
    let _ = headers.set("content-type", "application/json; charset=utf-8");
    headers
}

/// CORS is wide open on purpose: this is a read-only, credential-free public API.
fn with_standard_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    let _ = headers.set("access-control-allow-origin", "*");
    let _ = headers.set("access-control-allow-methods", "GET, OPTIONS");
    let _ = headers.set(
        "access-control-allow-headers",
        "content-type, cf-turnstile-response",
    );
    let _ = headers.set("access-control-max-age", "86400");
    let _ = headers.set("x-content-type-options", "nosniff");
    let _ = headers.set("referrer-policy", "no-referrer");
    response
}
