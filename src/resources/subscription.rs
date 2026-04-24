//! `client.subscription` — plans, per-hopper state, credits, billing.

use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::Transport;
use crate::error::Error;

#[derive(Clone, Debug)]
pub struct Subscription {
    transport: Arc<Transport>,
}

impl Subscription {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub async fn hopper(&self, hopper_id: impl ToString) -> Result<Value, Error> {
        let q = [("hopper_id", hopper_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/subscription/hopper", Some(&q[..]), None)
            .await
    }

    pub async fn get(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/subscription/get", None, None)
            .await
    }

    pub async fn plans(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/subscription/plans", None, None)
            .await
    }

    pub async fn remap(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/subscription/remap", None, Some(data))
            .await
    }

    pub async fn assign(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/subscription/assign", None, Some(data))
            .await
    }

    pub async fn get_credits(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/subscription/getcredits", None, None)
            .await
    }

    pub async fn order_sub(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/subscription/ordersub", None, Some(data))
            .await
    }

    pub async fn stop_subscription(&self, data: Option<&Value>) -> Result<Value, Error> {
        let empty = json!({});
        let body = data.unwrap_or(&empty);
        self.transport
            .request::<(), _>(
                Method::POST,
                "/subscription/stopsubscription",
                None,
                Some(body),
            )
            .await
    }
}
