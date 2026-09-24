//! Errors rendered as RFC 9457 `application/problem+json`.

use serde::Serialize;
use worker::{Response, ResponseBody};

#[derive(Debug, Clone)]
pub enum ApiError {
    InvalidInput(String),
    TenantNotFound(String),
    RateLimited { retry_after: u32 },
    Upstream(String),
    Internal(String),
}

impl ApiError {
    pub fn invalid_input(detail: impl Into<String>) -> Self {
        Self::InvalidInput(detail.into())
    }

    pub fn upstream(detail: impl Into<String>) -> Self {
        Self::Upstream(detail.into())
    }

    pub fn internal(detail: impl Into<String>) -> Self {
        Self::Internal(detail.into())
    }

    pub fn status(&self) -> u16 {
        match self {
            Self::InvalidInput(_) => 400,
            Self::TenantNotFound(_) => 404,
            Self::RateLimited { .. } => 429,
            Self::Upstream(_) => 502,
            Self::Internal(_) => 500,
        }
    }

    fn slug(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "invalid-input",
            Self::TenantNotFound(_) => "tenant-not-found",
            Self::RateLimited { .. } => "rate-limited",
            Self::Upstream(_) => "upstream-failure",
            Self::Internal(_) => "internal-error",
        }
    }

    fn title(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "Invalid input",
            Self::TenantNotFound(_) => "Tenant not found",
            Self::RateLimited { .. } => "Rate limit exceeded",
            Self::Upstream(_) => "Upstream request failed",
            Self::Internal(_) => "Internal error",
        }
    }

    fn detail(&self) -> String {
        match self {
            Self::InvalidInput(detail)
            | Self::TenantNotFound(detail)
            | Self::Upstream(detail)
            | Self::Internal(detail) => detail.clone(),
            Self::RateLimited { retry_after } => format!(
                "Too many lookups. Retry in {retry_after} seconds, or solve the Turnstile challenge for a higher limit."
            ),
        }
    }

    pub fn to_response(&self) -> worker::Result<Response> {
        let problem = Problem {
            r#type: format!("https://msid.info/errors/{}", self.slug()),
            title: self.title(),
            status: self.status(),
            detail: self.detail(),
        };

        let body = serde_json::to_string(&problem).unwrap_or_else(|_| {
            r#"{"title":"Internal error","status":500,"detail":"Serialization failed."}"#.into()
        });

        let mut response =
            Response::from_body(ResponseBody::Body(body.into_bytes()))?.with_status(self.status());
        response
            .headers_mut()
            .set("content-type", "application/problem+json; charset=utf-8")?;
        if let Self::RateLimited { retry_after } = self {
            response
                .headers_mut()
                .set("retry-after", &retry_after.to_string())?;
        }
        Ok(response)
    }
}

#[derive(Serialize)]
struct Problem {
    r#type: String,
    title: &'static str,
    status: u16,
    detail: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.title(), self.detail())
    }
}

impl From<worker::Error> for ApiError {
    fn from(err: worker::Error) -> Self {
        Self::Internal(err.to_string())
    }
}
