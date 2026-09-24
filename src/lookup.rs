//! Fans out to every Microsoft endpoint and folds the results into one response.
//!
//! Port of `getTenantInfos` in `_old/main.py`. `futures::join!` replaces `asyncio.TaskGroup`:
//! Workers are single-threaded, so this is cooperative concurrency over in-flight subrequests.

use worker::Env;

use crate::enums::CloudEnvironment;
use crate::error::ApiError;
use crate::model::response::{
    CacheInfo, CallMetadata, CredentialParameters, CredentialSummary, Endpoints, FederationInfo,
    LoginExperiences, LookupResponse, Meta, Query, RawResponses, TenantEnvironment, UserInfo,
};
use crate::model::upstream::{
    CredentialTypeResponse, FederationProvider, OpenIdConfiguration, TenantInformation,
    UserRealmLegacy, UserRealmV1, UserRealmV2,
};
use crate::msft::{autodiscover, credential_type, graph, http::Recorder, odc, openid, userrealm};
use crate::validate::Target;

pub async fn run(env: &Env, target: &Target) -> Result<LookupResponse, ApiError> {
    let started = worker::Date::now().as_millis();
    let recorder = Recorder::default();
    let mut degraded: Vec<String> = Vec::new();

    let odc_outcome = odc::federation_provider(&recorder, &target.domain).await;
    let raw_odc = odc_outcome.json();
    let provider: FederationProvider = parse(&raw_odc);

    let tenant_id = provider.tenant_id.clone().ok_or_else(|| {
        if odc_outcome.call.ok {
            ApiError::TenantNotFound(format!(
                "{} is not backed by a Microsoft Entra ID tenant.",
                target.domain
            ))
        } else {
            ApiError::upstream(
                "The Office discovery service did not answer. Try again in a moment.",
            )
        }
    })?;

    let environment_key = provider.environment.clone().unwrap_or_default();
    let cloud = CloudEnvironment::from_key(&environment_key).ok_or_else(|| {
        ApiError::upstream(format!(
            "Unknown Microsoft cloud environment '{environment_key}'."
        ))
    })?;

    let graph_credentials = graph::credentials(env);

    let (
        v1_outcome,
        v2_outcome,
        legacy_outcome,
        autodiscover_outcome,
        openid_outcome,
        credential_outcome,
        tenant_outcome,
    ) = futures::join!(
        userrealm::v1(&recorder, cloud.login_base, &target.username),
        userrealm::v2(&recorder, cloud.login_base, &target.username),
        userrealm::legacy(&recorder, cloud.login_base, &target.username),
        autodiscover::federation_information(&recorder, cloud.autodiscover_base, &target.domain),
        openid::configuration(&recorder, cloud.login_base, &target.domain),
        async {
            match cloud.supports_credential_type() {
                true => Some(
                    credential_type::get_credential_type(
                        &recorder,
                        cloud.login_base,
                        &target.username,
                    )
                    .await,
                ),
                false => None,
            }
        },
        async {
            match (&graph_credentials, cloud.supports_graph()) {
                (Some(credentials), true) => {
                    Some(graph::tenant_information(&recorder, env, credentials, &tenant_id).await)
                }
                _ => None,
            }
        }
    );

    if graph_credentials.is_none() {
        degraded.push("tenant_information:not-configured".into());
    } else if !cloud.supports_graph() {
        degraded.push("tenant_information:unsupported-cloud".into());
    }
    if !cloud.supports_credential_type() {
        degraded.push("credential_type:unsupported-cloud".into());
    }

    let raw_v1 = v1_outcome.json();
    let raw_v2 = v2_outcome.json();
    let raw_legacy = legacy_outcome.json();
    let raw_openid = openid_outcome.json();
    let raw_autodiscover = autodiscover_outcome.text();
    let raw_credential = credential_outcome
        .as_ref()
        .map(|o| o.json())
        .unwrap_or(serde_json::Value::Null);
    let raw_tenant = tenant_outcome
        .as_ref()
        .map(|o| o.json())
        .unwrap_or(serde_json::Value::Null);

    let v1: UserRealmV1 = parse(&raw_v1);
    let v2: UserRealmV2 = parse(&raw_v2);
    let legacy: UserRealmLegacy = parse(&raw_legacy);
    let openid: OpenIdConfiguration = parse(&raw_openid);
    let credential: CredentialTypeResponse = parse(&raw_credential);
    let tenant: TenantInformation = parse(&raw_tenant);

    let branding = v2
        .tenant_branding_info
        .as_ref()
        .and_then(|list| list.first())
        .cloned()
        .unwrap_or_default();

    let domains = autodiscover_outcome
        .body
        .as_deref()
        .map(autodiscover::parse_domains)
        .unwrap_or_default();

    let credentials = credential.credentials.clone().unwrap_or_default();
    let ests = credential.ests_properties.clone().unwrap_or_default();
    let call_metadata = ests.call_metadata.clone().unwrap_or_default();

    let calls = recorder.into_calls();
    degraded.extend(
        calls
            .iter()
            .filter(|call| !call.ok)
            .map(|call| format!("{}:failed", call.name)),
    );

    Ok(LookupResponse {
        query: Query {
            input: target.input.clone(),
            domain: target.domain.clone(),
            username: target.username.clone(),
            username_provided: target.username_provided,
        },
        tenant_id,
        tenant_name: v2
            .federation_brand_name
            .clone()
            .or_else(|| tenant.display_name.clone())
            .or_else(|| tenant.federation_brand_name.clone()),
        default_domain: tenant.default_domain_name.clone(),
        tenant_environment: TenantEnvironment {
            tenant_region_scope: openid.tenant_region_scope.clone(),
            tenant_region_sub_scope: openid.tenant_region_sub_scope.clone(),
            cloud_instance_display_name: cloud.display_name.to_string(),
            cloud_instance: openid.cloud_instance_name.clone(),
            audience_urn: v1.cloud_audience_urn.clone(),
        },
        name_space_type: v2.name_space_type.clone(),
        endpoints: Endpoints {
            issuer: openid.issuer.clone(),
            authorization_endpoint: openid.authorization_endpoint.clone(),
            token_endpoint: openid.token_endpoint.clone(),
            graph_host: openid.msgraph_host.clone(),
            cloud_graph_host: openid.cloud_graph_host_name.clone(),
        },
        federation_info: FederationInfo {
            brand_name: legacy.federation_brand_name.clone(),
            protocol: v2.federation_protocol.clone().or(v1.federation_protocol),
            global_version: legacy.federation_global_version,
            auth_url: legacy.auth_url.clone(),
            metadata_url: v1.federation_metadata_url,
            active_authentication_url: v1.federation_active_auth_url,
        },
        login_experiences: LoginExperiences {
            is_signup_allowed: credential
                .is_signup_disallowed
                .map(|disallowed| !disallowed),
            locale: branding.locale,
            banner_logo: branding.banner_logo,
            tile_logo: branding.tile_logo,
            tile_dark_logo: branding.tile_dark_logo,
            illustration: branding.illustration,
            background_color: branding.background_color,
            boiler_plate_text: branding.boiler_plate_text,
            user_id_label: branding.user_id_label,
            keep_me_signed_in_disabled: branding.keep_me_signed_in_disabled,
            use_transparent_light_box: branding.use_transparent_light_box,
        },
        user_info: UserInfo {
            username: credential.username.clone(),
            display_name: credential.display.clone(),
            state: credential.if_exists_result,
            domain_type: ests.domain_type,
            is_managed: credential.is_unmanaged.map(|unmanaged| !unmanaged),
            throttle_status: credential.throttle_status,
            credentials: CredentialSummary {
                preferred_credential: credentials.pref_credential,
                has_password: credentials.has_password,
                has_access_pass: credentials.has_access_pass,
                has_desktop_sso: ests.desktop_sso_enabled,
                has_remote_ngc: credentials.has_remote_ngc,
                has_fido: credentials.has_fido,
                has_cert_auth: credentials.has_cert_auth,
                has_google_federation: credentials.has_google_fed,
                has_qr_code_pin: credentials.has_qr_code_pin,
                otc_not_auto_sent: credentials.otc_not_auto_sent,
                parameters: CredentialParameters {
                    remote_ngc: credentials.remote_ngc_params,
                    fido: credentials.fido_params,
                    qr_code_pin: credentials.qr_code_pin_params,
                    sas: credentials.sas_params,
                    cert_auth: credentials.cert_auth_params,
                    google: credentials.google_params,
                    facebook: credentials.facebook_params,
                },
            },
            call_metadata: CallMetadata {
                long_running_transaction_partition: call_metadata
                    .long_running_transaction_partition,
                region: call_metadata.region,
                scale_unit: call_metadata.scale_unit,
                is_long_running_transaction: call_metadata.is_long_running_transaction,
            },
        },
        domains,
        raw: RawResponses {
            odc_federationprovider: raw_odc,
            userrealm_v1: raw_v1,
            userrealm_v2: raw_v2,
            userrealm_old: raw_legacy,
            credential_type: raw_credential,
            autodiscover_federation_information: raw_autodiscover,
            openid_configuration: raw_openid,
            tenant_information: raw_tenant,
        },
        meta: Meta {
            duration_ms: worker::Date::now().as_millis().saturating_sub(started) as u32,
            cache: CacheInfo {
                status: "MISS",
                ttl_seconds: 0,
            },
            degraded,
            calls,
        },
    })
}

fn parse<T: serde::de::DeserializeOwned + Default>(value: &serde_json::Value) -> T {
    serde_json::from_value(value.clone()).unwrap_or_default()
}
