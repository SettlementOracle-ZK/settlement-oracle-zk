use std::str::FromStr;
use std::sync::Arc;

use anyhow::Context;
use axum::http::{HeaderValue, Method};
use settlement_api::{router, AccountSource, AppState, Config, SolanaRpc};
use tower_http::cors::{AllowOrigin, CorsLayer};
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

    let origins: Vec<HeaderValue> = config
        .cors_origins
        .iter()
        .map(|o| o.parse())
        .collect::<Result<_, _>>()
        .context("parse CORS_ORIGINS")?;

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(tower_http::cors::Any);

    let state = AppState {
        pool,
        rpc: Arc::new(SolanaRpc::new(config.solana_rpc_url.clone())) as Arc<dyn AccountSource>,
        program_id,
        public_base_url: config.public_base_url.clone(),
        hermes_url: config.hermes_url.clone(),
        pyth_feed_id: config.pyth_feed_id.clone(),
        http: reqwest::Client::new(),
        app_env: config.app_env.clone(),
    };

    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .with_context(|| format!("bind {}", config.bind_addr))?;
    tracing::info!("listening on {}", config.bind_addr);
    axum::serve(listener, router(state).layer(cors)).await?;
    Ok(())
}
