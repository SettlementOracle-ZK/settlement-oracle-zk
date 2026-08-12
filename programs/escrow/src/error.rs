use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Only the policy authority can perform this action")]
    Unauthorized,
    #[msg("Policy has expired")]
    PolicyExpired,
    #[msg("Expiry must be in the future")]
    InvalidExpiry,
    #[msg("Asset class must be non-empty")]
    InvalidAssetClass,
    #[msg("Oracle price data is stale")]
    OracleStale,
    #[msg("Oracle confidence is below the required threshold")]
    OracleLowConfidence,
    #[msg("Escrow is paused")]
    Paused,
    #[msg("Trigger condition has not been met")]
    TriggerNotMet,
    #[msg("Escrow has already been settled")]
    AlreadySettled,
}
