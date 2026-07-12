mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn seed_loan(app: &axum::Router, cookie: &str) -> String {
    let body = serde_json::json!({
        "label": "Test",
        "setup_mode": "quick",
        "remaining_balance_minor": 1000000,
        "payment_frequency": "monthly",
        "payment_type": "tilgung_euro",
        "tilgung_euro_minor": 50000,
        "apr_basis_points": 350
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
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    json["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn immediate_sonderzahlung_reduces_balance() {
    let app = common::app().await;
    let cookie = common::session_cookie(&app).await;
    let id = seed_loan(&app, &cookie).await;
    let body = serde_json::json!({
        "amount_minor": 100000,
        "paid_at": "2025-06-01",
        "confirm_overpayment": true
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/loans/{id}/sonderzahlungen/immediate"))
                .header("content-type", "application/json")
                .header("cookie", cookie)
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn backdated_sonderzahlung_rebuilds_balance() {
    let app = common::app().await;
    let cookie = common::session_cookie(&app).await;
    let start = chrono::Utc::now()
        .date_naive()
        .checked_sub_months(chrono::Months::new(6))
        .unwrap()
        .to_string();
    let create_body = serde_json::json!({
        "label": "Backdated",
        "setup_mode": "advanced",
        "remaining_balance_minor": 1_000_000,
        "original_principal_minor": 1_000_000,
        "payment_frequency": "monthly",
        "payment_type": "tilgung_euro",
        "tilgung_euro_minor": 50_000,
        "apr_basis_points": 400,
        "loan_start_date": start
    });
    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/loans")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(create_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);
    let created: serde_json::Value =
        serde_json::from_slice(&create.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let id = created["id"].as_str().unwrap();

    let dash = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/dashboard")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let dash_json: serde_json::Value =
        serde_json::from_slice(&dash.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let balance_before = dash_json["loans"][0]["remaining_balance"]["amount_minor"]
        .as_i64()
        .unwrap();

    let past = chrono::Utc::now()
        .date_naive()
        .checked_sub_months(chrono::Months::new(2))
        .unwrap()
        .to_string();
    let body = serde_json::json!({
        "amount_minor": 100_000,
        "paid_at": past,
        "confirm_overpayment": true,
        "recalculate_from_past": true
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/loans/{id}/sonderzahlungen/immediate"))
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let detail: serde_json::Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let balance_after = detail["remaining_balance"]["amount_minor"].as_i64().unwrap();
    assert!(balance_after < balance_before);

    let payments = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/loans/{id}/payments"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let pay_json: Vec<serde_json::Value> =
        serde_json::from_slice(&payments.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert!(
        pay_json
            .iter()
            .any(|p| p["event_type"] == "sonderzahlung" && p["paid_at"] == past)
    );
}

#[tokio::test]
async fn schedule_future_sonderzahlung_stays_pending() {
    let app = common::app().await;
    let cookie = common::session_cookie(&app).await;
    let id = seed_loan(&app, &cookie).await;
    let future = chrono::Utc::now()
        .date_naive()
        .checked_add_months(chrono::Months::new(2))
        .unwrap()
        .to_string();
    let body = serde_json::json!({
        "amount_minor": 50_000,
        "due_date": future
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/loans/{id}/sonderzahlungen/scheduled"))
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let scheduled: serde_json::Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(scheduled["due_date"], future);
    assert_eq!(scheduled["status"], "pending");

    let detail = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/loans/{id}"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let detail_json: serde_json::Value =
        serde_json::from_slice(&detail.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let upcoming = detail_json["upcoming_scheduled"]
        .as_array()
        .expect("upcoming scheduled");
    assert!(upcoming.iter().any(|s| s["due_date"] == future));
}

#[tokio::test]
async fn plan_past_date_equivalent_to_backdated_immediate() {
    let app = common::app().await;
    let cookie = common::session_cookie(&app).await;
    let start = chrono::Utc::now()
        .date_naive()
        .checked_sub_months(chrono::Months::new(4))
        .unwrap()
        .to_string();
    let create_body = serde_json::json!({
        "label": "Plan Past",
        "setup_mode": "advanced",
        "remaining_balance_minor": 800_000,
        "original_principal_minor": 800_000,
        "payment_frequency": "monthly",
        "payment_type": "tilgung_euro",
        "tilgung_euro_minor": 40_000,
        "apr_basis_points": 350,
        "loan_start_date": start
    });
    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/loans")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(create_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);
    let created: serde_json::Value =
        serde_json::from_slice(&create.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let id = created["id"].as_str().unwrap();

    let past = chrono::Utc::now()
        .date_naive()
        .checked_sub_months(chrono::Months::new(1))
        .unwrap()
        .to_string();
    let body = serde_json::json!({
        "amount_minor": 75_000,
        "paid_at": past,
        "confirm_overpayment": true,
        "recalculate_from_past": true
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/loans/{id}/sonderzahlungen/immediate"))
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let payments = app
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
    let pay_json: Vec<serde_json::Value> =
        serde_json::from_slice(&payments.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert!(
        pay_json
            .iter()
            .any(|p| p["event_type"] == "sonderzahlung" && p["paid_at"] == past)
    );

    let detail = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/loans/{id}"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let detail_json: serde_json::Value =
        serde_json::from_slice(&detail.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let upcoming = detail_json["upcoming_scheduled"].as_array().unwrap();
    assert!(upcoming.is_empty());
}
