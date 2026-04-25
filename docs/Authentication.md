# Authentication

Every SDK request (except a handful of public endpoints) requires an OAuth2 bearer token:

```
Authorization: Bearer <40-char token>
```

## Obtaining a token

1. Log in to [cryptohopper.com](https://www.cryptohopper.com).
2. **Developer → Create App** — gives you a `client_id` + `client_secret`.
3. Complete the OAuth consent flow for your app, which returns a bearer token.

Options to automate step 3:

- **The official CLI**: `cryptohopper login` opens the consent page, runs a loopback listener, and persists the token to `~/.cryptohopper/config.json`. Read it from your Rust binary.
- **Your own code**: call the server's `/oauth2/authorize` + `/oauth2/token` endpoints directly. The CLI's implementation is short (~300 lines of TypeScript) and a reasonable reference.

## Client construction

```rust
use std::time::Duration;
use cryptohopper::Client;

let client = Client::builder()
    .api_key(std::env::var("CRYPTOHOPPER_TOKEN")?)
    .app_key(std::env::var("CRYPTOHOPPER_APP_KEY").unwrap_or_default())
    .base_url("https://api.cryptohopper.com/v1")
    .timeout(Duration::from_secs(30))
    .max_retries(3)
    .user_agent("my-app/1.0")
    .build()?;
```

`Client::new(api_key)` is a shortcut for the most common case (defaults for everything else). Use the builder when you need to set anything beyond the token.

### `.app_key(...)`

Cryptohopper lets OAuth apps identify themselves on every request via the `x-api-app-key` header (value = your OAuth `client_id`). When set, the SDK adds the header automatically. Reasons to set it:

- Shows up in Cryptohopper's server-side telemetry — you can attribute your own traffic.
- Drives per-app rate limits — if two apps share a token, they get independent quotas.
- Harmless to omit; the server accepts unattributed requests.

Empty strings are treated as "not set," so passing `unwrap_or_default()` from a missing env var is safe.

### `.base_url(...)`

Override for staging or a local dev server. The default is `https://api.cryptohopper.com/v1`. The trailing `/v1` is part of the base; resource paths are relative to it.

### `.http_client(reqwest::Client)`

Bring your own `reqwest::Client` for proxies, custom CA bundles, connection-pool tuning, or `tower`-based middleware:

```rust
let custom = reqwest::Client::builder()
    .proxy(reqwest::Proxy::http("http://corporate-proxy.internal:3128")?)
    .danger_accept_invalid_certs(false)  // keep TLS verification ON; supply a custom root if needed
    .timeout(Duration::from_secs(30))
    .build()?;

let client = Client::builder()
    .api_key(token)
    .http_client(custom)
    .build()?;
```

When you supply your own `reqwest::Client`, the `.timeout(...)` builder option on the Cryptohopper client is overridden — your `reqwest::Client` controls the per-request connect/read/write timeout. The body-read timeout the SDK applies on top of `resp.text()` still uses the value passed via `.timeout(...)` on the builder; pair the two so the body-read timeout isn't tighter than your reqwest timeout.

For **rustls vs OpenSSL**: the SDK's default-built reqwest client uses rustls + webpki-roots (mozilla CA bundle). To use your OS cert store, supply your own client built with `rustls-native-certs`:

```rust
let custom = reqwest::Client::builder()
    // ... build with rustls-native-certs feature enabled ...
    .build()?;
```

### `.timeout(...)` and `.max_retries(...)`

`.timeout(Duration)` — per-request total timeout. Applied to the connect + headers phase by reqwest, AND to the body read by the SDK (a wrapper landed in iter 12 to close that gap). Defaults to 30 seconds.

`.max_retries(u32)` — automatic retries on HTTP 429. Default 3. Set to 0 to disable. See [Rate Limits](Rate-Limits.md) for details.

## IP allowlisting

If your Cryptohopper app has IP allowlisting enabled, requests from unlisted IPs return `403 FORBIDDEN`. The SDK surfaces this as `cryptohopper::Error` with `code == ErrorCode::Forbidden` and `ip_address` populated:

```rust
use cryptohopper::ErrorCode;

if let Err(err) = client.hoppers.list().await {
    if matches!(err.code, ErrorCode::Forbidden) {
        eprintln!("blocked: caller IP was {:?}", err.ip_address);
    }
}
```

For CI where the runner IP isn't stable, either disable IP allowlisting for that app or route outbound traffic through a stable IP (NAT gateway, VPN, dedicated proxy).

## Rotating tokens

Cryptohopper bearer tokens are long-lived but can be revoked:

- Manually from the dashboard.
- When the user revokes consent.

The SDK surfaces revocation as `Unauthorized` on the next call. There is no automatic refresh-token handling in the SDK today — if your app uses refresh tokens, handle the `Unauthorized` branch by exchanging your refresh token for a new access token and constructing a fresh client. Use a `tokio::sync::RwLock` (or `arc_swap::ArcSwap` for higher concurrency) to swap the client atomically:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use cryptohopper::{Client, ErrorCode};

#[derive(Clone)]
pub struct AutoRefresh {
    inner: Arc<RwLock<Client>>,
}

impl AutoRefresh {
    pub async fn call<T, F, Fut>(&self, f: F) -> Result<T, cryptohopper::Error>
    where
        F: Fn(Client) -> Fut,
        Fut: std::future::Future<Output = Result<T, cryptohopper::Error>>,
    {
        let snapshot = self.inner.read().await.clone();
        match f(snapshot).await {
            Ok(v) => Ok(v),
            Err(e) if matches!(e.code, ErrorCode::Unauthorized) => {
                let new_token = refresh_token().await?;
                let new_client = Client::new(new_token)?;
                {
                    let mut w = self.inner.write().await;
                    *w = new_client.clone();
                }
                f(new_client).await
            }
            Err(e) => Err(e),
        }
    }
}
```

`Client` is cheaply cloneable, so swapping is safe. In-flight requests on the old token complete with `Unauthorized` and trigger their own refresh on retry — there's no shared state to invalidate.

## Concurrency

`Client` is `Send + Sync + Clone`. One client serving many tokio tasks is fine. The underlying `reqwest::Client` has its own connection pool which is also thread-safe.

```rust
use futures::stream::{FuturesUnordered, StreamExt};

let mut futures = FuturesUnordered::new();
for id in hopper_ids {
    let c = client.clone();
    futures.push(async move { c.hoppers.get(&id).await });
}
while let Some(res) = futures.next().await {
    handle(res);
}
```

See [Rate Limits](Rate-Limits.md) for guidance on capping concurrency.

## Public-only access (no token)

A handful of endpoints accept anonymous calls:

- `/market/*` — marketplace browse
- `/platform/*` — i18n, country list, blog feed
- `/exchange/ticker`, `/exchange/candle`, `/exchange/orderbook`, `/exchange/markets`, `/exchange/exchanges`, `/exchange/forex-rates` — public market data

The SDK still requires a non-empty `api_key` at construction; pass any placeholder if you only intend to hit public endpoints. The server ignores the bearer header on whitelisted routes.

```rust
let client = Client::new("anonymous")?;
let btc = client.exchange.ticker("binance", "BTC/USDT").await?;
```
