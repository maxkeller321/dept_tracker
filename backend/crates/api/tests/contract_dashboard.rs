mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn dashboard_response_has_required_fields() {
    let app = common::app().await;
    let cookie = common::session_cookie(&app).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/dashboard")
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("household").is_some());
    assert!(json["household"].get("total_balance").is_some());
    assert!(json["household"].get("total_monthly_obligation").is_some());
    assert!(json.get("loans").is_some());
}
