use std::sync::Arc;

use anchor_lang::prelude::Pubkey;
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

use crate::error::ApiError;
use crate::rpc::{
    bytes32_to_string, decode_escrow, decode_policy, escrow_pda, policy_pda, status_label,
    AccountSource,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub rpc: Arc<dyn AccountSource>,
    pub program_id: Pubkey,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub db: &'static str,
}

#[derive(Serialize)]
pub struct PolicyResponse {
    pub policy_id: String,
    pub holder: String,
    pub expiry: i64,
    pub asset_class: String,
    pub escrow: EscrowView,
    pub pdas: PdasView,
}

#[derive(Serialize)]
pub struct EscrowView {
    pub status: String,
    pub amount: u64,
    pub trigger_threshold: i64,
    pub paused: bool,
    pub authority: String,
}

#[derive(Serialize)]
pub struct PdasView {
    pub policy: String,
    pub escrow: String,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/policies/{id}", get(get_policy))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, ApiError> {
    crate::db::health_check(&state.pool).await?;
    Ok(Json(HealthResponse {
        status: "ok",
        db: "ok",
    }))
}

pub fn parse_policy_id(id: &str) -> Result<[u8; 32], ApiError> {
    let hex_str = id.strip_prefix("0x").unwrap_or(id);
    let bytes = hex::decode(hex_str).map_err(|_| ApiError::InvalidPolicyId)?;
    bytes.try_into().map_err(|_| ApiError::InvalidPolicyId)
}

async fn get_policy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PolicyResponse>, ApiError> {
    let policy_id = parse_policy_id(&id)?;
    let policy_addr = policy_pda(&state.program_id, &policy_id);
    let escrow_addr = escrow_pda(&state.program_id, &policy_id);

    let policy_data = state
        .rpc
        .get_account_data(&policy_addr)
        .await?
        .ok_or(ApiError::PolicyNotFound)?;
    let policy = decode_policy(&policy_data)?;

    let escrow_data = state
        .rpc
        .get_account_data(&escrow_addr)
        .await?
        .ok_or(ApiError::PolicyNotFound)?;
    let escrow = decode_escrow(&escrow_data)?;

    Ok(Json(PolicyResponse {
        policy_id: hex::encode(policy.policy_id),
        holder: policy.holder.to_string(),
        expiry: policy.expiry,
        asset_class: bytes32_to_string(&policy.asset_class),
        escrow: EscrowView {
            status: status_label(escrow.status).to_string(),
            amount: escrow.amount,
            trigger_threshold: escrow.trigger_threshold,
            paused: escrow.paused,
            authority: escrow.authority.to_string(),
        },
        pdas: PdasView {
            policy: policy_addr.to_string(),
            escrow: escrow_addr.to_string(),
        },
    }))
}
