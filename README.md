# cryptohopper

[![Crates.io](https://img.shields.io/crates/v/cryptohopper.svg)](https://crates.io/crates/cryptohopper)
[![docs.rs](https://docs.rs/cryptohopper/badge.svg)](https://docs.rs/cryptohopper)

Official Rust SDK for the [Cryptohopper](https://www.cryptohopper.com) API.

> **Status: 0.1.0-alpha.1** — full coverage of all 18 public API domains from day one. Matches `@cryptohopper/sdk`, `cryptohopper` (Python), `cryptohopper-go-sdk`, and the Ruby gem at v0.4.0.

## Install

```bash
cargo add cryptohopper
```

Or in `Cargo.toml`:

```toml
[dependencies]
cryptohopper = "0.1.0-alpha.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

Requires Rust 1.76+.

## Quickstart

```rust
use cryptohopper::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), cryptohopper::Error> {
    let ch = Client::new(std::env::var("CRYPTOHOPPER_TOKEN").unwrap())?;

    let me = ch.user.get().await?;
    println!("{}", me["email"]);

    let ticker = ch.exchange.ticker(&json!({
        "exchange": "binance",
        "market": "BTC/USDT",
    })).await?;
    println!("{}", ticker["last"]);

    Ok(())
}
```

## Authentication

Cryptohopper uses OAuth2 bearer tokens — see [cryptohopper.com → developer dashboard](https://www.cryptohopper.com) to register an OAuth app.

```rust
use cryptohopper::Client;

let ch = Client::builder()
    .api_key(std::env::var("CRYPTOHOPPER_TOKEN").unwrap())
    .app_key(std::env::var("CRYPTOHOPPER_CLIENT_ID").unwrap_or_default()) // optional
    .build()?;
```

## Resources

```rust
use serde_json::json;

// Read-only
ch.user.get().await?;
ch.hoppers.list(Some("binance")).await?;
ch.hoppers.get(42).await?;
ch.exchange.ticker(&json!({"exchange": "binance", "market": "BTC/USDT"})).await?;
ch.strategy.list().await?;
ch.backtest.limits().await?;
ch.market.homepage().await?;

// Write / trade
ch.hoppers.buy(&json!({
    "hopper_id": 42, "market": "BTC/USDT", "amount": 0.001
})).await?;
ch.hoppers.config_update(42, &json!({"strategy_id": 99})).await?;
ch.hoppers.panic(42).await?;

// A1 — signals / arbitrage / marketmaker / template
ch.signals.performance(None).await?;
ch.arbitrage.exchange_history(None).await?;
ch.marketmaker.get(Some(&json!({"hopper_id": 42}))).await?;
ch.template.load(3, 42).await?;  // apply template 3 to hopper 42

// A2 — ai / platform / chart / subscription
ch.ai.get_credits().await?;
ch.ai.llm_analyze(&json!({"strategy_id": 42})).await?;
ch.platform.bot_types().await?;
ch.subscription.plans().await?;

// A3 — social / tournaments / webhooks / app
ch.social.get_profile("pim").await?;
ch.social.create_post(&json!({"content": "New post"})).await?;
ch.tournaments.active().await?;
ch.webhooks.create(&json!({"url": "https://example.com/hook"})).await?;
```

## Client options

| Builder method | Default | Description |
|---|---|---|
| `api_key(...)` | — (required) | OAuth2 bearer token |
| `app_key(...)` | — | Optional OAuth `client_id`, sent as `x-api-app-key` |
| `base_url(...)` | `https://api.cryptohopper.com/v1` | Override for staging |
| `timeout(...)` | `30s` | Per-request timeout |
| `max_retries(...)` | `3` | Retries on HTTP 429 (respects `Retry-After`). `0` disables. |
| `user_agent(...)` | — | Appended after `cryptohopper-sdk-rust/<version>` |
| `http_client(...)` | — | Bring your own `reqwest::Client` |

## Errors

Every non-2xx response becomes a `cryptohopper::Error`:

```rust
use cryptohopper::{Client, ErrorCode};
use serde_json::json;

# async fn run(ch: Client) {
match ch.user.get().await {
    Ok(me) => println!("{}", me["email"]),
    Err(e) => match e.code {
        ErrorCode::Unauthorized => eprintln!("token expired or invalid"),
        ErrorCode::Forbidden => eprintln!("missing scope (ip={:?})", e.ip_address),
        ErrorCode::RateLimited => eprintln!("slow down, wait {:?}", e.retry_after_ms),
        _ => eprintln!("{}", e),
    },
}
# }
```

Known codes: `Unauthorized`, `Forbidden`, `NotFound`, `RateLimited`, `ValidationError`, `DeviceUnauthorized`, `Conflict`, `ServerError`, `ServiceUnavailable`, `NetworkError`, `Timeout`, `Unknown`. Unknown server-side codes pass through as `ErrorCode::Other(String)`.

## Rate limiting

On HTTP 429 the SDK retries with exponential backoff up to `max_retries` (default 3), honouring `Retry-After`. Pass `.max_retries(0)` to disable auto-retry.

## Development

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
cargo doc --no-deps --open
```

## Release

Push a `rs-v<version>` git tag. The release workflow runs fmt + clippy + tests, verifies tag-version parity, and publishes to crates.io via `CARGO_REGISTRY_TOKEN`.

## Related packages

| Language | Package | Install |
|---|---|---|
| Node.js | [`@cryptohopper/sdk`](https://www.npmjs.com/package/@cryptohopper/sdk) | `npm i @cryptohopper/sdk` |
| Python | [`cryptohopper`](https://pypi.org/project/cryptohopper/) | `pip install cryptohopper` |
| Ruby | [`cryptohopper`](https://rubygems.org/gems/cryptohopper) | `gem install cryptohopper --pre` |
| Go | `github.com/cryptohopper/cryptohopper-go-sdk` | `go get github.com/cryptohopper/cryptohopper-go-sdk` |
| CLI | [`cryptohopper-cli`](https://github.com/cryptohopper/cryptohopper-cli) | GitHub Releases binaries |

## License

MIT — see [LICENSE](./LICENSE).
