use chrono::Utc;
use domain::payment_split::split_payment;
use domain::types::{LoanCalcInput, LoanStatus, PaymentFrequency, PaymentType};
use domain::validation::{validate_create_loan, CreateLoanValidation};
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct CreateLoanParams {
    pub label: String,
    pub setup_mode: String,
    pub remaining_balance_minor: i64,
    pub original_principal_minor: Option<i64>,
    pub payment_frequency: PaymentFrequency,
    pub payment_type: PaymentType,
    pub tilgung_euro_minor: Option<i64>,
    pub tilgung_percent_basis_points: Option<i32>,
    pub apr_basis_points: Option<i32>,
    pub loan_start_date: Option<chrono::NaiveDate>,
    pub first_payment_date: Option<chrono::NaiveDate>,
    pub recurring: Vec<(i64, u8, u8)>,
    pub backfill: Vec<(i64, chrono::NaiveDate)>,
}

pub async fn create_loan(pool: &SqlitePool, params: CreateLoanParams) -> Result<String, String> {
    validate_create_loan(&CreateLoanValidation {
        label: params.label.clone(),
        remaining_balance_minor: params.remaining_balance_minor,
        payment_frequency: params.payment_frequency,
        payment_type: params.payment_type,
        tilgung_euro_minor: params.tilgung_euro_minor,
        tilgung_percent_basis_points: params.tilgung_percent_basis_points,
        apr_basis_points: params.apr_basis_points,
    })
    .map_err(|e| e.to_string())?;

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let freq = match params.payment_frequency {
        PaymentFrequency::Monthly => "monthly",
        PaymentFrequency::Yearly => "yearly",
    };
    let ptype = params.payment_type.as_str();
    let start = params
        .loan_start_date
        .map(|d| d.to_string())
        .unwrap_or_else(|| now[..10].to_string());
    let first_payment = params
        .first_payment_date
        .or(params.loan_start_date)
        .map(|d| d.to_string())
        .unwrap_or_else(|| start.clone());

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        r#"INSERT INTO loans (
            id, label, status, setup_mode, original_principal_minor, remaining_balance_minor,
            payment_frequency, payment_type, fixed_payment_minor, tilgung_percent_basis_points,
            apr_basis_points, loan_start_date, first_payment_date, created_at, updated_at
        ) VALUES (?, ?, 'active', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&id)
    .bind(&params.label)
    .bind(&params.setup_mode)
    .bind(params.original_principal_minor)
    .bind(params.remaining_balance_minor)
    .bind(freq)
    .bind(ptype)
    .bind(params.tilgung_euro_minor)
    .bind(params.tilgung_percent_basis_points)
    .bind(params.apr_basis_points)
    .bind(&start)
    .bind(&first_payment)
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    for (amount, month, day) in params.recurring {
        let rid = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO recurring_sonderzahlungen (id, loan_id, amount_minor, month, day, enabled) VALUES (?, ?, ?, ?, ?, 1)",
        )
        .bind(&rid)
        .bind(&id)
        .bind(amount)
        .bind(month)
        .bind(day)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    let mut balance = params.remaining_balance_minor;
    if params.setup_mode == "advanced" {
        for (amount, paid_at) in params.backfill {
            let stub = LoanCalcInput {
                id: id.clone(),
                label: params.label.clone(),
                status: LoanStatus::Active,
                remaining_balance_minor: balance,
                original_principal_minor: params.original_principal_minor,
                payment_frequency: params.payment_frequency,
                payment_type: params.payment_type,
                tilgung_euro_minor: params.tilgung_euro_minor,
                tilgung_percent_basis_points: params.tilgung_percent_basis_points,
                apr_basis_points: params.apr_basis_points,
                loan_start_date: params.loan_start_date.unwrap_or_else(|| Utc::now().date_naive()),
                first_payment_date: params
                    .first_payment_date
                    .or(params.loan_start_date)
                    .unwrap_or_else(|| Utc::now().date_naive()),
                recurring_extras: vec![],
                scheduled_extras: vec![],
                payments: vec![],
            };
            let split = split_payment(&stub, amount, balance);
            let pid = Uuid::new_v4().to_string();
            sqlx::query(
                r#"INSERT INTO payment_events (
                    id, loan_id, event_type, amount_minor, interest_portion_minor,
                    principal_portion_minor, balance_after_minor, paid_at, created_at
                ) VALUES (?, ?, 'regular', ?, ?, ?, ?, ?, ?)"#,
            )
            .bind(&pid)
            .bind(&id)
            .bind(amount)
            .bind(split.interest_portion_minor)
            .bind(split.principal_portion_minor)
            .bind(split.balance_after_minor)
            .bind(paid_at.to_string())
            .bind(&now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            balance = split.balance_after_minor;
        }
        sqlx::query("UPDATE loans SET remaining_balance_minor = ? WHERE id = ?")
            .bind(balance)
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(id)
}
