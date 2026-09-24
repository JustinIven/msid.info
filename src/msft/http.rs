//! Thin wrapper around the Workers `fetch` API.
//!
//! `reqwest` is unusable on `wasm32-unknown-unknown` inside Workers, so every outbound call goes
//! through `worker::Fetch`. Each call is timed and recorded so the API can report exactly which
//! upstream endpoint was slow or broken.

use std::cell::RefCell;

use worker::wasm_bindgen::JsValue;
use worker::{Fetch, Headers, Method, Request, RequestInit};

use crate::model::response::UpstreamCall;

/// Identifies the service to Microsoft. These are public, unauthenticated endpoints; there is no
/// reason to pretend to be a browser except on the login page, which serves different markup.
pub const USER_AGENT: &str = "msid.info/2.0 (+https://msid.info)";
pub const BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36";

/// Collects per-endpoint diagnostics across concurrently running lookups.
///
/// Worker futures are single-threaded and `!Send`, so a `RefCell` is sufficient here.
#[derive(Default)]
pub struct Recorder {
    calls: RefCell<Vec<UpstreamCall>>,
}

impl Recorder {
    pub fn record(&self, call: UpstreamCall) {
        self.calls.borrow_mut().push(call);
    }

    pub fn into_calls(self) -> Vec<UpstreamCall> {
        let mut calls = self.calls.into_inner();
        calls.sort_by_key(|call| call.name);
        calls
    }
}

pub struct Outcome {
    pub body: Option<String>,
    pub call: UpstreamCall,
}

impl Outcome {
    pub fn json(&self) -> serde_json::Value {
        self.body
            .as_deref()
            .and_then(|body| serde_json::from_str(body).ok())
            .unwrap_or(serde_json::Value::Null)
    }

    pub fn text(&self) -> serde_json::Value {
        match &self.body {
            Some(body) => serde_json::Value::String(body.clone()),
            None => serde_json::Value::Null,
        }
    }
}

pub struct Req<'a> {
    pub name: &'static str,
    pub method: Method,
    pub url: String,
    pub body: Option<String>,
    pub headers: Vec<(&'a str, &'a str)>,
}

impl<'a> Req<'a> {
    pub fn get(name: &'static str, url: impl Into<String>) -> Self {
        Self {
            name,
            method: Method::Get,
            url: url.into(),
            body: None,
            headers: Vec::new(),
        }
    }

    pub fn post(name: &'static str, url: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            name,
            method: Method::Post,
            url: url.into(),
            body: Some(body.into()),
            headers: Vec::new(),
        }
    }

    pub fn header(mut self, key: &'a str, value: &'a str) -> Self {
        self.headers.push((key, value));
        self
    }
}

pub async fn send(recorder: &Recorder, req: Req<'_>) -> Outcome {
    let started = worker::Date::now().as_millis();
    let method_label = method_label(&req.method);
    let url = req.url.clone();
    let name = req.name;

    let result = perform(req).await;
    let duration_ms = worker::Date::now().as_millis().saturating_sub(started) as u32;

    let outcome = match result {
        Ok((status, body)) => {
            let ok = (200..300).contains(&status);
            Outcome {
                body: ok.then_some(body),
                call: UpstreamCall {
                    name,
                    method: method_label,
                    url,
                    status: Some(status),
                    duration_ms,
                    ok,
                    error: (!ok).then(|| format!("Upstream returned HTTP {status}.")),
                },
            }
        }
        Err(err) => Outcome {
            body: None,
            call: UpstreamCall {
                name,
                method: method_label,
                url,
                status: None,
                duration_ms,
                ok: false,
                error: Some(err),
            },
        },
    };

    recorder.record(outcome.call.clone());
    outcome
}

async fn perform(req: Req<'_>) -> Result<(u16, String), String> {
    let headers = Headers::new();
    headers
        .set("user-agent", USER_AGENT)
        .map_err(|e| e.to_string())?;
    headers.set("accept", "*/*").map_err(|e| e.to_string())?;
    for (key, value) in &req.headers {
        headers.set(key, value).map_err(|e| e.to_string())?;
    }

    let mut init = RequestInit::new();
    init.with_method(req.method).with_headers(headers);
    if let Some(body) = &req.body {
        init.with_body(Some(JsValue::from_str(body)));
    }

    let request = Request::new_with_init(&req.url, &init).map_err(|e| e.to_string())?;
    let mut response = Fetch::Request(request)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = response.status_code();
    let body = response.text().await.map_err(|e| e.to_string())?;
    Ok((status, body))
}

fn method_label(method: &Method) -> &'static str {
    match method {
        Method::Get => "GET",
        Method::Post => "POST",
        _ => "OTHER",
    }
}
