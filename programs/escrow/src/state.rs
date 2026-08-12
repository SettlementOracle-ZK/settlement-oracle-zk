use anchor_lang::prelude::*;

/// Settlement status for an escrow vault.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum EscrowStatus {
    Active = 0,
    Triggered = 1,
    Settled = 2,
    Cancelled = 3,
}

/// On-chain parametric insurance policy metadata.
#[account]
#[derive(InitSpace)]
pub struct PolicyAccount {
    /// Unique policy key (also used as PDA seed).
    pub policy_id: Pubkey,
    /// Insurer who created the policy.
    pub authority: Pubkey,
    /// Insured payout recipient.
    pub holder: Pubkey,
    /// Unix expiry timestamp; must be in the future at create.
    pub expiry: i64,
    /// Fixed UTF-8 asset class (e.g. `agriculture_climate`), zero-padded.
    pub asset_class: [u8; 32],
    /// Unix timestamp when the policy was created.
    pub created_at: i64,
    /// PDA bump.
    pub bump: u8,
}

/// Escrow vault state (defined for the next slice; unused by `create_policy`).
#[account]
#[derive(InitSpace)]
pub struct EscrowAccount {
    pub policy_id: Pubkey,
    pub authority: Pubkey,
    pub amount: u64,
    pub trigger_threshold: i64,
    pub status: EscrowStatus,
    pub paused: bool,
    pub bump: u8,
}
