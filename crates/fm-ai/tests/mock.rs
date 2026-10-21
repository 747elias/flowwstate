use axum::{
    Json, Router,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use fm_ai::{Client, Message};
use fm_core::Config;
use serde_json::{Value, json};
async fn mock() -> (String, tokio::task::JoinHandle<()>) {
    async fn models(h: HeaderMap) -> axum::response::Response {
        if h.get("authorization").and_then(|v| v.to_str().ok()) == Some("Bearer wrong") {
            return StatusCode::UNAUTHORIZED.into_response();
        }
        Json(json!({"data":[{"id":"mock"}]})).into_response()
    }
    async fn chat(Json(v): Json<Value>) -> axum::response::Response {
        if v["model"] == "broken" {
            return ([("content-type", "text/event-stream")], "data: {bad}\n\n").into_response();
        }
        if v["model"] == "slow" {
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        }
        ([("content-type","text/event-stream")],"data: {\"choices\":[{\"delta\":{\"content\":\"Grüezi \"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"Welt\"}}]}\n\ndata: [DONE]\n\n").into_response()
    }
    let app = Router::new()
        .route("/v1/models", get(models))
        .route("/v1/chat/completions", post(chat))
        .route(
            "/v1/embeddings",
            post(|| async { Json(json!({"data":[{"index":0,"embedding":[0.1,0.9]}]})) }),
        );
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = l.local_addr().unwrap();
    let task = tokio::spawn(async move { axum::serve(l, app).await.unwrap() });
    (format!("http://{addr}/v1"), task)
}
#[tokio::test]
async fn api_contract_errors_and_embeddings() {
    let (base, task) = mock().await;
    let mut c = Config {
        base_url: base,
        chat_model: "mock".into(),
        embedding_model: "embed".into(),
        ..Config::default()
    };
    let client = Client::new(c.clone(), String::new()).unwrap();
    assert_eq!(client.test().await.unwrap(), ["mock"]);
    assert_eq!(
        client
            .complete(vec![Message {
                role: "user".into(),
                content: "Hallo".into()
            }])
            .await
            .unwrap()
            .concat(),
        "Grüezi Welt"
    );
    assert_eq!(
        client.embeddings(vec!["text".into()]).await.unwrap()[0].len(),
        2
    );
    let e = Client::new(c.clone(), "wrong".into())
        .unwrap()
        .test()
        .await
        .unwrap_err()
        .to_string();
    assert!(e.contains("API-Key ungültig"));
    assert!(!e.contains("wrong"));
    c.chat_model = "missing".into();
    assert!(
        Client::new(c.clone(), String::new())
            .unwrap()
            .test()
            .await
            .is_err()
    );
    c.chat_model = "broken".into();
    assert!(
        Client::new(c.clone(), String::new())
            .unwrap()
            .complete(vec![])
            .await
            .is_err()
    );
    c.chat_model = "slow".into();
    assert!(
        Client::with_timeout(c, String::new(), std::time::Duration::from_millis(20))
            .unwrap()
            .complete(vec![])
            .await
            .unwrap_err()
            .to_string()
            .contains("Zeitüberschreitung")
    );
    task.abort();
}
#[test]
fn cloud_consent_and_url() {
    assert!(fm_ai::validate_url("http://example.org/v1").is_err());
    assert!(fm_ai::validate_url("https://key:secret@example.org/v1").is_err());
    let c = Config {
        base_url: "https://example.org/v1".into(),
        ..Config::default()
    };
    assert!(Client::new(c, String::new()).is_err());
}
