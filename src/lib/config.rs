use anyhow::Context;
use url::Url;

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: Url,
    pub token_url: Url,
    pub api_url: Url,
    pub client_id: String,
    pub client_secret: String,
    pub source_file_name: String,
}

const BASE_URL_KEY: &str = "BASE_URL";
const CLIENT_ID_KEY: &str = "CLIENT_ID";
const CLIENT_SECRET_KEY: &str = "CLIENT_SECRET";
const SOURCE_FILE_NAME_KEY: &str = "SOURCE_FILE_NAME";

const TOKEN_URL_PATH: &str = "auth/token";
const API_URL_PATH: &str = "api";

impl Config {
    /// Reads `.env` from the working directory, then the process environment.
    pub fn load_from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().context("failed to load environment")?;
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    /// Builds the config from `lookup`, which answers `None` for an unset key.
    /// The base URL must be https: the client secret travels on every request.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> anyhow::Result<Self> {
        let var = |key: &'static str| lookup(key).with_context(|| format!("{key} is not set"));

        let base_url_str = var(BASE_URL_KEY)?;
        let base_url = Url::parse(&base_url_str)
            .with_context(|| format!("{BASE_URL_KEY} is not a URL: {base_url_str}"))?;
        if base_url.scheme() != "https" {
            anyhow::bail!("{BASE_URL_KEY} must start with https://, got {base_url_str}");
        }
        let mut token_url = base_url.clone();
        token_url.set_path(TOKEN_URL_PATH);
        let mut api_url = base_url.clone();
        api_url.set_path(API_URL_PATH);
        let client_id = var(CLIENT_ID_KEY)?;
        let client_secret = var(CLIENT_SECRET_KEY)?;
        let source_file_name = var(SOURCE_FILE_NAME_KEY)?;

        Ok(Self {
            base_url,
            token_url,
            api_url,
            client_id,
            client_secret,
            source_file_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn vars(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        move |key| map.get(key).cloned()
    }

    const COMPLETE: &[(&str, &str)] = &[
        ("BASE_URL", "https://example.halo.com"),
        ("CLIENT_ID", "id"),
        ("CLIENT_SECRET", "secret"),
        ("SOURCE_FILE_NAME", "source.csv"),
    ];

    fn with(key: &str, value: &str) -> anyhow::Result<Config> {
        let mut pairs: Vec<(&str, &str)> = COMPLETE
            .iter()
            .copied()
            .filter(|(k, _)| *k != key)
            .collect();
        pairs.push((key, value));
        Config::from_lookup(vars(&pairs))
    }

    #[test]
    fn a_complete_set_parses_and_derives_the_token_and_api_urls() {
        let config = Config::from_lookup(vars(COMPLETE)).unwrap();
        assert_eq!(
            config.token_url.as_str(),
            "https://example.halo.com/auth/token"
        );
        assert_eq!(config.api_url.as_str(), "https://example.halo.com/api");
    }

    #[test]
    fn a_missing_variable_is_named() {
        for (key, _) in COMPLETE {
            let without: Vec<(&str, &str)> =
                COMPLETE.iter().copied().filter(|(k, _)| k != key).collect();
            let message = Config::from_lookup(vars(&without)).unwrap_err().to_string();
            assert!(message.contains(key), "{message}");
        }
    }

    #[test]
    fn a_base_url_that_is_not_https_is_refused() {
        for bad in [
            "http://example.halo.com",
            "ftp://example.halo.com",
            "example.halo.com",
        ] {
            let message = with("BASE_URL", bad).unwrap_err().to_string();
            assert!(message.contains("BASE_URL"), "{bad}: {message}");
        }
        assert!(
            with("BASE_URL", "HTTPS://example.halo.com").is_ok(),
            "schemes are case-insensitive"
        );
    }
}
