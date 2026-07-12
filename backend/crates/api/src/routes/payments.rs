use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;

use crate::error::ApiError;
use crate::routes::loans::loan_detail;
use crate::AppState;

#[derive(Deserialize)]
pub struct RecordPaymentBody {
    pub amount_minor: i64,
    pub paid_at: String,
    pub note: Option<String>,
}

pub async fn list_payments(
    State(state): State<AppState>,
    Path(loan_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let events = db::payment_events::list_payments(&state.pool, &loan_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::json!(events)))
}

pub async fn record_payment(
    State(state): State<AppState>,
    Path(loan_id): Path<String>,
    Json(body): Json<RecordPaymentBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let paid_at = chrono::NaiveDate::parse_from_str(&body.paid_at, "%Y-%m-%d")
        .map_err(|_| ApiError::bad_request("invalid paid_at"))?;
    let row = db::loans::get_loan(&state.pool, &loan_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .ok_or_else(|| ApiError::not_found("loan not found"))?;
    let calc = db::loans::load_loan_calc(&state.pool, &row)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let split = domain::payment_split::split_payment(&calc, body.amount_minor, calc.remaining_balance_minor);
    db::payment_events::record_regular_payment(
        &state.pool,
        &loan_id,
        body.amount_minor,
        paid_at,
        split,
        body.note,
    )
    .await
    .map_err(ApiError::bad_request)?;
    loan_detail(State(state), Path(loan_id)).await
}
