use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

pub async fn app() -> axum::Router {
    let db = db::test_support::test_pool().await;
    api::router::app(api::AppState { pool: db.pool }, None)
}

pub async fn session_cookie(app: &axum::Router) -> String {
    let body = serde_json::json!({ "username": "test", "password": "test" });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    response
        .headers()
        .get("set-cookie")
        .expect("set-cookie")
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string()
}
