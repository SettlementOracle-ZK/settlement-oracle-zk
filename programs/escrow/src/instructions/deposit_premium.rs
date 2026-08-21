use anchor_lang::prelude::*;

use crate::{
    constants::*,
    error::ErrorCode,
    state::{EscrowAccount, EscrowStatus},
};

#[derive(Accounts)]
pub struct DepositPremium<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [ESCROW_SEED, escrow.policy_id.as_ref()],
        bump = escrow.bump,
        has_one = authority @ ErrorCode::Unauthorized,
    )]
    pub escrow: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
}

pub fn handle_deposit_premium(ctx: Context<DepositPremium>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InsufficientDeposit);
    require!(
        ctx.accounts.escrow.status == EscrowStatus::Active,
        ErrorCode::EscrowNotActive
    );

    let cpi_accounts = anchor_lang::system_program::Transfer {
        from: ctx.accounts.authority.to_account_info(),
        to: ctx.accounts.escrow.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(anchor_lang::system_program::ID, cpi_accounts);
    anchor_lang::system_program::transfer(cpi_ctx, amount)?;

    ctx.accounts.escrow.amount = ctx
        .accounts
        .escrow
        .amount
        .checked_add(amount)
        .ok_or(ErrorCode::InsufficientDeposit)?;

    Ok(())
}
