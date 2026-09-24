//! Cloudflare Turnstile verification.
//!
//! Turnstile is a *tier upgrade*, never a gate: a request without a token is still served, just
//! under the stricter anonymous rate limit. That keeps the documented public API usable from
//! curl and CI.

use worker::Env;

use crate::msft::http::{send, Recorder, Req};

const SITEVERIFY_URL: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Verified,
    Anonymous,
}

impl Tier {
    pub fn rate_limiter_binding(&self) -> &'static str {
        match self {
            Self::Verified => "RL_VERIFIED",
            Self::Anonymous => "RL_ANON",
        }
    }
}

pub fn is_enabled(env: &Env) -> bool {
    !secret(env).unwrap_or_default().is_empty()
}

pub fn sitekey(env: &Env) -> String {
    env.var("TURNSTILE_SITEKEY")
        .map(|v| v.to_string())
        .unwrap_or_default()
}

pub async fn classify(env: &Env, token: Option<String>, remote_ip: Option<String>) -> Tier {
    let (Some(secret), Some(token)) = (secret(env), token) else {
        return Tier::Anonymous;
    };
    if secret.is_empty() || token.is_empty() {
        return Tier::Anonymous;
    }

    let mut body = format!(
        "secret={}&response={}",
        form_encode(&secret),
        form_encode(&token)
    );
    if let Some(ip) = remote_ip {
        body.push_str(&format!("&remoteip={}", form_encode(&ip)));
    }

    let recorder = Recorder::default();
    let outcome = send(
        &recorder,
        Req::post("turnstile", SITEVERIFY_URL, body)
            .header("content-type", "application/x-www-form-urlencoded"),
    )
    .await;

    match outcome.json().get("success").and_then(|v| v.as_bool()) {
        Some(true) => Tier::Verified,
        _ => Tier::Anonymous,
    }
}

fn secret(env: &Env) -> Option<String> {
    env.secret("TURNSTILE_SECRET").ok().map(|s| s.to_string())
}

fn form_encode(value: &str) -> String {
    percent_encoding::utf8_percent_encode(value, percent_encoding::NON_ALPHANUMERIC).to_string()
}
