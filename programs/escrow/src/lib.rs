pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("C4YXYjWeEuEcoNACGFLdU3muCaS9S9vTb73rr3WboKCi");

#[program]
pub mod escrow {
    use super::*;

    pub fn create_policy(
        ctx: Context<CreatePolicy>,
        policy_id: Pubkey,
        holder: Pubkey,
        expiry: i64,
        asset_class: [u8; 32],
    ) -> Result<()> {
        crate::instructions::create_policy::handle_create_policy(
            ctx,
            policy_id,
            holder,
            expiry,
            asset_class,
        )
    }
}
