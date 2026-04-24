//! `client.hoppers` — user trading bots (CRUD, positions, orders, trade, config).

use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::Transport;
use crate::error::Error;

#[derive(Clone, Debug)]
pub struct Hoppers {
    transport: Arc<Transport>,
}

impl Hoppers {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    /// List the user's hoppers. `exchange` filter is optional.
    pub async fn list(&self, exchange: Option<&str>) -> Result<Value, Error> {
        let query = exchange.map(|e| [("exchange", e.to_string())]);
        self.transport
            .request::<_, ()>(
                Method::GET,
                "/hopper/list",
                query.as_ref().map(|q| q.as_slice()),
                None,
            )
            .await
    }

    /// Fetch a single hopper.
    pub async fn get(&self, hopper_id: impl ToString) -> Result<Value, Error> {
        let q = [("hopper_id", hopper_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/hopper/get", Some(&q[..]), None)
            .await
    }

    /// Create a new hopper.
    pub async fn create(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/hopper/create", None, Some(data))
            .await
    }

    /// Update a hopper.
    pub async fn update(&self, hopper_id: impl ToString, data: &Value) -> Result<Value, Error> {
        let mut body = data.clone();
        merge_id(&mut body, "hopper_id", &hopper_id.to_string());
        self.transport
            .request::<(), _>(Method::POST, "/hopper/update", None, Some(&body))
            .await
    }

    pub async fn delete(&self, hopper_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/hopper/delete",
                None,
                Some(&json!({ "hopper_id": hopper_id.to_string() })),
            )
            .await
    }

    pub async fn positions(&self, hopper_id: impl ToString) -> Result<Value, Error> {
        let q = [("hopper_id", hopper_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/hopper/positions", Some(&q[..]), None)
            .await
    }

    pub async fn position(
        &self,
        hopper_id: impl ToString,
        position_id: impl ToString,
    ) -> Result<Value, Error> {
        let q = [
            ("hopper_id", hopper_id.to_string()),
            ("position_id", position_id.to_string()),
        ];
        self.transport
            .request::<_, ()>(Method::GET, "/hopper/position", Some(&q[..]), None)
            .await
    }

    pub async fn orders(
        &self,
        hopper_id: impl ToString,
        extra: Option<&Value>,
    ) -> Result<Value, Error> {
        let mut query: Vec<(String, String)> = vec![("hopper_id".into(), hopper_id.to_string())];
        if let Some(Value::Object(map)) = extra {
            for (k, v) in map {
                if let Some(s) = value_to_query_string(v) {
                    query.push((k.clone(), s));
                }
            }
        }
        self.transport
            .request::<_, ()>(Method::GET, "/hopper/orders", Some(&query), None)
            .await
    }

    /// Place a market/limit buy. Subject to the `order` rate bucket.
    pub async fn buy(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/hopper/buy", None, Some(data))
            .await
    }

    /// Place a market/limit sell. Subject to the `order` rate bucket.
    pub async fn sell(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/hopper/sell", None, Some(data))
            .await
    }

    pub async fn config_get(&self, hopper_id: impl ToString) -> Result<Value, Error> {
        let q = [("hopper_id", hopper_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/hopper/configget", Some(&q[..]), None)
            .await
    }

    pub async fn config_update(
        &self,
        hopper_id: impl ToString,
        config: &Value,
    ) -> Result<Value, Error> {
        let mut body = config.clone();
        merge_id(&mut body, "hopper_id", &hopper_id.to_string());
        self.transport
            .request::<(), _>(Method::POST, "/hopper/configupdate", None, Some(&body))
            .await
    }

    pub async fn config_pools(&self, hopper_id: impl ToString) -> Result<Value, Error> {
        let q = [("hopper_id", hopper_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/hopper/configpools", Some(&q[..]), None)
            .await
    }

    /// Panic-sell every position on this hopper. Requires `trade`.
    pub async fn panic(&self, hopper_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/hopper/panic",
                None,
                Some(&json!({ "hopper_id": hopper_id.to_string() })),
            )
            .await
    }
}

pub(crate) fn merge_id(body: &mut Value, key: &str, id: &str) {
    if let Value::Object(map) = body {
        map.insert(key.to_string(), json!(id));
    } else {
        *body = json!({ key: id });
    }
}

pub(crate) fn value_to_query_string(v: &Value) -> Option<String> {
    match v {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => Some(v.to_string()),
    }
}
