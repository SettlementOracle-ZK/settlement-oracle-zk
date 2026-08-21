use anchor_lang::prelude::*;

use crate::{
    constants::*,
    error::ErrorCode,
    oracle::read_validated_price,
    state::{EscrowAccount, EscrowStatus, PolicyAccount},
};

#[derive(Accounts)]
pub struct EvaluateTrigger<'info> {
    /// Fee payer; permissionless crank — any signer may submit when oracle conditions are met.
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [ESCROW_SEED, escrow.policy_id.as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, EscrowAccount>,

    #[account(
        seeds = [POLICY_SEED, escrow.policy_id.as_ref()],
        bump = policy.bump,
        constraint = policy.policy_id == escrow.policy_id @ ErrorCode::InvalidPolicy,
    )]
    pub policy: Account<'info, PolicyAccount>,

    /// CHECK: Pyth legacy price feed account; validated in handler via `pyth_legacy` parser.
    pub price_feed: UncheckedAccount<'info>,
}

pub fn handle_evaluate_trigger(ctx: Context<EvaluateTrigger>) -> Result<()> {
    require!(!ctx.accounts.escrow.paused, ErrorCode::Paused);
    require!(
        ctx.accounts.escrow.status == EscrowStatus::Active,
        ErrorCode::EscrowNotActive
    );

    let clock = Clock::get()?;
    require!(
        ctx.accounts.policy.expiry > clock.unix_timestamp,
        ErrorCode::PolicyExpired
    );

    let price = read_validated_price(
        &ctx.accounts.price_feed.to_account_info(),
        clock.unix_timestamp,
    )?;

    require!(
        price.price < ctx.accounts.escrow.trigger_threshold,
        ErrorCode::TriggerNotMet
    );

    let escrow = &mut ctx.accounts.escrow;
    escrow.status = EscrowStatus::Triggered;

    Ok(())
}
