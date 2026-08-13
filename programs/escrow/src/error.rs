use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Only the escrow authority can perform this action")]
    Unauthorized,
    #[msg("Policy ID does not match the policy account")]
    InvalidPolicy,
    #[msg("Deposit amount must be greater than zero")]
    InsufficientDeposit,
    #[msg("Escrow is not in an active state for this operation")]
    EscrowNotActive,
    #[msg("Policy has expired")]
    PolicyExpired,
    #[msg("Expiry must be in the future")]
    InvalidExpiry,
    #[msg("Asset class must be non-empty")]
    InvalidAssetClass,
    #[msg("Oracle data is stale")]
    OracleStale,
    #[msg("Oracle confidence interval is too wide")]
    OracleLowConfidence,
    #[msg("Escrow is paused")]
    Paused,
    #[msg("Trigger condition has not been met")]
    TriggerNotMet,
    #[msg("Escrow has already been settled")]
    AlreadySettled,
}
