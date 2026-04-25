# Error Handling

Every non-2xx response and every transport failure returns a `cryptohopper::Error`. Same idea as the Node/Python/Go/Ruby/PHP/Dart SDKs but laid out idiomatically as a struct with public fields, plus a `code` enum that pattern-matches cleanly.

```rust
pub struct Error {
    pub code: ErrorCode,
    pub status: u16,           // 0 for transport-level failures
    pub message: String,
    pub server_code: Option<i64>,    // numeric `code` from envelope
    pub ip_address: Option<String>,  // server-reported caller IP
    pub retry_after_ms: Option<u64>, // parsed Retry-After (only on 429)
}
```

`Error` implements `std::error::Error` + `Display` + `Debug`, so it works with `?`, `anyhow`, `thiserror`, `tracing`, and any logger.

## ErrorCode variants

```rust
pub enum ErrorCode {
    ValidationError,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    RateLimited,
    ServerError,
    ServiceUnavailable,
    DeviceUnauthorized,
    NetworkError,
    Timeout,
    Unknown,
    Other(String),  // server returned something the SDK didn't recognise
}
```

| Variant | HTTP | When you'll see it | Recover by |
|---|---|---|---|
| `ValidationError` | 400, 422 | Missing or malformed parameter | Fix the request |
| `Unauthorized` | 401 | Token missing, wrong, or revoked | Re-auth |
| `DeviceUnauthorized` | 402 | Internal Cryptohopper device-auth flow rejected you | Shouldn't happen via the public API; contact support |
| `Forbidden` | 403 | Scope missing, or IP not allowlisted | Check `err.ip_address`; add to allowlist or grant the scope |
| `NotFound` | 404 | Resource or endpoint doesn't exist | Check the ID; check you're using the latest SDK |
| `Conflict` | 409 | Resource is in a conflicting state | Cancel the existing job or wait |
| `RateLimited` | 429 | Bucket exhausted | The SDK auto-retries; see [Rate Limits](Rate-Limits.md) |
| `ServerError` | 500–502, 504 | Cryptohopper's end | Retry with back-off |
| `ServiceUnavailable` | 503 | Planned maintenance or downstream outage | Respect `retry_after_ms`; retry |
| `NetworkError` | — | DNS failure, TCP reset, TLS handshake failure | Retry; check your network |
| `Timeout` | — | Hit the per-request `timeout` | Retry; bump timeout if legitimately slow |
| `Unknown` | any | The SDK's status-mapping fall-through (rare; used for unmapped 4xx) | Inspect `err.status` and `err.message` |
| `Other(String)` | any | Server returned a code string the SDK doesn't know | Pass through; future-proof your handler |

These are stable across SDK versions. New server-side codes pass through as `Other(String)` rather than breaking your `match`.

## Exhaustive matching with `Other`

Since `Other(String)` is non-exhaustive at compile time, clippy won't warn on missing arms. Always include a catchall:

```rust
use cryptohopper::ErrorCode;

fn classify(code: &ErrorCode) -> &'static str {
    match code {
        ErrorCode::Unauthorized
        | ErrorCode::Forbidden
        | ErrorCode::DeviceUnauthorized => "auth",
        ErrorCode::ValidationError => "bad-request",
        ErrorCode::NotFound => "not-found",
        ErrorCode::Conflict => "conflict",
        ErrorCode::RateLimited => "throttled",
        ErrorCode::ServerError | ErrorCode::ServiceUnavailable => "server",
        ErrorCode::NetworkError | ErrorCode::Timeout => "transient",
        ErrorCode::Unknown | ErrorCode::Other(_) => "unknown",
    }
}
```

The exhaustiveness check from the compiler covers everything except `Other(String)` content — and you usually want `Other(_)` to fall into a generic "unknown" bucket anyway.

## Working with `?` and `anyhow`

The SDK error implements `std::error::Error`, so `?` chains naturally:

```rust
use anyhow::Result;

async fn run(client: &cryptohopper::Client) -> Result<()> {
    let me = client.user.get().await?;
    let hoppers = client.hoppers.list().await?;
    println!("{} hopper(s) for {}", hoppers.as_array().map(|a| a.len()).unwrap_or(0), me["email"]);
    Ok(())
}
```

For library code, embed the SDK error in a `thiserror`-derived enum:

```rust
#[derive(thiserror::Error, Debug)]
pub enum MyError {
    #[error(transparent)]
    Cryptohopper(#[from] cryptohopper::Error),

    #[error("missing field {field}")]
    MissingField { field: &'static str },
}
```

`#[from]` converts on `?`; `transparent` reuses the SDK error's Display.

## A robust retry wrapper

```rust
use cryptohopper::{Error, ErrorCode};
use std::future::Future;
use std::time::Duration;

pub async fn with_retry<T, F, Fut>(
    mut f: F,
    max_attempts: u32,
    base_ms: u64,
) -> Result<T, Error>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, Error>>,
{
    let transient = |c: &ErrorCode| matches!(
        c,
        ErrorCode::ServerError | ErrorCode::ServiceUnavailable
            | ErrorCode::NetworkError | ErrorCode::Timeout
    );

    for attempt in 1..=max_attempts {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) if !transient(&e.code) || attempt == max_attempts => return Err(e),
            Err(e) => {
                let wait = e
                    .retry_after_ms
                    .unwrap_or(base_ms * 2u64.pow(attempt - 1));
                tokio::time::sleep(Duration::from_millis(wait)).await;
            }
        }
    }
    unreachable!()
}
```

Don't include `RateLimited` in `transient` — the SDK already retries 429s internally. Wrapping it here would multiply attempts unhelpfully.

## Logging

`Error` implements `Display`, which renders compactly:

```
cryptohopper: [FORBIDDEN 403] IP not in allowlist
```

For `tracing` structured logs, pull individual fields:

```rust
match client.hoppers.list().await {
    Ok(_) => {}
    Err(err) => tracing::error!(
        code = %err.code,           // uses the Display impl on ErrorCode
        status = err.status,
        server_code = ?err.server_code,
        ip = ?err.ip_address,
        retry_after_ms = ?err.retry_after_ms,
        message = %err.message,
        "cryptohopper request failed"
    ),
}
```

`%` (Display) on `code` and `message` keeps the log line readable; `?` (Debug) on the optional fields renders `Some(...)` / `None` clearly.

## Extracting the server code for diagnostics

If you're talking to Cryptohopper support about a flaky endpoint, the `server_code` is the most useful single field:

```rust
if let Err(err) = client.hoppers.list().await {
    if let Some(server_code) = err.server_code {
        tracing::warn!("contact support with server_code={server_code}");
    }
}
```

The SDK doesn't interpret `server_code` itself; it's an opaque numeric diagnostic the server includes in the JSON envelope. Cryptohopper support can map it to the specific failure on their end.
