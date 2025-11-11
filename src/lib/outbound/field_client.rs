
use anyhow::Context;
use log::debug;
use reqwest::Client as ReqwestClient;
use std::time::Duration;
use tokio::time::sleep;

use crate::config::Config;
use crate::domain::models::custom_field::CustomField;
use crate::outbound::http_custom_field::HttpCustomField;

pub struct FieldClient {
    config: Config,
    http_client: ReqwestClient,
    auth_token: String,
}

impl FieldClient {
    pub fn new(config: Config, auth_token: String) -> Self {
        Self {
            config,
            http_client: ReqwestClient::new(),
            auth_token,
        }
    }

    pub async fn create_field(&self, custom_field: &CustomField) -> anyhow::Result<()> {
        // rate limiting: 500ms delay between requests
        // max 120 requests/minute, staying under the 700/5min limit
        sleep(Duration::from_millis(500)).await;

        let endpoint = format!("{}/fieldinfo", self.config.api_url);
        let http_custom_field = HttpCustomField::from(custom_field);

        debug!("sending field creation request for: {}", custom_field.label);

        let response = self
            .http_client
            .post(&endpoint)
            .header("Authorization", &self.auth_token)
            .header("Content-Type", "application/json")
            .json(&http_custom_field)
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
