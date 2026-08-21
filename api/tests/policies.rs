use std::collections::HashMap;
use std::sync::Arc;

use anchor_lang::prelude::Pubkey;
use http_body_util::BodyExt;
use settlement_api::rpc::{escrow_pda, policy_pda};
use settlement_api::{parse_policy_id, router, AccountSource, AppState};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

struct MockRpc {
    accounts: HashMap<Pubkey, Vec<u8>>,
}

impl AccountSource for MockRpc {
    fn get_account_data<'a>(
        &'a self,
        address: &'a Pubkey,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Option<Vec<u8>>, settlement_api::ApiError>>
                + Send
                + 'a,
        >,
    > {
        let data = self.accounts.get(address).cloned();
        Box::pin(async move { Ok(data) })
    }
}

#[test]
fn parse_policy_id_accepts_hex() {
    let id = "01".repeat(32);
    let bytes = parse_policy_id(&id).unwrap();
    assert_eq!(bytes, [1u8; 32]);
    assert_eq!(parse_policy_id(&format!("0x{id}")).unwrap(), [1u8; 32]);
}

#[test]
fn parse_policy_id_rejects_short_hex() {
    assert!(parse_policy_id("abcd").is_err());
}

#[test]
fn pdas_are_deterministic() {
    let program_id = escrow::ID;
    let policy_id = [1u8; 32];
    let a = policy_pda(&program_id, &policy_id);
    let b = policy_pda(&program_id, &policy_id);
    assert_eq!(a, b);
    assert_ne!(a, escrow_pda(&program_id, &policy_id));
}

fn fixture_policy_account(holder: Pubkey) -> Vec<u8> {
    use anchor_lang::AccountSerialize;
    use escrow::state::PolicyAccount;
    let account = PolicyAccount {
        policy_id: [1u8; 32],
        holder,
        expiry: 4_102_444_800,
        asset_class: *b"agriculture_climate\0\0\0\0\0\0\0\0\0\0\0\0\0",
        created_at: 1_700_000_000,
        bump: 255,
    };
    let mut data = Vec::new();
    account.try_serialize(&mut data).unwrap();
    data
}

fn fixture_escrow_account(authority: Pubkey) -> Vec<u8> {
    use anchor_lang::AccountSerialize;
    use escrow::state::{EscrowAccount, EscrowStatus};
    let account = EscrowAccount {
        policy_id: [1u8; 32],
        authority,
        amount: 500_000_000,
        trigger_threshold: 120,
        status: EscrowStatus::Active,
        paused: false,
        bump: 255,
    };
    let mut data = Vec::new();
    account.try_serialize(&mut data).unwrap();
    data
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

#[tokio::test]
async fn get_policies_reads_mocked_on_chain_accounts() {
    let Some(pool) = try_pool().await else {
        eprintln!("skipping: DATABASE_URL not set or postgres unavailable");
        return;
    };

    let program_id = escrow::ID;
    let policy_id = [1u8; 32];
    let holder = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let policy_addr = policy_pda(&program_id, &policy_id);
    let escrow_addr = escrow_pda(&program_id, &policy_id);

    let mut accounts = HashMap::new();
    accounts.insert(policy_addr, fixture_policy_account(holder));
    accounts.insert(escrow_addr, fixture_escrow_account(authority));

    let app = router(AppState::for_test(
        pool,
        Arc::new(MockRpc { accounts }) as Arc<dyn AccountSource>,
        program_id,
    ));

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/policies/{}", hex::encode(policy_id)))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["escrow"]["status"], "Active");
    assert_eq!(json["escrow"]["amount"], 500_000_000);
    assert_eq!(json["escrow"]["paused"], false);
    assert_eq!(json["asset_class"], "agriculture_climate");
}

#[tokio::test]
async fn get_policies_404_when_missing_on_chain() {
    let Some(pool) = try_pool().await else {
        eprintln!("skipping: DATABASE_URL not set or postgres unavailable");
        return;
    };

    let app = router(AppState::for_test(
        pool,
        Arc::new(MockRpc {
            accounts: HashMap::new(),
        }) as Arc<dyn AccountSource>,
        escrow::ID,
    ));

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/policies/{}", hex::encode([1u8; 32])))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn health_ok_when_db_up() {
    let Some(pool) = try_pool().await else {
        eprintln!("skipping: DATABASE_URL not set or postgres unavailable");
        return;
    };

    let app = router(AppState::for_test(
        pool,
        Arc::new(MockRpc {
            accounts: HashMap::new(),
        }) as Arc<dyn AccountSource>,
        escrow::ID,
    ));

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
}
