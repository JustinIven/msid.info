//! Port of the lookup tables in `_old/static.py`.
//!
//! Every coded enum keeps an `Unknown(i64)` arm so that new values invented by Microsoft
//! surface as `"Unknown(<code>)"` instead of failing the whole deserialization.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

macro_rules! coded_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident = $code:literal => $label:literal),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $($variant,)*
            Unknown(i64),
        }

        impl $name {
            pub fn from_code(code: i64) -> Self {
                match code {
                    $($code => Self::$variant,)*
                    other => Self::Unknown(other),
                }
            }

            pub fn label(&self) -> String {
                match self {
                    $(Self::$variant => $label.to_string(),)*
                    Self::Unknown(code) => format!("Unknown({code})"),
                }
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.label())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                Ok(Self::from_code(i64::deserialize(deserializer)?))
            }
        }
    };
}

coded_enum!(ThrottleStatus {
    NotThrottled = 0 => "NotThrottled",
    AadThrottled = 1 => "AadThrottled",
    MsaThrottled = 2 => "MsaThrottled",
});

coded_enum!(CredentialType {
    None = 0 => "None",
    Password = 1 => "Password",
    RemoteNgc = 2 => "RemoteNGC",
    OneTimeCode = 3 => "OneTimeCode",
    Federation = 4 => "Federation",
    CloudFederation = 5 => "CloudFederation",
    OtherMicrosoftIdpFederation = 6 => "OtherMicrosoftIdpFederation",
    Fido = 7 => "Fido",
    GitHub = 8 => "GitHub",
    PublicIdentifierCode = 9 => "PublicIdentifierCode",
    LinkedIn = 10 => "LinkedIn",
    RemoteLogin = 11 => "RemoteLogin",
    Google = 12 => "Google",
    AccessPass = 13 => "AccessPass",
    Facebook = 14 => "Facebook",
    Certificate = 15 => "Certificate",
    OfflineAccount = 16 => "OfflineAccount",
    VerifiableCredential = 17 => "VerifiableCredential",
    QrCodePin = 18 => "QrCodePin",
    NoPreferredCredential = 1000 => "NoPreferredCredential",
});

coded_enum!(DomainType {
    Unknown_ = 1 => "Unknown",
    Consumer = 2 => "Consumer",
    Managed = 3 => "Managed",
    Federated = 4 => "Federated",
    CloudFederated = 5 => "CloudFederated",
});

coded_enum!(UserState {
    Unknown_ = -1 => "Unknown",
    Exists = 0 => "Exists",
    NotExist = 1 => "NotExist",
    Throttled = 2 => "Throttled",
    Error = 4 => "Error",
    ExistsInOtherMicrosoftIdp = 5 => "ExistsInOtherMicrosoftIDP",
    ExistsBothIdps = 6 => "ExistsBothIDPs",
});

/// One of the four sovereign Microsoft 365 clouds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloudEnvironment {
    /// Value of `environment` in the ODC federationprovider response.
    pub key: &'static str,
    pub display_name: &'static str,
    pub login_base: &'static str,
    pub autodiscover_base: &'static str,
}

pub const GLOBAL: CloudEnvironment = CloudEnvironment {
    key: "Global",
    display_name: "Worldwide",
    login_base: "https://login.microsoftonline.com",
    autodiscover_base: "https://autodiscover-s.outlook.com",
};

pub const US_GOV_GCC_HIGH: CloudEnvironment = CloudEnvironment {
    key: "microsoftonline.us",
    display_name: "U.S. Government GCC High",
    login_base: "https://login.microsoftonline.us",
    autodiscover_base: "https://autodiscover-s.office365.us",
};

pub const US_GOV_DOD: CloudEnvironment = CloudEnvironment {
    key: "microsoftonline.mil",
    display_name: "U.S. Government DoD",
    login_base: "https://login.microsoftonline.us",
    autodiscover_base: "https://autodiscover-s-dod.office365.us",
};

pub const CHINA: CloudEnvironment = CloudEnvironment {
    key: "partner.microsoftonline.cn",
    display_name: "Microsoft 365 operated by 21Vianet (China)",
    login_base: "https://login.partner.microsoftonline.cn",
    autodiscover_base: "https://autodiscover-s.partner.outlook.cn",
};

pub const ALL_ENVIRONMENTS: [CloudEnvironment; 4] = [GLOBAL, US_GOV_GCC_HIGH, US_GOV_DOD, CHINA];

impl CloudEnvironment {
    pub fn from_key(key: &str) -> Option<Self> {
        ALL_ENVIRONMENTS.into_iter().find(|env| env.key == key)
    }

    /// Microsoft Graph is only reachable for the commercial cloud with our app registration.
    pub fn supports_graph(&self) -> bool {
        self.key == GLOBAL.key
    }

    /// `GetCredentialType` only exists on the commercial and 21Vianet login endpoints.
    pub fn supports_credential_type(&self) -> bool {
        self.key == GLOBAL.key || self.key == CHINA.key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_known_codes_to_labels() {
        assert_eq!(CredentialType::from_code(7).label(), "Fido");
        assert_eq!(
            CredentialType::from_code(1000).label(),
            "NoPreferredCredential"
        );
        assert_eq!(UserState::from_code(-1).label(), "Unknown");
        assert_eq!(ThrottleStatus::from_code(2).label(), "MsaThrottled");
    }

    #[test]
    fn preserves_unknown_codes() {
        assert_eq!(CredentialType::from_code(42).label(), "Unknown(42)");
        assert_eq!(UserState::from_code(99).label(), "Unknown(99)");
    }

    #[test]
    fn deserializes_from_integers() {
        let value: UserState = serde_json::from_str("0").unwrap();
        assert_eq!(value, UserState::Exists);
        assert_eq!(serde_json::to_string(&value).unwrap(), "\"Exists\"");
    }

    #[test]
    fn resolves_environments() {
        assert_eq!(CloudEnvironment::from_key("Global"), Some(GLOBAL));
        assert_eq!(CloudEnvironment::from_key("nope"), None);
        assert!(GLOBAL.supports_graph());
        assert!(!CHINA.supports_graph());
        assert!(CHINA.supports_credential_type());
        assert!(!US_GOV_DOD.supports_credential_type());
    }
}
