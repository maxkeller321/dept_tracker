use chrono::NaiveDate;
use serde::Serialize;

use crate::money::{apr_basis_to_percent, minor_to_decimal};
use crate::payoff_timeline::{build_payoff_timeline, PayoffTimeline};
use crate::projection::{monthly_equivalent_payment, project_payoff};
use crate::types::LoanCalcInput;

#[derive(Debug, Clone, Serialize)]
pub struct MoneyDto {
    pub amount_minor: i64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoanSummary {
    pub id: String,
    pub label: String,
    pub remaining_balance: MoneyDto,
    /// Original loan amount at origination (None if not recorded).
    pub original_principal: Option<MoneyDto>,
    pub periodic_payment: MoneyDto,
    pub payment_frequency: String,
    pub last_payment_date: Option<String>,
    pub projected_payoff_date: Option<String>,
    pub progress_percent: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct HouseholdSummary {
    pub total_balance: MoneyDto,
    pub total_monthly_obligation: MoneyDto,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardResponse {
    pub household: HouseholdSummary,
    pub loans: Vec<LoanSummary>,
    pub payoff_timeline: PayoffTimeline,
}

pub fn build_dashboard(loans: &[LoanCalcInput], currency: &str, as_of: NaiveDate) -> DashboardResponse {
    let mut total_balance = 0i64;
    let mut total_monthly = 0i64;
    let mut summaries = Vec::new();

    for loan in loans {
        if loan.status != crate::types::LoanStatus::Active {
            continue;
        }
        let projection = project_payoff(loan, as_of);
        let last_payment = loan
            .payments
            .iter()
            .filter(|p| p.event_type == "regular")
            .map(|p| p.paid_at)
            .max();
        let progress = progress_percent(loan);
        total_balance += loan.remaining_balance_minor;
        total_monthly += monthly_equivalent_payment(
            projection.periodic_payment_minor,
            loan.payment_frequency,
        );
        summaries.push(LoanSummary {
            id: loan.id.clone(),
            label: loan.label.clone(),
            remaining_balance: money(loan.remaining_balance_minor, currency),
            original_principal: loan.original_principal_minor.map(|p| money(p, currency)),
            periodic_payment: money(projection.periodic_payment_minor, currency),
            payment_frequency: match loan.payment_frequency {
                crate::types::PaymentFrequency::Monthly => "monthly".into(),
                crate::types::PaymentFrequency::Yearly => "yearly".into(),
            },
            last_payment_date: last_payment.map(|d| d.to_string()),
            projected_payoff_date: projection
                .projected_payoff_date
                .map(|d| d.to_string()),
            progress_percent: progress,
        });
    }

    DashboardResponse {
        household: HouseholdSummary {
            total_balance: money(total_balance, currency),
            total_monthly_obligation: money(total_monthly, currency),
        },
        loans: summaries,
        payoff_timeline: build_payoff_timeline(loans, as_of),
    }
}

fn money(amount_minor: i64, currency: &str) -> MoneyDto {
    MoneyDto {
        amount_minor,
        currency: currency.to_string(),
    }
}

fn progress_percent(loan: &LoanCalcInput) -> f64 {
    let original = loan
        .original_principal_minor
        .unwrap_or(loan.remaining_balance_minor);
    if original <= 0 {
        return 0.0;
    }
    let paid_down = original - loan.remaining_balance_minor;
    ((paid_down as f64 / original as f64) * 100.0).clamp(0.0, 100.0)
}

#[allow(dead_code)]
pub fn format_display_amount(amount_minor: i64) -> String {
    minor_to_decimal(amount_minor)
}

#[allow(dead_code)]
pub fn format_apr(basis: i32) -> f64 {
    apr_basis_to_percent(basis)
}
