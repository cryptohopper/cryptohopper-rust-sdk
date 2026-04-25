//! Client + transport layer.
//!
//! [`Client`] is constructed via [`Client::new`] (for the common case) or
//! [`Client::builder`] (for full configuration). It's cheaply cloneable —
//! internally it's an `Arc` around the transport plus the resource
//! namespaces. Resources share the same transport.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{header, Method, StatusCode};
use serde::Serialize;
use serde_json::Value;

use crate::error::{Error, ErrorCode};
use crate::resources::{
    ai::Ai, app::App, arbitrage::Arbitrage, backtest::Backtests, chart::Chart, exchange::Exchange,
    hoppers::Hoppers, market::Market, marketmaker::MarketMaker, platform::Platform,
    signals::Signals, social::Social, strategy::Strategies, subscription::Subscription,
    template::Templates, tournaments::Tournaments, user::User, webhooks::Webhooks,
};

/// Current SDK version, kept in lockstep with `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_BASE_URL: &str = "https://api.cryptohopper.com/v1";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_MAX_RETRIES: u32 = 3;

/// The public entry point. Clone freely — resources share transport via `Arc`.
#[derive(Clone, Debug)]
pub struct Client {
    // Held to keep the shared transport alive even when every resource is
    // dropped; also an escape hatch for future raw-request methods.
    #[allow(dead_code)]
    transport: Arc<Transport>,
    pub user: User,
    pub hoppers: Hoppers,
    pub exchange: Exchange,
    pub strategy: Strategies,
    pub backtest: Backtests,
    pub market: Market,
    pub signals: Signals,
    pub arbitrage: Arbitrage,
    pub marketmaker: MarketMaker,
    pub template: Templates,
    pub ai: Ai,
    pub platform: Platform,
    pub chart: Chart,
    pub subscription: Subscription,
    pub social: Social,
    pub tournaments: Tournaments,
    pub webhooks: Webhooks,
    pub app: App,
}

impl Client {
    /// Shortcut for `Client::builder().api_key(key).build()`.
    pub fn new(api_key: impl Into<String>) -> Result<Self, Error> {
        Client::builder().api_key(api_key).build()
    }

    /// Builder for full configuration.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }
}

/// Builder for [`Client`].
#[derive(Default, Debug)]
pub struct ClientBuilder {
    api_key: Option<String>,
    app_key: Option<String>,
    base_url: Option<String>,
    timeout: Option<Duration>,
    max_retries: Option<u32>,
    user_agent: Option<String>,
    http_client: Option<reqwest::Client>,
}

impl ClientBuilder {
    /// OAuth2 bearer token. Required.
    pub fn api_key(mut self, v: impl Into<String>) -> Self {
        self.api_key = Some(v.into());
        self
    }

    /// OAuth client_id, sent as the `x-api-app-key` header. Optional.
    pub fn app_key(mut self, v: impl Into<String>) -> Self {
        self.app_key = Some(v.into());
        self
    }

    /// Override the API base URL (e.g. a staging environment). Defaults
    /// to `https://api.cryptohopper.com/v1`.
    pub fn base_url(mut self, v: impl Into<String>) -> Self {
        self.base_url = Some(v.into());
        self
    }

    /// Per-request timeout. Defaults to 30 seconds.
    pub fn timeout(mut self, v: Duration) -> Self {
        self.timeout = Some(v);
        self
    }

    /// Retries on HTTP 429 (respecting `Retry-After`). Set to 0 to
    /// disable auto-retry. Defaults to 3.
    pub fn max_retries(mut self, v: u32) -> Self {
        self.max_retries = Some(v);
        self
    }

    /// Appended after `cryptohopper-sdk-rust/<version>` in the
    /// User-Agent header.
    pub fn user_agent(mut self, v: impl Into<String>) -> Self {
        self.user_agent = Some(v.into());
        self
    }

    /// Bring your own `reqwest::Client` (for proxies, custom timeouts,
    /// tests). Overrides `timeout`.
    pub fn http_client(mut self, v: reqwest::Client) -> Self {
        self.http_client = Some(v);
        self
    }

    /// Build the [`Client`]. Returns an error if `api_key` is missing.
    pub fn build(self) -> Result<Client, Error> {
        let api_key = self.api_key.ok_or_else(|| Error {
            code: ErrorCode::ValidationError,
            status: 0,
            message: "api_key is required".into(),
            server_code: None,
            ip_address: None,
            retry_after_ms: None,
        })?;

        if api_key.is_empty() {
            return Err(Error {
                code: ErrorCode::ValidationError,
                status: 0,
                message: "api_key must not be empty".into(),
                server_code: None,
                ip_address: None,
                retry_after_ms: None,
            });
        }

        let base_url = self
            .base_url
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string())
            .trim_end_matches('/')
            .to_string();

        let timeout = self.timeout.unwrap_or(DEFAULT_TIMEOUT);

        let http = match self.http_client {
            Some(c) => c,
            None => reqwest::Client::builder()
                .timeout(timeout)
                .build()
                .map_err(|e| Error::network(format!("failed to build HTTP client: {e}")))?,
        };

        let user_agent = match self.user_agent {
            Some(suffix) if !suffix.is_empty() => {
                format!("cryptohopper-sdk-rust/{VERSION} {suffix}")
            }
            _ => format!("cryptohopper-sdk-rust/{VERSION}"),
        };

        let transport = Arc::new(Transport {
            api_key,
            app_key: self.app_key,
            base_url,
            user_agent,
            max_retries: self.max_retries.unwrap_or(DEFAULT_MAX_RETRIES),
            timeout,
            http,
        });

        Ok(Client {
            user: User::new(transport.clone()),
            hoppers: Hoppers::new(transport.clone()),
            exchange: Exchange::new(transport.clone()),
            strategy: Strategies::new(transport.clone()),
            backtest: Backtests::new(transport.clone()),
            market: Market::new(transport.clone()),
            signals: Signals::new(transport.clone()),
            arbitrage: Arbitrage::new(transport.clone()),
            marketmaker: MarketMaker::new(transport.clone()),
            template: Templates::new(transport.clone()),
            ai: Ai::new(transport.clone()),
            platform: Platform::new(transport.clone()),
            chart: Chart::new(transport.clone()),
            subscription: Subscription::new(transport.clone()),
            social: Social::new(transport.clone()),
            tournaments: Tournaments::new(transport.clone()),
            webhooks: Webhooks::new(transport.clone()),
            app: App::new(transport.clone()),
            transport,
        })
    }
}

/// Internal transport shared across resources.
#[derive(Debug)]
pub(crate) struct Transport {
    api_key: String,
    app_key: Option<String>,
    base_url: String,
    user_agent: String,
    max_retries: u32,
    // Per-request total deadline. reqwest's `Client::timeout` covers the
    // connect + initial-response phase; the response body stream that
    // backs `resp.text()` is *not* covered. We re-apply the same total
    // timeout to the body read so a slow/stalled body can't hang the call.
    timeout: Duration,
    http: reqwest::Client,
}

impl Transport {
    /// Execute a request with auto-retry on 429.
    pub(crate) async fn request<Q, B>(
        &self,
        method: Method,
        path: &str,
        query: Option<&Q>,
        body: Option<&B>,
    ) -> Result<Value, Error>
    where
        Q: Serialize + ?Sized,
        B: Serialize + ?Sized,
    {
        let mut attempt = 0u32;
        loop {
            match self.do_request(&method, path, query, body).await {
                Ok(v) => return Ok(v),
                Err(e) if e.code == ErrorCode::RateLimited && attempt < self.max_retries => {
                    let wait = e
                        .retry_after_ms
                        .map(Duration::from_millis)
                        .unwrap_or_else(|| Duration::from_secs(1u64 << attempt.min(6)));
                    tokio::time::sleep(wait).await;
                    attempt += 1;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn do_request<Q, B>(
        &self,
        method: &Method,
        path: &str,
        query: Option<&Q>,
        body: Option<&B>,
    ) -> Result<Value, Error>
    where
        Q: Serialize + ?Sized,
        B: Serialize + ?Sized,
    {
        let mut url = self.base_url.clone();
        if !path.starts_with('/') {
            url.push('/');
        }
        url.push_str(path);

        let mut builder = self.http.request(method.clone(), &url);

        if let Some(q) = query {
            builder = builder.query(q);
        }

        builder = builder
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(header::ACCEPT, "application/json")
            .header(header::USER_AGENT, &self.user_agent);

        if let Some(app_key) = &self.app_key {
            builder = builder.header("x-api-app-key", app_key);
        }

        if let Some(b) = body {
            builder = builder.json(b);
        }

        let resp = match builder.send().await {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    return Err(Error::timeout(format!("request timed out: {e}")));
                }
                return Err(Error::network(format!(
                    "could not reach {}: {}",
                    self.base_url, e
                )));
            }
        };

        let status = resp.status();
        let retry_after = resp
            .headers()
            .get(header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(parse_retry_after);

        // Bound the body read by the configured per-request timeout —
        // reqwest's Client::timeout doesn't cover the body stream once
        // .send() has resolved, so a slow body would otherwise hang.
        let text = match tokio::time::timeout(self.timeout, resp.text()).await {
            Ok(Ok(t)) => t,
            Ok(Err(e)) => return Err(Error::network(format!("failed to read body: {e}"))),
            Err(_) => {
                return Err(Error::timeout(format!(
                    "response body read timed out after {}s",
                    self.timeout.as_secs()
                )))
            }
        };
        let parsed: Option<Value> = if text.is_empty() {
            None
        } else {
            serde_json::from_str(&text).ok()
        };

        if !status.is_success() {
            return Err(build_api_error(status, &parsed, retry_after));
        }

        if let Some(mut v) = parsed {
            if let Some(obj) = v.as_object_mut() {
                if let Some(data) = obj.remove("data") {
                    return Ok(data);
                }
            }
            return Ok(v);
        }
        Ok(Value::Null)
    }
}

fn build_api_error(
    status: StatusCode,
    parsed: &Option<Value>,
    retry_after_ms: Option<u64>,
) -> Error {
    let (message, server_code, ip_address) = match parsed.as_ref().and_then(|v| v.as_object()) {
        Some(obj) => {
            let msg = obj
                .get("message")
                .and_then(|m| m.as_str())
                .map(str::to_string)
                .unwrap_or_else(|| format!("Request failed ({})", status.as_u16()));
            let code = obj.get("code").and_then(|c| c.as_i64()).filter(|n| *n > 0);
            let ip = obj
                .get("ip_address")
                .and_then(|i| i.as_str())
                .map(str::to_string);
            (msg, code, ip)
        }
        None => (format!("Request failed ({})", status.as_u16()), None, None),
    };

    Error {
        code: ErrorCode::from_status(status.as_u16()),
        status: status.as_u16(),
        message,
        server_code,
        ip_address,
        retry_after_ms,
    }
}

fn parse_retry_after(header: &str) -> Option<u64> {
    if let Ok(secs) = header.parse::<f64>() {
        if secs < 0.0 {
            return None;
        }
        return Some((secs * 1000.0).round() as u64);
    }
    let when = httpdate::parse_http_date(header).ok()?;
    let now = std::time::SystemTime::now();
    when.duration_since(now).ok().map(|d| d.as_millis() as u64)
}
