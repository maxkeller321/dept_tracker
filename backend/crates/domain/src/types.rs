use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentFrequency {
    Monthly,
    Yearly,
}

/// How principal (Tilgung) is determined each period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentType {
    /// Annual % of original loan amount (Prozentuale Tilgung).
    TilgungPercent,
    /// Fixed principal per period in EUR (Tilgung €).
    TilgungEuro,
}

impl PaymentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentType::TilgungPercent => "tilgung_percent",
            PaymentType::TilgungEuro => "tilgung_euro",
        }
    }
}

impl std::str::FromStr for PaymentType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tilgung_percent" | "percent" | "apr" => Ok(PaymentType::TilgungPercent),
            "tilgung_euro" | "euro" | "fixed" => Ok(PaymentType::TilgungEuro),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoanStatus {
    Active,
    Archived,
}

impl LoanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoanStatus::Active => "active",
            LoanStatus::Archived => "archived",
        }
    }
}

impl std::str::FromStr for LoanStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(LoanStatus::Active),
            "archived" => Ok(LoanStatus::Archived),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecurringExtra {
    pub amount_minor: i64,
    pub month: u8,
    pub day: u8,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ScheduledExtra {
    pub amount_minor: i64,
    pub due_date: NaiveDate,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct PaymentRecord {
    pub paid_at: NaiveDate,
    pub amount_minor: i64,
    pub interest_portion_minor: i64,
    pub principal_portion_minor: i64,
    pub event_type: String,
}

/// Loan data needed for calculations (DB-agnostic).
#[derive(Debug, Clone)]
pub struct LoanCalcInput {
    pub id: String,
    pub label: String,
    pub status: LoanStatus,
    pub remaining_balance_minor: i64,
    pub original_principal_minor: Option<i64>,
    pub payment_frequency: PaymentFrequency,
    pub payment_type: PaymentType,
    /// Tilgung € per period (principal only).
    pub tilgung_euro_minor: Option<i64>,
    /// Annual tilgung % of original principal (basis points, e.g. 200 = 2.00%).
    pub tilgung_percent_basis_points: Option<i32>,
    /// APR for interest calculation (basis points).
    pub apr_basis_points: Option<i32>,
    /// Contract / disbursement start (date-only, no timezone).
    pub loan_start_date: NaiveDate,
    /// First regular installment due date; auto-payments anchor here, not created_at.
    pub first_payment_date: NaiveDate,
    pub recurring_extras: Vec<RecurringExtra>,
    pub scheduled_extras: Vec<ScheduledExtra>,
    pub payments: Vec<PaymentRecord>,
}
