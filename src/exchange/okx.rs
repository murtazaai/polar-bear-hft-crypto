//! # OKX REST API Authentication
//!
//! ## Signing scheme (HMAC-SHA256 + base64)
//!
//! ```text
//! timestamp   = ISO-8601 UTC, e.g. "2024-01-15T12:00:00.000Z"
//! prehash     = timestamp + method + requestPath + body
//! signature   = base64( HMAC-SHA256(apiSecret, prehash) )
//! ```
//!
//! ## Required headers
//! - `OK-ACCESS-KEY`        : API key
//! - `OK-ACCESS-SIGN`       : base64-encoded HMAC-SHA256 signature
//! - `OK-ACCESS-TIMESTAMP`  : ISO-8601 timestamp
//! - `OK-ACCESS-PASSPHRASE` : passphrase set at API key creation
//!
//! ## Reference
//! - https://www.okx.com/docs-v5/en/#overview-rest-authentication

use crate::crypto::hmac::hmac_sha256_base64;
use crate::exchange::auth::{ExchangeAuth, SignedRequest, timestamp_iso8601};
use anyhow::Result;
use std::collections::HashMap;

const BASE_URL: &str = "https://www.okx.com";

/// OKX credentials.
///
/// # Examples
///
/// ```
/// let creds = OkxCredentials::new("api_key", "api_secret", "passphrase");
/// ```
#[derive(Clone)]
pub struct OkxCredentials {
    /// API key.
    pub api_key: String,
    /// API secret.
    pub api_secret: String,
    /// Passphrase.
    pub passphrase: String,
}

/// OKX authentication.
///
/// # Examples
///
/// ```
/// let creds = OkxCredentials::new("api_key", "api_secret", "passphrase");
/// ```
impl OkxCredentials {
    /// Creates a new [`OkxCredentials`] instance.
    ///
    /// # Arguments
    ///
    /// * `api_key` - API key.
    /// * `api_secret` - API secret.
    /// * `passphrase` - Passphrase.
    ///
    /// # Returns
    ///
    /// A new [`OkxCredentials`] instance.
    pub fn new(
        // API key.
        api_key: impl Into<String>,
        // API secret.
        api_secret: impl Into<String>,
        // Passphrase.
        passphrase: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            api_secret: api_secret.into(),
            passphrase: passphrase.into(),
        }
    }

    /// Creates a new [`OkxCredentials`] instance from environment variables.
    ///
    /// # Returns
    ///
    /// A new [`OkxCredentials`] instance.
    ///
    /// # Panics
    ///
    /// Panics if the environment variables are not set.
    ///
    /// # Environment Variables
    ///
    /// * `OKX_API_KEY` - API key.
    /// * `OKX_API_SECRET` - API secret.
    /// * `OKX_PASSPHRASE` - Passphrase.
    ///
    /// # Default Values
    ///
    /// * `DRY_RUN_KEY` - API key.
    /// * `dry-run-secret` - API secret.
    /// * `dry-run-passphrase` - Passphrase.
    pub fn from_env() -> Self {
        // Creates a new [`OkxCredentials`] instance from environment variables.
        //
        // # Panics
        //
        // Panics if the environment variables are not set.
        Self::new(
            std::env::var("OKX_API_KEY").unwrap_or_else(|_| "DRY_RUN_KEY".into()),
            std::env::var("OKX_API_SECRET").unwrap_or_else(|_| "dry-run-secret".into()),
            std::env::var("OKX_PASSPHRASE").unwrap_or_else(|_| "dry-run-passphrase".into()),
        )
    }
}

/// Authenticates with the OKX API using the provided credentials.
///
/// # Example
///
/// ```
/// use polar_bear_hft_crypto::exchange::okx::OkxAuth;
///
/// let auth = OkxAuth::new(OkxCredentials::from_env());
/// ```
pub struct OkxAuth {
    /// The credentials used to authenticate with the OKX API.
    creds: OkxCredentials,
}

/// Authenticates with the OKX API using the provided credentials.
///
/// # Example
///
/// ```
/// use polar_bear_hft_crypto::exchange::okx::OkxAuth;
///
/// let auth = OkxAuth::new(OkxCredentials::from_env());
/// ```
impl OkxAuth {
    /// Creates a new `OkxAuth` instance with the provided credentials.
    ///
    /// # Arguments
    ///
    /// * `creds` - The credentials used to authenticate with the OKX API.
    ///
    /// # Returns
    ///
    /// A new `OkxAuth` instance.
    pub fn new(creds: OkxCredentials) -> Self {
        Self { creds }
    }

    fn sign(
        &self,
        timestamp: &str,
        method: &str,
        request_path: &str,
        body: &str,
    ) -> Result<String> {
        let prehash = format!("{timestamp}{method}{request_path}{body}");
        hmac_sha256_base64(self.creds.api_secret.as_bytes(), prehash.as_bytes())
    }

    fn build_auth_headers(
        &self,
        method: &str,
        path: &str,
        body: &str,
    ) -> Result<(HashMap<String, String>, String)> {
        let timestamp = timestamp_iso8601();
        let sig = self.sign(&timestamp, method, path, body)?;

        let mut headers = HashMap::new();
        headers.insert("OK-ACCESS-KEY".to_string(), self.creds.api_key.clone());
        headers.insert("OK-ACCESS-SIGN".to_string(), sig);
        headers.insert("OK-ACCESS-TIMESTAMP".to_string(), timestamp.clone());
        headers.insert(
            "OK-ACCESS-PASSPHRASE".to_string(),
            self.creds.passphrase.clone(),
        );
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        Ok((headers, timestamp))
    }
}

impl ExchangeAuth for OkxAuth {
    fn exchange_name(&self) -> &'static str {
        "OKX"
    }

    fn sign_order(
        &self,
        symbol: &str,
        side: &str,
        quantity: f64,
        price: Option<f64>,
    ) -> Result<SignedRequest> {
        let path = "/api/v5/trade/order";
        let order_type = if price.is_some() { "limit" } else { "market" };

        let mut body_map = serde_json::json!({
            "instId": symbol,
            "tdMode": "cash",
            "side": side.to_lowercase(),
            "ordType": order_type,
            "sz": format!("{quantity:.8}"),
        });
        if let Some(p) = price {
            body_map["px"] = serde_json::json!(format!("{p:.2}"));
        }
        let body_str = body_map.to_string();

        let (headers, _) = self.build_auth_headers("POST", path, &body_str)?;

        Ok(SignedRequest {
            method: "POST".to_string(),
            url: format!("{BASE_URL}{path}"),
            headers,
            body: Some(body_str),
            exchange: self.exchange_name().to_string(),
            description: format!("{side} {quantity} {symbol}"),
        })
    }

    fn sign_balance_query(&self) -> Result<SignedRequest> {
        let path = "/api/v5/account/balance";
        let (headers, _) = self.build_auth_headers("GET", path, "")?;

        Ok(SignedRequest {
            method: "GET".to_string(),
            url: format!("{BASE_URL}{path}"),
            headers,
            body: None,
            exchange: self.exchange_name().to_string(),
            description: "account balance query".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dry_run() -> OkxAuth {
        OkxAuth::new(OkxCredentials::new("key", "secret", "passphrase"))
    }

    #[test]
    fn sign_order_has_required_headers() {
        let auth = dry_run();
        let req = auth
            .sign_order("BTC-USDT", "buy", 0.001, Some(65000.0))
            .unwrap();
        assert!(req.headers.contains_key("OK-ACCESS-KEY"));
        assert!(req.headers.contains_key("OK-ACCESS-SIGN"));
        assert!(req.headers.contains_key("OK-ACCESS-TIMESTAMP"));
        assert!(req.headers.contains_key("OK-ACCESS-PASSPHRASE"));
    }

    #[test]
    fn signature_is_base64() {
        let auth = dry_run();
        let req = auth.sign_balance_query().unwrap();
        let sig = req.headers.get("OK-ACCESS-SIGN").unwrap();
        // base64 of 32 bytes = 44 chars
        assert_eq!(sig.len(), 44);
    }
}
