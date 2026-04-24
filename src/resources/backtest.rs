//! `client.backtest` — run and inspect backtests.

use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::Transport;
use crate::error::Error;

#[derive(Clone, Debug)]
pub struct Backtests {
    transport: Arc<Transport>,
}

impl Backtests {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub async fn create(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/backtest/new", None, Some(data))
            .await
    }

    pub async fn get(&self, backtest_id: impl ToString) -> Result<Value, Error> {
        let q = [("backtest_id", backtest_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/backtest/get", Some(&q[..]), None)
            .await
    }

    pub async fn list(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/backtest/list", params, None)
            .await
    }

    pub async fn cancel(&self, backtest_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/backtest/cancel",
                None,
                Some(&json!({ "backtest_id": backtest_id.to_string() })),
            )
            .await
    }

    pub async fn restart(&self, backtest_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/backtest/restart",
                None,
                Some(&json!({ "backtest_id": backtest_id.to_string() })),
            )
            .await
    }

    pub async fn limits(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/backtest/limits", None, None)
            .await
    }
}
