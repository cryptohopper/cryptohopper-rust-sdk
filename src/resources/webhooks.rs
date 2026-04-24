//! `client.webhooks` — developer webhook registration.
//! Maps to the server's `/api/webhook_*` endpoints.

use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::Transport;
use crate::error::Error;

#[derive(Clone, Debug)]
pub struct Webhooks {
    transport: Arc<Transport>,
}

impl Webhooks {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub async fn create(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/api/webhook_create", None, Some(data))
            .await
    }

    pub async fn delete(&self, webhook_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/api/webhook_delete",
                None,
                Some(&json!({ "webhook_id": webhook_id.to_string() })),
            )
            .await
    }
}
