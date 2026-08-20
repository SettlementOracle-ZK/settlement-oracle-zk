pub mod constants;
pub mod error;
pub mod instructions;
pub mod oracle;
pub mod pyth_legacy;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("DqdgQv57RWZ4RVUQ7v6SMvTfA11gFph2TNKZDz7sa3Ap");

#[program]
pub mod escrow {
    use super::*;

    pub fn initialize_policy(
        ctx: Context<InitializePolicy>,
        policy_id: [u8; 32],
        holder: Pubkey,
        expiry: i64,
        asset_class: [u8; 32],
    ) -> Result<()> {
        instructions::initialize_policy::handle_initialize_policy(
            ctx,
            policy_id,
            holder,
            expiry,
            asset_class,
        )
    }

    pub fn initialize_escrow(
        ctx: Context<InitializeEscrow>,
        policy_id: [u8; 32],
        trigger_threshold: i64,
    ) -> Result<()> {
        instructions::initialize_escrow::handle_initialize_escrow(ctx, policy_id, trigger_threshold)
    }

    pub fn deposit_premium(ctx: Context<DepositPremium>, amount: u64) -> Result<()> {
        instructions::deposit_premium::handle_deposit_premium(ctx, amount)
    }

    pub fn pause(ctx: Context<PauseEscrow>) -> Result<()> {
        instructions::pause::handle_pause(ctx)
    }

    pub fn unpause(ctx: Context<PauseEscrow>) -> Result<()> {
        instructions::pause::handle_unpause(ctx)
    }

    pub fn execute_payout(ctx: Context<ExecutePayout>) -> Result<()> {
        instructions::execute_payout::handle_execute_payout(ctx)
    }

    pub fn evaluate_trigger(ctx: Context<EvaluateTrigger>) -> Result<()> {
        instructions::evaluate_trigger::handle_evaluate_trigger(ctx)
    }
}
