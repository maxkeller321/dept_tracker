use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;

use crate::error::ApiError;
use crate::routes::loans::loan_detail;
use crate::AppState;

#[derive(Deserialize)]
pub struct ImmediateBody {
    pub amount_minor: i64,
    pub paid_at: String,
    #[serde(default)]
    pub confirm_overpayment: bool,
    /// When true, treat `paid_at` as a past date and rebuild loan balances from that point.
    #[serde(default)]
    pub recalculate_from_past: bool,
}

#[derive(Deserialize)]
pub struct ScheduleBody {
    pub amount_minor: i64,
    pub due_date: String,
}

pub async fn immediate(
    State(state): State<AppState>,
    Path(loan_id): Path<String>,
    Json(body): Json<ImmediateBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let paid_at = chrono::NaiveDate::parse_from_str(&body.paid_at, "%Y-%m-%d")
        .map_err(|_| ApiError::bad_request("invalid paid_at"))?;
    let row = db::loans::get_loan(&state.pool, &loan_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .ok_or_else(|| ApiError::not_found("loan not found"))?;
    if body.amount_minor > row.remaining_balance_minor && !body.confirm_overpayment {
        return Err(ApiError::bad_request(
            "payment exceeds balance; set confirm_overpayment=true",
        ));
    }
    let today = chrono::Utc::now().date_naive();
    if body.recalculate_from_past {
        if paid_at > today {
            return Err(ApiError::bad_request(
                "recalculate_from_past requires paid_at on or before today",
            ));
        }
        db::payment_events::record_backdated_sonderzahlung(
            &state.pool,
            &loan_id,
            body.amount_minor,
            paid_at,
            today,
        )
        .await
        .map_err(ApiError::bad_request)?;
    } else {
        db::payment_events::record_sonderzahlung(&state.pool, &loan_id, body.amount_minor, paid_at)
            .await
            .map_err(ApiError::bad_request)?;
    }
    loan_detail(State(state), Path(loan_id)).await
}

pub async fn list_scheduled(
    State(state): State<AppState>,
    Path(loan_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let list = db::scheduled_sonderzahlungen::list_pending(&state.pool, &loan_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::json!(list)))
}

pub async fn schedule(
    State(state): State<AppState>,
    Path(loan_id): Path<String>,
    Json(body): Json<ScheduleBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let due = chrono::NaiveDate::parse_from_str(&body.due_date, "%Y-%m-%d")
        .map_err(|_| ApiError::bad_request("invalid due_date"))?;
    let id = db::scheduled_sonderzahlungen::schedule(&state.pool, &loan_id, body.amount_minor, due)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::json!({
        "id": id,
        "amount_minor": body.amount_minor,
        "due_date": body.due_date,
        "status": "pending"
    })))
}

pub async fn cancel_scheduled(
    State(state): State<AppState>,
    Path((loan_id, schedule_id)): Path<(String, String)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let _ = loan_id;
    db::scheduled_sonderzahlungen::cancel(&state.pool, &schedule_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
