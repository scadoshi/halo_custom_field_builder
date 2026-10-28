use anyhow::Context;
use chrono::{DateTime, Duration, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthToken {
    access_token: String,
    token_type: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    expires_at: DateTime<Utc>,
}

impl AuthToken {
    /// A token that expires `expires_in` seconds from now.
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

    pub fn is_expired(&self) -> bool {
        // Add a small buffer (30 seconds) to prevent edge cases
        Utc::now() + Duration::seconds(30) >= self.expires_at
    }

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
    fn the_header_is_the_type_then_the_token() {
        assert_eq!(
            AuthToken::new("abc".into(), "Bearer".into(), 60)
                .unwrap()
                .header_value(),
            "Bearer abc"
        );
    }
}
