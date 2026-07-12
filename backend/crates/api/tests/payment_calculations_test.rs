mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{Months, NaiveDate, Utc};
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn create_loan(app: &axum::Router, cookie: &str, body: serde_json::Value) -> String {
    let create = app
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
    assert_eq!(create.status(), StatusCode::OK);
    let created: serde_json::Value =
        serde_json::from_slice(&create.into_body().collect().await.unwrap().to_bytes()).unwrap();
    created["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn first_payment_date_delays_auto_catch_up() {
    let app = common::app().await;
    let cookie = common::session_cookie(&app).await;
    let today = Utc::now().date_naive();
    let loan_start = today
        .checked_sub_months(Months::new(6))
        .unwrap()
        .to_string();
    let first_payment = today
        .checked_sub_months(Months::new(1))
        .unwrap()
        .to_string();

    let id = create_loan(
        &app,
        &cookie,
        serde_json::json!({
            "label": "Delayed First Rate",
            "setup_mode": "advanced",
            "remaining_balance_minor": 1_000_000,
            "original_principal_minor": 1_000_000,
            "payment_frequency": "monthly",
            "payment_type": "tilgung_euro",
            "tilgung_euro_minor": 50_000,
            "apr_basis_points": 400,
            "loan_start_date": loan_start,
            "first_payment_date": first_payment
        }),
    )
    .await;

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
    let first_naive = NaiveDate::parse_from_str(&first_payment, "%Y-%m-%d").unwrap();
    assert!(
        pay_json.len() < 6,
        "must not backfill from loan_start ({loan_start}); got {} payments",
        pay_json.len()
    );
    for p in &pay_json {
        let paid_at = NaiveDate::parse_from_str(p["paid_at"].as_str().unwrap(), "%Y-%m-%d").unwrap();
        assert!(paid_at >= first_naive, "payment before first_payment_date");
    }
    assert!(!pay_json.is_empty());
}

#[tokio::test]
async fn regular_payment_splits_interest_and_principal() {
    let app = common::app().await;
    let cookie = common::session_cookie(&app).await;
    let first = Utc::now()
        .date_naive()
        .checked_sub_months(Months::new(1))
        .unwrap()
        .to_string();

    let id = create_loan(
        &app,
        &cookie,
        serde_json::json!({
            "label": "Split Test",
            "setup_mode": "advanced",
            "remaining_balance_minor": 10_000_000,
            "original_principal_minor": 10_000_000,
            "payment_frequency": "monthly",
            "payment_type": "tilgung_euro",
            "tilgung_euro_minor": 500_000,
            "apr_basis_points": 400,
            "loan_start_date": first,
            "first_payment_date": first
        }),
    )
    .await;

    let _ = app
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

    let payments = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/loans/{id}/payments"))
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let pay_json: Vec<serde_json::Value> =
        serde_json::from_slice(&payments.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert!(!pay_json.is_empty());
    let p = &pay_json[0];
    let interest = p["interest_portion_minor"].as_i64().unwrap();
    let principal = p["principal_portion_minor"].as_i64().unwrap();
    assert!(interest > 0);
    assert_eq!(principal, 500_000);
    assert_eq!(interest + principal, p["amount_minor"].as_i64().unwrap());
}

#[tokio::test]
async fn sonderzahlung_reduces_balance() {
    let app = common::app().await;
    let cookie = common::session_cookie(&app).await;
    let today = Utc::now().date_naive().to_string();

    let id = create_loan(
        &app,
        &cookie,
        serde_json::json!({
            "label": "Sonder Test",
            "setup_mode": "advanced",
            "remaining_balance_minor": 5_000_000,
            "payment_frequency": "monthly",
            "payment_type": "tilgung_euro",
            "tilgung_euro_minor": 100_000,
            "apr_basis_points": 300,
            "loan_start_date": today,
            "first_payment_date": today
        }),
    )
    .await;

    let before = app
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
    let before_json: serde_json::Value =
        serde_json::from_slice(&before.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let balance_before = before_json["remaining_balance"]["amount_minor"].as_i64().unwrap();

    let extra = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/loans/{id}/sonderzahlungen/immediate"))
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(
                    serde_json::json!({
                        "amount_minor": 500_000,
                        "paid_at": today,
                        "confirm_overpayment": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(extra.status(), StatusCode::OK);

    let after = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/loans/{id}"))
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let after_json: serde_json::Value =
        serde_json::from_slice(&after.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let balance_after = after_json["remaining_balance"]["amount_minor"].as_i64().unwrap();
    assert_eq!(balance_after, balance_before - 500_000);
}

#[tokio::test]
async fn percent_tilgung_yearly_applies_annual_principal() {
    let app = common::app().await;
    let cookie = common::session_cookie(&app).await;
    let first = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().to_string();

    let id = create_loan(
        &app,
        &cookie,
        serde_json::json!({
            "label": "Yearly Percent",
            "setup_mode": "advanced",
            "remaining_balance_minor": 10_000_000,
            "original_principal_minor": 10_000_000,
            "payment_frequency": "yearly",
            "payment_type": "tilgung_percent",
            "tilgung_percent_basis_points": 200,
            "apr_basis_points": 400,
            "loan_start_date": first,
            "first_payment_date": first
        }),
    )
    .await;

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
    assert!(detail_json["projected_payoff_date"].as_str().is_some());
    assert!(detail_json["periodic_payment"]["amount_minor"].as_i64().unwrap() > 200_000);
}
