//! One sanity test per resource group — method + path + body shape.

use cryptohopper::Client;
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn build(server: &MockServer) -> Client {
    Client::builder()
        .api_key("ch_test")
        .base_url(server.uri())
        .max_retries(0)
        .build()
        .expect("client")
}

async fn ok_get(server: &MockServer, p: &str, body: serde_json::Value) {
    Mock::given(method("GET"))
        .and(path(p))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(server)
        .await;
}

async fn ok_post(server: &MockServer, p: &str, body: serde_json::Value) {
    Mock::given(method("POST"))
        .and(path(p))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(server)
        .await;
}

// ─── Hoppers ────────────────────────────────────────────────────────────

#[tokio::test]
async fn hoppers_list_with_exchange_filter() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/hopper/list"))
        .and(query_param("exchange", "binance"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
        .mount(&server)
        .await;
    build(&server).hoppers.list(Some("binance")).await.unwrap();
}

#[tokio::test]
async fn hoppers_config_update_merges_hopper_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/hopper/configupdate"))
        .and(body_json(json!({"hopper_id": "7", "strategy_id": 99})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {}})))
        .mount(&server)
        .await;
    build(&server)
        .hoppers
        .config_update(7, &json!({"strategy_id": 99}))
        .await
        .unwrap();
}

// ─── Exchange ───────────────────────────────────────────────────────────

#[tokio::test]
async fn exchange_forex_rates_hyphenated_path() {
    let server = MockServer::start().await;
    ok_get(&server, "/exchange/forex-rates", json!({"data": {}})).await;
    build(&server).exchange.forex_rates().await.unwrap();
}

// ─── Strategy ───────────────────────────────────────────────────────────

#[tokio::test]
async fn strategy_list_hits_strategies_plural() {
    let server = MockServer::start().await;
    ok_get(&server, "/strategy/strategies", json!({"data": []})).await;
    build(&server).strategy.list().await.unwrap();
}

#[tokio::test]
async fn strategy_update_hits_edit() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/strategy/edit"))
        .and(body_json(json!({"strategy_id": "5", "name": "renamed"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {}})))
        .mount(&server)
        .await;
    build(&server)
        .strategy
        .update(5, &json!({"name": "renamed"}))
        .await
        .unwrap();
}

// ─── Backtest ───────────────────────────────────────────────────────────

#[tokio::test]
async fn backtest_create_hits_new() {
    let server = MockServer::start().await;
    ok_post(&server, "/backtest/new", json!({"data": {"id": 1}})).await;
    build(&server)
        .backtest
        .create(&json!({"hopper_id": 42}))
        .await
        .unwrap();
}

// ─── Market ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn market_items_hits_marketitems() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/market/marketitems"))
        .and(query_param("type", "strategy"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
        .mount(&server)
        .await;
    build(&server)
        .market
        .items(Some(&json!({"type": "strategy"})))
        .await
        .unwrap();
}

// ─── Signals ────────────────────────────────────────────────────────────

#[tokio::test]
async fn signals_chart_data_single_word_path() {
    let server = MockServer::start().await;
    ok_get(&server, "/signals/chartdata", json!({"data": {}})).await;
    build(&server).signals.chart_data(None).await.unwrap();
}

// ─── Arbitrage ──────────────────────────────────────────────────────────

#[tokio::test]
async fn arbitrage_market_cancel_hyphenated() {
    let server = MockServer::start().await;
    ok_post(&server, "/arbitrage/market-cancel", json!({"data": {}})).await;
    build(&server).arbitrage.market_cancel(None).await.unwrap();
}

#[tokio::test]
async fn arbitrage_delete_backlog_posts_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/arbitrage/delete-backlog"))
        .and(body_json(json!({"backlog_id": "7"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {}})))
        .mount(&server)
        .await;
    build(&server).arbitrage.delete_backlog(7).await.unwrap();
}

// ─── MarketMaker ────────────────────────────────────────────────────────

#[tokio::test]
async fn marketmaker_set_market_trend_hyphenated() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/marketmaker/set-market-trend"))
        .and(body_json(json!({"hopper_id": 1, "trend": "bull"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {}})))
        .mount(&server)
        .await;
    build(&server)
        .marketmaker
        .set_market_trend(&json!({"hopper_id": 1, "trend": "bull"}))
        .await
        .unwrap();
}

// ─── Template ───────────────────────────────────────────────────────────

#[tokio::test]
async fn template_save_hyphenated_path() {
    let server = MockServer::start().await;
    ok_post(
        &server,
        "/template/save-template",
        json!({"data": {"id": 4}}),
    )
    .await;
    build(&server)
        .template
        .save(&json!({"name": "t"}))
        .await
        .unwrap();
}

#[tokio::test]
async fn template_load_sends_both_ids() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/template/load"))
        .and(body_json(json!({"template_id": "3", "hopper_id": "5"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {}})))
        .mount(&server)
        .await;
    build(&server).template.load(3, 5).await.unwrap();
}

// ─── AI ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn ai_get_credits_keeps_server_prefix() {
    let server = MockServer::start().await;
    ok_get(
        &server,
        "/ai/getaicredits",
        json!({"data": {"balance": 100}}),
    )
    .await;
    let out = build(&server).ai.get_credits().await.unwrap();
    assert_eq!(out["balance"], 100);
}

#[tokio::test]
async fn ai_llm_analyze() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/ai/doaillmanalyze"))
        .and(body_json(json!({"strategy_id": 42})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {}})))
        .mount(&server)
        .await;
    build(&server)
        .ai
        .llm_analyze(&json!({"strategy_id": 42}))
        .await
        .unwrap();
}

// ─── Platform ───────────────────────────────────────────────────────────

#[tokio::test]
async fn platform_search_documentation_with_q() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/platform/searchdocumentation"))
        .and(query_param("q", "rsi"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
        .mount(&server)
        .await;
    build(&server)
        .platform
        .search_documentation("rsi")
        .await
        .unwrap();
}

// ─── Chart ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn chart_share_save_hyphenated() {
    let server = MockServer::start().await;
    ok_post(&server, "/chart/share-save", json!({"data": {}})).await;
    build(&server)
        .chart
        .share_save(&json!({"title": "BTC"}))
        .await
        .unwrap();
}

// ─── Subscription ───────────────────────────────────────────────────────

#[tokio::test]
async fn subscription_stop_posts_empty_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/subscription/stopsubscription"))
        .and(body_json(json!({})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {}})))
        .mount(&server)
        .await;
    build(&server)
        .subscription
        .stop_subscription(None)
        .await
        .unwrap();
}

// ─── Social ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn social_create_post_maps_to_bare_post() {
    let server = MockServer::start().await;
    ok_post(&server, "/social/post", json!({"data": {"id": 1}})).await;
    build(&server)
        .social
        .create_post(&json!({"content": "hi"}))
        .await
        .unwrap();
}

#[tokio::test]
async fn social_get_conversation_maps_to_loadconversation() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/social/loadconversation"))
        .and(query_param("conversation_id", "42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
        .mount(&server)
        .await;
    build(&server).social.get_conversation(42).await.unwrap();
}

// ─── Tournaments ────────────────────────────────────────────────────────

#[tokio::test]
async fn tournaments_list_gettournaments() {
    let server = MockServer::start().await;
    ok_get(&server, "/tournaments/gettournaments", json!({"data": []})).await;
    build(&server).tournaments.list(None).await.unwrap();
}

#[tokio::test]
async fn tournaments_tournament_leaderboard_underscored() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tournaments/leaderboard_tournament"))
        .and(query_param("tournament_id", "7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
        .mount(&server)
        .await;
    build(&server)
        .tournaments
        .tournament_leaderboard(7)
        .await
        .unwrap();
}

// ─── Webhooks ───────────────────────────────────────────────────────────

#[tokio::test]
async fn webhooks_create_posts_to_api_webhook_create() {
    let server = MockServer::start().await;
    ok_post(&server, "/api/webhook_create", json!({"data": {"id": 1}})).await;
    build(&server)
        .webhooks
        .create(&json!({"url": "https://e.com"}))
        .await
        .unwrap();
}

// ─── App ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn app_in_app_purchase_underscored() {
    let server = MockServer::start().await;
    ok_post(&server, "/app/in_app_purchase", json!({"data": {}})).await;
    build(&server)
        .app
        .in_app_purchase(&json!({"receipt": "abc"}))
        .await
        .unwrap();
}

// ─── User ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn user_get_hits_user_get() {
    let server = MockServer::start().await;
    ok_get(
        &server,
        "/user/get",
        json!({"data": {"id": 1, "email": "x@y.com"}}),
    )
    .await;
    let out = build(&server).user.get().await.unwrap();
    assert_eq!(out["email"], "x@y.com");
}
