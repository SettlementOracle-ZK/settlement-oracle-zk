//! Minimal reader for Pyth legacy Solana price accounts (layout from pyth-sdk-solana 0.10.6).
//! Avoids pulling pyth-sdk-solana into the program — it conflicts with anchor-lang borsh versions.

use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::error::ErrorCode;

pub const MAGIC: u32 = 0xa1b2c3d4;
pub const VERSION_2: u32 = 2;
pub const ACCOUNT_TYPE_PRICE: u32 = 3;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PriceStatus {
    Unknown = 0,
    Trading,
    Halted,
    Auction,
    Ignored,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PriceInfo {
    pub price: i64,
    pub conf: u64,
    pub status: u8,
    pub corp_act: u8,
    pub _pad: [u8; 6],
    pub pub_slot: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Rational {
    pub val: i64,
    pub numer: i64,
    pub denom: i64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PriceComp {
    pub publisher: [u8; 32],
    pub agg: PriceInfo,
    pub latest: PriceInfo,
}

/// Solana-specific Pyth price account (`GenericPriceAccount<32, ()>`).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SolanaPriceAccount {
    pub magic: u32,
    pub ver: u32,
    pub atype: u32,
    pub size: u32,
    pub ptype: u8,
    pub _ptype_pad: [u8; 3],
    pub expo: i32,
    pub num: u32,
    pub num_qt: u32,
    pub last_slot: u64,
    pub valid_slot: u64,
    pub ema_price: Rational,
    pub ema_conf: Rational,
    pub timestamp: i64,
    pub min_pub: u8,
    pub drv2: u8,
    pub drv3: u16,
    pub drv4: u32,
    pub prod: [u8; 32],
    pub next: [u8; 32],
    pub prev_slot: u64,
    pub prev_price: i64,
    pub prev_conf: u64,
    pub prev_timestamp: i64,
    pub agg: PriceInfo,
    pub comp: [PriceComp; 32],
}

pub struct ValidatedOraclePrice {
    pub price: i64,
    pub conf: u64,
    pub publish_time: i64,
}

pub fn load_price_account(data: &[u8]) -> Result<&SolanaPriceAccount> {
    let account: &SolanaPriceAccount = bytemuck::try_from_bytes(data)
        .map_err(|_| error!(ErrorCode::OracleStale))?;

    require!(account.magic == MAGIC, ErrorCode::OracleStale);
    require!(account.ver == VERSION_2, ErrorCode::OracleStale);
    require!(account.atype == ACCOUNT_TYPE_PRICE, ErrorCode::OracleStale);

    Ok(account)
}

impl SolanaPriceAccount {
    pub fn current_price(&self, now: i64, max_age_seconds: u64) -> Result<ValidatedOraclePrice> {
        let (price, conf, publish_time) = if self.agg.status == PriceStatus::Trading as u8 {
            (self.agg.price, self.agg.conf, self.timestamp)
        } else {
            (self.prev_price, self.prev_conf, self.prev_timestamp)
        };

        let age = (publish_time - now).unsigned_abs();
        require!(age <= max_age_seconds, ErrorCode::OracleStale);

        Ok(ValidatedOraclePrice {
            price,
            conf,
            publish_time,
        })
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;
    use std::mem::{align_of, offset_of, size_of};

    #[test]
    fn price_info_layout_matches_pyth_sdk() {
        assert_eq!(size_of::<PriceInfo>(), 32);
        assert_eq!(offset_of!(PriceInfo, price), 0);
        assert_eq!(offset_of!(PriceInfo, conf), 8);
        assert_eq!(offset_of!(PriceInfo, status), 16);
        assert_eq!(offset_of!(PriceInfo, pub_slot), 24);
    }

    #[test]
    fn solana_price_account_agg_offset() {
        assert_eq!(offset_of!(SolanaPriceAccount, agg), 208);
        assert_eq!(align_of::<SolanaPriceAccount>(), 8);
    }
}
