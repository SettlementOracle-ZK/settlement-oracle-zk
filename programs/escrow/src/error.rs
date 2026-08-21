use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    Unauthorized,
    InvalidPolicy,
    InsufficientDeposit,
    EscrowNotActive,
    PolicyExpired,
    InvalidExpiry,
    InvalidAssetClass,
    OracleStale,
    OracleLowConfidence,
    Paused,
    TriggerNotMet,
    AlreadySettled,
}
