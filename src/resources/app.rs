//! `client.app` — mobile app store receipts + in-app purchases.

use std::sync::Arc;

use reqwest::Method;
use serde_json::Value;

use crate::client::Transport;
use crate::error::Error;

#[derive(Clone, Debug)]
pub struct App {
    transport: Arc<Transport>,
}

impl App {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub async fn receipt(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/app/receipt", None, Some(data))
            .await
    }

    pub async fn in_app_purchase(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/app/in_app_purchase", None, Some(data))
            .await
    }
}
