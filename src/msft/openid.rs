use super::http::{send, Outcome, Recorder, Req};

pub async fn configuration(recorder: &Recorder, login_base: &str, domain: &str) -> Outcome {
    let url = format!(
        "{login_base}/{}/.well-known/openid-configuration",
        super::encode_segment(domain)
    );
    send(recorder, Req::get("openid_configuration", url)).await
}
