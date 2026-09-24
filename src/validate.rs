//! Input validation.
//!
//! The caller-supplied string is interpolated into upstream URLs, so it is validated against a
//! strict allowlist *before* it is used anywhere. Anything outside `[a-z0-9.-]` (plus a small
//! local-part alphabet) is rejected rather than escaped, which rules out path traversal,
//! query/fragment injection and scheme smuggling by construction.

use crate::error::ApiError;

const MAX_INPUT_LEN: usize = 320;
const MAX_DOMAIN_LEN: usize = 253;
const MAX_LABEL_LEN: usize = 63;
const MAX_LOCAL_PART_LEN: usize = 64;

/// A validated lookup target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub input: String,
    pub domain: String,
    pub username: String,
    pub username_provided: bool,
}

impl Target {
    /// Stable key for caching and rate limiting.
    pub fn cache_key(&self) -> String {
        self.input.clone()
    }
}

pub fn parse(raw: &str) -> Result<Target, ApiError> {
    let input = raw.trim().trim_matches('.').to_ascii_lowercase();

    if input.is_empty() {
        return Err(ApiError::invalid_input("Query must not be empty."));
    }
    if input.len() > MAX_INPUT_LEN {
        return Err(ApiError::invalid_input(
            "Query is longer than 320 characters.",
        ));
    }
    if !input.is_ascii() {
        return Err(ApiError::invalid_input(
            "Query must be ASCII. Convert internationalized domains to punycode (xn--...) first.",
        ));
    }

    match input.split_once('@') {
        Some((local, domain)) => {
            if domain.contains('@') {
                return Err(ApiError::invalid_input(
                    "Query must contain at most one '@'.",
                ));
            }
            validate_local_part(local)?;
            let domain = validate_domain(domain)?;
            Ok(Target {
                username: format!("{local}@{domain}"),
                input,
                domain,
                username_provided: true,
            })
        }
        None => {
            let domain = validate_domain(&input)?;
            Ok(Target {
                username: format!("admin@{domain}"),
                input,
                domain,
                username_provided: false,
            })
        }
    }
}

fn validate_local_part(local: &str) -> Result<(), ApiError> {
    if local.is_empty() || local.len() > MAX_LOCAL_PART_LEN {
        return Err(ApiError::invalid_input(
            "The part before '@' must be 1-64 characters long.",
        ));
    }
    let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '+' | '\'');
    if !local.chars().all(allowed) {
        return Err(ApiError::invalid_input(
            "The part before '@' contains unsupported characters.",
        ));
    }
    Ok(())
}

fn validate_domain(domain: &str) -> Result<String, ApiError> {
    if domain.is_empty() || domain.len() > MAX_DOMAIN_LEN {
        return Err(ApiError::invalid_input(
            "Domain must be 1-253 characters long.",
        ));
    }

    let labels: Vec<&str> = domain.split('.').collect();
    if labels.len() < 2 {
        return Err(ApiError::invalid_input(
            "Domain must contain at least one dot, e.g. contoso.com.",
        ));
    }

    for label in &labels {
        if label.is_empty() || label.len() > MAX_LABEL_LEN {
            return Err(ApiError::invalid_input(
                "Every domain label must be 1-63 characters long.",
            ));
        }
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(ApiError::invalid_input(
                "Domain may only contain letters, digits, hyphens and dots.",
            ));
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(ApiError::invalid_input(
                "Domain labels must not start or end with a hyphen.",
            ));
        }
    }

    let tld = labels[labels.len() - 1];
    if tld.chars().all(|c| c.is_ascii_digit()) {
        return Err(ApiError::invalid_input(
            "IP addresses are not valid lookup targets.",
        ));
    }

    Ok(domain.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_bare_domain_and_synthesizes_upn() {
        let target = parse("Contoso.COM").unwrap();
        assert_eq!(target.domain, "contoso.com");
        assert_eq!(target.username, "admin@contoso.com");
        assert!(!target.username_provided);
    }

    #[test]
    fn accepts_upn() {
        let target = parse(" User.Name+tag@Contoso.com ").unwrap();
        assert_eq!(target.domain, "contoso.com");
        assert_eq!(target.username, "user.name+tag@contoso.com");
        assert!(target.username_provided);
    }

    #[test]
    fn rejects_path_traversal_and_url_injection() {
        for bad in [
            "evil.com/../secret",
            "contoso.com/foo",
            "contoso.com?x=1",
            "contoso.com#frag",
            "http://contoso.com",
            "https://contoso.com",
            "contoso.com:8443",
            "contoso.com\\foo",
            "cont oso.com",
            "contoso.com\nHost: evil",
            "a@b@c.com",
            "contoso.com%2f..",
        ] {
            assert!(parse(bad).is_err(), "expected rejection of {bad:?}");
        }
    }

    #[test]
    fn rejects_malformed_domains() {
        for bad in [
            "",
            "   ",
            "localhost",
            "-contoso.com",
            "contoso-.com",
            "contoso..com",
            "127.0.0.1",
            "@contoso.com",
            "user@",
        ] {
            assert!(parse(bad).is_err(), "expected rejection of {bad:?}");
        }
    }

    #[test]
    fn rejects_oversized_input() {
        let long_label = "a".repeat(64);
        assert!(parse(&format!("{long_label}.com")).is_err());
        assert!(parse(&"a".repeat(400)).is_err());
    }

    #[test]
    fn rejects_non_ascii() {
        assert!(parse("bücher.de").is_err());
        assert!(parse("xn--bcher-kva.de").is_ok());
    }
}
