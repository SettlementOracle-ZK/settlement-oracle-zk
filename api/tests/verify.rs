use std::sync::Arc;

use http_body_util::BodyExt;
use settlement_api::rpc::AccountSource;
use settlement_api::{parse_proof_hash, router, AppState};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

struct MockRpc;

impl AccountSource for MockRpc {
    fn get_account_data<'a>(
        &'a self,
        _address: &'a anchor_lang::prelude::Pubkey,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Option<Vec<u8>>, settlement_api::ApiError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move { Ok(None) })
    }
}

async fn try_pool() -> Option<sqlx::PgPool> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .ok()?;
    sqlx::migrate!("./migrations").run(&pool).await.ok()?;
    Some(pool)
}

fn app(pool: sqlx::PgPool) -> axum::Router {
    router(AppState::for_test(
        pool,
        Arc::new(MockRpc) as Arc<dyn AccountSource>,
        escrow::ID,
    ))
}

#[test]
fn parse_proof_hash_canonicalizes() {
    let hash = "abc123def4567890";
    assert_eq!(parse_proof_hash(hash).unwrap(), format!("0x{hash}"));
    assert_eq!(
        parse_proof_hash(&format!("0x{}", hash.to_uppercase())).unwrap(),
        format!("0x{hash}")
    );
}

#[test]
fn parse_proof_hash_rejects_invalid() {
    assert!(parse_proof_hash("abcd").is_err());
    assert!(parse_proof_hash("zz").is_err());
    assert!(parse_proof_hash("abc123def456789").is_err());
}

#[tokio::test]
async fn verify_returns_prd_payload() {
    let Some(pool) = try_pool().await else {
        eprintln!("skipping: DATABASE_URL not set or postgres unavailable");
        return;
    };

    let hash = "0xabc123def4567890abc123def4567890abc123def4567890abc123def4567890";
    sqlx::query("DELETE FROM proofs WHERE proof_hash = $1")
        .bind(hash)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        r#"
        INSERT INTO proofs (
            proof_hash, asset_class, risk_score, scale, model_confidence,
            proof_timestamp, public_inputs
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(hash)
    .bind("agriculture_climate")
    .bind(85.4)
    .bind("0-100")
    .bind("92%")
    .bind(
        chrono::DateTime::parse_from_rfc3339("2026-05-19T14:42:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc),
    )
    .bind(serde_json::json!({"triggered": true}))
    .execute(&pool)
    .await
    .unwrap();

    let response = app(pool)
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/verify/{hash}"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["asset_class"], "agriculture_climate");
    assert_eq!(json["risk_score"], 85.4);
    assert_eq!(json["scale"], "0-100");
    assert_eq!(json["model_confidence"], "92%");
    assert_eq!(json["timestamp"], "2026-05-19T14:42:00Z");
    assert_eq!(json["zk_proof"]["hash"], hash);
    assert_eq!(
        json["zk_proof"]["verification_url"],
        format!("http://127.0.0.1:3000/verify/{hash}")
    );
    assert_eq!(json["verified"], false);
    assert_eq!(json["attested"], true);
    assert_eq!(json["verification_method"], "stored_attestation");
}

#[tokio::test]
async fn verify_404_when_missing() {
    let Some(pool) = try_pool().await else {
        eprintln!("skipping: DATABASE_URL not set or postgres unavailable");
        return;
    };

    let response = app(pool)
        .oneshot(
            axum::http::Request::builder()
                .uri("/verify/0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn verify_400_when_hash_invalid() {
    let Some(pool) = try_pool().await else {
        eprintln!("skipping: DATABASE_URL not set or postgres unavailable");
        return;
    };

    let response = app(pool)
        .oneshot(
            axum::http::Request::builder()
                .uri("/verify/not-a-hash")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn list_settlements_includes_verify_link() {
    let Some(pool) = try_pool().await else {
        eprintln!("skipping: DATABASE_URL not set or postgres unavailable");
        return;
    };

    let policy_id = [0xBBu8; 32];
    sqlx::query("DELETE FROM settlements WHERE policy_id = $1")
        .bind(policy_id.as_slice())
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM policies WHERE policy_id = $1")
        .bind(policy_id.as_slice())
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        r#"
        INSERT INTO policies (policy_id, holder, expiry, asset_class, policy_pda, escrow_pda)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(policy_id.as_slice())
    .bind("Holder111")
    .bind(
        chrono::DateTime::parse_from_rfc3339("2099-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc),
    )
    .bind("agriculture_climate")
    .bind("PolicyPda111")
    .bind("EscrowPda111")
    .execute(&pool)
    .await
    .unwrap();

    let hash = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    sqlx::query(
        r#"
        INSERT INTO settlements (policy_id, status, payout_amount, tx_signature, proof_hash)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(policy_id.as_slice())
    .bind("PAID")
    .bind(500_000_000_i64)
    .bind("5TestSignature")
    .bind(hash)
    .execute(&pool)
    .await
    .unwrap();

    let response = app(pool)
        .oneshot(
            axum::http::Request::builder()
                .uri("/settlements")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let found = json
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["proof_hash"] == hash)
        .expect("seeded settlement");
    assert_eq!(found["tx_signature"], "5TestSignature");
    assert_eq!(
        found["verification_url"],
        format!("http://127.0.0.1:3000/verify/{hash}")
    );
}
