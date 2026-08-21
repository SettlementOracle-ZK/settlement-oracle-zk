use anchor_lang::prelude::*;

use crate::{constants::*, error::ErrorCode, state::EscrowAccount};

#[derive(Accounts)]
pub struct PauseEscrow<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [ESCROW_SEED, escrow.policy_id.as_ref()],
        bump = escrow.bump,
        has_one = authority @ ErrorCode::Unauthorized,
    )]
    pub escrow: Account<'info, EscrowAccount>,
}

pub fn handle_pause(ctx: Context<PauseEscrow>) -> Result<()> {
    ctx.accounts.escrow.paused = true;
    Ok(())
}

pub fn handle_unpause(ctx: Context<PauseEscrow>) -> Result<()> {
    ctx.accounts.escrow.paused = false;
    Ok(())
}
