//! The public API response. Field names and nesting follow `_old/main.py` so existing
//! consumers keep working; new diagnostics live under `meta`.

use serde::Serialize;

use crate::enums::{CredentialType, DomainType, ThrottleStatus, UserState};

#[derive(Debug, Clone, Serialize)]
pub struct LookupResponse {
    pub query: Query,
    #[serde(rename = "tenantId")]
    pub tenant_id: String,
    #[serde(rename = "tenantName")]
    pub tenant_name: Option<String>,
    #[serde(rename = "defaultDomain")]
    pub default_domain: Option<String>,
    #[serde(rename = "tenantEnvironment")]
    pub tenant_environment: TenantEnvironment,
    #[serde(rename = "nameSpaceType")]
    pub name_space_type: Option<String>,
    pub endpoints: Endpoints,
    #[serde(rename = "federationInfo")]
    pub federation_info: FederationInfo,
    #[serde(rename = "loginExperiences")]
    pub login_experiences: LoginExperiences,
    #[serde(rename = "userInfo")]
    pub user_info: UserInfo,
    pub domains: Vec<String>,
    pub raw: RawResponses,
    pub meta: Meta,
}

#[derive(Debug, Clone, Serialize)]
pub struct Query {
    /// Exactly what the caller asked for, after normalization.
    pub input: String,
    pub domain: String,
    /// The UPN probed against the login endpoints. Synthesized as `admin@{domain}`
    /// when the caller supplied a bare domain.
    pub username: String,
    #[serde(rename = "usernameProvided")]
    pub username_provided: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TenantEnvironment {
    #[serde(rename = "tenantRegionScope")]
    pub tenant_region_scope: Option<String>,
    #[serde(rename = "tenantRegionSubScope")]
    pub tenant_region_sub_scope: Option<String>,
    #[serde(rename = "cloudInstanceDisplayName")]
    pub cloud_instance_display_name: String,
    #[serde(rename = "cloudInstance")]
    pub cloud_instance: Option<String>,
    #[serde(rename = "audienceUrn")]
    pub audience_urn: Option<String>,
}

/// OIDC endpoints for the tenant, straight from its discovery document.
#[derive(Debug, Clone, Serialize)]
pub struct Endpoints {
    pub issuer: Option<String>,
    #[serde(rename = "authorizationEndpoint")]
    pub authorization_endpoint: Option<String>,
    #[serde(rename = "tokenEndpoint")]
    pub token_endpoint: Option<String>,
    #[serde(rename = "graphHost")]
    pub graph_host: Option<String>,
    #[serde(rename = "cloudGraphHost")]
    pub cloud_graph_host: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FederationInfo {
    #[serde(rename = "brandName")]
    pub brand_name: Option<String>,
    pub protocol: Option<String>,
    #[serde(rename = "globalVersion")]
    pub global_version: Option<i64>,
    #[serde(rename = "authUrl")]
    pub auth_url: Option<String>,
    #[serde(rename = "metadataUrl")]
    pub metadata_url: Option<String>,
    #[serde(rename = "activeAuthenticationUrl")]
    pub active_authentication_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginExperiences {
    /// `null` when `GetCredentialType` was not consulted, rather than a misleading `true`.
    #[serde(rename = "isSignupAllowed")]
    pub is_signup_allowed: Option<bool>,
    pub locale: Option<i64>,
    #[serde(rename = "bannerLogo")]
    pub banner_logo: Option<String>,
    #[serde(rename = "tileLogo")]
    pub tile_logo: Option<String>,
    #[serde(rename = "tileDarkLogo")]
    pub tile_dark_logo: Option<String>,
    pub illustration: Option<String>,
    #[serde(rename = "backgroundColor")]
    pub background_color: Option<String>,
    #[serde(rename = "boilerPlateText")]
    pub boiler_plate_text: Option<String>,
    #[serde(rename = "userIdLabel")]
    pub user_id_label: Option<String>,
    #[serde(rename = "keepMeSignedInDisabled")]
    pub keep_me_signed_in_disabled: Option<bool>,
    #[serde(rename = "useTransparentLightBox")]
    pub use_transparent_light_box: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    pub username: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub state: Option<UserState>,
    #[serde(rename = "domainType")]
    pub domain_type: Option<DomainType>,
    #[serde(rename = "isManaged")]
    pub is_managed: Option<bool>,
    #[serde(rename = "throttleStatus")]
    pub throttle_status: Option<ThrottleStatus>,
    pub credentials: CredentialSummary,
    #[serde(rename = "callMetadata")]
    pub call_metadata: CallMetadata,
}

#[derive(Debug, Clone, Serialize)]
pub struct CredentialSummary {
    #[serde(rename = "preferredCredential")]
    pub preferred_credential: Option<CredentialType>,
    #[serde(rename = "hasPassword")]
    pub has_password: Option<bool>,
    #[serde(rename = "hasAccessPass")]
    pub has_access_pass: Option<bool>,
    #[serde(rename = "hasDesktopSso")]
    pub has_desktop_sso: Option<bool>,
    #[serde(rename = "hasRemoteNGC")]
    pub has_remote_ngc: Option<bool>,
    #[serde(rename = "hasFido")]
    pub has_fido: Option<bool>,
    #[serde(rename = "hasCertAuth")]
    pub has_cert_auth: Option<bool>,
    #[serde(rename = "hasGoogleFederation")]
    pub has_google_federation: Option<bool>,
    #[serde(rename = "hasQrCodePin")]
    pub has_qr_code_pin: Option<bool>,
    #[serde(rename = "otcNotAutoSent")]
    pub otc_not_auto_sent: Option<bool>,
    pub parameters: CredentialParameters,
}

#[derive(Debug, Clone, Serialize)]
pub struct CredentialParameters {
    #[serde(rename = "remoteNgc")]
    pub remote_ngc: Option<serde_json::Value>,
    pub fido: Option<serde_json::Value>,
    #[serde(rename = "qrCodePin")]
    pub qr_code_pin: Option<serde_json::Value>,
    pub sas: Option<serde_json::Value>,
    #[serde(rename = "certAuth")]
    pub cert_auth: Option<serde_json::Value>,
    pub google: Option<serde_json::Value>,
    pub facebook: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RawResponses {
    pub odc_federationprovider: serde_json::Value,
    pub userrealm_v1: serde_json::Value,
    pub userrealm_v2: serde_json::Value,
    pub userrealm_old: serde_json::Value,
    pub credential_type: serde_json::Value,
    pub autodiscover_federation_information: serde_json::Value,
    pub openid_configuration: serde_json::Value,
    pub tenant_information: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct Meta {
    /// Wall-clock time spent on the whole lookup, in milliseconds.
    #[serde(rename = "durationMs")]
    pub duration_ms: u32,
    pub cache: CacheInfo,
    /// Endpoints that were skipped or failed, e.g. `"graph:no-credentials"`.
    pub degraded: Vec<String>,
    pub calls: Vec<UpstreamCall>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheInfo {
    pub status: &'static str,
    #[serde(rename = "ttlSeconds")]
    pub ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpstreamCall {
    pub name: &'static str,
    pub method: &'static str,
    pub url: String,
    pub status: Option<u16>,
    #[serde(rename = "durationMs")]
    pub duration_ms: u32,
    pub ok: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CallMetadata {
    #[serde(rename = "longRunningTransactionPartition")]
    pub long_running_transaction_partition: Option<String>,
    pub region: Option<String>,
    #[serde(rename = "scaleUnit")]
    pub scale_unit: Option<String>,
    #[serde(rename = "isLongRunningTransaction")]
    pub is_long_running_transaction: Option<bool>,
}
