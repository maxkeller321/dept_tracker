use chrono::Utc;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::loans::{get_loan, list_loans, LoanRow};

#[derive(Serialize)]
pub struct ExportBundle {
    pub schema_version: i32,
    pub exported_at: String,
    pub currency_code: String,
    pub loans: Vec<serde_json::Value>,
}

pub async fn export_all(pool: &SqlitePool) -> Result<ExportBundle, sqlx::Error> {
    let currency = crate::settings::get_currency(pool).await?;
    let rows = list_loans(pool, true).await?;
    let mut loans = Vec::new();
    for row in rows {
        loans.push(export_loan_json(pool, &row).await?);
    }
    Ok(ExportBundle {
        schema_version: 1,
        exported_at: Utc::now().to_rfc3339(),
        currency_code: currency,
        loans,
    })
}

async fn export_loan_json(pool: &SqlitePool, row: &LoanRow) -> Result<serde_json::Value, sqlx::Error> {
    let _ = get_loan(pool, &row.id).await?;
    let recurring = sqlx::query_as::<_, (String, i64, i32, i32, i32)>(
        "SELECT id, amount_minor, month, day, enabled FROM recurring_sonderzahlungen WHERE loan_id = ?",
    )
    .bind(&row.id)
    .fetch_all(pool)
    .await?;
    let scheduled = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT id, amount_minor, due_date, status FROM scheduled_sonderzahlungen WHERE loan_id = ?",
    )
    .bind(&row.id)
    .fetch_all(pool)
    .await?;
    let payments = sqlx::query_as::<_, (String, String, i64, i64, i64, i64, String)>(
        r#"SELECT id, event_type, amount_minor, interest_portion_minor,
           principal_portion_minor, balance_after_minor, paid_at FROM payment_events WHERE loan_id = ?"#,
    )
    .bind(&row.id)
    .fetch_all(pool)
    .await?;

    Ok(serde_json::json!({
        "loan": {
            "id": row.id,
            "label": row.label,
            "status": row.status,
            "setup_mode": row.setup_mode,
            "original_principal_minor": row.original_principal_minor,
            "remaining_balance_minor": row.remaining_balance_minor,
            "payment_frequency": row.payment_frequency,
            "payment_type": row.payment_type,
            "tilgung_euro_minor": row.fixed_payment_minor,
            "fixed_payment_minor": row.fixed_payment_minor,
            "tilgung_percent_basis_points": row.tilgung_percent_basis_points,
            "apr_basis_points": row.apr_basis_points,
            "loan_start_date": row.loan_start_date,
            "first_payment_date": row.first_payment_date,
            "created_at": row.created_at,
        },
        "recurring_sonderzahlungen": recurring.iter().map(|(id, amount, month, day, enabled)| serde_json::json!({
            "id": id, "amount_minor": amount, "month": month, "day": day, "enabled": *enabled != 0
        })).collect::<Vec<_>>(),
        "scheduled_sonderzahlungen": scheduled.iter().map(|(id, amount, due, status)| serde_json::json!({
            "id": id, "amount_minor": amount, "due_date": due, "status": status
        })).collect::<Vec<_>>(),
        "payment_events": payments.iter().map(|(id, et, amount, i, p, b, paid)| serde_json::json!({
            "id": id, "event_type": et, "amount_minor": amount,
            "interest_portion_minor": i, "principal_portion_minor": p,
            "balance_after_minor": b, "paid_at": paid
        })).collect::<Vec<_>>(),
    }))
}
