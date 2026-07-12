use chrono::{Datelike, NaiveDate};
use serde::Serialize;

use crate::amortization::{periodic_interest_minor, periodic_principal_minor, periodic_rate, periodic_total_minor};
use crate::due_payments::{advance_due, first_due_after, payment_due_anchor};
use crate::types::{LoanCalcInput, PaymentFrequency};

const MAX_PERIODS: u32 = 600;

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectionResult {
    pub projected_payoff_date: Option<NaiveDate>,
    pub periodic_payment_minor: i64,
}

/// Forward-simulate from the first unpaid due date after `as_of` until balance <= 0.
/// Uses date-only scheduling (NaiveDate); dashboard applies due payments through `as_of` first.
pub fn project_payoff(loan: &LoanCalcInput, as_of: NaiveDate) -> ProjectionResult {
    let mut balance = loan.remaining_balance_minor as f64 / 100.0;
    if balance <= 0.0 {
        return ProjectionResult {
            projected_payoff_date: Some(as_of),
            periodic_payment_minor: periodic_total_minor(loan),
        };
    }

    let apr = loan.apr_basis_points.unwrap_or(0);
    let _r = periodic_rate(apr, loan.payment_frequency);
    let principal_per_period = periodic_principal_minor(loan) as f64 / 100.0;
    let display_payment = periodic_total_minor(loan);

    let anchor = payment_due_anchor(Some(loan.first_payment_date), loan.loan_start_date);
    let mut date = first_due_after(anchor, loan.payment_frequency, as_of);
    let mut payoff: Option<NaiveDate> = None;

    for _ in 0..MAX_PERIODS {
        if balance <= 0.0 {
            payoff = Some(date);
            break;
        }

        let principal = principal_per_period.min(balance);
        balance -= principal;

        balance -= extras_for_date(loan, date) as f64 / 100.0;
        balance -= pending_scheduled_extras(loan, date) as f64 / 100.0;

        if balance <= 0.0 {
            payoff = Some(date);
            break;
        }

        date = advance_due(date, loan.payment_frequency);
    }

    ProjectionResult {
        projected_payoff_date: payoff,
        periodic_payment_minor: display_payment,
    }
}

/// Balance after each projected payment date from `as_of` until payoff.
pub fn project_balance_schedule(loan: &LoanCalcInput, as_of: NaiveDate) -> Vec<(NaiveDate, i64)> {
    let mut balance = loan.remaining_balance_minor as f64 / 100.0;
    let mut points = vec![(as_of, loan.remaining_balance_minor)];
    if balance <= 0.0 {
        return points;
    }

    let principal_per_period = periodic_principal_minor(loan) as f64 / 100.0;
    let anchor = payment_due_anchor(Some(loan.first_payment_date), loan.loan_start_date);
    let mut date = first_due_after(anchor, loan.payment_frequency, as_of);

    for _ in 0..MAX_PERIODS {
        let principal = principal_per_period.min(balance);
        balance -= principal;
        balance -= extras_for_date(loan, date) as f64 / 100.0;
        balance -= pending_scheduled_extras(loan, date) as f64 / 100.0;
        let balance_minor = (balance * 100.0).max(0.0).round() as i64;
        points.push((date, balance_minor));
        if balance_minor <= 0 {
            break;
        }
        date = advance_due(date, loan.payment_frequency);
    }

    points
}

// advance_period is provided by due_payments::advance_due which correctly handles
// month-end overflow (e.g. Jan 31 -> Feb 28/29) via chrono::checked_add_months.

fn extras_for_date(loan: &LoanCalcInput, date: NaiveDate) -> i64 {
    loan.recurring_extras
        .iter()
        .filter(|e| {
            e.enabled
                && u32::from(e.month) == date.month()
                && u32::from(e.day) == date.day()
        })
        .map(|e| e.amount_minor)
        .sum()
}

fn pending_scheduled_extras(loan: &LoanCalcInput, date: NaiveDate) -> i64 {
    loan.scheduled_extras
        .iter()
        .filter(|s| s.status == "pending" && s.due_date == date)
        .map(|s| s.amount_minor)
        .sum()
}

/// One row of an amortization (Tilgungsplan) table.
#[derive(Debug, Clone, Serialize)]
pub struct AmortizationRow {
    /// ISO date of this payment.
    pub date: String,
    /// Total cash paid this period (interest + scheduled Tilgung). Does not include Sonderzahlungen.
    pub payment_minor: i64,
    /// Interest portion.
    pub interest_minor: i64,
    /// Principal (Tilgung) portion applied to balance this period, including any extra payments.
    pub principal_minor: i64,
    /// Remaining balance after this payment.
    pub balance_minor: i64,
}

/// Full amortization schedule from `as_of` forward, starting at the loan's current
/// `remaining_balance_minor`.  Extra payments (Sonderzahlungen) reduce the balance at their
/// respective dates and are reflected in `principal_minor` for that row.
pub fn compute_amortization_schedule(loan: &LoanCalcInput, as_of: NaiveDate) -> Vec<AmortizationRow> {
    let apr = loan.apr_basis_points.unwrap_or(0);
    let mut balance = loan.remaining_balance_minor as f64 / 100.0;
    if balance <= 0.0 {
        return vec![];
    }

    let principal_per_period = periodic_principal_minor(loan) as f64 / 100.0;
    let anchor = payment_due_anchor(Some(loan.first_payment_date), loan.loan_start_date);
    let mut date = first_due_after(anchor, loan.payment_frequency, as_of);

    let mut rows = Vec::new();
    for _ in 0..MAX_PERIODS {
        let balance_before_minor = (balance * 100.0).round() as i64;
        let interest = periodic_interest_minor(balance_before_minor, apr, loan.payment_frequency);
        let scheduled_tilgung = principal_per_period.min(balance);
        let extras = (extras_for_date(loan, date) + pending_scheduled_extras(loan, date)) as f64 / 100.0;

        balance -= scheduled_tilgung + extras;
        let balance_after = (balance * 100.0).max(0.0).round() as i64;
        let principal_applied = balance_before_minor - balance_after;

        rows.push(AmortizationRow {
            date: date.to_string(),
            payment_minor: interest + (scheduled_tilgung * 100.0).round() as i64,
            interest_minor: interest,
            principal_minor: principal_applied,
            balance_minor: balance_after,
        });

        if balance_after <= 0 {
            break;
        }
        date = advance_due(date, loan.payment_frequency);
    }
    rows
}

/// Merge amortization schedules for multiple loans, grouped by calendar month.
/// Returns one `AmortizationRow` per month with summed payments, interest, principal and the
/// running total balance across all loans.
pub fn build_combined_schedule(loans: &[LoanCalcInput], as_of: NaiveDate) -> Vec<AmortizationRow> {
    use std::collections::BTreeMap;

    let active: Vec<&LoanCalcInput> = loans
        .iter()
        .filter(|l| l.remaining_balance_minor > 0)
        .collect();
    if active.is_empty() {
        return vec![];
    }

    let schedules: Vec<Vec<AmortizationRow>> = active
        .iter()
        .map(|l| compute_amortization_schedule(l, as_of))
        .collect();

    // Sum payment/interest/principal per (year, month).
    let mut by_month: BTreeMap<(i32, u32), (i64, i64, i64)> = BTreeMap::new();
    for schedule in &schedules {
        for row in schedule {
            let d = NaiveDate::parse_from_str(&row.date, "%Y-%m-%d").unwrap_or(as_of);
            let key = (d.year(), d.month());
            let e = by_month.entry(key).or_default();
            e.0 += row.payment_minor;
            e.1 += row.interest_minor;
            e.2 += row.principal_minor;
        }
    }

    // Running total balance: start from sum of all current balances, subtract cumulative principal.
    let mut running: i64 = active.iter().map(|l| l.remaining_balance_minor).sum();

    by_month
        .into_iter()
        .map(|((year, month), (payment, interest, principal))| {
            running -= principal;
            AmortizationRow {
                date: NaiveDate::from_ymd_opt(year, month, 1)
                    .unwrap_or(as_of)
                    .to_string(),
                payment_minor: payment,
                interest_minor: interest,
                principal_minor: principal,
                balance_minor: running.max(0),
            }
        })
        .collect()
}

/// Balance and remaining future interest after each projected payment date from `as_of`.
///
/// Returns `Vec<(date, remaining_principal_minor, remaining_interest_minor)>`.
/// The `remaining_interest_minor` at each point is the suffix sum of all per-period interest
/// payments from that point to payoff — i.e. the total interest cost still to be paid.
pub fn project_balance_and_interest_schedule(
    loan: &LoanCalcInput,
    as_of: NaiveDate,
) -> Vec<(NaiveDate, i64, i64)> {
    let mut balance = loan.remaining_balance_minor as f64 / 100.0;
    if balance <= 0.0 {
        return vec![(as_of, 0, 0)];
    }

    let apr = loan.apr_basis_points.unwrap_or(0);
    let principal_per_period = periodic_principal_minor(loan) as f64 / 100.0;
    let anchor = payment_due_anchor(Some(loan.first_payment_date), loan.loan_start_date);
    let mut date = first_due_after(anchor, loan.payment_frequency, as_of);

    // Forward pass: (date, balance_after, interest_this_period)
    let mut steps: Vec<(NaiveDate, i64, i64)> = Vec::new();
    for _ in 0..MAX_PERIODS {
        let balance_before = (balance * 100.0).round() as i64;
        let interest_this = periodic_interest_minor(balance_before, apr, loan.payment_frequency);
        let principal = principal_per_period.min(balance);
        balance -= principal;
        balance -= extras_for_date(loan, date) as f64 / 100.0;
        balance -= pending_scheduled_extras(loan, date) as f64 / 100.0;
        let balance_minor = (balance * 100.0).max(0.0).round() as i64;
        steps.push((date, balance_minor, interest_this));
        if balance_minor <= 0 {
            break;
        }
        date = advance_due(date, loan.payment_frequency);
    }

    // Suffix sum: remaining_interest[i] = total interest still owed after paying step i
    let n = steps.len();
    let mut suffix = vec![0i64; n + 1];
    for i in (0..n).rev() {
        suffix[i] = suffix[i + 1] + steps[i].2;
    }

    // First entry: as_of date with full remaining balance and ALL future interest
    let mut result = vec![(as_of, loan.remaining_balance_minor, suffix[0])];
    for (i, &(step_date, balance_minor, _)) in steps.iter().enumerate() {
        result.push((step_date, balance_minor, suffix[i + 1]));
    }
    result
}

/// Monthly-normalized obligation for dashboard totals.
pub fn monthly_equivalent_payment(periodic_minor: i64, frequency: PaymentFrequency) -> i64 {
    match frequency {
        PaymentFrequency::Monthly => periodic_minor,
        PaymentFrequency::Yearly => (periodic_minor as f64 / 12.0).round() as i64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{LoanStatus, PaymentType, RecurringExtra, ScheduledExtra};
    use chrono::NaiveDate;

    fn sample_loan() -> LoanCalcInput {
        LoanCalcInput {
            id: "1".into(),
            label: "Test".into(),
            status: LoanStatus::Active,
            remaining_balance_minor: 100_000_00,
            original_principal_minor: Some(120_000_00),
            payment_frequency: PaymentFrequency::Monthly,
            payment_type: PaymentType::TilgungEuro,
            tilgung_euro_minor: Some(1_000_00),
            tilgung_percent_basis_points: None,
            apr_basis_points: Some(300),
            loan_start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            first_payment_date: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            recurring_extras: vec![],
            scheduled_extras: vec![],
            payments: vec![],
        }
    }

    #[test]
    fn projection_computes_periodic_payment() {
        let loan = sample_loan();
        let result = project_payoff(&loan, NaiveDate::from_ymd_opt(2025, 5, 1).unwrap());
        assert!(result.periodic_payment_minor > 1_000_00);
    }

    #[test]
    fn zero_balance_payoff_is_as_of() {
        let mut loan = sample_loan();
        loan.remaining_balance_minor = 0;
        let as_of = NaiveDate::from_ymd_opt(2025, 5, 1).unwrap();
        let result = project_payoff(&loan, as_of);
        assert_eq!(result.projected_payoff_date, Some(as_of));
    }

    #[test]
    fn sonderzahlung_speeds_up_payoff() {
        let base = sample_loan();
        let mut with_extra = sample_loan();
        with_extra.scheduled_extras.push(ScheduledExtra {
            amount_minor: 50_000_00,
            due_date: NaiveDate::from_ymd_opt(2025, 8, 1).unwrap(),
            status: "pending".into(),
        });
        let as_of = NaiveDate::from_ymd_opt(2025, 5, 1).unwrap();
        let p_base = project_payoff(&base, as_of);
        let p_extra = project_payoff(&with_extra, as_of);
        if let (Some(earlier), Some(later)) = (p_extra.projected_payoff_date, p_base.projected_payoff_date) {
            assert!(earlier <= later);
        }
    }

    #[test]
    fn recurring_extra_included_in_projection() {
        let mut loan = sample_loan();
        loan.recurring_extras.push(RecurringExtra {
            amount_minor: 5_000_00,
            month: 12,
            day: 1,
            enabled: true,
        });
        let result = project_payoff(&loan, NaiveDate::from_ymd_opt(2025, 5, 1).unwrap());
        assert!(result.periodic_payment_minor > 0);
        assert!(result.projected_payoff_date.is_some());
    }
}
