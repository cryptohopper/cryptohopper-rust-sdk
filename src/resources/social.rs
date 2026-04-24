//! `client.social` — profiles, feed, posts, conversations, social graph.
//! Largest resource in the SDK (27 methods).

use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::Transport;
use crate::error::Error;

#[derive(Clone, Debug)]
pub struct Social {
    transport: Arc<Transport>,
}

impl Social {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    // ─── Profiles ─────────────────────────────────────────────────────

    pub async fn get_profile(&self, alias_or_id: impl ToString) -> Result<Value, Error> {
        let q = [("alias", alias_or_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/social/getprofile", Some(&q[..]), None)
            .await
    }

    pub async fn edit_profile(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/social/editprofile", None, Some(data))
            .await
    }

    pub async fn check_alias(&self, alias: &str) -> Result<Value, Error> {
        let q = [("alias", alias.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/social/checkalias", Some(&q[..]), None)
            .await
    }

    // ─── Feed / discovery ─────────────────────────────────────────────

    pub async fn get_feed(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/social/getfeed", params, None)
            .await
    }

    pub async fn get_trends(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/social/gettrends", None, None)
            .await
    }

    pub async fn who_to_follow(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/social/whotofollow", None, None)
            .await
    }

    pub async fn search(&self, query: &str) -> Result<Value, Error> {
        let q = [("q", query.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/social/search", Some(&q[..]), None)
            .await
    }

    // ─── Notifications ────────────────────────────────────────────────

    pub async fn get_notifications(&self, params: Option<&Value>) -> Result<Value, Error> {
        self.transport
            .request::<_, ()>(Method::GET, "/social/getnotifications", params, None)
            .await
    }

    // ─── Conversations / messages ─────────────────────────────────────

    pub async fn get_conversation_list(&self) -> Result<Value, Error> {
        self.transport
            .request::<(), ()>(Method::GET, "/social/getconversationlist", None, None)
            .await
    }

    pub async fn get_conversation(&self, conversation_id: impl ToString) -> Result<Value, Error> {
        let q = [("conversation_id", conversation_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/social/loadconversation", Some(&q[..]), None)
            .await
    }

    pub async fn send_message(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/social/sendmessage", None, Some(data))
            .await
    }

    pub async fn delete_message(&self, message_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/social/deletemessage",
                None,
                Some(&json!({ "message_id": message_id.to_string() })),
            )
            .await
    }

    // ─── Posts ────────────────────────────────────────────────────────

    pub async fn create_post(&self, data: &Value) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(Method::POST, "/social/post", None, Some(data))
            .await
    }

    pub async fn get_post(&self, post_id: impl ToString) -> Result<Value, Error> {
        let q = [("post_id", post_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/social/getpost", Some(&q[..]), None)
            .await
    }

    pub async fn delete_post(&self, post_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/social/deletepost",
                None,
                Some(&json!({ "post_id": post_id.to_string() })),
            )
            .await
    }

    pub async fn pin_post(&self, post_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/social/pinpost",
                None,
                Some(&json!({ "post_id": post_id.to_string() })),
            )
            .await
    }

    // ─── Comments ─────────────────────────────────────────────────────

    pub async fn get_comment(&self, comment_id: impl ToString) -> Result<Value, Error> {
        let q = [("comment_id", comment_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/social/getcomment", Some(&q[..]), None)
            .await
    }

    pub async fn get_comments(&self, post_id: impl ToString) -> Result<Value, Error> {
        let q = [("post_id", post_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/social/getcomments", Some(&q[..]), None)
            .await
    }

    pub async fn delete_comment(&self, comment_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/social/deletecomment",
                None,
                Some(&json!({ "comment_id": comment_id.to_string() })),
            )
            .await
    }

    // ─── Media ────────────────────────────────────────────────────────

    pub async fn get_media(&self, media_id: impl ToString) -> Result<Value, Error> {
        let q = [("media_id", media_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/social/getmedia", Some(&q[..]), None)
            .await
    }

    // ─── Social graph ─────────────────────────────────────────────────

    pub async fn follow(&self, alias_or_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/social/follow",
                None,
                Some(&json!({ "alias": alias_or_id.to_string() })),
            )
            .await
    }

    pub async fn get_followers(&self, alias_or_id: impl ToString) -> Result<Value, Error> {
        let q = [("alias", alias_or_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/social/followers", Some(&q[..]), None)
            .await
    }

    pub async fn get_following(&self, alias_or_id: impl ToString) -> Result<Value, Error> {
        let q = [("alias", alias_or_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/social/following", Some(&q[..]), None)
            .await
    }

    pub async fn get_following_profiles(&self, alias_or_id: impl ToString) -> Result<Value, Error> {
        let q = [("alias", alias_or_id.to_string())];
        self.transport
            .request::<_, ()>(Method::GET, "/social/followingprofiles", Some(&q[..]), None)
            .await
    }

    // ─── Engagement ───────────────────────────────────────────────────

    pub async fn like(&self, post_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/social/like",
                None,
                Some(&json!({ "post_id": post_id.to_string() })),
            )
            .await
    }

    pub async fn repost(&self, post_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/social/repost",
                None,
                Some(&json!({ "post_id": post_id.to_string() })),
            )
            .await
    }

    // ─── Moderation ───────────────────────────────────────────────────

    pub async fn block_user(&self, alias_or_id: impl ToString) -> Result<Value, Error> {
        self.transport
            .request::<(), _>(
                Method::POST,
                "/social/blockuser",
                None,
                Some(&json!({ "alias": alias_or_id.to_string() })),
            )
            .await
    }
}
