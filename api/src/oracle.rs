//! Pyth Hermes reader for the Trigger Monitor. Fail closed on HTTP errors.

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

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
    Ok(mantissa * 10f64.powi(expo))
}

pub fn is_stale(publish_time: i64, now_seconds: i64, max_staleness_seconds: i64) -> bool {
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
        symbol: "SOL/USD",
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

pub async fn fetch_latest_feed(
    http: &reqwest::Client,
    hermes_url: &str,
    feed_id: &str,
) -> Result<OracleFeedView, ApiError> {
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
        .and_then(|mut rows| rows.pop())
        .ok_or_else(|| ApiError::Oracle("empty hermes payload".into()))?;
    let price = normalize_pyth_price(&parsed.price.price, parsed.price.expo)?;
    let conf = normalize_pyth_price(&parsed.price.conf, parsed.price.expo)?;
    let now_seconds = Utc::now().timestamp();
    let id = if parsed.id.starts_with("0x") {
        parsed.id
    } else {
        format!("0x{}", parsed.id)
    };

    Ok(feed_view(
        id,
        price,
        conf,
        parsed.price.expo,
        parsed.price.publish_time,
        now_seconds,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_when_older_than_window() {
        assert!(!is_stale(100, 160, 60));
        assert!(is_stale(100, 161, 60));
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
}
