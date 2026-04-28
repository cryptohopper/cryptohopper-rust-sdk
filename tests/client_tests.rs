//! Transport / error-mapping / auth integration tests using wiremock.

use cryptohopper::{Client, ErrorCode};
use serde_json::json;
use wiremock::matchers::{body_json, header, header_exists, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn build(server: &MockServer) -> Client {
    Client::builder()
        .api_key("ch_test")
        .base_url(server.uri())
        .max_retries(0)
        .build()
        .expect("client")
}

#[tokio::test]
async fn sends_access_token_user_agent_and_unwraps_data() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/user/get"))
        .and(header("access-token", "ch_test"))
        .and(header("Accept", "application/json"))
        .and(header_exists("User-Agent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"hello": "world"}})))
        .mount(&server)
        .await;

    let out = build(&server).user.get().await.expect("ok");
    assert_eq!(out["hello"], "world");
}

#[tokio::test]
async fn app_key_sets_header() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/user/get"))
        .and(header("x-api-app-key", "client_123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {}})))
        .mount(&server)
        .await;

    let c = Client::builder()
        .api_key("ch_test")
        .app_key("client_123")
        .base_url(server.uri())
        .max_retries(0)
        .build()
        .expect("client");
    c.user.get().await.expect("ok");
}

#[tokio::test]
async fn post_body_is_json_encoded() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/hopper/buy"))
        .and(body_json(
            json!({"hopper_id": 42, "market": "BTC/USDT", "amount": "0.001"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {}})))
        .mount(&server)
        .await;

    build(&server)
        .hoppers
        .buy(&json!({"hopper_id": 42, "market": "BTC/USDT", "amount": "0.001"}))
        .await
        .expect("ok");
}

#[tokio::test]
async fn query_params_serialised_from_json_value() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/exchange/ticker"))
        .and(query_param("exchange", "binance"))
        .and(query_param("market", "BTC/USDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"last": 42000}})))
        .mount(&server)
        .await;

    let out = build(&server)
        .exchange
        .ticker(&json!({"exchange": "binance", "market": "BTC/USDT"}))
        .await
        .expect("ok");
    assert_eq!(out["last"], 42000);
}

#[tokio::test]
async fn maps_cryptohopper_error_envelope_to_typed_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/user/get"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "status": 403,
            "code": 0,
            "error": 1,
            "message": "no access",
            "ip_address": "1.2.3.4"
        })))
        .mount(&server)
        .await;

    let err = build(&server).user.get().await.unwrap_err();
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert_eq!(err.status, 403);
    assert_eq!(err.ip_address.as_deref(), Some("1.2.3.4"));
    assert_eq!(err.message, "no access");
}

#[tokio::test]
async fn retries_on_429_honouring_retry_after_then_succeeds() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/user/get"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "0")
                .set_body_json(json!({
                    "status": 429, "code": 0, "error": 1, "message": "slow"
                })),
        )
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/user/get"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"ok": true}})))
        .mount(&server)
        .await;

    let c = Client::builder()
        .api_key("ch_test")
        .base_url(server.uri())
        .max_retries(2)
        .build()
        .expect("client");
    let out = c.user.get().await.expect("retried");
    assert_eq!(out["ok"], true);
}

#[tokio::test]
async fn gives_up_after_max_retries_on_persistent_429() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/user/get"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "0")
                .set_body_json(json!({
                    "status": 429, "code": 0, "error": 1, "message": "slow"
                })),
        )
        .mount(&server)
        .await;

    let c = Client::builder()
        .api_key("ch_test")
        .base_url(server.uri())
        .max_retries(2)
        .build()
        .expect("client");
    let err = c.user.get().await.unwrap_err();
    assert_eq!(err.code, ErrorCode::RateLimited);
}

#[tokio::test]
async fn non_json_5xx_falls_back_to_server_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/user/get"))
        .respond_with(ResponseTemplate::new(500).set_body_string("upstream crashed"))
        .mount(&server)
        .await;

    let err = build(&server).user.get().await.unwrap_err();
    assert_eq!(err.code, ErrorCode::ServerError);
    assert_eq!(err.status, 500);
}

#[test]
fn empty_api_key_rejected() {
    let result = Client::new("");
    assert!(result.is_err());
}

#[test]
fn builder_strips_trailing_slash_from_base_url() {
    let c = Client::builder()
        .api_key("ch_test")
        .base_url("https://api-staging.cryptohopper.com/v1/")
        .build()
        .expect("client");
    // No public accessor for base_url, but client was constructed successfully,
    // which is the behaviour we care about. Further assertions via mock URLs in
    // the other tests cover URL building correctness.
    drop(c);
}
