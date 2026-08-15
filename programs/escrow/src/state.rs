use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace, Debug)]
pub enum EscrowStatus {
    Pending,
    Active,
    Triggered,
    Paid,
    Failed,
}

#[account]
#[derive(InitSpace)]
pub struct PolicyAccount {
    pub policy_id: [u8; 32],
    pub holder: Pubkey,
    pub expiry: i64,
    pub asset_class: [u8; 32],
    pub created_at: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct EscrowAccount {
    pub policy_id: [u8; 32],
    pub authority: Pubkey,
    pub amount: u64,
    pub trigger_threshold: i64,
    pub status: EscrowStatus,
    pub paused: bool,
    pub bump: u8,
}
