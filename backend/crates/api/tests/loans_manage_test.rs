mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn seed(app: &axum::Router, cookie: &str) -> String {
    let body = serde_json::json!({
        "label": "Manage Test",
        "setup_mode": "quick",
        "remaining_balance_minor": 5000000,
        "payment_frequency": "monthly",
        "payment_type": "tilgung_euro",
        "tilgung_euro_minor": 100000,
        "apr_basis_points": 400
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
async fn patch_archive_delete_and_list_payments() {
    let app = common::app().await;
    let cookie = common::session_cookie(&app).await;
    let id = seed(&app, &cookie).await;

    let patch = serde_json::json!({ "label": "Renamed Loan" });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/loans/{id}"))
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(patch.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let pay = serde_json::json!({ "amount_minor": 100000, "paid_at": "2025-06-01" });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/loans/{id}/payments"))
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(pay.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/loans/{id}/payments"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let list: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert!(!list.is_empty());

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/loans/{id}/archive"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/loans/{id}?confirm=true"))
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}
