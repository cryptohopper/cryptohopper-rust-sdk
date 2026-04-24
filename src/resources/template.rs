//! `client.template` — bot templates.

use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::Transport;
use crate::error::Error;
use crate::resources::hoppers::merge_id;

#[derive(Clone, Debug)]
pub struct Templates {
    transport: Arc<Transport>,
}

impl Templates {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub async fn list(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/template/templates", None, None)
            .await
    }

    pub async fn get(&self, template_id: impl ToString) -> Result<Value, Error> {
        let q = [("template_id", template_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/template/get", Some(&q[..]), None)
            .await
    }

    pub async fn basic(&self, template_id: impl ToString) -> Result<Value, Error> {
        let q = [("template_id", template_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/template/basic", Some(&q[..]), None)
            .await
    }

    pub async fn save(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/template/save-template", None, Some(data))
            .await
    }

    pub async fn update(&self, template_id: impl ToString, data: &Value) -> Result<Value, Error> {
        let mut body = data.clone();
        merge_id(&mut body, "template_id", &template_id.to_string());
        self.transport
            .request::<(), _>(Method::POST, "/template/update", None, Some(&body))
            .await
    }

    pub async fn load(
        &self,
        template_id: impl ToString,
        hopper_id: impl ToString,
    ) -> Result<Value, Error> {
        let body = json!({
            "template_id": template_id.to_string(),
            "hopper_id": hopper_id.to_string(),
        });
        self.transport
            .request::<(), _>(Method::POST, "/template/load", None, Some(&body))
            .await
    }

    pub async fn delete(&self, template_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/template/delete",
                None,
                Some(&json!({ "template_id": template_id.to_string() })),
            )
            .await
    }
}
