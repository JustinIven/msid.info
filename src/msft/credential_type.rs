use super::http::{send, Outcome, Recorder, Req, BROWSER_USER_AGENT};

/// Probes which credential types a user can authenticate with.
///
/// The endpoint historically required an `originalRequest` (`sCtx`) value scraped from the login
/// page. It now generally accepts a bare request, so the scrape is only performed as a fallback -
/// it costs two extra subrequests and is the most fragile part of the whole lookup.
pub async fn get_credential_type(recorder: &Recorder, login_base: &str, username: &str) -> Outcome {
    let first = post(recorder, login_base, username, None).await;
    if is_usable(&first) {
        return first;
    }

    match fetch_sctx(recorder, login_base).await {
        Some(sctx) => post(recorder, login_base, username, Some(&sctx)).await,
        None => first,
    }
}

async fn post(
    recorder: &Recorder,
    login_base: &str,
    username: &str,
    original_request: Option<&str>,
) -> Outcome {
    let mut payload = serde_json::json!({
        "username": username,
        "isOtherIdpSupported": true,
        "isRemoteNGCSupported": true,
        "isFidoSupported": true,
        "isRemoteConnectSupported": true,
        "isAccessPassSupported": true,
        "isCookieBannerShown": false,
        "isSignupPost": false,
        "checkPhones": true,
        "isExternalFederationDisallowed": false,
    });
    if let Some(sctx) = original_request {
        payload["originalRequest"] = serde_json::Value::String(sctx.to_string());
    }

    let name = if original_request.is_some() {
        "credential_type_retry"
    } else {
        "credential_type"
    };

    send(
        recorder,
        Req::post(
            name,
            format!("{login_base}/common/GetCredentialType"),
            payload.to_string(),
        )
        .header("content-type", "application/json; charset=utf-8"),
    )
    .await
}

/// A 200 that omits `IfExistsResult` means the endpoint rejected the request shape.
fn is_usable(outcome: &Outcome) -> bool {
    outcome.call.ok && outcome.json().get("IfExistsResult").is_some()
}

async fn fetch_sctx(recorder: &Recorder, login_base: &str) -> Option<String> {
    let outcome = send(
        recorder,
        Req::get("login_page", login_base.to_string()).header("user-agent", BROWSER_USER_AGENT),
    )
    .await;
    extract_sctx(outcome.body.as_deref()?)
}

fn extract_sctx(html: &str) -> Option<String> {
    let start = html.find("\"sCtx\":\"")? + "\"sCtx\":\"".len();
    let rest = &html[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_sctx_from_config_blob() {
        let html = r#"<script>//<![CDATA[ $Config={"iMaxStackForKnockoutAsyncComponents":10000,"sCtx":"rQQIARAAtVLLbtNAFB","fTenant":true};//]]></script>"#;
        assert_eq!(extract_sctx(html).as_deref(), Some("rQQIARAAtVLLbtNAFB"));
    }

    #[test]
    fn returns_none_when_absent() {
        assert!(extract_sctx("<html></html>").is_none());
        assert!(extract_sctx("\"sCtx\":\"unterminated").is_none());
    }
}
