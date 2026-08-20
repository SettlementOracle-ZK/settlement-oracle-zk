use crate::error::ApiError;

pub const DEFAULT_PYTH_FEED_ID: &str =
    "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
pub const DEFAULT_HERMES_URL: &str = "https://hermes.pyth.network";

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub solana_rpc_url: String,
    pub escrow_program_id: String,
    pub bind_addr: String,
    pub public_base_url: String,
    pub hermes_url: String,
    pub pyth_feed_id: String,
    pub cors_origins: Vec<String>,
    pub app_env: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ApiError> {
        let bind_addr = std::env::var("API_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".into());
        let public_base_url =
            std::env::var("API_PUBLIC_BASE_URL").unwrap_or_else(|_| format!("http://{bind_addr}"));
        let cors_origins = std::env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3001,http://127.0.0.1:3001".into())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| ApiError::Config("DATABASE_URL is required".into()))?,
            solana_rpc_url: std::env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.devnet.solana.com".into()),
            escrow_program_id: std::env::var("ESCROW_PROGRAM_ID")
                .unwrap_or_else(|_| escrow::ID.to_string()),
            bind_addr,
            public_base_url,
            hermes_url: std::env::var("HERMES_URL").unwrap_or_else(|_| DEFAULT_HERMES_URL.into()),
            pyth_feed_id: std::env::var("PYTH_FEED_ID")
                .unwrap_or_else(|_| DEFAULT_PYTH_FEED_ID.into()),
            cors_origins,
            app_env: std::env::var("APP_ENV").unwrap_or_else(|_| "development".into()),
        })
    }
}
