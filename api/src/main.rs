use std::str::FromStr;
use std::sync::Arc;

use anyhow::Context;
use settlement_api::{router, AccountSource, AppState, Config, SolanaRpc};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let config = Config::from_env().map_err(|e| anyhow::anyhow!(e))?;
    let pool = settlement_api::db::connect(&config.database_url)
        .await
        .context("connect postgres")?;
    let program_id = anchor_lang::prelude::Pubkey::from_str(&config.escrow_program_id)
        .map_err(|e| anyhow::anyhow!("invalid ESCROW_PROGRAM_ID: {e}"))?;

    let state = AppState {
        pool,
        rpc: Arc::new(SolanaRpc::new(config.solana_rpc_url.clone())) as Arc<dyn AccountSource>,
        program_id,
    };

    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .with_context(|| format!("bind {}", config.bind_addr))?;
    tracing::info!("listening on {}", config.bind_addr);
    axum::serve(listener, router(state)).await?;
    Ok(())
}
