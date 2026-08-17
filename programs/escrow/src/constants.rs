use anchor_lang::prelude::*;

#[constant]
pub const POLICY_SEED: &[u8] = b"policy";

#[constant]
pub const ESCROW_SEED: &[u8] = b"escrow";

/// Reject oracle data older than this (seconds). Matches oracle-connector MVP default.
#[constant]
pub const MAX_STALENESS_SECONDS: u64 = 60;

/// Reject when confidence / |price| exceeds this ratio (500 bps = 5%).
#[constant]
pub const MAX_CONFIDENCE_BPS: u64 = 500;
