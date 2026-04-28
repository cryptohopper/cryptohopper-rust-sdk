# Changelog

All notable changes to the `cryptohopper` crate are documented in this file.

## 0.1.0-alpha.2 — Unreleased

### Fixed
- **Critical: every authenticated request was rejected by the API gateway.** The transport sent `Authorization: Bearer <token>`, which the AWS API Gateway in front of `api.cryptohopper.com/v1/*` rejects (`405 Missing Authentication Token`). Cryptohopper's Public API v1 uses `access-token: <token>` — confirmed by the official [API documentation](https://www.cryptohopper.com/api-documentation/how-the-api-works) and the legacy iOS/Android SDKs. Switching to send `access-token`. The `Authorization` header is no longer set.

### Compatibility
No public-API change. Resource methods keep their signatures.

## 0.1.0-alpha.1 — 2026-04-24

Initial release. Full coverage of all 18 public API domains from day one.

### Transport
- `cryptohopper::Client` with `Client::new(api_key)` shorthand and a full `Client::builder()` for advanced config.
- Async-first on top of `reqwest` 0.12 + `rustls-tls` (no OpenSSL dep).
- `cryptohopper::Error` with a typed `ErrorCode` enum plus `ErrorCode::Other(String)` for forward-compatibility with unknown server codes.
- Automatic retry on HTTP 429 honouring `Retry-After` (default `max_retries: 3`, disableable).

### Resources
- **Core** — `user`, `hoppers`, `exchange`, `strategy`, `backtest`, `market`
- **A1** — `signals`, `arbitrage`, `marketmaker`, `template`
- **A2** — `ai`, `platform`, `chart`, `subscription`
- **A3** — `social` (27 methods), `tournaments`, `webhooks`, `app`

### Publishing
- Released to crates.io via `cargo publish` using `CARGO_REGISTRY_TOKEN` repo secret (trusted publishing for crates.io is still in beta).
- Tag prefix: `rs-v*`.
