//! `client.tournaments` — trading competitions.

use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::Transport;
use crate::error::Error;
use crate::resources::hoppers::merge_id;

#[derive(Clone, Debug)]
pub struct Tournaments {
    transport: Arc<Transport>,
}

impl Tournaments {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub async fn list(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/tournaments/gettournaments", params, None)
            .await
    }

    pub async fn active(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/tournaments/active", None, None)
            .await
    }

    pub async fn get(&self, tournament_id: impl ToString) -> Result<Value, Error> {
        let q = [("tournament_id", tournament_id.to_string())];
        self.transport
            .request::<_, ()>(
                Method::GET,
                "/tournaments/gettournament",
                Some(&q[..]),
                None,
            )
            .await
    }

    pub async fn search(&self, query: &str) -> Result<Value, Error> {
        let q = [("q", query.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/tournaments/search", Some(&q[..]), None)
            .await
    }

    pub async fn trades(&self, tournament_id: impl ToString) -> Result<Value, Error> {
        let q = [("tournament_id", tournament_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/tournaments/trades", Some(&q[..]), None)
            .await
    }

    pub async fn stats(&self, tournament_id: impl ToString) -> Result<Value, Error> {
        let q = [("tournament_id", tournament_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/tournaments/stats", Some(&q[..]), None)
            .await
    }

    pub async fn activity(&self, tournament_id: impl ToString) -> Result<Value, Error> {
        let q = [("tournament_id", tournament_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/tournaments/activity", Some(&q[..]), None)
            .await
    }

    pub async fn leaderboard(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/tournaments/leaderboard", params, None)
            .await
    }

    pub async fn tournament_leaderboard(
        &self,
        tournament_id: impl ToString,
    ) -> Result<Value, Error> {
        let q = [("tournament_id", tournament_id.to_string())];
        self.transport
            .request::<_, ()>(
                Method::GET,
                "/tournaments/leaderboard_tournament",
                Some(&q[..]),
                None,
            )
            .await
    }

    pub async fn join(
        &self,
        tournament_id: impl ToString,
        data: Option<&Value>,
    ) -> Result<Value, Error> {
        let mut body = data.cloned().unwrap_or_else(|| json!({}));
        merge_id(&mut body, "tournament_id", &tournament_id.to_string());
        self.transport
            .request::<(), _>(Method::POST, "/tournaments/join", None, Some(&body))
            .await
    }

    pub async fn leave(&self, tournament_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/tournaments/leave",
                None,
                Some(&json!({ "tournament_id": tournament_id.to_string() })),
            )
            .await
    }
}
