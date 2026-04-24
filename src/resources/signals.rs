//! `client.signals` — signal-provider analytics.

use std::sync::Arc;

use reqwest::Method;
use serde_json::Value;

use crate::client::Transport;
use crate::error::Error;

#[derive(Clone, Debug)]
pub struct Signals {
    transport: Arc<Transport>,
}

impl Signals {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub async fn list(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/signals/signals", params, None)
            .await
    }

    pub async fn performance(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/signals/performance", params, None)
            .await
    }

    pub async fn stats(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/signals/stats", None, None)
            .await
    }

    pub async fn distribution(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/signals/distribution", None, None)
            .await
    }

    pub async fn chart_data(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/signals/chartdata", params, None)
            .await
    }
}
