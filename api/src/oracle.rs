//! Pyth Hermes reader for the Trigger Monitor. Fail closed on HTTP errors.

use anchor_lang::prelude::Pubkey;
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::rpc::AccountSource;

/// Matches `oracle-connector` defaults.
pub const MAX_STALENESS_SECONDS: i64 = 60;
pub const MAX_CONFIDENCE_RATIO: f64 = 0.05;

#[derive(Debug, Deserialize)]
struct HermesLatestResponse {
    parsed: Option<Vec<HermesParsedPrice>>,
}

#[derive(Debug, Deserialize)]
struct HermesParsedPrice {
    id: String,
    price: HermesPrice,
}

#[derive(Debug, Deserialize)]
struct HermesPrice {
    price: String,
    conf: String,
    expo: i32,
    publish_time: i64,
}

#[derive(Debug, Serialize)]
pub struct OracleFeedView {
    pub feed_id: String,
    pub symbol: &'static str,
    pub price: f64,
    pub conf: f64,
    pub expo: i32,
    pub publish_time: i64,
    pub timestamp: String,
    pub age_seconds: i64,
    pub stale: bool,
    pub low_confidence: bool,
    pub max_staleness_seconds: i64,
    pub max_confidence_ratio: f64,
}

pub fn normalize_pyth_price(raw: &str, expo: i32) -> Result<f64, ApiError> {
    let mantissa: f64 = raw
        .parse()
        .map_err(|_| ApiError::Oracle("invalid pyth mantissa".into()))?;
    if !mantissa.is_finite() {
        return Err(ApiError::Oracle("non-finite pyth mantissa".into()));
    }
    let value = mantissa * 10f64.powi(expo);
    if !value.is_finite() {
        return Err(ApiError::Oracle("non-finite pyth price".into()));
    }
    Ok(value)
}

pub fn is_stale(publish_time: i64, now_seconds: i64, max_staleness_seconds: i64) -> bool {
    if publish_time > now_seconds {
        return true;
    }
    now_seconds.saturating_sub(publish_time) > max_staleness_seconds
}

pub fn is_low_confidence(price: f64, conf: f64, max_ratio: f64) -> bool {
    if price == 0.0 {
        return true;
    }
    (conf / price).abs() > max_ratio
}

pub fn feed_view(
    feed_id: String,
    symbol: &'static str,
    price: f64,
    conf: f64,
    expo: i32,
    publish_time: i64,
    now_seconds: i64,
) -> OracleFeedView {
    let age_seconds = now_seconds.saturating_sub(publish_time);
    let timestamp = chrono::DateTime::from_timestamp(publish_time, 0)
        .unwrap_or_else(Utc::now)
        .to_rfc3339_opts(SecondsFormat::Secs, true);
    OracleFeedView {
        feed_id,
        symbol,
        price,
        conf,
        expo,
        publish_time,
        timestamp,
        age_seconds,
        stale: is_stale(publish_time, now_seconds, MAX_STALENESS_SECONDS),
        low_confidence: is_low_confidence(price, conf, MAX_CONFIDENCE_RATIO),
        max_staleness_seconds: MAX_STALENESS_SECONDS,
        max_confidence_ratio: MAX_CONFIDENCE_RATIO,
    }
}

pub fn normalize_feed_id(id: &str) -> String {
    let hex = id
        .strip_prefix("0x")
        .or_else(|| id.strip_prefix("0X"))
        .unwrap_or(id);
    format!("0x{}", hex.to_ascii_lowercase())
}

pub async fn fetch_latest_feed(
    http: &reqwest::Client,
    hermes_url: &str,
    feed_id: &str,
) -> Result<OracleFeedView, ApiError> {
    let want = normalize_feed_id(feed_id);
    let url = format!(
        "{}/v2/updates/price/latest?ids[]={}",
        hermes_url.trim_end_matches('/'),
        feed_id
    );
    let resp: HermesLatestResponse = http
        .get(&url)
        .send()
        .await
        .map_err(|e| ApiError::Oracle(e.to_string()))?
        .error_for_status()
        .map_err(|e| ApiError::Oracle(e.to_string()))?
        .json()
        .await
        .map_err(|e| ApiError::Oracle(e.to_string()))?;

    let parsed = resp
        .parsed
        .unwrap_or_default()
        .into_iter()
        .find(|row| normalize_feed_id(&row.id) == want)
        .ok_or_else(|| ApiError::Oracle("hermes payload missing requested feed".into()))?;
    let price = normalize_pyth_price(&parsed.price.price, parsed.price.expo)?;
    let conf = normalize_pyth_price(&parsed.price.conf, parsed.price.expo)?;
    let now_seconds = Utc::now().timestamp();

    Ok(feed_view(
        normalize_feed_id(&parsed.id),
        "SOL/USD",
        price,
        conf,
        parsed.price.expo,
        parsed.price.publish_time,
        now_seconds,
    ))
}

fn delay_low_confidence(price: i64, conf: u64) -> bool {
    if price == 0 {
        return true;
    }
    let abs_price = price.unsigned_abs();
    conf.saturating_mul(10_000) > abs_price.saturating_mul(escrow::constants::MAX_CONFIDENCE_BPS)
}

pub async fn fetch_delay_feed(
    rpc: &dyn AccountSource,
    feed_pubkey: &Pubkey,
) -> Result<OracleFeedView, ApiError> {
    let data = rpc
        .get_account_data(feed_pubkey)
        .await?
        .ok_or_else(|| ApiError::Oracle("delay feed account missing".into()))?;

    let now_seconds = Utc::now().timestamp();
    let quote = escrow::pyth_legacy::parse_validated_price(
        &data,
        now_seconds,
        escrow::constants::MAX_STALENESS_SECONDS,
    )
    .map_err(|_| ApiError::Oracle("invalid or stale delay feed".into()))?;

    let price = quote.price as f64;
    let conf = quote.conf as f64;
    let publish_time = quote.publish_time;
    let age_seconds = now_seconds.saturating_sub(publish_time);

    Ok(OracleFeedView {
        feed_id: feed_pubkey.to_string(),
        symbol: "Delay",
        price,
        conf,
        expo: 0,
        publish_time,
        timestamp: chrono::DateTime::from_timestamp(publish_time, 0)
            .unwrap_or_else(Utc::now)
            .to_rfc3339_opts(SecondsFormat::Secs, true),
        age_seconds,
        stale: is_stale(publish_time, now_seconds, MAX_STALENESS_SECONDS),
        low_confidence: delay_low_confidence(quote.price, quote.conf),
        max_staleness_seconds: MAX_STALENESS_SECONDS,
        max_confidence_ratio: MAX_CONFIDENCE_RATIO,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_when_older_than_window() {
        assert!(!is_stale(100, 160, 60));
        assert!(is_stale(100, 161, 60));
        assert!(is_stale(200, 100, 60));
    }

    #[test]
    fn low_confidence_when_ratio_exceeds_cap() {
        assert!(!is_low_confidence(100.0, 4.0, 0.05));
        assert!(is_low_confidence(100.0, 6.0, 0.05));
        assert!(is_low_confidence(0.0, 1.0, 0.05));
    }

    #[test]
    fn normalize_applies_exponent() {
        assert_eq!(normalize_pyth_price("14250", -2).unwrap(), 142.5);
    }

    #[test]
    fn normalize_feed_id_is_canonical() {
        assert_eq!(
            normalize_feed_id("EF0D8B6FDA2CEBA41DA15D4095D1DA392A0D2F8ED0C6C7BC0F4CFAC8C280B56D"),
            normalize_feed_id("0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d")
        );
    }
}
