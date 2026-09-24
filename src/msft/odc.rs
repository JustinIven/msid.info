use super::http::{send, Outcome, Recorder, Req};

const ENDPOINT: &str = "https://odc.officeapps.live.com/odc/v2.1/federationprovider";

/// Resolves a domain to its tenant ID and sovereign cloud. This is the entry point of every
/// lookup: without a `tenantId` here, the domain is not backed by an Entra ID tenant.
pub async fn federation_provider(recorder: &Recorder, domain: &str) -> Outcome {
    let url = format!("{ENDPOINT}?domain={}", super::encode_segment(domain));
    send(recorder, Req::get("odc_federationprovider", url)).await
}
