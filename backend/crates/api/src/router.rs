use std::path::PathBuf;

use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::middleware::require_auth;
use crate::routes::{auth, backup, dashboard, loans, payments, sonderzahlungen};
use crate::AppState;

pub fn app(state: AppState, static_dir: Option<PathBuf>) -> Router {
    let api = Router::new()
        .route("/health", get(health))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/status", get(auth::status))
        .route("/dashboard", get(dashboard::get_dashboard))
        .route("/amortization", get(dashboard::combined_amortization))
        .route("/loans", post(loans::create_loan))
        .route(
            "/loans/:id",
            get(loans::loan_detail)
                .patch(loans::update_loan)
                .delete(loans::delete_loan),
        )
        .route("/loans/:id/archive", post(loans::archive_loan))
        .route("/loans/:id/amortization", get(loans::loan_amortization))
        .route("/loans/:id/payments", get(payments::list_payments).post(payments::record_payment))
        .route(
            "/loans/:id/sonderzahlungen/immediate",
            post(sonderzahlungen::immediate),
        )
        .route(
            "/loans/:id/sonderzahlungen/scheduled",
            get(sonderzahlungen::list_scheduled).post(sonderzahlungen::schedule),
        )
        .route(
            "/loans/:id/sonderzahlungen/scheduled/:schedule_id",
            axum::routing::delete(sonderzahlungen::cancel_scheduled),
        )
        .route("/export", get(backup::export_data))
        .route("/import", post(backup::import_data))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let mut router = Router::new().nest("/api/v1", api).with_state(state);

    if let Some(dir) = static_dir {
        let index = dir.join("index.html");
        router = router.fallback_service(
            ServeDir::new(dir).not_found_service(ServeFile::new(index)),
        );
    }

    router.layer(TraceLayer::new_for_http())
}

async fn health() -> &'static str {
    "ok"
}
