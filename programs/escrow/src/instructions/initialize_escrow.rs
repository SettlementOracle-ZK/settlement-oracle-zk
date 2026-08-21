use anchor_lang::prelude::*;

use crate::{
    constants::*,
    error::ErrorCode,
    state::{EscrowAccount, EscrowStatus, PolicyAccount},
};

#[derive(Accounts)]
#[instruction(policy_id: [u8; 32])]
pub struct InitializeEscrow<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [POLICY_SEED, policy_id.as_ref()],
        bump = policy.bump,
    )]
    pub policy: Account<'info, PolicyAccount>,

    #[account(
        init,
        payer = authority,
        space = 8 + EscrowAccount::INIT_SPACE,
        seeds = [ESCROW_SEED, policy_id.as_ref()],
        bump
    )]
    pub escrow: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_escrow(
    ctx: Context<InitializeEscrow>,
    policy_id: [u8; 32],
    trigger_threshold: i64,
) -> Result<()> {
    require!(
        policy_id == ctx.accounts.policy.policy_id,
        ErrorCode::InvalidPolicy
    );

    let clock = Clock::get()?;
    require!(
        ctx.accounts.policy.expiry > clock.unix_timestamp,
        ErrorCode::PolicyExpired
    );

    let escrow = &mut ctx.accounts.escrow;
    escrow.policy_id = policy_id;
    escrow.authority = ctx.accounts.authority.key();
    escrow.amount = 0;
    escrow.trigger_threshold = trigger_threshold;
    escrow.status = EscrowStatus::Active;
    escrow.paused = false;
    escrow.bump = ctx.bumps.escrow;

    Ok(())
}
