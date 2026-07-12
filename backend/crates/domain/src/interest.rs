use crate::amortization::{periodic_principal_minor, periodic_rate};
use crate::due_payments::advance_due;
use crate::projection::project_payoff;
use crate::types::LoanCalcInput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterestSummary {
    pub interest_paid_minor: i64,
    pub interest_remaining_minor: i64,
    pub computable: bool,
    pub message: Option<String>,
}

pub fn compute_interest_summary(loan: &LoanCalcInput, as_of: chrono::NaiveDate) -> InterestSummary {
    let paid: i64 = loan
        .payments
        .iter()
        .map(|p| p.interest_portion_minor)
        .sum();

    if loan.apr_basis_points.is_none() {
        return InterestSummary {
            interest_paid_minor: paid,
            interest_remaining_minor: 0,
            computable: false,
            message: Some("missing_apr".into()),
        };
    }

    let remaining = estimate_remaining_interest(loan, as_of);
    InterestSummary {
        interest_paid_minor: paid,
        interest_remaining_minor: remaining,
        computable: true,
        message: None,
    }
}

fn estimate_remaining_interest(loan: &LoanCalcInput, as_of: chrono::NaiveDate) -> i64 {
    let projection = project_payoff(loan, as_of);
    let r = loan
        .apr_basis_points
        .map(|a| periodic_rate(a, loan.payment_frequency))
        .unwrap_or(0.0);
    let mut balance = loan.remaining_balance_minor as f64 / 100.0;
    let mut total_interest = 0.0;
    let principal_per_period = periodic_principal_minor(loan) as f64 / 100.0;
    let mut date = as_of;
    for _ in 0..600 {
        if balance <= 0.0 {
            break;
        }
        let interest = balance * r;
        total_interest += interest;
        let principal = principal_per_period.min(balance);
        balance -= principal;
        date = advance_due(date, loan.payment_frequency);
        if projection
            .projected_payoff_date
            .map(|p| date > p)
            .unwrap_or(false)
        {
            break;
        }
    }
    (total_interest * 100.0).round() as i64
}
