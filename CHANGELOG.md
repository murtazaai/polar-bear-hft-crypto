# Changelog

All notable changes to `polar-bear-hft-crypto` are documented here.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

---

## [0.2.0] - 2026-05-16

### Added
- `rustfmt.toml` - code-style rules (100 cols, Rust 2024 edition, crate-level imports)
- `.clippy.toml` - Clippy config with MSRV 1.93.1 and complexity thresholds
- `.env.example` - template for all 7 exchange API keys + Anthropic key
- `LICENSE-PBS` - Polar Bear Systems proprietary licence
- `CHANGELOG.md` - this file
- `CONTRIBUTING.md` - contribution guide with full workflow
- `FILE_STRUCTURE.md` - annotated repository map
- `BUG-FIXES.md` - root-cause analysis of resolved issues
- `docs/architecture.md` - system architecture deep-dive with ASCII diagram
- `docs/dsa_math.md` - detailed ECDSA and Ed25519 mathematical commentary
- `examples/crypto_demo.rs` - standalone ECDSA + Ed25519 + HMAC demo
- `examples/exchange_demo.rs` - 7-exchange signed request demo
- `examples/agent_demo.rs` - Rig AI agent demo (requires `ai-agent` feature)
- `tests/providers/anthropic.rs` - live Anthropic integration tests (`#[ignore]`)
- `.zed/tasks.json` / `debug.json` - Zed IDE task and debug config

### Changed
- `Cargo.toml` - upgraded to **Rust 2024 edition**; added `rust-version = "1.93.1"` (MSRV),
  `[package.metadata.docs.rs]`, and `[lints]` tables; relaxed `=` exact version pins to
  `^` (semver-compatible) for all deps; bumped `thiserror` to `^2`; added `dotenvy ^0.15`;
  added `[profile.release]` with LTO + single-codegen-unit (mirrors rig upstream)
- `.github/workflows/ci.yml` - added MSRV check step, `cargo doc` validation, `--workspace`
  flag on test, improved cache key, added `ai-agent` feature build step
- `.gitignore` - consolidated with focused, Rust-only ignore rules matching rig-hft

### Fixed
- **Fix 1** - `src/agent/hft_agent.rs`: removed `Arc<anthropic::Client>` wrapper.
  `Client::from_env()` in rig-core 0.37 returns a bare `Client`; wrapping in `Arc` produced
  `Arc<Client>` on which `.agent()` could not be resolved. Removed `use std::sync::Arc`.
- **Fix 2** - `src/agent/hft_agent.rs`: added `rig::client::{CompletionClient, ProviderClient}`
  to the `use` import. Both traits must be in scope for `.agent()` to resolve in rig-core ≥ 0.36.

---

## [0.1.0] - 2025-01-01

Initial release:

- ECDSA (secp256k1) forward-engineering from FIPS 186-5
- Ed25519 (Curve25519) forward-engineering from RFC 8032
- HMAC-SHA256 / HMAC-SHA512 implementations validated against NIST RFC 4231 §4.2 vectors
- 7-exchange REST API authentication: Binance, Kraken, OKX, Bybit, Coinbase, KuCoin, Hyperliquid
- `ExchangeAuth` trait for uniform sign/verify interface
- Rig (ARC) `HftAgent` integration behind `ai-agent` feature flag
- 52 passing unit + integration tests
- GitHub Actions CI: fmt → clippy → build → test
