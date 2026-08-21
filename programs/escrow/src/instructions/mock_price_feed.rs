use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke_signed, system_instruction};

use crate::{
    constants::MOCK_PYTH_SEED,
    error::ErrorCode,
    pyth_legacy::{write_mock_legacy_price, MOCK_ACCOUNT_SIZE},
};

/// Devnet/local helper: program-owned PDA with legacy Pyth layout (below trigger threshold).
pub const MOCK_PRICE_DEFAULT: i64 = 50_000_000_000;
pub const MOCK_CONF_DEFAULT: u64 = 1_000_000;

#[derive(Accounts)]
pub struct MockPriceFeedAccounts<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: legacy Pyth layout bytes stored on program-owned PDA
    #[account(
        mut,
        seeds = [MOCK_PYTH_SEED],
        bump,
    )]
    pub price_feed: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_init_mock_price_feed(ctx: Context<MockPriceFeedAccounts>) -> Result<()> {
    let account = &ctx.accounts.price_feed;
    let space = MOCK_ACCOUNT_SIZE;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);

    if account.lamports() == 0 {
        invoke_signed(
            &system_instruction::create_account(
                ctx.accounts.authority.key,
                account.key,
                lamports,
                space as u64,
                ctx.program_id,
            ),
            &[
                ctx.accounts.authority.to_account_info(),
                account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[MOCK_PYTH_SEED, &[ctx.bumps.price_feed]]],
        )?;
    } else {
        require!(account.owner == ctx.program_id, ErrorCode::Unauthorized);
        require!(account.data_len() == space, ErrorCode::Unauthorized);
    }

    let now = Clock::get()?.unix_timestamp;
    write_account_data(account, MOCK_PRICE_DEFAULT, MOCK_CONF_DEFAULT, now)
}

fn write_account_data(
    account: &UncheckedAccount<'_>,
    price: i64,
    conf: u64,
    publish_time: i64,
) -> Result<()> {
    let mut data = account.try_borrow_mut_data()?;
    write_mock_legacy_price(&mut data, price, conf, publish_time)
}
