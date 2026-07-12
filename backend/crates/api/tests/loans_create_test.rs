mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn create_loan_quick() {
    let app = common::app().await;
    let cookie = common::session_cookie(&app).await;
    let body = serde_json::json!({
        "label": "Mortgage",
        "setup_mode": "quick",
        "remaining_balance_minor": 20000000,
        "payment_frequency": "monthly",
        "payment_type": "tilgung_euro",
        "tilgung_euro_minor": 120000,
        "apr_basis_points": 375
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/loans")
                .header("content-type", "application/json")
                .header("cookie", cookie)
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8_lossy(&bytes);
    assert_eq!(status, StatusCode::OK, "body: {text}");
}

#[tokio::test]
async fn create_loan_tilgung_percent_infers_original_principal() {
    let app = common::app().await;
    let cookie = common::session_cookie(&app).await;
    let today = chrono::Utc::now().date_naive().to_string();
    let body = serde_json::json!({
        "label": "Percent Without Original",
        "setup_mode": "advanced",
        "remaining_balance_minor": 10_000_000,
        "payment_frequency": "monthly",
        "payment_type": "tilgung_percent",
        "tilgung_percent_basis_points": 200,
        "apr_basis_points": 500,
        "loan_start_date": today,
        "first_payment_date": today
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/loans")
                .header("content-type", "application/json")
                .header("cookie", cookie)
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8_lossy(&bytes);
    assert_eq!(status, StatusCode::OK, "body: {text}");
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["payment_type"], "tilgung_percent");
    assert_eq!(json["tilgung_percent_basis_points"], 200);
}
