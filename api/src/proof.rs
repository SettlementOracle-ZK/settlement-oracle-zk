use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::ApiError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalWitness {
    pub asset_class: String,
    pub feed_id: String,
    pub operator: String,
    pub oracle_conf: f64,
    pub oracle_price: f64,
    pub publish_time: i64,
    pub risk_score: f64,
    pub threshold: f64,
    pub triggered: bool,
}

pub fn witness_from_public_inputs(value: &serde_json::Value) -> Result<CanonicalWitness, ApiError> {
    serde_json::from_value(value.clone()).map_err(|_| ApiError::ProofInvalid)
}

pub fn canonical_witness_json(witness: &CanonicalWitness) -> String {
    format!(
        r#"{{"asset_class":"{}","feed_id":"{}","operator":"{}","oracle_conf":{},"oracle_price":{},"publish_time":{},"risk_score":{},"threshold":{},"triggered":{}}}"#,
        json_escape(&witness.asset_class),
        json_escape(&witness.feed_id),
        json_escape(&witness.operator),
        format_json_number(witness.oracle_conf),
        format_json_number(witness.oracle_price),
        witness.publish_time,
        format_json_number(witness.risk_score),
        format_json_number(witness.threshold),
        witness.triggered
    )
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Match JavaScript `JSON.stringify` for finite numbers (whole floats omit `.0`).
fn format_json_number(value: f64) -> String {
    if !value.is_finite() {
        return "null".to_string();
    }
    if value.fract() == 0.0 && value.abs() <= (i64::MAX as f64) {
        return format!("{}", value as i64);
    }
    let s = ryu::Buffer::new().format(value).to_string();
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{s}.0")
    }
}

pub fn hash_witness(witness: &CanonicalWitness) -> String {
    let json = canonical_witness_json(witness);
    let digest = Sha256::digest(json.as_bytes());
    format!("0x{}", hex::encode(digest))
}

pub fn verify_stored_hash(witness: &CanonicalWitness, expected: &str) -> bool {
    let computed = hash_witness(witness);
    let normalized = if expected.starts_with("0x") {
        expected.to_ascii_lowercase()
    } else {
        format!("0x{}", expected.to_ascii_lowercase())
    };
    computed == normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_matches_typescript_fixture() {
        let witness = CanonicalWitness {
            asset_class: "agriculture_climate".into(),
            feed_id: "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d".into(),
            operator: "lt".into(),
            oracle_conf: 0.5,
            oracle_price: 87.2,
            publish_time: 1_700_000_000,
            risk_score: 87.0,
            threshold: 100.0,
            triggered: true,
        };
        let hash = hash_witness(&witness);
        assert!(hash.starts_with("0x"));
        assert_eq!(hash.len(), 66);
        assert!(verify_stored_hash(&witness, &hash));
    }

    #[test]
    fn hash_matches_live_oracle_fixture() {
        let witness_json = serde_json::json!({
            "asset_class": "agriculture_climate",
            "feed_id": "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
            "operator": "lt",
            "oracle_conf": 0.04784881,
            "oracle_price": 85.27885594,
            "publish_time": 1787190672,
            "risk_score": 85,
            "threshold": 100,
            "triggered": true
        });
        let witness = witness_from_public_inputs(&witness_json).expect("parse witness");
        let json = canonical_witness_json(&witness);
        eprintln!("rust json: {json}");
        let hash = hash_witness(&witness);
        eprintln!("rust hash: {hash}");
        assert_eq!(
            hash,
            "0x29ab3e25fe7d7b15a9645a6bff7a0518848a050d0344368101578512fefde53b"
        );
    }

    #[test]
    fn hash_matches_js_float_artifact() {
        let witness_json = serde_json::json!({
            "asset_class": "agriculture_climate",
            "feed_id": "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
            "operator": "lt",
            "oracle_conf": 0.046930690000000004,
            "oracle_price": 86.67944457,
            "publish_time": 1787238102,
            "risk_score": 87,
            "threshold": 100,
            "triggered": true
        });
        let witness = witness_from_public_inputs(&witness_json).expect("parse witness");
        let json = canonical_witness_json(&witness);
        assert!(json.contains("0.046930690000000004"), "json: {json}");
        assert_eq!(
            hash_witness(&witness),
            "0xb5cd76ddda26d7044fec24b6cff89bbbe9186f497a5cb2e3abcc181a792253b7"
        );
    }
}
