use crate::error::ApiError;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub solana_rpc_url: String,
    pub escrow_program_id: String,
    pub bind_addr: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ApiError> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| ApiError::Config("DATABASE_URL is required".into()))?,
            solana_rpc_url: std::env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.devnet.solana.com".into()),
            escrow_program_id: std::env::var("ESCROW_PROGRAM_ID")
                .unwrap_or_else(|_| escrow::ID.to_string()),
            bind_addr: std::env::var("API_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".into()),
        })
    }
}
