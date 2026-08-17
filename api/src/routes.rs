use std::sync::Arc;

use anchor_lang::prelude::Pubkey;
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::Serialize;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

use crate::config::{DEFAULT_HERMES_URL, DEFAULT_PYTH_FEED_ID};
use crate::error::ApiError;
use crate::oracle::{fetch_latest_feed, OracleFeedView};
use crate::rpc::{
    bytes32_to_string, decode_escrow, decode_policy, escrow_pda, policy_pda, status_label,
    AccountSource,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub rpc: Arc<dyn AccountSource>,
    pub program_id: Pubkey,
    pub public_base_url: String,
    pub hermes_url: String,
    pub pyth_feed_id: String,
    pub http: reqwest::Client,
}

impl AppState {
    pub fn for_test(pool: PgPool, rpc: Arc<dyn AccountSource>, program_id: Pubkey) -> Self {
        Self {
            pool,
            rpc,
            program_id,
            public_base_url: "http://127.0.0.1:3000".into(),
            hermes_url: DEFAULT_HERMES_URL.into(),
            pyth_feed_id: DEFAULT_PYTH_FEED_ID.into(),
            http: reqwest::Client::new(),
        }
    }
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

#[derive(Serialize)]
pub struct PolicyIndexRow {
    pub policy_id: String,
    pub holder: String,
    pub expiry: String,
    pub asset_class: String,
    pub policy_pda: String,
    pub escrow_pda: String,
}

#[derive(Serialize)]
pub struct SettlementIndexRow {
    pub id: String,
    pub policy_id: String,
    pub status: String,
    pub payout_amount: Option<i64>,
    pub tx_signature: Option<String>,
    pub proof_hash: Option<String>,
    pub verification_url: Option<String>,
    pub settled_at: Option<String>,
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub asset_class: String,
    pub risk_score: f64,
    pub scale: String,
    pub model_confidence: String,
    pub timestamp: String,
    pub zk_proof: ZkProofView,
    pub attested: bool,
    pub verified: bool,
    pub verification_method: &'static str,
    pub public_inputs: serde_json::Value,
}

#[derive(Serialize)]
pub struct ZkProofView {
    pub hash: String,
    pub verification_url: String,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/policies", get(list_policies))
        .route("/policies/{id}", get(get_policy))
        .route("/settlements", get(list_settlements))
        .route("/verify/{proof_hash}", get(get_verify))
        .route("/oracle/latest", get(get_oracle_latest))
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

/// Canonical form: `0x` + lowercase even-length hex (16..=128 chars).
pub fn parse_proof_hash(raw: &str) -> Result<String, ApiError> {
    let hex_str = raw
        .strip_prefix("0x")
        .or_else(|| raw.strip_prefix("0X"))
        .unwrap_or(raw);
    let hex_str = hex_str.to_ascii_lowercase();
    if hex_str.len() < 16 || hex_str.len() > 128 || hex_str.len() % 2 != 0 {
        return Err(ApiError::InvalidProofHash);
    }
    if !hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ApiError::InvalidProofHash);
    }
    Ok(format!("0x{hex_str}"))
}

fn rfc3339(ts: DateTime<Utc>) -> String {
    ts.to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn verification_url(base: &str, hash: &str) -> String {
    format!("{}/verify/{hash}", base.trim_end_matches('/'))
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

async fn list_policies(
    State(state): State<AppState>,
) -> Result<Json<Vec<PolicyIndexRow>>, ApiError> {
    let rows = sqlx::query_as::<_, PolicyIndexDb>(
        r#"
        SELECT encode(policy_id, 'hex') AS policy_id,
               holder,
               expiry,
               asset_class,
               policy_pda,
               escrow_pda
        FROM policies
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|row| PolicyIndexRow {
                policy_id: row.policy_id,
                holder: row.holder,
                expiry: rfc3339(row.expiry),
                asset_class: row.asset_class,
                policy_pda: row.policy_pda,
                escrow_pda: row.escrow_pda,
            })
            .collect(),
    ))
}

async fn list_settlements(
    State(state): State<AppState>,
) -> Result<Json<Vec<SettlementIndexRow>>, ApiError> {
    let rows = sqlx::query_as::<_, SettlementIndexDb>(
        r#"
        SELECT id::text AS id,
               encode(policy_id, 'hex') AS policy_id,
               status,
               payout_amount,
               tx_signature,
               proof_hash,
               settled_at
        FROM settlements
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|row| {
                let verification_url = row
                    .proof_hash
                    .as_ref()
                    .map(|hash| verification_url(&state.public_base_url, hash));
                SettlementIndexRow {
                    id: row.id,
                    policy_id: row.policy_id,
                    status: row.status,
                    payout_amount: row.payout_amount,
                    tx_signature: row.tx_signature,
                    proof_hash: row.proof_hash,
                    verification_url,
                    settled_at: row.settled_at.map(rfc3339),
                }
            })
            .collect(),
    ))
}

async fn get_verify(
    State(state): State<AppState>,
    Path(proof_hash): Path<String>,
) -> Result<Json<VerifyResponse>, ApiError> {
    let hash = parse_proof_hash(&proof_hash)?;
    let row = sqlx::query_as::<_, ProofDb>(
        r#"
        SELECT proof_hash, asset_class, risk_score, scale, model_confidence,
               proof_timestamp, public_inputs
        FROM proofs
        WHERE proof_hash = $1
        "#,
    )
    .bind(&hash)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::ProofNotFound)?;

    Ok(Json(VerifyResponse {
        asset_class: row.asset_class,
        risk_score: row.risk_score,
        scale: row.scale,
        model_confidence: row.model_confidence,
        timestamp: rfc3339(row.proof_timestamp),
        zk_proof: ZkProofView {
            hash: row.proof_hash.clone(),
            verification_url: verification_url(&state.public_base_url, &row.proof_hash),
        },
        attested: true,
        verified: false,
        verification_method: "stored_attestation",
        public_inputs: row.public_inputs,
    }))
}

async fn get_oracle_latest(
    State(state): State<AppState>,
) -> Result<Json<OracleFeedView>, ApiError> {
    let feed = fetch_latest_feed(&state.http, &state.hermes_url, &state.pyth_feed_id).await?;
    Ok(Json(feed))
}

#[derive(sqlx::FromRow)]
struct PolicyIndexDb {
    policy_id: String,
    holder: String,
    expiry: DateTime<Utc>,
    asset_class: String,
    policy_pda: String,
    escrow_pda: String,
}

#[derive(sqlx::FromRow)]
struct SettlementIndexDb {
    id: String,
    policy_id: String,
    status: String,
    payout_amount: Option<i64>,
    tx_signature: Option<String>,
    proof_hash: Option<String>,
    settled_at: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow)]
struct ProofDb {
    proof_hash: String,
    asset_class: String,
    risk_score: f64,
    scale: String,
    model_confidence: String,
    proof_timestamp: DateTime<Utc>,
    public_inputs: serde_json::Value,
}
