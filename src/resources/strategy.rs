//! `client.strategy` — user-defined trading strategies.

use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::Transport;
use crate::error::Error;
use crate::resources::hoppers::merge_id;

#[derive(Clone, Debug)]
pub struct Strategies {
    transport: Arc<Transport>,
}

impl Strategies {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub async fn list(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/strategy/strategies", None, None)
            .await
    }

    pub async fn get(&self, strategy_id: impl ToString) -> Result<Value, Error> {
        let q = [("strategy_id", strategy_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/strategy/get", Some(&q[..]), None)
            .await
    }

    pub async fn create(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/strategy/create", None, Some(data))
            .await
    }

    pub async fn update(&self, strategy_id: impl ToString, data: &Value) -> Result<Value, Error> {
        let mut body = data.clone();
        merge_id(&mut body, "strategy_id", &strategy_id.to_string());
        self.transport
            .request::<(), _>(Method::POST, "/strategy/edit", None, Some(&body))
            .await
    }

    pub async fn delete(&self, strategy_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/strategy/delete",
                None,
                Some(&json!({ "strategy_id": strategy_id.to_string() })),
            )
            .await
    }
}
