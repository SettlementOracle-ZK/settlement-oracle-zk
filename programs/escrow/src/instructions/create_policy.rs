use anchor_lang::prelude::*;

use crate::{constants::*, error::ErrorCode, state::PolicyAccount};

#[derive(Accounts)]
#[instruction(policy_id: Pubkey)]
pub struct CreatePolicy<'info> {
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

pub fn handle_create_policy(
    ctx: Context<CreatePolicy>,
    policy_id: Pubkey,
    holder: Pubkey,
    expiry: i64,
    asset_class: [u8; 32],
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(expiry > now, ErrorCode::InvalidExpiry);
    require!(asset_class != [0u8; 32], ErrorCode::InvalidAssetClass);

    let policy = &mut ctx.accounts.policy;
    policy.policy_id = policy_id;
    policy.authority = ctx.accounts.authority.key();
    policy.holder = holder;
    policy.expiry = expiry;
    policy.asset_class = asset_class;
    policy.created_at = now;
    policy.bump = ctx.bumps.policy;

    msg!("Policy created: {}", policy_id);
    Ok(())
}
