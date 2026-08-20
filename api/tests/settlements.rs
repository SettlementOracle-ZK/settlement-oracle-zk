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

#[tokio::test]
async fn settlement_detail_returns_prd_payload() {
    let Some(pool) = try_pool().await else {
        eprintln!("skipping: DATABASE_URL not set or postgres unavailable");
        return;
    };

    let settlement_id = "22222222-2222-4222-8222-222222222222";
    let policy_id = [0xCCu8; 32];
    let witness_json = serde_json::json!({
        "asset_class": "agriculture_climate",
        "feed_id": "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
        "operator": "lt",
        "oracle_conf": 1.0,
        "oracle_price": 72.5,
        "publish_time": 1700000000,
        "risk_score": 72,
        "threshold": 100.0,
        "triggered": true
    });
    let witness = settlement_api::witness_from_public_inputs(&witness_json).unwrap();
    let hash = settlement_api::hash_witness(&witness);

    sqlx::query("DELETE FROM settlements WHERE id = $1::uuid")
        .bind(settlement_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM proofs WHERE proof_hash = $1")
        .bind(&hash)
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
    .bind("Holder222")
    .bind(
        chrono::DateTime::parse_from_rfc3339("2099-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc),
    )
    .bind("agriculture_climate")
    .bind("PolicyPda222")
    .bind("EscrowPda222")
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
    .bind(&hash)
    .bind("agriculture_climate")
    .bind(72.5)
    .bind("0-100")
    .bind("90%")
    .bind(
        chrono::DateTime::parse_from_rfc3339("2026-06-01T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc),
    )
    .bind(witness_json)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO settlements (id, policy_id, status, payout_amount, tx_signature, proof_hash, settled_at)
        VALUES ($1::uuid, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(settlement_id)
    .bind(policy_id.as_slice())
    .bind("PAID")
    .bind(500_000_000_i64)
    .bind("Sig222")
    .bind(&hash)
    .bind(
        chrono::DateTime::parse_from_rfc3339("2026-06-01T12:05:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc),
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = app(pool)
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/settlements/{settlement_id}"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["status"], "PAID");
    assert_eq!(json["asset_class"], "agriculture_climate");
    assert_eq!(json["risk_score"], 72.5);
    assert_eq!(json["zk_proof"]["hash"], hash);
    assert_eq!(json["verification_method"], "circuit_commitment");
    assert_eq!(json["verified"], true);

    sqlx::query("DELETE FROM settlements WHERE id = $1::uuid")
        .bind(settlement_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM proofs WHERE proof_hash = $1")
        .bind(&hash)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM policies WHERE policy_id = $1")
        .bind(policy_id.as_slice())
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn register_proof_dev_endpoint() {
    let Some(pool) = try_pool().await else {
        eprintln!("skipping: DATABASE_URL not set or postgres unavailable");
        return;
    };

    let witness = serde_json::json!({
        "asset_class": "agriculture_climate",
        "feed_id": "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
        "operator": "lt",
        "oracle_conf": 0.5,
        "oracle_price": 87.2,
        "publish_time": 1700000000,
        "risk_score": 87,
        "threshold": 100.0,
        "triggered": true
    });

    let canonical = settlement_api::proof::witness_from_public_inputs(&witness).unwrap();
    let hash = settlement_api::proof::hash_witness(&canonical);

    sqlx::query("DELETE FROM proofs WHERE proof_hash = $1")
        .bind(&hash)
        .execute(&pool)
        .await
        .unwrap();

    let body = serde_json::json!({
        "proof_hash": hash,
        "asset_class": "agriculture_climate",
        "risk_score": 87.0,
        "scale": "0-100",
        "model_confidence": "95%",
        "timestamp": "2026-05-19T14:42:00Z",
        "public_inputs": witness
    });

    let response = app(pool.clone())
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/proofs")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let parsed = parse_proof_hash(&hash).unwrap();
    let verify = app(pool.clone())
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/verify/{parsed}"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(verify.status(), axum::http::StatusCode::OK);
    let bytes = verify.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["verified"], true);
    assert_eq!(json["verification_method"], "circuit_commitment");

    sqlx::query("DELETE FROM proofs WHERE proof_hash = $1")
        .bind(&hash)
        .execute(&pool)
        .await
        .unwrap();
}
