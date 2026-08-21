//! Minimal reader for Pyth legacy Solana price accounts (layout from pyth-sdk-solana 0.10.6).
//! Offset-based parsing keeps the on-chain binary small for devnet deploy budgets.

use anchor_lang::prelude::*;

use crate::error::ErrorCode;

pub const MAGIC: u32 = 0xa1b2c3d4;
pub const VERSION_2: u32 = 2;
pub const ACCOUNT_TYPE_PRICE: u32 = 3;

/// Bytes required through `agg` (mock PDA size). Full legacy feeds are ~3312 B.
pub const MOCK_ACCOUNT_SIZE: usize = 240;

const OFF_MAGIC: usize = 0;
const OFF_VER: usize = 4;
const OFF_ATYPE: usize = 8;
const OFF_TIMESTAMP: usize = 96;
const OFF_PREV_PRICE: usize = 184;
const OFF_PREV_CONF: usize = 192;
const OFF_PREV_TIMESTAMP: usize = 200;
const OFF_AGG_PRICE: usize = 208;
const OFF_AGG_CONF: usize = 216;
const OFF_AGG_STATUS: usize = 224;

const STATUS_TRADING: u8 = 1;

pub struct ValidatedOraclePrice {
    pub price: i64,
    pub conf: u64,
    pub publish_time: i64,
}

pub fn parse_validated_price(
    data: &[u8],
    now: i64,
    max_age_seconds: u64,
) -> Result<ValidatedOraclePrice> {
    require!(data.len() >= MOCK_ACCOUNT_SIZE, ErrorCode::OracleStale);

    let magic = read_u32(data, OFF_MAGIC)?;
    let ver = read_u32(data, OFF_VER)?;
    let atype = read_u32(data, OFF_ATYPE)?;
    require!(magic == MAGIC, ErrorCode::OracleStale);
    require!(ver == VERSION_2, ErrorCode::OracleStale);
    require!(atype == ACCOUNT_TYPE_PRICE, ErrorCode::OracleStale);

    let timestamp = read_i64(data, OFF_TIMESTAMP)?;
    let agg_status = data[OFF_AGG_STATUS];

    let (price, conf, publish_time) = if agg_status == STATUS_TRADING {
        (
            read_i64(data, OFF_AGG_PRICE)?,
            read_u64(data, OFF_AGG_CONF)?,
            timestamp,
        )
    } else {
        (
            read_i64(data, OFF_PREV_PRICE)?,
            read_u64(data, OFF_PREV_CONF)?,
            read_i64(data, OFF_PREV_TIMESTAMP)?,
        )
    };

    let age = (publish_time - now).unsigned_abs();
    require!(age <= max_age_seconds, ErrorCode::OracleStale);

    Ok(ValidatedOraclePrice {
        price,
        conf,
        publish_time,
    })
}

/// Fill a compact legacy header for devnet/local smoke (delay minutes stand-in).
pub fn write_mock_legacy_price(data: &mut [u8], price: i64, conf: u64, publish_time: i64) -> Result<()> {
    require!(data.len() >= MOCK_ACCOUNT_SIZE, ErrorCode::Unauthorized);
    data.fill(0);
    write_u32(data, OFF_MAGIC, MAGIC);
    write_u32(data, OFF_VER, VERSION_2);
    write_u32(data, OFF_ATYPE, ACCOUNT_TYPE_PRICE);
    write_i64(data, OFF_TIMESTAMP, publish_time);
    write_i64(data, OFF_AGG_PRICE, price);
    write_u64(data, OFF_AGG_CONF, conf);
    data[OFF_AGG_STATUS] = STATUS_TRADING;
    Ok(())
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32> {
    let bytes: [u8; 4] = data[offset..offset + 4]
        .try_into()
        .map_err(|_| error!(ErrorCode::OracleStale))?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_i64(data: &[u8], offset: usize) -> Result<i64> {
    let bytes: [u8; 8] = data[offset..offset + 8]
        .try_into()
        .map_err(|_| error!(ErrorCode::OracleStale))?;
    Ok(i64::from_le_bytes(bytes))
}

fn read_u64(data: &[u8], offset: usize) -> Result<u64> {
    let bytes: [u8; 8] = data[offset..offset + 8]
        .try_into()
        .map_err(|_| error!(ErrorCode::OracleStale))?;
    Ok(u64::from_le_bytes(bytes))
}

fn write_u32(data: &mut [u8], offset: usize, value: u32) {
    data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_i64(data: &mut [u8], offset: usize, value: i64) {
    data[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn write_u64(data: &mut [u8], offset: usize, value: u64) {
    data[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod layout_tests {
    use super::*;
    use std::mem::{align_of, offset_of, size_of};

    #[repr(C)]
    #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    struct PriceInfo {
        price: i64,
        conf: u64,
        status: u8,
        corp_act: u8,
        _pad: [u8; 6],
        pub_slot: u64,
    }

    #[repr(C)]
    #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    struct Rational {
        val: i64,
        numer: i64,
        denom: i64,
    }

    #[repr(C)]
    #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    struct SolanaPriceAccount {
        magic: u32,
        ver: u32,
        atype: u32,
        size: u32,
        ptype: u8,
        _ptype_pad: [u8; 3],
        expo: i32,
        num: u32,
        num_qt: u32,
        last_slot: u64,
        valid_slot: u64,
        ema_price: Rational,
        ema_conf: Rational,
        timestamp: i64,
        min_pub: u8,
        drv2: u8,
        drv3: u16,
        drv4: u32,
        prod: [u8; 32],
        next: [u8; 32],
        prev_slot: u64,
        prev_price: i64,
        prev_conf: u64,
        prev_timestamp: i64,
        agg: PriceInfo,
    }

    #[test]
    fn mock_account_size_matches_agg_end() {
        assert_eq!(size_of::<PriceInfo>(), 32);
        assert_eq!(offset_of!(SolanaPriceAccount, timestamp), OFF_TIMESTAMP);
        assert_eq!(offset_of!(SolanaPriceAccount, prev_price), OFF_PREV_PRICE);
        assert_eq!(offset_of!(SolanaPriceAccount, agg), OFF_AGG);
        assert_eq!(MOCK_ACCOUNT_SIZE, offset_of!(SolanaPriceAccount, agg) + size_of::<PriceInfo>());
        assert_eq!(align_of::<SolanaPriceAccount>(), 8);
    }

    const OFF_AGG: usize = 208;
}
