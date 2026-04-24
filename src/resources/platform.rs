//! `client.platform` — public discovery / i18n / marketing reads.

use std::sync::Arc;

use reqwest::Method;
use serde_json::Value;

use crate::client::Transport;
use crate::error::Error;

#[derive(Clone, Debug)]
pub struct Platform {
    transport: Arc<Transport>,
}

impl Platform {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub async fn latest_blog(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/platform/latestblog", params, None)
            .await
    }

    pub async fn documentation(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/platform/documentation", params, None)
            .await
    }

    pub async fn promo_bar(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/platform/promobar", None, None)
            .await
    }

    pub async fn search_documentation(&self, query: &str) -> Result<Value, Error> {
        let q = [("q", query.to_string())];
        self.transport
            .request::<_, ()>(
                Method::GET,
                "/platform/searchdocumentation",
                Some(&q[..]),
                None,
            )
            .await
    }

    pub async fn countries(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/platform/countries", None, None)
            .await
    }

    pub async fn country_allowlist(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/platform/countryallowlist", None, None)
            .await
    }

    pub async fn ip_country(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/platform/ipcountry", None, None)
            .await
    }

    pub async fn languages(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/platform/languages", None, None)
            .await
    }

    pub async fn bot_types(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/platform/bottypes", None, None)
            .await
    }
}
