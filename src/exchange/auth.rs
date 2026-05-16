//! Shared authentication trait and request types used across all exchange modules.

use anyhow::Result;
use std::collections::HashMap;

/// A signed HTTP request ready to be dispatched.
#[derive(Debug, Clone)]
pub struct SignedRequest {
    /// HTTP method (GET, POST, DELETE).
    pub method: String,
    /// Full URL including signed query string (where applicable).
    pub url: String,
    /// HTTP headers carrying authentication material.
    pub headers: HashMap<String, String>,
    /// Optional JSON body for POST requests.
    pub body: Option<String>,
    /// The exchange this request targets.
    pub exchange: String,
    /// Human-readable description of what this request does.
    pub description: String,
}

impl SignedRequest {
    /// Print a formatted summary (safe to log - no secret material).
    pub fn display(&self) {
        println!("┌─ {} ─ {} ──────────────", self.exchange, self.description);
        println!("│  Method : {}", self.method);
        println!("│  URL    : {}", self.url);
        for (k, v) in &self.headers {
            println!("│  Header : {k}: {v}");
        }
        if let Some(body) = &self.body {
            println!("│  Body   : {body}");
        }
        println!("└────────────────────────────────────────────────────────────");
    }
}

/// Credential set for HMAC-based exchanges.
#[derive(Debug, Clone)]
pub struct HmacCredentials {
    pub api_key: String,
    pub api_secret: String,
}

impl HmacCredentials {
    pub fn new(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            api_secret: api_secret.into(),
        }
    }

    /// Load from environment variables.
    ///
    /// # Example
    /// ```
    /// use polar_bear_hft_crypto::exchange::auth::HmacCredentials;
    /// std::env::set_var("BINANCE_API_KEY", "test-key");
    /// std::env::set_var("BINANCE_API_SECRET", "test-secret");
    /// let creds = HmacCredentials::from_env("BINANCE_API_KEY", "BINANCE_API_SECRET").unwrap();
    /// ```
    pub fn from_env(key_var: &str, secret_var: &str) -> Result<Self> {
        let api_key = std::env::var(key_var).unwrap_or_else(|_| format!("DRY_RUN_{key_var}"));
        let api_secret =
            std::env::var(secret_var).unwrap_or_else(|_| format!("dry-run-secret-{secret_var}"));
        Ok(Self::new(api_key, api_secret))
    }
}

/// Returns current UTC timestamp in milliseconds.
pub fn timestamp_ms() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}

/// Returns current UTC timestamp in seconds.
pub fn timestamp_s() -> u64 {
    chrono::Utc::now().timestamp() as u64
}

/// Returns ISO-8601 UTC timestamp string (OKX format).
pub fn timestamp_iso8601() -> String {
    chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string()
}

/// Trait that all exchange authenticators implement.
pub trait ExchangeAuth {
    /// Sign a spot market order request (dry-run by default).
    fn sign_order(
        &self,
        symbol: &str,
        side: &str,
        quantity: f64,
        price: Option<f64>,
    ) -> Result<SignedRequest>;

    /// Sign an account balance query.
    fn sign_balance_query(&self) -> Result<SignedRequest>;

    /// Name of the exchange.
    fn exchange_name(&self) -> &'static str;
}
