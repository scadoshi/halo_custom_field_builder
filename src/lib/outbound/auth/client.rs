use std::fmt;

use anyhow::Context;
use reqwest::Client as ReqwestClient;
use serde::{Deserialize, Serialize};
use url::Url;

use super::token::AuthToken;
use crate::config::{Config, REDACTED};

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
}

#[derive(Serialize)]
struct TokenRequest<'config> {
    client_id: &'config str,
    client_secret: &'config str,
    grant_type: &'static str,
}

pub struct AuthClient {
    token_url: Url,
    client_id: String,
    client_secret: String,
    http_client: ReqwestClient,
    current_token: Option<AuthToken>,
}

impl fmt::Debug for AuthClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthClient")
            .field("token_url", &self.token_url)
            .field("client_id", &self.client_id)
            .field("client_secret", &REDACTED)
            .field("http_client", &self.http_client)
            .field("current_token", &self.current_token)
            .finish()
    }
}

impl AuthClient {
    pub fn new(config: &Config) -> Self {
        Self {
            token_url: config.token_url.clone(),
            client_id: config.client_id.clone(),
            client_secret: config.client_secret.clone(),
            http_client: ReqwestClient::new(),
            current_token: None,
        }
    }

    pub async fn fetch_token(&mut self) -> anyhow::Result<String> {
        if let Some(token) = &self.current_token
            && !token.is_expired()
        {
            return Ok(token.header_value());
        }

        let new_token = self.request_token().await?;
        let header = new_token.header_value();
        self.current_token = Some(new_token);

        Ok(header)
    }

    async fn request_token(&self) -> anyhow::Result<AuthToken> {
        let token_request = TokenRequest {
            client_id: &self.client_id,
            client_secret: &self.client_secret,
            grant_type: "client_credentials",
        };

        let response = self
            .http_client
            .post(self.token_url.as_str())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&token_request)
            .send()
            .await
            .context("failed to send token request")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("failed to read token response")?;

        // A failed request's body is an error from Halo, never a token.
        if !status.is_success() {
            anyhow::bail!("token request failed with status {status}: {response_text}");
        }

        let token_response: TokenResponse =
            serde_json::from_str(&response_text).context("failed to parse token response")?;

        AuthToken::new(
            token_response.access_token,
            token_response.token_type,
            token_response.expires_in,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_server::{CLIENT_SECRET, Reply, TOKEN, TestServer};

    async fn token_error(reply: Reply) -> anyhow::Error {
        let server = TestServer::start(vec![reply]).await;
        AuthClient::new(&server.config())
            .fetch_token()
            .await
            .unwrap_err()
    }

    #[test]
    fn debug_output_hides_the_client_secret() {
        let config = Config::from_lookup(|key| match key {
            "BASE_URL" => Some("https://example.halo.com".to_string()),
            "CLIENT_SECRET" => Some("hunter2".to_string()),
            _ => Some("x".to_string()),
        })
        .unwrap();
        let debug = format!("{:?}", AuthClient::new(&config));
        assert!(!debug.contains("hunter2"), "{debug}");
    }

    #[tokio::test]
    async fn a_token_response_becomes_the_authorization_header() {
        let server = TestServer::start(vec![TOKEN]).await;
        let header = AuthClient::new(&server.config())
            .fetch_token()
            .await
            .unwrap();
        assert_eq!(header, "Bearer tok");
        let requests = server.requests();
        assert!(requests[0].body.contains("grant_type=client_credentials"));
    }

    #[tokio::test]
    async fn a_failed_token_request_shows_the_status_and_body() {
        let cases = [
            (
                Reply::Json(
                    401,
                    r#"{"error":"invalid_client","error_description":"bad client"}"#,
                ),
                "401",
                "bad client",
            ),
            (
                Reply::Json(400, r#"{"error":"unsupported_grant_type"}"#),
                "400",
                "unsupported_grant_type",
            ),
            (
                Reply::Json(404, "<html><body>not here</body></html>"),
                "404",
                "not here",
            ),
            (Reply::Json(500, ""), "500", ""),
        ];
        for (reply, status, body) in cases {
            let message = format!("{:#}", token_error(reply).await);
            assert!(message.contains(status), "{status}: {message}");
            assert!(message.contains(body), "{status}: {message}");
            assert!(!message.contains(CLIENT_SECRET), "{status}: {message}");
        }
    }

    #[tokio::test]
    async fn an_unreadable_token_response_keeps_the_parse_error_as_its_cause() {
        let error = token_error(Reply::Json(200, "<html>sign in</html>")).await;
        assert!(
            error
                .root_cause()
                .downcast_ref::<serde_json::Error>()
                .is_some(),
            "{error:#}"
        );
    }

    #[tokio::test]
    async fn an_expiry_too_far_out_is_an_error() {
        for expires_in in [
            "9223372036854775807",
            "9223372036854775",
            "-9223372036854775808",
        ] {
            let body = format!(
                r#"{{"access_token":"tok","token_type":"Bearer","expires_in":{expires_in}}}"#
            );
            let server = TestServer::start(vec![Reply::Json(200, body.leak())]).await;
            let message = format!(
                "{:#}",
                AuthClient::new(&server.config())
                    .fetch_token()
                    .await
                    .unwrap_err()
            );
            assert!(message.contains("expires_in"), "{expires_in}: {message}");
        }
    }
}
