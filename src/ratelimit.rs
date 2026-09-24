//! Rate limiting via the Workers rate limiting binding.
//!
//! The binding is optional: if it is not configured (local `wrangler dev`, forks without the
//! bindings) the limiter fails open rather than taking the API down.

use worker::Env;

use crate::error::ApiError;
use crate::turnstile::Tier;

const RETRY_AFTER_SECONDS: u32 = 60;

pub async fn enforce(env: &Env, tier: Tier, key: &str) -> Result<(), ApiError> {
    let Ok(limiter) = env.rate_limiter(tier.rate_limiter_binding()) else {
        return Ok(());
    };

    match limiter.limit(key.to_string()).await {
        Ok(outcome) if !outcome.success => Err(ApiError::RateLimited {
            retry_after: RETRY_AFTER_SECONDS,
        }),
        _ => Ok(()),
    }
}
