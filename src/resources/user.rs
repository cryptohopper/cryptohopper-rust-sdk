//! `client.user` — authenticated user profile.

use std::sync::Arc;

use reqwest::Method;
use serde_json::Value;

use crate::client::Transport;
use crate::error::Error;

#[derive(Clone, Debug)]
pub struct User {
    transport: Arc<Transport>,
}

impl User {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    /// Fetch the authenticated user's profile. Requires `user` scope.
    pub async fn get(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/user/get", None, None)
            .await
    }
}
