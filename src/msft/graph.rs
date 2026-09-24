use worker::Env;

use super::http::{send, Outcome, Recorder, Req};

const GRAPH_TOKEN_CACHE_KEY: &str = "graph:token";
/// Workers KV rejects TTLs below 60 seconds.
const MIN_TOKEN_TTL: u64 = 60;

pub struct Credentials {
    tenant_id: String,
    client_id: String,
    client_secret: String,
}

/// Returns `None` when the Graph app registration is not configured, which is a supported mode:
/// the lookup then simply reports `defaultDomain` as null.
pub fn credentials(env: &Env) -> Option<Credentials> {
    let read = |name: &str| -> Option<String> {
        let value = env.secret(name).ok()?.to_string();
        (!value.is_empty()).then_some(value)
    };

    Some(Credentials {
        tenant_id: read("GRAPH_TENANT_ID")?,
        client_id: read("GRAPH_CLIENT_ID")?,
        client_secret: read("GRAPH_CLIENT_SECRET")?,
    })
}

pub async fn tenant_information(
    recorder: &Recorder,
    env: &Env,
    credentials: &Credentials,
    tenant_id: &str,
) -> Outcome {
    let token = access_token(recorder, env, credentials).await;
    let url = format!(
        "https://graph.microsoft.com/v1.0/tenantRelationships/findTenantInformationByTenantId(tenantId='{}')",
        super::encode_segment(tenant_id)
    );

    let Some(token) = token else {
        return send(recorder, Req::get("tenant_information", url)).await;
    };

    let authorization = format!("Bearer {token}");
    send(
        recorder,
        Req::get("tenant_information", url).header("authorization", &authorization),
    )
    .await
}

/// Client-credentials token, cached in KV so it is shared across isolates and colos.
async fn access_token(recorder: &Recorder, env: &Env, credentials: &Credentials) -> Option<String> {
    let kv = env.kv("CACHE").ok();

    if let Some(kv) = &kv {
        if let Ok(Some(cached)) = kv.get(GRAPH_TOKEN_CACHE_KEY).text().await {
            return Some(cached);
        }
    }

    let body = format!(
        "grant_type=client_credentials&client_id={}&client_secret={}&scope={}",
        form_encode(&credentials.client_id),
        form_encode(&credentials.client_secret),
        form_encode("https://graph.microsoft.com/.default"),
    );

    let outcome = send(
        recorder,
        Req::post(
            "graph_token",
            format!(
                "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
                super::encode_segment(&credentials.tenant_id)
            ),
            body,
        )
        .header("content-type", "application/x-www-form-urlencoded"),
    )
    .await;

    let json = outcome.json();
    let token = json.get("access_token")?.as_str()?.to_string();
    let expires_in = json
        .get("expires_in")
        .and_then(|v| v.as_u64())
        .unwrap_or(3600);

    if let Some(kv) = &kv {
        let ttl = expires_in.saturating_sub(300).max(MIN_TOKEN_TTL);
        if let Ok(builder) = kv.put(GRAPH_TOKEN_CACHE_KEY, token.clone()) {
            let _ = builder.expiration_ttl(ttl).execute().await;
        }
    }

    Some(token)
}

fn form_encode(value: &str) -> String {
    percent_encoding::utf8_percent_encode(value, percent_encoding::NON_ALPHANUMERIC).to_string()
}
