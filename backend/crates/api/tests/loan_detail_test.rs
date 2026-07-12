mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn seed_percent_loan(app: &axum::Router, cookie: &str) -> String {
    let body = serde_json::json!({
        "label": "Percent Loan",
        "setup_mode": "quick",
        "remaining_balance_minor": 10000000,
        "original_principal_minor": 10000000,
        "payment_frequency": "monthly",
        "payment_type": "tilgung_percent",
        "tilgung_percent_basis_points": 200,
        "apr_basis_points": 375
    });
    let response = app
        .clone()
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
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn loan_detail_includes_interest_fields() {
    let app = common::app().await;
    let cookie = common::session_cookie(&app).await;
    let id = seed_percent_loan(&app, &cookie).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/loans/{id}"))
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json.get("interest_paid_to_date").is_some());
    assert!(json.get("interest_remaining_estimate").is_some());
}
