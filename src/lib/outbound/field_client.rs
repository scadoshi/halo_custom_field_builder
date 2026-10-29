//! Creates fields through Halo's `fieldinfo` endpoint.

use std::time::Duration;

use anyhow::Context;
use log::debug;
use reqwest::Client as ReqwestClient;
use tokio::time;

use crate::config::Config;
use crate::domain::models::custom_field::CustomField;
use crate::outbound::auth::client::AuthClient;
use crate::outbound::http_custom_field::HttpCustomField;

/// The wait before each field request. Halo allows 700 requests per five
/// minutes; this caps the program at 120 a minute.
const REQUEST_DELAY: Duration = Duration::from_millis(500);

/// Creates fields in Halo, one request at a time, at most two a second.
#[derive(Debug)]
pub struct FieldClient {
    /// Halo's field creation endpoint.
    endpoint: String,
    http_client: ReqwestClient,
    auth_client: AuthClient,
}

impl FieldClient {
    /// A client for the API in `config` that authenticates with `auth_client`.
    pub fn new(config: &Config, auth_client: AuthClient) -> Self {
        Self {
            endpoint: format!("{}/fieldinfo", config.api_url),
            http_client: ReqwestClient::new(),
            auth_client,
        }
    }

    /// Waits 500 ms, then creates the field.
    ///
    /// # Errors
    ///
    /// Fails when no token can be had, the request can't be sent, or Halo
    /// answers with an error status (the message holds the status and body).
    pub async fn create_field(&mut self, custom_field: &CustomField) -> anyhow::Result<()> {
        time::sleep(REQUEST_DELAY).await;

        let http_custom_field = HttpCustomField::from(custom_field);
        // halo api requires an array of custom field update objects
        let wrapped_http_custom_field = vec![http_custom_field];

        // The cached token, or a new one when it is about to expire.
        let auth_header = self.auth_client.fetch_token().await?;

        debug!("sending field creation request for: {}", custom_field.label);

        let response = self
            .http_client
            .post(&self.endpoint)
            .header("Authorization", auth_header)
            .json(&wrapped_http_custom_field)
            .send()
            .await
            .context("failed to send request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "failed to get error response".to_string());

            anyhow::bail!(
                "field creation failed for '{}': status {}, error: {}",
                custom_field.label,
                status,
                error_text
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::test_server::{self, Reply, TestServer};

    /// A token that counts as expired two seconds after it arrives.
    const SHORT_TOKEN: Reply = Reply::Json(
        200,
        r#"{"access_token":"one","token_type":"Bearer","expires_in":32}"#,
    );
    const NEXT_TOKEN: Reply = Reply::Json(
        200,
        r#"{"access_token":"two","token_type":"Bearer","expires_in":3600}"#,
    );
    const CREATED: Reply = Reply::Json(200, "[]");

    fn field() -> CustomField {
        CustomField::new("notes", "notes", 1, None, None).unwrap()
    }

    #[tokio::test]
    async fn each_request_names_its_content_type_once() {
        let server = TestServer::start(vec![test_server::TOKEN, CREATED]).await;
        let config = server.config();
        let mut client = FieldClient::new(&config, AuthClient::new(&config));

        client.create_field(&field()).await.unwrap();

        let requests = server.requests();
        let content_types = |request: &test_server::Request| {
            request
                .headers
                .iter()
                .filter(|line| line.to_ascii_lowercase().starts_with("content-type:"))
                .count()
        };
        assert_eq!(
            requests[0].header("content-type"),
            Some("application/x-www-form-urlencoded")
        );
        assert_eq!(requests[1].header("content-type"), Some("application/json"));
        assert_eq!(content_types(&requests[0]), 1);
        assert_eq!(content_types(&requests[1]), 1);
    }

    #[tokio::test]
    async fn a_token_that_expires_mid_run_is_replaced_before_the_next_request() {
        let server = TestServer::start(vec![SHORT_TOKEN, CREATED, NEXT_TOKEN, CREATED]).await;
        let config = server.config();
        let mut auth = AuthClient::new(&config);
        auth.fetch_token().await.unwrap();
        let mut client = FieldClient::new(&config, auth);

        client.create_field(&field()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(2500)).await;
        client.create_field(&field()).await.unwrap();

        let requests = server.requests();
        assert_eq!(
            test_server::lines(&requests),
            [
                "POST /auth/token HTTP/1.1",
                "POST /api/fieldinfo HTTP/1.1",
                "POST /auth/token HTTP/1.1",
                "POST /api/fieldinfo HTTP/1.1",
            ]
        );
        assert_eq!(requests[1].header("authorization"), Some("Bearer one"));
        assert_eq!(requests[3].header("authorization"), Some("Bearer two"));
    }
}
