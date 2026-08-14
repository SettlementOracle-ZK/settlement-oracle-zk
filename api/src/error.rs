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
    #[error("policy not found on-chain")]
    PolicyNotFound,
    #[error("failed to deserialize on-chain account")]
    AccountDecode,
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
            ApiError::InvalidPolicyId => StatusCode::BAD_REQUEST,
            ApiError::PolicyNotFound => StatusCode::NOT_FOUND,
            ApiError::AccountDecode => StatusCode::BAD_GATEWAY,
            ApiError::Database(_) | ApiError::Migration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Rpc(_) => StatusCode::BAD_GATEWAY,
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
