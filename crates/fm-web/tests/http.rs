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
        .extension(axum::extract::ConnectInfo(
            "127.0.0.1:50000".parse::<std::net::SocketAddr>().unwrap(),
        ))
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
            serde_json::json!({"path":data,"config":{"stable_seconds":0}})
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
            serde_json::json!({"path":data,"config":{"stable_seconds":0}})
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
                .extension(axum::extract::ConnectInfo(
                    "127.0.0.1:50000".parse::<std::net::SocketAddr>().unwrap(),
                ))
                .header("host", "evil.example:8484")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn unknown_question_needs_no_ai_and_sources_stay_private() {
    let t = tempfile::tempdir().unwrap();
    let data = t.path().join("data");
    std::fs::create_dir(&data).unwrap();
    let s = AppState::new(t.path().join("pointer")).unwrap();
    let csrf = s.csrf.clone();
    let r = router(s);
    request(
        r.clone(),
        "POST",
        "/api/setup",
        &csrf,
        serde_json::json!({"path":data,"config":{"stable_seconds":0}}),
    )
    .await;
    let response = r
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/chat")
                .extension(axum::extract::ConnectInfo(
                    "127.0.0.1:50000".parse::<std::net::SocketAddr>().unwrap(),
                ))
                .header("host", "127.0.0.1:8484")
                .header("content-type", "application/json")
                .header("x-csrf-token", &csrf)
                .body(Body::from(
                    serde_json::json!({"question":"Quantenbanane","mode":"erklaeren"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1_000_000).await.unwrap();
    assert!(String::from_utf8_lossy(&body).contains("keine belegte Antwort"));
    assert_eq!(
        request(
            r,
            "GET",
            "/api/source?id=../../etc/passwd",
            "",
            serde_json::Value::Null
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn remote_whitelist_login_and_local_administration() {
    let t = tempfile::tempdir().unwrap();
    let data = t.path().join("data");
    std::fs::create_dir(&data).unwrap();
    let s = AppState::new(t.path().join("pointer")).unwrap();
    let csrf = s.csrf.clone();
    let r = router(s.clone());
    request(
        r.clone(),
        "POST",
        "/api/setup",
        &csrf,
        serde_json::json!({"path":data}),
    )
    .await;
    {
        let l = s.lib().unwrap();
        let mut c = l.config.write().unwrap();
        c.network_enabled = true;
        c.allowed_ips = vec!["10.0.0.0/24".into()];
        c.password_hash = fm_access::password_hash("Ein sicheres Passwort").unwrap();
    }
    async fn remote(
        r: axum::Router,
        ip: &str,
        method: &str,
        path: &str,
        csrf: &str,
        cookie: &str,
        body: serde_json::Value,
    ) -> axum::response::Response {
        r.oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .extension(axum::extract::ConnectInfo(
                    format!("{ip}:50000")
                        .parse::<std::net::SocketAddr>()
                        .unwrap(),
                ))
                .header("host", "192.168.1.1:8485")
                .header("content-type", "application/json")
                .header("x-csrf-token", csrf)
                .header("cookie", cookie)
                .header("x-forwarded-for", "127.0.0.1")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
    }
    let null = serde_json::Value::Null;
    assert_eq!(
        remote(
            r.clone(),
            "192.0.2.5",
            "GET",
            "/api/documents",
            "",
            "",
            null.clone()
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        remote(
            r.clone(),
            "10.0.0.2",
            "GET",
            "/api/documents",
            "",
            "",
            null.clone()
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        remote(
            r.clone(),
            "10.0.0.2",
            "POST",
            "/api/settings",
            &csrf,
            "",
            serde_json::json!({})
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        remote(
            r.clone(),
            "10.0.0.2",
            "POST",
            "/api/login",
            "",
            "",
            serde_json::json!({"password":"Ein sicheres Passwort"})
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    let login = remote(
        r.clone(),
        "10.0.0.2",
        "POST",
        "/api/login",
        &csrf,
        "",
        serde_json::json!({"password":"Ein sicheres Passwort"}),
    )
    .await;
    assert_eq!(login.status(), StatusCode::OK);
    let cookie = login
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(cookie.contains("Secure; HttpOnly; SameSite=Strict"));
    assert_eq!(
        remote(
            r.clone(),
            "10.0.0.2",
            "GET",
            "/api/documents",
            "",
            &cookie,
            null.clone()
        )
        .await
        .status(),
        StatusCode::OK
    );
    assert_eq!(
        remote(
            r.clone(),
            "10.0.0.3",
            "GET",
            "/api/documents",
            "",
            &cookie,
            null.clone()
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        remote(
            r.clone(),
            "10.0.0.2",
            "POST",
            "/api/access/allow",
            &csrf,
            &cookie,
            serde_json::json!({"ip":"1.1.1.1","confirmed":true})
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    let response = r
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/settings")
                .extension(axum::extract::ConnectInfo(
                    "[::1]:50000".parse::<std::net::SocketAddr>().unwrap(),
                ))
                .header("host", "[::1]:8484")
                .header("x-csrf-token", csrf)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(
        !s.lib()
            .unwrap()
            .catalog
            .rows("SELECT ip FROM access_log WHERE allowed=0")
            .unwrap()
            .is_empty()
    );
}
