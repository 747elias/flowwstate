use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use fm_web::{AppState, router};
use tower::ServiceExt;
async fn request(
    r: axum::Router,
    method: &str,
    path: &str,
    csrf: &str,
    value: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .method(method)
        .uri(path)
        .header("host", "127.0.0.1:8484")
        .header("content-type", "application/json")
        .header("x-csrf-token", csrf)
        .body(Body::from(value.to_string()))
        .unwrap();
    let response = r.oneshot(req).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 10_000_000).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or_default())
}
#[tokio::test]
async fn setup_scan_search_and_dedup() {
    let t = tempfile::tempdir().unwrap();
    let data = t.path().join("Daten mit Leerzeichen");
    std::fs::create_dir(&data).unwrap();
    let s = AppState::new(t.path().join("profile/pointer")).unwrap();
    let csrf = s.csrf.clone();
    let r = router(s.clone());
    assert_eq!(
        request(
            r.clone(),
            "POST",
            "/api/setup",
            "",
            serde_json::json!({"path":data})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            r.clone(),
            "POST",
            "/api/setup",
            &csrf,
            serde_json::json!({"path":data})
        )
        .await
        .0,
        StatusCode::OK
    );
    let lib = s.lib().unwrap();
    let p = "unterlagen/1-lehrjahr/modul-117-netzwerk/kapitel-01-grundlagen/test.txt";
    lib.guard
        .upload(p, b"Ethernet verbindet lokale Netzwerke")
        .unwrap();
    for _ in 0..2 {
        assert_eq!(
            request(r.clone(), "POST", "/api/scan", &csrf, serde_json::json!({}))
                .await
                .0,
            StatusCode::OK
        );
    }
    assert_eq!(lib.catalog.documents().unwrap().len(), 1);
    assert_eq!(lib.catalog.chunks().unwrap().len(), 1);
    assert!(
        lib.catalog
            .rows("SELECT CAST(COUNT(*) AS TEXT) FROM document_versions")
            .unwrap()[0][0]
            == "0"
    );
    let (status, result) = request(
        r,
        "GET",
        "/api/search?q=Ethernet",
        "",
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(result[0]["text"].as_str().unwrap().contains("Ethernet"));
    assert!(lib.guard.write("../escape", b"x").is_err());
}
#[tokio::test]
async fn rebinding_rejected() {
    let t = tempfile::tempdir().unwrap();
    let s = AppState::new(t.path().join("pointer")).unwrap();
    let response = router(s)
        .oneshot(
            Request::builder()
                .uri("/api/state")
                .header("host", "evil.example:8484")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
