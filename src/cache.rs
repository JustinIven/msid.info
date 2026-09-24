//! Two-layer response cache.
//!
//! The Cache API is colo-local and free; KV is shared across colos and shields the throttle-prone
//! `GetCredentialType` endpoint from repeated lookups of popular domains. Both layers are
//! best-effort - any failure degrades to a live lookup.
//!
//! Note that the Cache API only serves URLs belonging to the zone the Worker runs on, and is a
//! no-op on `*.workers.dev`. When it is unavailable every request simply reports `HIT-KV` or
//! `MISS` instead of `HIT-EDGE`.

use worker::{Cache, Env};

const DEFAULT_TTL_SECONDS: u64 = 900;
/// Workers KV rejects TTLs below 60 seconds.
const MIN_TTL_SECONDS: u64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Edge,
    Kv,
}

impl Source {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Edge => "HIT-EDGE",
            Self::Kv => "HIT-KV",
        }
    }
}

pub fn ttl_seconds(env: &Env) -> u64 {
    env.var("CACHE_TTL_SECONDS")
        .ok()
        .and_then(|value| value.to_string().parse::<u64>().ok())
        .unwrap_or(DEFAULT_TTL_SECONDS)
        .max(MIN_TTL_SECONDS)
}

/// Synthetic key into the colo-local cache. It is built from the caller's own origin because the
/// Cache API ignores URLs outside the Worker's zone; `__cache` keeps it clear of real routes.
fn edge_key(origin: &str, key: &str) -> String {
    format!("{origin}/__cache/v1/lookup/{key}")
}

fn kv_key(key: &str) -> String {
    format!("lookup:v1:{key}")
}

pub async fn get(env: &Env, origin: Option<&str>, key: &str) -> Option<(String, Source)> {
    if let Some(origin) = origin {
        if let Ok(Some(mut response)) = Cache::default().get(edge_key(origin, key), false).await {
            if let Ok(body) = response.text().await {
                return Some((body, Source::Edge));
            }
        }
    }

    let kv = env.kv("CACHE").ok()?;
    let body = kv.get(&kv_key(key)).text().await.ok()??;
    Some((body, Source::Kv))
}

pub async fn put(env: &Env, origin: Option<&str>, key: &str, body: &str, ttl: u64) {
    if let Ok(kv) = env.kv("CACHE") {
        if let Ok(builder) = kv.put(&kv_key(key), body.to_string()) {
            let _ = builder.expiration_ttl(ttl).execute().await;
        }
    }

    let Some(origin) = origin else {
        return;
    };
    let Ok(mut response) = worker::Response::ok(body.to_string()) else {
        return;
    };
    let headers = response.headers_mut();
    let _ = headers.set("content-type", "application/json; charset=utf-8");
    let _ = headers.set("cache-control", &format!("public, max-age={ttl}"));
    let _ = Cache::default().put(edge_key(origin, key), response).await;
}
