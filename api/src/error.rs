use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("invalid policy id; expected 32-byte hex")]
    InvalidPolicyId,
    #[error("invalid proof hash; expected even-length hex")]
    InvalidProofHash,
    #[error("policy not found on-chain")]
    PolicyNotFound,
    #[error("proof not found")]
    ProofNotFound,
    #[error("settlement not found")]
    SettlementNotFound,
    #[error("invalid proof witness")]
    ProofInvalid,
    #[error("endpoint disabled outside development")]
    DevOnly,
    #[error("failed to deserialize on-chain account")]
    AccountDecode,
    #[error("oracle unavailable: {0}")]
    Oracle(String),
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("migration error")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("rpc error: {0}")]
    Rpc(String),
    #[error("configuration error: {0}")]
    Config(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            ApiError::PolicyNotFound | ApiError::ProofNotFound | ApiError::SettlementNotFound => {
                StatusCode::NOT_FOUND
            }
            ApiError::InvalidPolicyId | ApiError::InvalidProofHash | ApiError::ProofInvalid => {
                StatusCode::BAD_REQUEST
            }
            ApiError::DevOnly => StatusCode::FORBIDDEN,
            ApiError::AccountDecode => StatusCode::BAD_GATEWAY,
            ApiError::Database(_) | ApiError::Migration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Rpc(_) | ApiError::Oracle(_) => StatusCode::BAD_GATEWAY,
            ApiError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (
            status,
            Json(ErrorBody {
                error: self.to_string(),
            }),
        )
            .into_response()
    }
}
