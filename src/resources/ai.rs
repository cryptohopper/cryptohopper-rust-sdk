//! `client.ai` — AI credits + LLM analysis.
//!
//! Named `Ai` (not `AI`) to satisfy Rust's CamelCase convention. Access
//! via `client.ai.*`.

use std::sync::Arc;

use reqwest::Method;
use serde_json::Value;

use crate::client::Transport;
use crate::error::Error;

#[derive(Clone, Debug)]
pub struct Ai {
    transport: Arc<Transport>,
}

impl Ai {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub async fn list(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/ai/list", params, None)
            .await
    }

    pub async fn get(&self, id: impl ToString) -> Result<Value, Error> {
        let q = [("id", id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/ai/get", Some(&q[..]), None)
            .await
    }

    pub async fn available_models(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/ai/availablemodels", None, None)
            .await
    }

    // Credits

    pub async fn get_credits(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/ai/getaicredits", None, None)
            .await
    }

    pub async fn credit_invoices(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/ai/aicreditinvoices", params, None)
            .await
    }

    pub async fn credit_transactions(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/ai/aicredittransactions", params, None)
            .await
    }

    pub async fn buy_credits(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/ai/buyaicredits", None, Some(data))
            .await
    }

    // LLM analysis

    pub async fn llm_analyze_options(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/ai/aillmanalyzeoptions", None, None)
            .await
    }

    pub async fn llm_analyze(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/ai/doaillmanalyze", None, Some(data))
            .await
    }

    pub async fn llm_analyze_results(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/ai/aillmanalyzeresults", params, None)
            .await
    }

    pub async fn llm_results(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/ai/aillmresults", params, None)
            .await
    }
}
