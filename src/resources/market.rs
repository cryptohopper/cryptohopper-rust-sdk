//! `client.market` — marketplace browse (public).

use std::sync::Arc;

use reqwest::Method;
use serde_json::Value;

use crate::client::Transport;
use crate::error::Error;

#[derive(Clone, Debug)]
pub struct Market {
    transport: Arc<Transport>,
}

impl Market {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub async fn signals(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/market/signals", params, None)
            .await
    }

    pub async fn signal(&self, signal_id: impl ToString) -> Result<Value, Error> {
        let q = [("signal_id", signal_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/market/signal", Some(&q[..]), None)
            .await
    }

    pub async fn items(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/market/marketitems", params, None)
            .await
    }

    pub async fn item(&self, item_id: impl ToString) -> Result<Value, Error> {
        let q = [("item_id", item_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/market/marketitem", Some(&q[..]), None)
            .await
    }

    pub async fn homepage(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/market/homepage", None, None)
            .await
    }
}
