//! `client.exchange` — public market data.

use std::sync::Arc;

use reqwest::Method;
use serde_json::Value;

use crate::client::Transport;
use crate::error::Error;

#[derive(Clone, Debug)]
pub struct Exchange {
    transport: Arc<Transport>,
}

impl Exchange {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    /// `params` should be a JSON object (e.g. `json!({"exchange":"binance","market":"BTC/USDT"})`).
    pub async fn ticker(&self, params: &Value) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/exchange/ticker", Some(params), None)
            .await
    }

    /// OHLCV candles. `params` must include `exchange`, `market`, `timeframe`.
    pub async fn candles(&self, params: &Value) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/exchange/candle", Some(params), None)
            .await
    }

    pub async fn orderbook(&self, params: &Value) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/exchange/orderbook", Some(params), None)
            .await
    }

    pub async fn markets(&self, exchange: &str) -> Result<Value, Error> {
        let q = [("exchange", exchange.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/exchange/markets", Some(&q[..]), None)
            .await
    }

    pub async fn currencies(&self, exchange: &str) -> Result<Value, Error> {
        let q = [("exchange", exchange.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/exchange/currencies", Some(&q[..]), None)
            .await
    }

    pub async fn exchanges(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/exchange/exchanges", None, None)
            .await
    }

    pub async fn forex_rates(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/exchange/forex-rates", None, None)
            .await
    }
}
