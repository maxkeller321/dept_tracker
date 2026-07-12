mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn export_import_round_trip() {
    let app = common::app().await;
    let cookie = common::session_cookie(&app).await;
    let create = serde_json::json!({
        "label": "Round Trip",
        "setup_mode": "quick",
        "remaining_balance_minor": 3000000,
        "payment_frequency": "monthly",
        "payment_type": "tilgung_euro",
        "tilgung_euro_minor": 50000,
        "apr_basis_points": 300
    });
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/loans")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(create.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let export_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/export")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(export_res.status(), StatusCode::OK);
    let export_bytes = export_res.into_body().collect().await.unwrap().to_bytes();
    let bundle: serde_json::Value = serde_json::from_slice(&export_bytes).unwrap();
    assert_eq!(bundle["schema_version"], 1);
    assert_eq!(bundle["loans"].as_array().unwrap().len(), 1);

    let import_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/import?confirm=true")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(export_bytes.to_vec()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(import_res.status(), StatusCode::OK);

    let dash = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/dashboard")
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let dash_bytes = dash.into_body().collect().await.unwrap().to_bytes();
    let dash_json: serde_json::Value = serde_json::from_slice(&dash_bytes).unwrap();
    assert_eq!(dash_json["loans"].as_array().unwrap().len(), 1);
    assert_eq!(dash_json["loans"][0]["label"], "Round Trip");
}
