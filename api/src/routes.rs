use std::str::FromStr;
use std::sync::Arc;

use anchor_lang::prelude::Pubkey;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

use crate::config::{DEFAULT_HERMES_URL, DEFAULT_PYTH_FEED_ID};
use crate::error::ApiError;
use crate::oracle::{fetch_delay_feed, fetch_latest_feed, OracleFeedView};
use crate::proof::{hash_witness, verify_stored_hash, witness_from_public_inputs};
use crate::rpc::{
    bytes32_to_string, decode_escrow, decode_policy, escrow_pda, mock_pyth_pda, policy_pda,
    status_label, AccountSource,
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
    pub app_env: String,
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
            app_env: "development".into(),
        }
    }

    fn is_development(&self) -> bool {
        self.app_env.eq_ignore_ascii_case("development")
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
pub struct SettlementDetailResponse {
    pub id: String,
    pub policy_id: String,
    pub status: String,
    pub payout_amount: Option<i64>,
    pub tx_signature: Option<String>,
    pub settled_at: Option<String>,
    pub asset_class: String,
    pub risk_score: f64,
    pub scale: String,
    pub model_confidence: String,
    pub timestamp: String,
    pub zk_proof: ZkProofView,
    pub verified: bool,
    pub attested: bool,
    pub verification_method: &'static str,
    pub public_inputs: serde_json::Value,
}

#[derive(Deserialize)]
pub struct RegisterProofRequest {
    pub proof_hash: String,
    pub asset_class: String,
    pub risk_score: f64,
    pub scale: Option<String>,
    pub model_confidence: String,
    pub timestamp: String,
    pub public_inputs: serde_json::Value,
}

#[derive(Deserialize)]
pub struct RegisterSettlementRequest {
    pub policy_id: String,
    pub status: String,
    pub proof_hash: Option<String>,
    pub payout_amount: Option<i64>,
    pub tx_signature: Option<String>,
    pub holder: Option<String>,
    pub asset_class: Option<String>,
    pub policy_pda: Option<String>,
    pub escrow_pda: Option<String>,
}

#[derive(Deserialize)]
pub struct RegisterPolicyRequest {
    pub policy_id: String,
    pub holder: String,
    pub expiry: String,
    pub asset_class: String,
    pub policy_pda: String,
    pub escrow_pda: String,
    /// Optional devnet tx signatures for audit trail (not stored in MVP schema).
    pub init_policy_tx: Option<String>,
}

#[derive(Serialize)]
pub struct RegisterProofResponse {
    pub proof_hash: String,
    pub verification_url: String,
    pub verified: bool,
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
        .route("/policies/register", post(register_policy))
        .route("/policies/{id}", get(get_policy))
        .route("/settlements", get(list_settlements))
        .route("/settlements/{id}", get(get_settlement))
        .route("/proofs", post(register_proof))
        .route("/settlements/register", post(register_settlement))
        .route("/verify/{proof_hash}", get(get_verify))
        .route("/oracle/latest", get(get_oracle_latest))
        .route("/oracle/delay", get(get_oracle_delay))
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

fn proof_verification(row: &ProofDb) -> (bool, &'static str) {
    if let Ok(witness) = witness_from_public_inputs(&row.public_inputs) {
        let verified = verify_stored_hash(&witness, &row.proof_hash);
        if verified {
            return (true, "circuit_commitment");
        }
        let computed = hash_witness(&witness);
        if computed == row.proof_hash.to_ascii_lowercase() {
            return (true, "circuit_commitment");
        }
    }
    (false, "stored_attestation")
}

fn verify_response_from_proof(row: ProofDb, base_url: &str) -> VerifyResponse {
    let (verified, method) = proof_verification(&row);
    VerifyResponse {
        asset_class: row.asset_class,
        risk_score: row.risk_score,
        scale: row.scale,
        model_confidence: row.model_confidence,
        timestamp: rfc3339(row.proof_timestamp),
        zk_proof: ZkProofView {
            hash: row.proof_hash.clone(),
            verification_url: verification_url(base_url, &row.proof_hash),
        },
        attested: true,
        verified,
        verification_method: method,
        public_inputs: row.public_inputs,
    }
}

fn parse_pubkey(raw: &str) -> Result<Pubkey, ApiError> {
    Pubkey::from_str(raw.trim()).map_err(|_| ApiError::InvalidPolicyId)
}

async fn indexed_policy_pdas(
    pool: &PgPool,
    policy_id: &[u8; 32],
) -> Result<Option<(Pubkey, Pubkey)>, ApiError> {
    let row = sqlx::query_as::<_, PolicyIndexDb>(
        r#"
        SELECT encode(policy_id, 'hex') AS policy_id,
               holder,
               expiry,
               asset_class,
               policy_pda,
               escrow_pda
        FROM policies
        WHERE policy_id = $1
        "#,
    )
    .bind(policy_id.as_slice())
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some((
        parse_pubkey(&row.policy_pda)?,
        parse_pubkey(&row.escrow_pda)?,
    )))
}

async fn resolve_policy_pdas(
    state: &AppState,
    policy_id: &[u8; 32],
) -> Result<(Pubkey, Pubkey), ApiError> {
    let derived_policy = policy_pda(&state.program_id, policy_id);
    let derived_escrow = escrow_pda(&state.program_id, policy_id);

    let derived_policy_data = state.rpc.get_account_data(&derived_policy).await?;
    let derived_escrow_data = state.rpc.get_account_data(&derived_escrow).await?;

    if derived_policy_data.is_some() && derived_escrow_data.is_some() {
        return Ok((derived_policy, derived_escrow));
    }

    if let Some((indexed_policy, indexed_escrow)) =
        indexed_policy_pdas(&state.pool, policy_id).await?
    {
        if indexed_policy != derived_policy || indexed_escrow != derived_escrow {
            let indexed_policy_data = state.rpc.get_account_data(&indexed_policy).await?;
            let indexed_escrow_data = state.rpc.get_account_data(&indexed_escrow).await?;
            if indexed_policy_data.is_some() && indexed_escrow_data.is_some() {
                return Ok((indexed_policy, indexed_escrow));
            }
        }
    }

    Err(ApiError::PolicyNotFound)
}

async fn get_policy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PolicyResponse>, ApiError> {
    let policy_id = parse_policy_id(&id)?;
    let (policy_addr, escrow_addr) = resolve_policy_pdas(&state, &policy_id).await?;

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

async fn get_settlement(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SettlementDetailResponse>, ApiError> {
    let row = sqlx::query_as::<_, SettlementDetailDb>(
        r#"
        SELECT s.id::text AS id,
               encode(s.policy_id, 'hex') AS policy_id,
               s.status,
               s.payout_amount,
               s.tx_signature,
               s.proof_hash,
               s.settled_at,
               p.asset_class,
               p.risk_score,
               p.scale,
               p.model_confidence,
               p.proof_timestamp,
               p.public_inputs
        FROM settlements s
        LEFT JOIN proofs p ON p.proof_hash = s.proof_hash
        WHERE s.id::text = $1
        "#,
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::SettlementNotFound)?;

    let proof_hash = row
        .proof_hash
        .clone()
        .unwrap_or_else(|| "0x0".into());
    let (verified, method) = if let Some(ref inputs) = row.public_inputs {
        if let Ok(witness) = witness_from_public_inputs(inputs) {
            let v = verify_stored_hash(&witness, &proof_hash);
            (v, if v { "circuit_commitment" } else { "stored_attestation" })
        } else {
            (false, "stored_attestation")
        }
    } else {
        (false, "stored_attestation")
    };

    Ok(Json(SettlementDetailResponse {
        id: row.id,
        policy_id: row.policy_id,
        status: row.status,
        payout_amount: row.payout_amount,
        tx_signature: row.tx_signature,
        settled_at: row.settled_at.map(rfc3339),
        asset_class: row.asset_class.unwrap_or_else(|| "unknown".into()),
        risk_score: row.risk_score.unwrap_or(0.0),
        scale: row.scale.unwrap_or_else(|| "0-100".into()),
        model_confidence: row.model_confidence.unwrap_or_else(|| "0%".into()),
        timestamp: row
            .proof_timestamp
            .map(rfc3339)
            .unwrap_or_else(|| Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
        zk_proof: ZkProofView {
            hash: proof_hash.clone(),
            verification_url: verification_url(&state.public_base_url, &proof_hash),
        },
        verified,
        attested: row.public_inputs.is_some(),
        verification_method: method,
        public_inputs: row.public_inputs.unwrap_or(serde_json::json!({})),
    }))
}

async fn register_proof(
    State(state): State<AppState>,
    Json(body): Json<RegisterProofRequest>,
) -> Result<Json<RegisterProofResponse>, ApiError> {
    if !state.is_development() {
        return Err(ApiError::DevOnly);
    }

    let hash = parse_proof_hash(&body.proof_hash)?;
    let witness = witness_from_public_inputs(&body.public_inputs)?;
    let computed = hash_witness(&witness);
    if computed != hash {
        return Err(ApiError::ProofInvalid);
    }

    let ts = chrono::DateTime::parse_from_rfc3339(&body.timestamp)
        .map_err(|_| ApiError::ProofInvalid)?
        .with_timezone(&Utc);

    sqlx::query(
        r#"
        INSERT INTO proofs (
            proof_hash, asset_class, risk_score, scale, model_confidence,
            proof_timestamp, public_inputs
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (proof_hash) DO UPDATE SET
            asset_class = EXCLUDED.asset_class,
            risk_score = EXCLUDED.risk_score,
            scale = EXCLUDED.scale,
            model_confidence = EXCLUDED.model_confidence,
            proof_timestamp = EXCLUDED.proof_timestamp,
            public_inputs = EXCLUDED.public_inputs
        "#,
    )
    .bind(&hash)
    .bind(&body.asset_class)
    .bind(body.risk_score)
    .bind(body.scale.unwrap_or_else(|| "0-100".into()))
    .bind(&body.model_confidence)
    .bind(ts)
    .bind(&body.public_inputs)
    .execute(&state.pool)
    .await?;

    Ok(Json(RegisterProofResponse {
        proof_hash: hash.clone(),
        verification_url: verification_url(&state.public_base_url, &hash),
        verified: true,
    }))
}

async fn register_settlement(
    State(state): State<AppState>,
    Json(body): Json<RegisterSettlementRequest>,
) -> Result<Json<SettlementIndexRow>, ApiError> {
    if !state.is_development() {
        return Err(ApiError::DevOnly);
    }

    let policy_id = parse_policy_id(&body.policy_id)?;
    if let Some(ref hash) = body.proof_hash {
        parse_proof_hash(hash)?;
    }

    if let (Some(holder), Some(asset_class), Some(policy_pda), Some(escrow_pda)) = (
        &body.holder,
        &body.asset_class,
        &body.policy_pda,
        &body.escrow_pda,
    ) {
        sqlx::query(
            r#"
            INSERT INTO policies (policy_id, holder, expiry, asset_class, policy_pda, escrow_pda)
            VALUES ($1, $2, '2099-12-31T00:00:00Z', $3, $4, $5)
            ON CONFLICT (policy_id) DO UPDATE SET
                holder = EXCLUDED.holder,
                asset_class = EXCLUDED.asset_class,
                policy_pda = EXCLUDED.policy_pda,
                escrow_pda = EXCLUDED.escrow_pda
            "#,
        )
        .bind(policy_id.as_slice())
        .bind(holder)
        .bind(asset_class)
        .bind(policy_pda)
        .bind(escrow_pda)
        .execute(&state.pool)
        .await?;
    }

    let row = sqlx::query_as::<_, SettlementIndexDb>(
        r#"
        UPDATE settlements s
        SET status = $2,
            payout_amount = COALESCE($3, s.payout_amount),
            tx_signature = COALESCE($4, s.tx_signature),
            proof_hash = COALESCE($5, s.proof_hash),
            settled_at = CASE WHEN $2 = 'PAID' THEN now() ELSE s.settled_at END
        FROM (
            SELECT id FROM settlements
            WHERE policy_id = $1
            ORDER BY created_at DESC
            LIMIT 1
        ) latest
        WHERE s.id = latest.id
        RETURNING s.id::text AS id,
                  encode(s.policy_id, 'hex') AS policy_id,
                  s.status,
                  s.payout_amount,
                  s.tx_signature,
                  s.proof_hash,
                  s.settled_at
        "#,
    )
    .bind(policy_id.as_slice())
    .bind(&body.status)
    .bind(body.payout_amount)
    .bind(&body.tx_signature)
    .bind(&body.proof_hash)
    .fetch_optional(&state.pool)
    .await?;

    let row = if let Some(row) = row {
        row
    } else {
        sqlx::query_as::<_, SettlementIndexDb>(
            r#"
            INSERT INTO settlements (policy_id, status, payout_amount, tx_signature, proof_hash, settled_at)
            VALUES ($1, $2, $3, $4, $5, CASE WHEN $2 = 'PAID' THEN now() ELSE NULL END)
            RETURNING id::text AS id,
                      encode(policy_id, 'hex') AS policy_id,
                      status,
                      payout_amount,
                      tx_signature,
                      proof_hash,
                      settled_at
            "#,
        )
        .bind(policy_id.as_slice())
        .bind(&body.status)
        .bind(body.payout_amount)
        .bind(&body.tx_signature)
        .bind(&body.proof_hash)
        .fetch_one(&state.pool)
        .await?
    };

    let verification_url = row
        .proof_hash
        .as_ref()
        .map(|hash| verification_url(&state.public_base_url, hash));

    Ok(Json(SettlementIndexRow {
        id: row.id,
        policy_id: row.policy_id,
        status: row.status,
        payout_amount: row.payout_amount,
        tx_signature: row.tx_signature,
        proof_hash: row.proof_hash,
        verification_url,
        settled_at: row.settled_at.map(rfc3339),
    }))
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

    Ok(Json(verify_response_from_proof(row, &state.public_base_url)))
}

async fn register_policy(
    State(state): State<AppState>,
    Json(body): Json<RegisterPolicyRequest>,
) -> Result<Json<PolicyIndexRow>, ApiError> {
    if !state.is_development() {
        return Err(ApiError::DevOnly);
    }

    let policy_id = parse_policy_id(&body.policy_id)?;
    let expiry = chrono::DateTime::parse_from_rfc3339(&body.expiry)
        .map_err(|_| ApiError::InvalidPolicyId)?
        .with_timezone(&Utc);

    sqlx::query(
        r#"
        INSERT INTO policies (policy_id, holder, expiry, asset_class, policy_pda, escrow_pda)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (policy_id) DO UPDATE SET
            holder = EXCLUDED.holder,
            expiry = EXCLUDED.expiry,
            asset_class = EXCLUDED.asset_class,
            policy_pda = EXCLUDED.policy_pda,
            escrow_pda = EXCLUDED.escrow_pda
        "#,
    )
    .bind(policy_id.as_slice())
    .bind(&body.holder)
    .bind(expiry)
    .bind(&body.asset_class)
    .bind(&body.policy_pda)
    .bind(&body.escrow_pda)
    .execute(&state.pool)
    .await?;

    Ok(Json(PolicyIndexRow {
        policy_id: body.policy_id.strip_prefix("0x").unwrap_or(&body.policy_id).to_string(),
        holder: body.holder,
        expiry: rfc3339(expiry),
        asset_class: body.asset_class,
        policy_pda: body.policy_pda,
        escrow_pda: body.escrow_pda,
    }))
}

async fn get_oracle_latest(
    State(state): State<AppState>,
) -> Result<Json<OracleFeedView>, ApiError> {
    let feed = fetch_latest_feed(&state.http, &state.hermes_url, &state.pyth_feed_id).await?;
    Ok(Json(feed))
}

async fn get_oracle_delay(
    State(state): State<AppState>,
) -> Result<Json<OracleFeedView>, ApiError> {
    let feed_pubkey = mock_pyth_pda(&state.program_id);
    let feed = fetch_delay_feed(state.rpc.as_ref(), &feed_pubkey).await?;
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
struct SettlementDetailDb {
    id: String,
    policy_id: String,
    status: String,
    payout_amount: Option<i64>,
    tx_signature: Option<String>,
    proof_hash: Option<String>,
    settled_at: Option<DateTime<Utc>>,
    asset_class: Option<String>,
    risk_score: Option<f64>,
    scale: Option<String>,
    model_confidence: Option<String>,
    proof_timestamp: Option<DateTime<Utc>>,
    public_inputs: Option<serde_json::Value>,
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
