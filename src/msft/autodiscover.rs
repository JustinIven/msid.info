use quick_xml::events::Event;
use quick_xml::Reader;

use super::http::{send, Outcome, Recorder, Req};

/// SOAP `GetFederationInformation` — the only endpoint that enumerates every verified domain
/// attached to a tenant.
pub async fn federation_information(
    recorder: &Recorder,
    autodiscover_base: &str,
    domain: &str,
) -> Outcome {
    // `domain` is already restricted to `[a-z0-9.-]` by `validate`, so it cannot break out of the
    // XML element.
    let body = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<soap:Envelope xmlns:exm="http://schemas.microsoft.com/exchange/services/2006/messages" xmlns:ext="http://schemas.microsoft.com/exchange/services/2006/types" xmlns:a="http://www.w3.org/2005/08/addressing" xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
    <soap:Header>
        <a:Action soap:mustUnderstand="1">http://schemas.microsoft.com/exchange/2010/Autodiscover/Autodiscover/GetFederationInformation</a:Action>
    </soap:Header>
    <soap:Body>
        <GetFederationInformationRequestMessage xmlns="http://schemas.microsoft.com/exchange/2010/Autodiscover">
            <Request>
                <Domain>{domain}</Domain>
            </Request>
        </GetFederationInformationRequestMessage>
    </soap:Body>
</soap:Envelope>"#
    );

    let url = format!("{autodiscover_base}/autodiscover/autodiscover.svc");
    send(
        recorder,
        Req::post("autodiscover_federation_information", url, body)
            .header("content-type", "text/xml; charset=utf-8"),
    )
    .await
}

/// Extracts `<Domain>` values from the SOAP envelope, deduplicated and sorted.
pub fn parse_domains(xml: &str) -> Vec<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut domains: Vec<String> = Vec::new();
    let mut depth_in_domain = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(tag)) if tag.local_name().as_ref() == b"Domain" => {
                depth_in_domain = true;
            }
            Ok(Event::End(tag)) if tag.local_name().as_ref() == b"Domain" => {
                depth_in_domain = false;
            }
            Ok(Event::Text(text)) if depth_in_domain => {
                let value = text
                    .xml_content()
                    .map(|cow| cow.into_owned())
                    .unwrap_or_else(|_| String::from_utf8_lossy(text.as_ref()).into_owned());
                let value = value.trim().to_ascii_lowercase();
                if !value.is_empty() {
                    domains.push(value);
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }

    domains.sort();
    domains.dedup();
    domains
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
  <s:Body>
    <GetFederationInformationResponseMessage xmlns="http://schemas.microsoft.com/exchange/2010/Autodiscover">
      <Response>
        <Domains>
          <Domain>zebra.contoso.com</Domain>
          <Domain>Contoso.com</Domain>
          <Domain>contoso.com</Domain>
        </Domains>
      </Response>
    </GetFederationInformationResponseMessage>
  </s:Body>
</s:Envelope>"#;

    #[test]
    fn extracts_sorted_unique_domains() {
        assert_eq!(
            parse_domains(SAMPLE),
            vec!["contoso.com", "zebra.contoso.com"]
        );
    }

    #[test]
    fn tolerates_garbage() {
        assert!(parse_domains("").is_empty());
        assert!(parse_domains("<html>not soap</html>").is_empty());
        assert!(parse_domains("<Domains><Domain>").is_empty());
    }
}
