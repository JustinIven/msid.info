use super::http::{send, Outcome, Recorder, Req};

pub async fn v1(recorder: &Recorder, login_base: &str, username: &str) -> Outcome {
    let url = format!(
        "{login_base}/common/userrealm/{}?api-version=1.0",
        super::encode_segment(username)
    );
    send(recorder, Req::get("userrealm_v1", url)).await
}

pub async fn v2(recorder: &Recorder, login_base: &str, username: &str) -> Outcome {
    let url = format!(
        "{login_base}/common/userrealm/{}?api-version=2.0",
        super::encode_segment(username)
    );
    send(recorder, Req::get("userrealm_v2", url)).await
}

/// The pre-ADAL endpoint. Still the only source of `FederationGlobalVersion`.
pub async fn legacy(recorder: &Recorder, login_base: &str, username: &str) -> Outcome {
    let url = format!(
        "{login_base}/GetUserRealm.srf?login={}",
        super::encode_segment(username)
    );
    send(recorder, Req::get("userrealm_old", url)).await
}
