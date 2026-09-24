//! Typed views over the Microsoft responses.
//!
//! Every field is optional: these are undocumented endpoints and Microsoft adds, removes and
//! renames fields without notice. Unknown fields are ignored rather than rejected, and the
//! untouched payload is always echoed back under `raw` so nothing is silently lost.

use serde::Deserialize;

use crate::enums::{CredentialType, DomainType, ThrottleStatus, UserState};

/// `GET https://odc.officeapps.live.com/odc/v2.1/federationprovider`
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FederationProvider {
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<String>,
    pub environment: Option<String>,
}

/// `GET {login_base}/common/userrealm/{upn}?api-version=1.0`
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserRealmV1 {
    pub cloud_audience_urn: Option<String>,
    pub federation_protocol: Option<String>,
    pub federation_metadata_url: Option<String>,
    pub federation_active_auth_url: Option<String>,
}

/// `GET {login_base}/common/userrealm/{upn}?api-version=2.0`
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserRealmV2 {
    #[serde(rename = "NameSpaceType")]
    pub name_space_type: Option<String>,
    #[serde(rename = "FederationBrandName")]
    pub federation_brand_name: Option<String>,
    #[serde(rename = "TenantBrandingInfo")]
    pub tenant_branding_info: Option<Vec<TenantBrandingInfo>>,
    pub federation_protocol: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TenantBrandingInfo {
    #[serde(rename = "Locale")]
    pub locale: Option<i64>,
    #[serde(rename = "BannerLogo")]
    pub banner_logo: Option<String>,
    #[serde(rename = "TileLogo")]
    pub tile_logo: Option<String>,
    #[serde(rename = "TileDarkLogo")]
    pub tile_dark_logo: Option<String>,
    #[serde(rename = "Illustration")]
    pub illustration: Option<String>,
    #[serde(rename = "BackgroundColor")]
    pub background_color: Option<String>,
    #[serde(rename = "BoilerPlateText")]
    pub boiler_plate_text: Option<String>,
    #[serde(rename = "UserIdLabel")]
    pub user_id_label: Option<String>,
    #[serde(rename = "KeepMeSignedInDisabled")]
    pub keep_me_signed_in_disabled: Option<bool>,
    #[serde(rename = "UseTransparentLightBox")]
    pub use_transparent_light_box: Option<bool>,
}

/// `GET {login_base}/GetUserRealm.srf?login={upn}`
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserRealmLegacy {
    #[serde(rename = "FederationBrandName")]
    pub federation_brand_name: Option<String>,
    #[serde(rename = "FederationGlobalVersion")]
    pub federation_global_version: Option<i64>,
    #[serde(rename = "AuthURL")]
    pub auth_url: Option<String>,
}

/// `POST {login_base}/common/GetCredentialType`
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CredentialTypeResponse {
    #[serde(rename = "Username")]
    pub username: Option<String>,
    #[serde(rename = "Display")]
    pub display: Option<String>,
    #[serde(rename = "IfExistsResult")]
    pub if_exists_result: Option<UserState>,
    #[serde(rename = "IsUnmanaged")]
    pub is_unmanaged: Option<bool>,
    #[serde(rename = "IsSignupDisallowed")]
    pub is_signup_disallowed: Option<bool>,
    #[serde(rename = "ThrottleStatus")]
    pub throttle_status: Option<ThrottleStatus>,
    #[serde(rename = "EstsProperties")]
    pub ests_properties: Option<EstsProperties>,
    #[serde(rename = "Credentials")]
    pub credentials: Option<Credentials>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Credentials {
    #[serde(rename = "PrefCredential")]
    pub pref_credential: Option<CredentialType>,
    #[serde(rename = "HasPassword")]
    pub has_password: Option<bool>,
    #[serde(rename = "HasAccessPass")]
    pub has_access_pass: Option<bool>,
    #[serde(rename = "HasRemoteNGC")]
    pub has_remote_ngc: Option<bool>,
    #[serde(rename = "HasFido")]
    pub has_fido: Option<bool>,
    #[serde(rename = "HasCertAuth")]
    pub has_cert_auth: Option<bool>,
    #[serde(rename = "HasGoogleFed")]
    pub has_google_fed: Option<bool>,
    #[serde(rename = "HasQrCodePin")]
    pub has_qr_code_pin: Option<bool>,
    #[serde(rename = "OtcNotAutoSent")]
    pub otc_not_auto_sent: Option<bool>,
    #[serde(rename = "RemoteNgcParams")]
    pub remote_ngc_params: Option<serde_json::Value>,
    #[serde(rename = "FidoParams")]
    pub fido_params: Option<serde_json::Value>,
    #[serde(rename = "QrCodePinParams")]
    pub qr_code_pin_params: Option<serde_json::Value>,
    #[serde(rename = "SasParams")]
    pub sas_params: Option<serde_json::Value>,
    #[serde(rename = "CertAuthParams")]
    pub cert_auth_params: Option<serde_json::Value>,
    #[serde(rename = "GoogleParams")]
    pub google_params: Option<serde_json::Value>,
    #[serde(rename = "FacebookParams")]
    pub facebook_params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct EstsProperties {
    #[serde(rename = "DesktopSsoEnabled")]
    pub desktop_sso_enabled: Option<bool>,
    #[serde(rename = "DomainType")]
    pub domain_type: Option<DomainType>,
    #[serde(rename = "CallMetadata")]
    pub call_metadata: Option<CallMetadata>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CallMetadata {
    #[serde(rename = "LongRunningTransactionPartition")]
    pub long_running_transaction_partition: Option<String>,
    #[serde(rename = "Region")]
    pub region: Option<String>,
    #[serde(rename = "ScaleUnit")]
    pub scale_unit: Option<String>,
    #[serde(rename = "IsLongRunningTransaction")]
    pub is_long_running_transaction: Option<bool>,
}

/// `GET {login_base}/{domain}/.well-known/openid-configuration`
#[derive(Debug, Clone, Default, Deserialize)]
pub struct OpenIdConfiguration {
    pub issuer: Option<String>,
    pub tenant_region_scope: Option<String>,
    pub tenant_region_sub_scope: Option<String>,
    pub cloud_instance_name: Option<String>,
    pub cloud_graph_host_name: Option<String>,
    pub msgraph_host: Option<String>,
    pub authorization_endpoint: Option<String>,
    pub token_endpoint: Option<String>,
}

/// `GET https://graph.microsoft.com/v1.0/tenantRelationships/findTenantInformationByTenantId(...)`
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TenantInformation {
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "federationBrandName")]
    pub federation_brand_name: Option<String>,
    #[serde(rename = "defaultDomainName")]
    pub default_domain_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_unknown_fields() {
        let raw = serde_json::json!({
            "tenantId": "1234",
            "environment": "Global",
            "somethingBrandNew": { "nested": true }
        });
        let parsed: FederationProvider = serde_json::from_value(raw).unwrap();
        assert_eq!(parsed.tenant_id.as_deref(), Some("1234"));
    }

    #[test]
    fn parses_credential_type_payload() {
        let raw = serde_json::json!({
            "Username": "admin@contoso.com",
            "IfExistsResult": 0,
            "ThrottleStatus": 0,
            "IsUnmanaged": false,
            "Credentials": { "PrefCredential": 1, "HasPassword": true, "FidoParams": null },
            "EstsProperties": { "DesktopSsoEnabled": true, "CallMetadata": { "Region": "EU" } }
        });
        let parsed: CredentialTypeResponse = serde_json::from_value(raw).unwrap();
        assert_eq!(parsed.if_exists_result, Some(UserState::Exists));
        assert_eq!(
            parsed.credentials.unwrap().pref_credential,
            Some(CredentialType::Password)
        );
        assert_eq!(
            parsed
                .ests_properties
                .unwrap()
                .call_metadata
                .unwrap()
                .region
                .as_deref(),
            Some("EU")
        );
    }

    #[test]
    fn missing_optional_blocks_stay_none() {
        let parsed: CredentialTypeResponse = serde_json::from_value(serde_json::json!({})).unwrap();
        assert!(parsed.is_unmanaged.is_none());
        assert!(parsed.credentials.is_none());
    }
}
