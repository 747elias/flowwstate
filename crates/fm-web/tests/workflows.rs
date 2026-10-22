use axum::{
    Json,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    response::IntoResponse,
};
use fm_catalog::Collection;
use fm_web::{AppState, router};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn send(
    s: &AppState,
    method: &str,
    path: &str,
    value: Value,
    csrf: bool,
) -> (StatusCode, String) {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .extension(axum::extract::ConnectInfo(
            "127.0.0.1:50000".parse::<std::net::SocketAddr>().unwrap(),
        ))
        .header("host", "127.0.0.1:8484")
        .header("content-type", "application/json")
        .header("x-csrf-token", if csrf { s.csrf.as_str() } else { "" })
        .body(Body::from(value.to_string()))
        .unwrap();
    let response = router(s.clone()).oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 20_000_000).await.unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}
async fn api(s: &AppState, path: &str, value: Value) -> Value {
    let (status, body) = send(s, "POST", path, value, true).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    serde_json::from_str(&body).unwrap()
}
fn done(body: &str) -> Value {
    let part = body.split("event: done\n").nth(1).unwrap();
    serde_json::from_str(part.lines().find_map(|l| l.strip_prefix("data: ")).unwrap()).unwrap()
}
async fn initialized() -> (tempfile::TempDir, AppState) {
    let t = tempfile::tempdir().unwrap();
    let data = t.path().join("Daten mit Leerzeichen");
    std::fs::create_dir(&data).unwrap();
    let s = AppState::new(t.path().join("profile/pointer")).unwrap();
    api(
        &s,
        "/api/setup",
        json!({"path":data,"name":"Bestehender Nutzer","config":{"stable_seconds":0}}),
    )
    .await;
    (t, s)
}
async fn mock() -> (String, tokio::task::JoinHandle<()>) {
    async fn completion(Json(value): Json<Value>) -> axum::response::Response {
        assert!(value.get("tools").is_none());
        let messages = value["messages"].as_array().unwrap();
        let system = messages[0]["content"].as_str().unwrap();
        let input = messages.last().unwrap()["content"].as_str().unwrap();
        let response = if system.starts_with("Ordne genau") {
            if value["model"] == "escape" {
                json!({"target_directory":"../ausbruch","confidence":1.0,"reason":"Angriff","question":""}).to_string()
            } else if value["model"] == "low" {
                json!({"target_directory":"unterlagen/unklar","confidence":0.4,"reason":"Nicht eindeutig","question":"Welches Fach ist gemeint?"}).to_string()
            } else {
                json!({"target_directory":"unterlagen/neues-thema/ki","confidence":0.99,"reason":"Eindeutiger Text und Kontext","question":""}).to_string()
            }
        } else if system.contains("Agent-Builder von Folder-Mind") {
            json!({"message":"Hier ist dein Entwurf. Bitte prüfe und bestätige ihn.","ready":true,"agent":{"name":"Prüfungscoach","description":"Karteikarten aus Unterlagen","instructions":"Erstelle kurze Karteikarten aus den gelieferten Quellen.","grounded":true,"scope":""}}).to_string()
        } else if system.contains("ausschliesslich der Ablage-Agent") {
            let input: Value = serde_json::from_str(input).unwrap();
            let file = input["dateien"]
                .as_array()
                .unwrap()
                .iter()
                .find(|f| f["path"] == "unterlagen/wissen.txt")
                .unwrap();
            json!({"message":"Bitte bestätige die vorgeschlagene Verschiebung.","operation":"move","source_id":file["id"],"target_directory":"unterlagen/neuer-ordner","matches":[]}).to_string()
        } else {
            let id = input
                .split("\"id\":\"")
                .nth(1)
                .unwrap()
                .split('"')
                .next()
                .unwrap();
            format!(
                "# Überblick\n\n**Ethernet** verbindet Computer. [{id}]\n\n- Ein Kernpunkt\n\n```rust\nlet test = 1;\n```\n\n<script>alert(1)</script>"
            )
        };
        let event = json!({"choices":[{"delta":{"content":response}}]});
        (
            [("content-type", "text/event-stream")],
            format!("data: {event}\n\ndata: [DONE]\n\n"),
        )
            .into_response()
    }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            axum::Router::new().route("/v1/chat/completions", axum::routing::post(completion)),
        )
        .await
        .unwrap();
    });
    (format!("http://{addr}/v1"), task)
}
fn connect(s: &AppState, url: &str, model: &str) {
    let l = s.lib().unwrap();
    let mut c = l.config.write().unwrap();
    c.base_url = url.into();
    c.chat_model = model.into();
    c.save(&l.guard).unwrap();
}

#[tokio::test]
async fn classifier_uses_ai_but_never_trusts_outside_targets_or_low_confidence() {
    let (_t, s) = initialized().await;
    let (url, server) = mock().await;
    connect(&s, &url, "mock");
    let l = s.lib().unwrap();
    l.guard
        .upload("input/ethernet.txt", b"Ethernet Netzwerke")
        .unwrap();
    fm_web::inbox::process(&s).await.unwrap();
    assert!(
        l.guard
            .resolve("unterlagen/neues-thema/ki/ethernet.txt")
            .unwrap()
            .exists()
    );
    connect(&s, &url, "low");
    l.guard
        .upload("input/unsicher.md", b"Noch ein anderes Thema")
        .unwrap();
    fm_web::inbox::process(&s).await.unwrap();
    let items = l
        .catalog
        .entries::<fm_core::InboxItem>(Collection::Inbox)
        .unwrap();
    assert!(
        items
            .iter()
            .any(|i| i.question == "Welches Fach ist gemeint?")
    );
    assert!(l.guard.resolve("input/unsicher.md").unwrap().exists());
    connect(&s, &url, "escape");
    l.guard
        .upload("input/angriff.txt", b"Nicht ausbrechen")
        .unwrap();
    fm_web::inbox::process(&s).await.unwrap();
    assert!(l.guard.resolve("input/angriff.txt").unwrap().exists());
    assert!(!l.guard.root().join("ausbruch").exists());
    assert_eq!(l.catalog.entry_count(Collection::Activity).unwrap(), 1);
    server.abort();
}

#[tokio::test]
async fn organizer_finds_a_storage_location_by_content_without_a_model() {
    let (_t, s) = initialized().await;
    let l = s.lib().unwrap();
    l.guard
        .upload("unterlagen/a42.txt", b"Ethernet verbindet Computer.")
        .unwrap();
    l.scan(false).unwrap();
    let (status, body) = send(
        &s,
        "POST",
        "/api/chat",
        json!({"agent_id":"ablage","question":"Wo liegt Ethernet?"}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let result = done(&body);
    assert!(
        result["html"]
            .as_str()
            .unwrap()
            .contains("unterlagen/a42.txt")
    );
    assert!(result["proposals"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn organizer_and_builder_require_confirmation_and_reject_changed_files() {
    let (_t, s) = initialized().await;
    let (url, server) = mock().await;
    connect(&s, &url, "mock");
    let l = s.lib().unwrap();
    l.guard
        .upload("unterlagen/wissen.txt", b"Ethernet Wissen")
        .unwrap();
    let (status,body)=send(&s,"POST","/api/chat",json!({"agent_id":"ablage","question":"Verschiebe wissen.txt in einen neuen Ordner","save":true}),true).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let result = done(&body);
    let proposal = &result["proposals"][0];
    assert!(l.guard.resolve("unterlagen/wissen.txt").unwrap().exists());
    assert!(
        !l.guard
            .resolve("unterlagen/neuer-ordner/wissen.txt")
            .unwrap()
            .exists()
    );
    assert_eq!(
        send(
            &s,
            "POST",
            "/api/actions/confirm",
            json!({"id":proposal["id"]}),
            false
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    l.guard
        .write("unterlagen/wissen.txt", b"Zwischenzeitliche Aenderung")
        .unwrap();
    assert_eq!(
        send(
            &s,
            "POST",
            "/api/actions/confirm",
            json!({"id":proposal["id"]}),
            true
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (_, body) = send(
        &s,
        "POST",
        "/api/chat",
        json!({"agent_id":"ablage","question":"Verschiebe wissen.txt","save":true}),
        true,
    )
    .await;
    let result = done(&body);
    let proposal = &result["proposals"][0];
    api(&s, "/api/actions/confirm", json!({"id":proposal["id"]})).await;
    assert_eq!(
        std::fs::read(
            l.guard
                .resolve("unterlagen/neuer-ordner/wissen.txt")
                .unwrap()
        )
        .unwrap(),
        b"Zwischenzeitliche Aenderung"
    );
    assert_eq!(
        send(
            &s,
            "POST",
            "/api/actions/confirm",
            json!({"id":proposal["id"]}),
            true
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (_,body)=send(&s,"POST","/api/chat",json!({"agent_id":"agent-builder","question":"Erstelle einen Karteikarten-Agenten","save":true}),true).await;
    let result = done(&body);
    let proposal = &result["proposals"][0];
    assert_eq!(l.catalog.entry_count(Collection::Agents).unwrap(), 0);
    api(&s, "/api/actions/confirm", json!({"id":proposal["id"]})).await;
    let agent = l
        .catalog
        .entries::<fm_core::Agent>(Collection::Agents)
        .unwrap()
        .remove(0);
    assert_eq!(agent.name, "Prüfungscoach");
    assert_eq!(
        send(
            &s,
            "POST",
            "/api/agents",
            json!({"id":"ablage","name":"Ueberschreiben","instructions":"Test"}),
            true
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(l.catalog.entry_count(Collection::Activity).unwrap(), 1);
    let guard = l.guard.clone();
    drop(l);
    drop(s);
    let reopened = fm_web::Library::open(guard).unwrap();
    assert_eq!(
        reopened
            .catalog
            .entries::<fm_core::Agent>(Collection::Agents)
            .unwrap()[0]
            .name,
        "Prüfungscoach"
    );
    server.abort();
}

#[tokio::test]
async fn summary_and_saved_chats_render_markdown_without_running_html() {
    let (_t, s) = initialized().await;
    let (url, server) = mock().await;
    connect(&s, &url, "mock");
    let l = s.lib().unwrap();
    l.guard
        .upload("unterlagen/netzwerk.txt", b"Ethernet verbindet Computer.")
        .unwrap();
    l.scan(false).unwrap();
    let (status, body) = send(
        &s,
        "POST",
        "/api/chat",
        json!({"agent_id":"zusammenfassen","question":"Fasse alles zusammen","save":true}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let result = done(&body);
    let html = result["html"].as_str().unwrap();
    assert!(html.contains("<h1>Überblick</h1>"));
    assert!(html.contains("<strong>Ethernet</strong>"));
    assert!(html.contains("/api/source?id="));
    assert!(!html.contains("<script>"));
    let (_, body) = send(&s, "GET", "/api/chats", Value::Null, true).await;
    let history: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(history["messages"].as_array().unwrap().len(), 2);
    assert!(
        history["rendered"][1]
            .as_str()
            .unwrap()
            .contains("<h1>Überblick</h1>")
    );
    assert_eq!(history["chat_agents"][0][1], "zusammenfassen");
    server.abort();
}

#[tokio::test]
async fn update_preparation_saves_new_workflows_and_rejects_further_writes() {
    let (_t, s) = initialized().await;
    let l = s.lib().unwrap();
    l.guard
        .upload("input/noch-nicht-sortiert.bin", b"Bleibt im Eingang")
        .unwrap();
    api(&s,"/api/agents",json!({"name":"Eigener Agent","instructions":"Beantworte Fragen aus Quellen.","grounded":true})).await;
    assert_eq!(
        send(
            &s,
            "POST",
            "/api/update/prepare",
            json!({"confirmed":false}),
            true
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert!(!s.paused.load(std::sync::atomic::Ordering::Relaxed));
    let result = api(&s, "/api/update/prepare", json!({"confirmed":true})).await;
    assert!(
        l.guard
            .resolve(format!("backups/{}", result["backup"].as_str().unwrap()))
            .unwrap()
            .exists()
    );
    let archive = std::fs::File::open(
        l.guard
            .resolve(format!("backups/{}", result["backup"].as_str().unwrap()))
            .unwrap(),
    )
    .unwrap();
    let mut archive = zip::ZipArchive::new(archive).unwrap();
    assert!(archive.by_name("input/noch-nicht-sortiert.bin").is_ok());
    assert_eq!(
        send(
            &s,
            "POST",
            "/api/agents",
            json!({"name":"Zu spaet","instructions":"Test"}),
            true
        )
        .await
        .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    let guard = l.guard.clone();
    drop(l);
    drop(s);
    let reopened = fm_web::Library::open(guard).unwrap();
    assert_eq!(reopened.catalog.entry_count(Collection::Agents).unwrap(), 1);
    assert!(
        reopened
            .guard
            .resolve("input/noch-nicht-sortiert.bin")
            .unwrap()
            .exists()
    );
}
