use anchor_lang::prelude::*;

use crate::{
    constants::*,
    error::ErrorCode,
    state::{EscrowAccount, EscrowStatus, PolicyAccount},
};

#[derive(Accounts)]
pub struct ExecutePayout<'info> {
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

    #[account(
        mut,
        constraint = holder.key() == policy.holder @ ErrorCode::InvalidPolicy,
    )]
    pub holder: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_execute_payout(ctx: Context<ExecutePayout>) -> Result<()> {
    require!(!ctx.accounts.escrow.paused, ErrorCode::Paused);
    require!(
        ctx.accounts.escrow.status != EscrowStatus::Paid,
        ErrorCode::AlreadySettled
    );
    require!(
        ctx.accounts.escrow.status == EscrowStatus::Triggered,
        ErrorCode::TriggerNotMet
    );
    require!(
        ctx.accounts.escrow.amount > 0,
        ErrorCode::InsufficientDeposit
    );

    let amount = ctx.accounts.escrow.amount;
    ctx.accounts.escrow.sub_lamports(amount)?;
    ctx.accounts.holder.add_lamports(amount)?;

    let escrow = &mut ctx.accounts.escrow;
    escrow.amount = 0;
    escrow.status = EscrowStatus::Paid;

    Ok(())
}
