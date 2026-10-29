//! An access token and when it expires.

use std::fmt;

use anyhow::Context;
use chrono::{DateTime, TimeDelta, Utc};

use crate::config::REDACTED;

/// How long before its stated expiry a token counts as expired, so it never
/// runs out mid-request.
const EXPIRY_MARGIN: TimeDelta = TimeDelta::seconds(30);

/// An access token and when it expires. Debug output hides the token.
pub struct AuthToken {
    access_token: String,
    token_type: String,
    expires_at: DateTime<Utc>,
}

impl fmt::Debug for AuthToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthToken")
            .field("access_token", &REDACTED)
            .field("token_type", &self.token_type)
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

impl AuthToken {
    /// A token that expires `expires_in` seconds from now.
    ///
    /// # Errors
    ///
    /// Fails when `expires_in` puts the expiry out of the clock's range.
    pub fn new(access_token: String, token_type: String, expires_in: i64) -> anyhow::Result<Self> {
        let expires_at = TimeDelta::try_seconds(expires_in)
            .and_then(|lifetime| Utc::now().checked_add_signed(lifetime))
            .with_context(|| format!("token expires_in {expires_in} is out of range"))?;
        Ok(Self {
            access_token,
            token_type,
            expires_at,
        })
    }

    /// Whether the token expires within the next 30 seconds or already has.
    pub fn is_expired(&self) -> bool {
        // A clock too close to the end of time to add the margin to counts as expired.
        Utc::now()
            .checked_add_signed(EXPIRY_MARGIN)
            .is_none_or(|soon| soon >= self.expires_at)
    }

    /// The `Authorization` header value: the token type, a space, the token.
    pub fn header_value(&self) -> String {
        format!("{} {}", self.token_type, self.access_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_token_counts_as_expired_thirty_seconds_early() {
        assert!(
            AuthToken::new("t".into(), "Bearer".into(), 29)
                .unwrap()
                .is_expired()
        );
        assert!(
            !AuthToken::new("t".into(), "Bearer".into(), 31)
                .unwrap()
                .is_expired()
        );
        assert!(
            AuthToken::new("t".into(), "Bearer".into(), -1)
                .unwrap()
                .is_expired()
        );
    }

    #[test]
    fn debug_output_hides_the_access_token() {
        let token = AuthToken::new("hunter2".into(), "Bearer".into(), 60).unwrap();
        let debug = format!("{token:?}");
        assert!(!debug.contains("hunter2"), "{debug}");
        assert!(debug.contains("Bearer"), "{debug}");
    }

    #[test]
    fn the_header_is_the_type_then_the_token() {
        assert_eq!(
            AuthToken::new("abc".into(), "Bearer".into(), 60)
                .unwrap()
                .header_value(),
            "Bearer abc"
        );
    }
}
