# Rate Limits

Cryptohopper applies per-bucket rate limits on the server. When you hit one, you get a `429` with a `Retry-After` header. The SDK handles this for you.

## The default behaviour

On every `429`, the SDK:

1. Parses `Retry-After` (either seconds-as-integer or HTTP-date form) into milliseconds.
2. Sleeps that long via `tokio::time::sleep` (falling back to exponential back-off if the header is missing).
3. Retries the request.
4. Repeats up to `.max_retries(n)` (default 3).

If retries exhaust, the call returns `Err(Error)` with `code == ErrorCode::RateLimited` and `retry_after_ms` set to the last seen retry hint.

## Configuring it

```rust
use std::time::Duration;
use cryptohopper::Client;

let client = Client::builder()
    .api_key(token)
    .max_retries(10)
    .timeout(Duration::from_secs(60))  // bump if 10 retries push past 30s
    .build()?;
```

To **disable** retries entirely (e.g. you want to do your own back-off):

```rust
let client = Client::builder()
    .api_key(token)
    .max_retries(0)
    .build()?;
```

With `max_retries(0)` a 429 returns immediately as `RateLimited`. Inspect `err.retry_after_ms` and schedule the retry on your own timeline.

## Buckets

Cryptohopper has three named buckets:

| Bucket | Scope | Example endpoints |
|---|---|---|
| `normal` | Most reads + writes | `/user/get`, `/hopper/list`, `/hopper/update`, `/exchange/ticker` |
| `order` | Anything that places or modifies orders | `/hopper/buy`, `/hopper/sell`, `/hopper/panic` |
| `backtest` | The (expensive) backtest subsystem | `/backtest/new`, `/backtest/get` |

The SDK doesn't know which bucket a call hits — it only sees the 429. You don't need to either; the server tells you when you're limited.

## Backfill jobs (own back-off)

If you're ingesting historical data and need to fetch many pages, take ownership of the back-off:

```rust
use cryptohopper::{Client, ErrorCode};
use std::time::Duration;

let client = Client::builder()
    .api_key(token)
    .max_retries(0)
    .build()?;

for hopper_id in all_hopper_ids {
    loop {
        match client.hoppers.orders(&hopper_id, None).await {
            Ok(orders) => {
                process(orders);
                break;
            }
            Err(err) if matches!(err.code, ErrorCode::RateLimited) => {
                let wait_ms = err.retry_after_ms.unwrap_or(1_000);
                tokio::time::sleep(Duration::from_millis(wait_ms)).await;
            }
            Err(err) => return Err(err.into()),
        }
    }
}
```

This pattern lets a long-running job honour rate limits without stalling other work, because you decide the pacing.

## Concurrency caps with `tokio::sync::Semaphore`

`Client::clone()` is cheap (`Arc` internally), so you can pass it freely to spawned tasks. Cap concurrency with a semaphore so you don't trip the rate limit despite the SDK's per-call retries:

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;
use futures::stream::{FuturesUnordered, StreamExt};

const MAX_CONCURRENT: usize = 4;

let sem = Arc::new(Semaphore::new(MAX_CONCURRENT));
let mut tasks = FuturesUnordered::new();

for id in hopper_ids {
    let permit = sem.clone().acquire_owned().await?;
    let client = client.clone();
    tasks.push(tokio::spawn(async move {
        let _guard = permit;  // drops on task end, releasing the slot
        client.hoppers.get(&id).await
    }));
}

while let Some(res) = tasks.next().await {
    match res?? {
        hopper => handle(hopper),
    }
}
```

Empirically, **4–8 concurrent workers** is comfortable for most accounts. Higher is feasible with `.app_key(...)` set (which gives your OAuth app its own quota) but plan to back off explicitly.

## Avoiding the rate-limit thunder herd

If many tasks fail with `RateLimited` simultaneously, they each retry independently — and they each base their back-off on the same `Retry-After`. The result: a synchronised re-attempt that may retrip the bucket. To smooth this, add jitter:

```rust
use rand::Rng;

let base_ms = err.retry_after_ms.unwrap_or(1_000);
let jitter_ms = rand::thread_rng().gen_range(0..=base_ms / 4);
tokio::time::sleep(Duration::from_millis(base_ms + jitter_ms)).await;
```

The SDK's built-in retry doesn't add jitter (kept the algorithm simple + deterministic). If you have many concurrent tasks, do this in your own retry layer with `max_retries(0)`.

## What the SDK does NOT do

- **No global semaphore.** If you spawn 100 tasks each calling the SDK and the server rate-limits them, every task's retry is independent — you might get 100 simultaneous sleeps. Cap concurrency yourself.
- **No adaptive slow-down.** After a 429, the SDK waits and retries that one call. It doesn't throttle future calls.
- **No client-side bucket tracking.** The server is the source of truth.
- **No jitter** on retry sleeps. Add yourself if you have many concurrent tasks (see above).

## Diagnosing "always rate-limited"

If every request returns `Err(_)` with `RateLimited` even at low volume:

1. Check that your app hasn't been flagged for abuse in the Cryptohopper dashboard.
2. Confirm your retry logic doesn't accidentally retry on non-429 errors too — `matches!(err.code, ErrorCode::RateLimited)` is the canonical guard.
3. Inspect `err.server_code` — Cryptohopper sometimes includes a numeric detail there that clarifies which bucket you've tripped.
4. Confirm you're not sharing one token across many machines (one quota, divided across all). If you have multiple environments, give each a distinct token + `app_key` for clean attribution.

## Body-read timeout vs rate-limit retry

A subtle interaction: if a 429 retry's body read takes a long time (the server might send headers fast then trickle the error body), the per-request timeout (default 30s) applies to *each* attempt's body read individually. A pathological case where each retry's body read takes 25s could keep retries running for `max_retries × 30s = 90s` total before giving up.

If your callers have their own deadline (HTTP request handlers, tokio task with `Instant + timeout`), wrap the SDK call in `tokio::time::timeout(your_deadline, ...)` and the SDK's internal retry loop will be cancelled cleanly.

```rust
use tokio::time::{timeout, Duration};

match timeout(Duration::from_secs(10), client.hoppers.list()).await {
    Ok(Ok(hoppers)) => process(hoppers),
    Ok(Err(err)) => handle_sdk_error(err),
    Err(_) => log::warn!("hopper list deadline exceeded"),
}
```

The outer `timeout` aborts whatever phase the SDK is in — connect, headers, body read, or the back-off `sleep` between retries.
