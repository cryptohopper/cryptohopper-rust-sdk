# Getting Started

## Install

```bash
cargo add cryptohopper
```

Or in your `Cargo.toml`:

```toml
[dependencies]
cryptohopper = "0.1.0-alpha.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

Requires Rust 1.74 or newer (for `async fn` in traits, used internally) and a `tokio` runtime.

## First call

```rust
use cryptohopper::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = std::env::var("CRYPTOHOPPER_TOKEN")?;
    let client = Client::new(token)?;

    let me = client.user.get().await?;
    println!("Logged in as: {}", me["email"]);

    let ticker = client
        .exchange
        .ticker("binance", "BTC/USDT")
        .await?;
    println!("BTC/USDT: {}", ticker["last"]);
    Ok(())
}
```

`Client` is cheaply cloneable (`Arc` internally), so you can pass it to spawned tasks without wrapping it in another `Arc`.

## Getting a token

Every request (except a handful of public endpoints like `/exchange/ticker`) needs an OAuth2 bearer token. Create one via **Developer → Create App** on [cryptohopper.com](https://www.cryptohopper.com) and complete the consent flow. The token is a 40-character opaque string.

For local dev:

```bash
export CRYPTOHOPPER_TOKEN=<your-token>
```

For production, load from your secret store at startup. `dotenv` works for dev, but in CI use the runner's secret-injection mechanism.

## Idiomatic patterns

### Pattern matching on `ErrorCode`

The SDK returns a typed `cryptohopper::Error` whose `code` field is a `cryptohopper::ErrorCode` enum. Match on it directly — no string-comparison gymnastics:

```rust
use cryptohopper::{Client, ErrorCode};

match client.hoppers.get("999999").await {
    Ok(hopper) => println!("got: {hopper:?}"),
    Err(err) => match err.code {
        ErrorCode::NotFound => {
            // Expected; ignore.
        }
        ErrorCode::Unauthorized => {
            refresh_token().await?;
            // retry
        }
        ErrorCode::RateLimited => {
            // SDK already retried; back off harder
            if let Some(ms) = err.retry_after_ms {
                tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
            }
        }
        ErrorCode::Forbidden => {
            eprintln!("blocked from IP: {:?}", err.ip_address);
        }
        ErrorCode::Other(ref code) => {
            // Server returned a code the SDK doesn't know about yet —
            // pass through cleanly.
            tracing::warn!("unknown cryptohopper code: {code}");
        }
        _ => return Err(err.into()),
    },
}
```

The `Other(String)` variant catches any new server-side codes the SDK predates. You don't need to bump SDK versions to handle new error types.

### `?` propagation with `anyhow` or thiserror

The SDK error implements `std::error::Error`, so `?` works naturally with `anyhow::Result` or any custom error type built with `thiserror`:

```rust
use anyhow::Result;

async fn list_open_positions(client: &cryptohopper::Client, hopper_id: &str) -> Result<Vec<serde_json::Value>> {
    let raw = client.hoppers.positions(hopper_id).await?;
    Ok(raw.as_array().cloned().unwrap_or_default())
}
```

For library code, define your own error type:

```rust
#[derive(thiserror::Error, Debug)]
pub enum MyError {
    #[error("cryptohopper: {0}")]
    Cryptohopper(#[from] cryptohopper::Error),

    #[error("missing field: {0}")]
    MissingField(&'static str),
}
```

`#[from]` auto-converts the SDK error.

### Customising the client

```rust
use std::time::Duration;
use cryptohopper::Client;

let client = Client::builder()
    .api_key(std::env::var("CRYPTOHOPPER_TOKEN")?)
    .app_key(std::env::var("CRYPTOHOPPER_APP_KEY").unwrap_or_default())
    .base_url("https://api.staging.cryptohopper.com/v1")
    .timeout(Duration::from_secs(60))
    .max_retries(5)
    .user_agent("my-app/1.0")
    .build()?;
```

The builder is the full-control entry point. `Client::new(api_key)` is a shortcut for `Client::builder().api_key(...).build()` when defaults are fine.

### Async + concurrency

`Client` is `Send + Sync + Clone`. Spawn it across tasks:

```rust
use futures::stream::{FuturesUnordered, StreamExt};

let mut tasks = FuturesUnordered::new();
for id in hopper_ids {
    let client = client.clone();
    tasks.push(tokio::spawn(async move {
        client.hoppers.get(&id).await
    }));
}

while let Some(res) = tasks.next().await {
    match res? {
        Ok(hopper) => process(hopper),
        Err(err) => tracing::warn!("hopper fetch failed: {err}"),
    }
}
```

See [Rate Limits](Rate-Limits.md) for guidance on capping concurrency at the API quota.

## Common pitfalls

**`Error: Cryptohopper(Error { code: ValidationError, ... })`** with `message: "api_key must not be empty"` — you passed an empty string. Most often: `std::env::var("CRYPTOHOPPER_TOKEN").unwrap_or_default()` returns `""` when unset. Use `?` to fail loudly:

```rust
let token = std::env::var("CRYPTOHOPPER_TOKEN")
    .map_err(|_| anyhow::anyhow!("CRYPTOHOPPER_TOKEN is not set"))?;
```

**`code: Unauthorized` on every call** — token is wrong, expired, or revoked. Visit the app page in the Cryptohopper dashboard to confirm.

**`code: Forbidden` on endpoints that used to work** — IP allowlisting on the OAuth app blocked your current IP. The error includes `ip_address`:

```rust
if let Err(err) = client.hoppers.list().await {
    if matches!(err.code, ErrorCode::Forbidden) {
        eprintln!("blocked from {:?}", err.ip_address);
    }
}
```

**`error[E0277]: ?` operator can only be applied to values that implement `Try`** — usually means you're inside a function that returns something other than `Result`. Either change the function signature to `Result<T, E>` or pattern-match on the SDK call's return.

**Hangs in tests** — your tests don't have a tokio runtime. Use `#[tokio::test]` instead of `#[test]`:

```rust
#[tokio::test]
async fn it_lists_hoppers() {
    let client = Client::new("test-token").unwrap();
    // ...
}
```

## Type signatures

Response shapes are returned as `serde_json::Value` because the Cryptohopper API hasn't been frozen into stable models yet. To layer typed parsing on top, use `serde::Deserialize`:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Hopper {
    id: u64,
    name: String,
    exchange: String,
    enabled: bool,
}

let raw = client.hoppers.get(&id).await?;
let hopper: Hopper = serde_json::from_value(raw)?;
```

Future SDK versions may ship typed response structs as a feature flag — file an issue if you'd benefit.

## Next steps

- [Authentication](Authentication.md) — bearer flow, app keys, IP whitelisting, custom HTTP clients
- [Error Handling](Error-Handling.md) — every variant, pattern-match recipes, retry wrappers
- [Rate Limits](Rate-Limits.md) — auto-retry, customising back-off, concurrency caps
