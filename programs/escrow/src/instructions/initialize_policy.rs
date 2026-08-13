use anchor_lang::prelude::*;

use crate::{constants::*, state::PolicyAccount};

#[derive(Accounts)]
#[instruction(policy_id: [u8; 32])]
pub struct InitializePolicy<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + PolicyAccount::INIT_SPACE,
        seeds = [POLICY_SEED, policy_id.as_ref()],
        bump
    )]
    pub policy: Account<'info, PolicyAccount>,

    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_policy(
    ctx: Context<InitializePolicy>,
    policy_id: [u8; 32],
    holder: Pubkey,
    expiry: i64,
    asset_class: [u8; 32],
) -> Result<()> {
    let clock = Clock::get()?;
    require!(expiry > clock.unix_timestamp, crate::error::ErrorCode::PolicyExpired);

    let policy = &mut ctx.accounts.policy;
    policy.policy_id = policy_id;
    policy.holder = holder;
    policy.expiry = expiry;
    policy.asset_class = asset_class;
    policy.created_at = clock.unix_timestamp;
    policy.bump = ctx.bumps.policy;

    msg!("Policy initialized: {:?}", policy_id);
    Ok(())
}
