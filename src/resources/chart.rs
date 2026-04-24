//! `client.chart` — saved chart layouts + shared chart links.

use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::Transport;
use crate::error::Error;

#[derive(Clone, Debug)]
pub struct Chart {
    transport: Arc<Transport>,
}

impl Chart {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub async fn list(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/chart/list", None, None)
            .await
    }

    pub async fn get(&self, chart_id: impl ToString) -> Result<Value, Error> {
        let q = [("chart_id", chart_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/chart/get", Some(&q[..]), None)
            .await
    }

    pub async fn save(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/chart/save", None, Some(data))
            .await
    }

    pub async fn delete(&self, chart_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/chart/delete",
                None,
                Some(&json!({ "chart_id": chart_id.to_string() })),
            )
            .await
    }

    pub async fn share_save(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/chart/share-save", None, Some(data))
            .await
    }

    pub async fn share_get(&self, share_id: &str) -> Result<Value, Error> {
        let q = [("share_id", share_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/chart/share-get", Some(&q[..]), None)
            .await
    }
}
