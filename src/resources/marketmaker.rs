//! `client.marketmaker` — market-maker bot ops + trend overrides + backlog.

use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::Transport;
use crate::error::Error;

#[derive(Clone, Debug)]
pub struct MarketMaker {
    transport: Arc<Transport>,
}

impl MarketMaker {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub async fn get(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/marketmaker/get", params, None)
            .await
    }

    pub async fn cancel(&self, data: Option<&Value>) -> Result<Value, Error> {
        let empty = json!({});
        let body = data.unwrap_or(&empty);
        self.transport
            .request::<(), _>(Method::POST, "/marketmaker/cancel", None, Some(body))
            .await
    }

    pub async fn history(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/marketmaker/history", params, None)
            .await
    }

    // Market-trend overrides

    pub async fn get_market_trend(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/marketmaker/get-market-trend", params, None)
            .await
    }

    pub async fn set_market_trend(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/marketmaker/set-market-trend",
                None,
                Some(data),
            )
            .await
    }

    pub async fn delete_market_trend(&self, data: Option<&Value>) -> Result<Value, Error> {
        let empty = json!({});
        let body = data.unwrap_or(&empty);
        self.transport
            .request::<(), _>(
                Method::POST,
                "/marketmaker/delete-market-trend",
                None,
                Some(body),
            )
            .await
    }

    // Backlog

    pub async fn backlogs(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/marketmaker/get-backlogs", params, None)
            .await
    }

    pub async fn backlog(&self, backlog_id: impl ToString) -> Result<Value, Error> {
        let q = [("backlog_id", backlog_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/marketmaker/get-backlog", Some(&q[..]), None)
            .await
    }

    pub async fn delete_backlog(&self, backlog_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/marketmaker/delete-backlog",
                None,
                Some(&json!({ "backlog_id": backlog_id.to_string() })),
            )
            .await
    }
}
