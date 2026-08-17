use anchor_lang::prelude::*;

use crate::{
    constants::*,
    error::ErrorCode,
    pyth_legacy::{load_price_account, ValidatedOraclePrice},
};

/// Read a Pyth legacy price feed account and apply MVP staleness + confidence gates.
pub fn read_validated_price(price_feed: &AccountInfo, now: i64) -> Result<ValidatedOraclePrice> {
    let data = price_feed
        .try_borrow_data()
        .map_err(|_| error!(ErrorCode::OracleStale))?;

    let account = load_price_account(&data)?;
    let quote = account.current_price(now, MAX_STALENESS_SECONDS)?;

    require!(quote.price != 0, ErrorCode::TriggerNotMet);

    let abs_price = quote.price.unsigned_abs();
    if quote
        .conf
        .saturating_mul(10_000)
        > abs_price.saturating_mul(MAX_CONFIDENCE_BPS)
    {
        return Err(error!(ErrorCode::OracleLowConfidence));
    }

    Ok(quote)
}
