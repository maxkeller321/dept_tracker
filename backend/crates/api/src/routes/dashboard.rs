use axum::extract::{Query, State};
use axum::Json;
use chrono::Utc;
use domain::build_dashboard;
use domain::projection::build_combined_schedule;
use serde::Deserialize;
use crate::error::ApiError;
use crate::AppState;

#[derive(Deserialize)]
pub struct DashboardQuery {
    #[serde(default)]
    pub include_archived: bool,
}

pub async fn get_dashboard(
    State(state): State<AppState>,
    Query(q): Query<DashboardQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let currency = db::get_currency(&state.pool).await.map_err(|e| ApiError::internal(e.to_string()))?;
    let as_of = Utc::now().date_naive();
    // Date-only cutoff: apply all installments due on or before today (local calendar date).
    db::payment_events::sync_all_due_regular_payments(&state.pool, as_of)
        .await
        .map_err(ApiError::bad_request)?;
    let rows = db::loans::list_loans(&state.pool, q.include_archived)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let mut calcs = Vec::new();
    for row in rows {
        if row.status == "active" || q.include_archived {
            calcs.push(
                db::loans::load_loan_calc(&state.pool, &row)
                    .await
                    .map_err(|e| ApiError::internal(e.to_string()))?,
            );
        }
    }
    let active: Vec<_> = if q.include_archived {
        calcs.iter().filter(|l| l.status == domain::types::LoanStatus::Active).cloned().collect()
    } else {
        calcs.clone()
    };
    let dashboard = build_dashboard(&active, &currency, as_of);
    Ok(Json(serde_json::to_value(dashboard).unwrap()))
}

pub async fn combined_amortization(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let as_of = Utc::now().date_naive();
    db::payment_events::sync_all_due_regular_payments(&state.pool, as_of)
        .await
        .map_err(ApiError::bad_request)?;
    let rows = db::loans::list_loans(&state.pool, false)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let mut calcs = Vec::new();
    for row in rows {
        calcs.push(
            db::loans::load_loan_calc(&state.pool, &row)
                .await
                .map_err(|e| ApiError::internal(e.to_string()))?,
        );
    }
    let schedule = build_combined_schedule(&calcs, as_of);
    Ok(Json(serde_json::json!({
        "total_payments": schedule.len(),
        "rows": schedule,
    })))
}
