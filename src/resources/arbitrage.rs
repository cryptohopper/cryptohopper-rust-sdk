//! `client.arbitrage` — exchange + market arbitrage + shared backlog.

use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::Transport;
use crate::error::Error;

#[derive(Clone, Debug)]
pub struct Arbitrage {
    transport: Arc<Transport>,
}

impl Arbitrage {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    // ─── Cross-exchange arbitrage ─────────────────────────────────────

    pub async fn exchange_start(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/arbitrage/exchange", None, Some(data))
            .await
    }

    pub async fn exchange_cancel(&self, data: Option<&Value>) -> Result<Value, Error> {
        let empty = json!({});
        let body = data.unwrap_or(&empty);
        self.transport
            .request::<(), _>(Method::POST, "/arbitrage/cancel", None, Some(body))
            .await
    }

    pub async fn exchange_results(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/arbitrage/results", params, None)
            .await
    }

    pub async fn exchange_history(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/arbitrage/history", params, None)
            .await
    }

    pub async fn exchange_total(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/arbitrage/total", None, None)
            .await
    }

    pub async fn exchange_reset_total(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/arbitrage/resettotal",
                None,
                Some(&json!({})),
            )
            .await
    }

    // ─── Intra-exchange market arbitrage ──────────────────────────────

    pub async fn market_start(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/arbitrage/market", None, Some(data))
            .await
    }

    pub async fn market_cancel(&self, data: Option<&Value>) -> Result<Value, Error> {
        let empty = json!({});
        let body = data.unwrap_or(&empty);
        self.transport
            .request::<(), _>(Method::POST, "/arbitrage/market-cancel", None, Some(body))
            .await
    }

    pub async fn market_result(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/arbitrage/market-result", params, None)
            .await
    }

    pub async fn market_history(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/arbitrage/market-history", params, None)
            .await
    }

    // ─── Backlog (shared) ─────────────────────────────────────────────

    pub async fn backlogs(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/arbitrage/get-backlogs", params, None)
            .await
    }

    pub async fn backlog(&self, backlog_id: impl ToString) -> Result<Value, Error> {
        let q = [("backlog_id", backlog_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/arbitrage/get-backlog", Some(&q[..]), None)
            .await
    }

    pub async fn delete_backlog(&self, backlog_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/arbitrage/delete-backlog",
                None,
                Some(&json!({ "backlog_id": backlog_id.to_string() })),
            )
            .await
    }
}
