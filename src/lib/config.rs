use anyhow::Context;
use url::Url;

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: Url,
    // pub tenant: String,
    pub token_url: Url,
    pub api_url: Url,
    pub client_id: String,
    pub client_secret: String,
    pub source_file_name: String,
}

const BASE_URL_KEY: &str = "BASE_URL";
// const TENANT_KEY: &str = "TENANT";
const API_URL_KEY: &str = "API_URL";
const CLIENT_ID_KEY: &str = "CLIENT_ID";
const CLIENT_SECRET_KEY: &str = "CLIENT_SECRET";
const SOURCE_FILE_NAME_KEY: &str = "SOURCE_FILE_NAME";

const TOKEN_URL_PATH: &str = "auth/token";

impl Config {
    pub fn load_from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().context("failed to load environment")?;

        let base_url = Url::try_from(std::env::var(BASE_URL_KEY)?.as_str())?;
        // let tenant = std::env::var(TENANT_KEY)?;
        let mut token_url = base_url.clone();
        token_url.set_path(TOKEN_URL_PATH);
        let api_url = Url::try_from(std::env::var(API_URL_KEY)?.as_str())?;
        let client_id = std::env::var(CLIENT_ID_KEY)?;
        let client_secret = std::env::var(CLIENT_SECRET_KEY)?;
        let source_file_name = std::env::var(SOURCE_FILE_NAME_KEY)?;

        Ok(Self {
            base_url,
            // tenant,
            token_url,
            api_url,
            client_id,
            client_secret,
            source_file_name,
        })
    }
}
