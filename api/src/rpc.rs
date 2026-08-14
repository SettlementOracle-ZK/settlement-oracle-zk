use std::future::Future;
use std::pin::Pin;

use anchor_lang::prelude::Pubkey;
use anchor_lang::AccountDeserialize;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use escrow::state::{EscrowAccount, EscrowStatus, PolicyAccount};
use serde::Deserialize;

use crate::error::ApiError;

pub trait AccountSource: Send + Sync {
    fn get_account_data<'a>(
        &'a self,
        address: &'a Pubkey,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Vec<u8>>, ApiError>> + Send + 'a>>;
}

#[derive(Clone)]
pub struct SolanaRpc {
    client: reqwest::Client,
    url: String,
}

impl SolanaRpc {
    pub fn new(url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            url,
        }
    }
}

#[derive(Deserialize)]
struct RpcResponse {
    result: Option<RpcResult>,
    error: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct RpcResult {
    value: Option<RpcAccount>,
}

#[derive(Deserialize)]
struct RpcAccount {
    data: Vec<String>,
}

impl AccountSource for SolanaRpc {
    fn get_account_data<'a>(
        &'a self,
        address: &'a Pubkey,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Vec<u8>>, ApiError>> + Send + 'a>> {
        Box::pin(async move {
            let body = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getAccountInfo",
                "params": [
                    address.to_string(),
                    { "encoding": "base64" }
                ]
            });
            let resp: RpcResponse = self
                .client
                .post(&self.url)
                .json(&body)
                .send()
                .await
                .map_err(|e| ApiError::Rpc(e.to_string()))?
                .json()
                .await
                .map_err(|e| ApiError::Rpc(e.to_string()))?;

            if let Some(err) = resp.error {
                return Err(ApiError::Rpc(err.to_string()));
            }
            let Some(value) = resp.result.and_then(|r| r.value) else {
                return Ok(None);
            };
            let encoded = value
                .data
                .first()
                .ok_or_else(|| ApiError::Rpc("missing account data".into()))?;
            let bytes = STANDARD
                .decode(encoded)
                .map_err(|e| ApiError::Rpc(e.to_string()))?;
            Ok(Some(bytes))
        })
    }
}

pub fn policy_pda(program_id: &Pubkey, policy_id: &[u8; 32]) -> Pubkey {
    Pubkey::find_program_address(
        &[escrow::constants::POLICY_SEED, policy_id.as_ref()],
        program_id,
    )
    .0
}

pub fn escrow_pda(program_id: &Pubkey, policy_id: &[u8; 32]) -> Pubkey {
    Pubkey::find_program_address(
        &[escrow::constants::ESCROW_SEED, policy_id.as_ref()],
        program_id,
    )
    .0
}

pub fn decode_policy(data: &[u8]) -> Result<PolicyAccount, ApiError> {
    let mut slice = data;
    PolicyAccount::try_deserialize(&mut slice).map_err(|_| ApiError::AccountDecode)
}

pub fn decode_escrow(data: &[u8]) -> Result<EscrowAccount, ApiError> {
    let mut slice = data;
    EscrowAccount::try_deserialize(&mut slice).map_err(|_| ApiError::AccountDecode)
}

pub fn status_label(status: EscrowStatus) -> &'static str {
    match status {
        EscrowStatus::Pending => "Pending",
        EscrowStatus::Active => "Active",
        EscrowStatus::Triggered => "Triggered",
        EscrowStatus::Paid => "Paid",
        EscrowStatus::Failed => "Failed",
    }
}

pub fn bytes32_to_string(bytes: &[u8; 32]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}
