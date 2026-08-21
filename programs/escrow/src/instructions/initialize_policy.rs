use anchor_lang::prelude::*;

use crate::{constants::*, error::ErrorCode, state::PolicyAccount};

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
    require!(expiry > clock.unix_timestamp, ErrorCode::InvalidExpiry);
    require!(asset_class != [0u8; 32], ErrorCode::InvalidAssetClass);

    let policy = &mut ctx.accounts.policy;
    policy.policy_id = policy_id;
    policy.holder = holder;
    policy.expiry = expiry;
    policy.asset_class = asset_class;
    policy.created_at = clock.unix_timestamp;
    policy.bump = ctx.bumps.policy;

    Ok(())
}
