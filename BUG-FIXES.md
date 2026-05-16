# Bug Fixes

---

## Fix 1 - `Arc<anthropic::Client>` in `HftAgent::new()`

**File**: `src/agent/hft_agent.rs`

**Root Cause**: The original code wrapped `anthropic::Client::from_env()` in `Arc::new(...)`,
producing `Arc<anthropic::Client>`. The `.agent()` method is defined on the `CompletionClient`
trait, which is implemented by `anthropic::Client` directly - not by `Arc<anthropic::Client>`.
Rust's method resolution cannot find `.agent()` on the `Arc` wrapper, yielding E0599.

Additionally, `Arc` was unnecessary: the `client` field was consumed by `.agent().preamble().build()`
in the same method call and never shared across tasks.

**Fix**: Remove `Arc::new(...)`, remove `use std::sync::Arc`, and store `client: anthropic::Client`
directly.

```rust
// Before (broken)
use std::sync::Arc;
pub struct HftAgent { client: Arc<anthropic::Client> }
let client = Arc::new(anthropic::Client::from_env());

// After (correct)
pub struct HftAgent { client: anthropic::Client }
let client = anthropic::Client::from_env();
```

---

## Fix 2 - Missing `CompletionClient` + `ProviderClient` trait imports

**File**: `src/agent/hft_agent.rs`

**Root Cause**: In rig-core ≥ 0.36, `.agent()` is a method on the `CompletionClient` trait,
not an inherent method on `anthropic::Client`. Without bringing `CompletionClient` and
`ProviderClient` into scope via `use`, the compiler cannot resolve the method call even though
`Client<AnthropicExt>` implements both traits.

**Fix**: Add both traits to the `use rig_core::{...}` import:

```rust
use rig_core::{
    client::{CompletionClient, ProviderClient},
    completion::Prompt,
    providers::anthropic,
};
```

This matches the canonical import in all official rig 0.36+ examples and documentation.

---

## Fix 3 - Exact `=` version pins for `hmac`, `sha2`, `ed25519-dalek`, `clap`

**File**: `Cargo.toml`

**Root Cause**: Exact pins (`= "=2.1.1"`, `"=0.12.1"`, `"=0.10.9"`, `"=4.4.18"`) prevent
Cargo from selecting any patch or minor release to resolve transitive dependency conflicts.
This is unnecessarily restrictive for a binary crate and causes unnecessary resolver failures.

**Fix**: Changed all exact pins to semver-compatible (`^`) ranges. These crates follow semver;
`^` ensures compatible updates are allowed while still preventing breaking changes.

```toml
# Before
ed25519-dalek = { version = "=2.1.1", ... }
hmac           = "=0.12.1"
sha2           = "=0.10.9"
clap           = { version = "=4.4.18", ... }

# After
ed25519-dalek = { version = "^2", ... }
hmac           = "^0.12"
sha2           = "^0.10"
clap           = { version = "^4", ... }
```

---

## Fix 4 - `thiserror` version `1.0` → `^2`

**File**: `Cargo.toml`

**Root Cause**: `thiserror` 2.x includes ergonomic improvements over 1.x. Pinning to `1.0`
prevented use of 2.x features available since 2024 and mismatched the rig upstream preference
for `^2`.

**Fix**: `thiserror = "^2"` - semver-compatible range covering all 2.x releases.

---

## Fix 5 - Missing `dotenvy` dependency

**File**: `Cargo.toml`

**Root Cause**: The project reads environment variables for API keys but had no standard
mechanism for loading a `.env` file in development. Adding `dotenvy` aligns with rig-hft
and allows `dotenvy::dotenv().ok()` to be called at startup.

**Fix**: Added `dotenvy = "^0.15"` to `[dependencies]`.
