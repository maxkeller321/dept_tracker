use serde::Deserialize;
use serde_json::Value;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ImportBundle {
    pub schema_version: i32,
    pub currency_code: Option<String>,
    pub loans: Vec<Value>,
}

pub async fn import_replace(pool: &SqlitePool, bundle: ImportBundle) -> Result<(), String> {
    if bundle.schema_version != 1 {
        return Err("unsupported schema_version".into());
    }
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    for table in [
        "payment_events",
        "scheduled_sonderzahlungen",
        "recurring_sonderzahlungen",
        "loans",
    ] {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    if let Some(ccy) = bundle.currency_code {
        sqlx::query("UPDATE settings SET currency_code = ? WHERE id = 1")
            .bind(ccy)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    for entry in bundle.loans {
        import_loan_entry(&mut tx, &entry).await?;
    }
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

async fn import_loan_entry(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    entry: &Value,
) -> Result<(), String> {
    let loan = entry.get("loan").ok_or("missing loan object")?;
    let id = loan["id"]
        .as_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        r#"INSERT INTO loans (
            id, label, status, setup_mode, original_principal_minor, remaining_balance_minor,
            payment_frequency, payment_type, fixed_payment_minor, tilgung_percent_basis_points,
            apr_basis_points, loan_start_date, first_payment_date, created_at, updated_at, archived_at, notes
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&id)
    .bind(json_str(loan, "label")?)
    .bind(json_str_or(loan, "status", "active"))
    .bind(json_str_or(loan, "setup_mode", "quick"))
    .bind(json_i64(loan, "original_principal_minor"))
    .bind(json_i64_req(loan, "remaining_balance_minor")?)
    .bind(json_str(loan, "payment_frequency")?)
    .bind(json_str(loan, "payment_type")?)
    .bind(
        json_i64(loan, "fixed_payment_minor")
            .or_else(|| json_i64(loan, "tilgung_euro_minor")),
    )
    .bind(json_i64(loan, "tilgung_percent_basis_points").map(|v| v as i32))
    .bind(json_i64(loan, "apr_basis_points").map(|v| v as i32))
    .bind(loan.get("loan_start_date").and_then(|v| v.as_str()))
    .bind(
        loan.get("first_payment_date")
            .and_then(|v| v.as_str())
            .or_else(|| loan.get("loan_start_date").and_then(|v| v.as_str())),
    )
    .bind(loan.get("created_at").and_then(|v| v.as_str()).unwrap_or(&now))
    .bind(loan.get("updated_at").and_then(|v| v.as_str()).unwrap_or(&now))
    .bind(loan.get("archived_at").and_then(|v| v.as_str()))
    .bind(loan.get("notes").and_then(|v| v.as_str()))
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(recurring) = entry.get("recurring_sonderzahlungen").and_then(|v| v.as_array()) {
        for r in recurring {
            let rid = r["id"]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            sqlx::query(
                "INSERT INTO recurring_sonderzahlungen (id, loan_id, amount_minor, month, day, enabled) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&rid)
            .bind(&id)
            .bind(r["amount_minor"].as_i64().unwrap_or(0))
            .bind(r["month"].as_i64().unwrap_or(1) as i32)
            .bind(r["day"].as_i64().unwrap_or(1) as i32)
            .bind(if r["enabled"].as_bool().unwrap_or(true) { 1 } else { 0 })
            .execute(&mut **tx)
            .await
            .map_err(|e| e.to_string())?;
        }
    }

    if let Some(scheduled) = entry.get("scheduled_sonderzahlungen").and_then(|v| v.as_array()) {
        for s in scheduled {
            let sid = s["id"]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            sqlx::query(
                "INSERT INTO scheduled_sonderzahlungen (id, loan_id, amount_minor, due_date, status, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&sid)
            .bind(&id)
            .bind(s["amount_minor"].as_i64().unwrap_or(0))
            .bind(s["due_date"].as_str().unwrap_or(""))
            .bind(s["status"].as_str().unwrap_or("pending"))
            .bind(&now)
            .execute(&mut **tx)
            .await
            .map_err(|e| e.to_string())?;
        }
    }

    if let Some(payments) = entry.get("payment_events").and_then(|v| v.as_array()) {
        for p in payments {
            let pid = p["id"]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            sqlx::query(
                r#"INSERT INTO payment_events (
                    id, loan_id, event_type, amount_minor, interest_portion_minor,
                    principal_portion_minor, balance_after_minor, paid_at, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            )
            .bind(&pid)
            .bind(&id)
            .bind(p["event_type"].as_str().unwrap_or("regular"))
            .bind(p["amount_minor"].as_i64().unwrap_or(0))
            .bind(p["interest_portion_minor"].as_i64().unwrap_or(0))
            .bind(p["principal_portion_minor"].as_i64().unwrap_or(0))
            .bind(p["balance_after_minor"].as_i64().unwrap_or(0))
            .bind(p["paid_at"].as_str().unwrap_or(""))
            .bind(&now)
            .execute(&mut **tx)
            .await
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn json_str<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v.get(key)
        .and_then(|x| x.as_str())
        .ok_or_else(|| format!("missing {key}"))
}

fn json_str_or<'a>(v: &'a Value, key: &str, default: &'a str) -> &'a str {
    v.get(key).and_then(|x| x.as_str()).unwrap_or(default)
}

fn json_i64(v: &Value, key: &str) -> Option<i64> {
    v.get(key).and_then(|x| x.as_i64())
}

fn json_i64_req(v: &Value, key: &str) -> Result<i64, String> {
    json_i64(v, key).ok_or_else(|| format!("missing {key}"))
}
