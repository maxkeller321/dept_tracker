use chrono::NaiveDate;
use domain::amortization::{periodic_interest_minor, periodic_principal_minor, periodic_total_minor};
use domain::due_payments::{due_regular_payment_dates, payment_due_anchor};
use domain::payment_split::split_regular_payment;
use domain::projection::{compute_amortization_schedule, project_payoff};
use domain::types::{LoanCalcInput, LoanStatus, PaymentFrequency, PaymentType};

fn base_loan() -> LoanCalcInput {
    LoanCalcInput {
        id: "1".into(),
        label: "Calc".into(),
        status: LoanStatus::Active,
        remaining_balance_minor: 100_000_00,
        original_principal_minor: Some(100_000_00),
        payment_frequency: PaymentFrequency::Monthly,
        payment_type: PaymentType::TilgungEuro,
        tilgung_euro_minor: Some(500_00),
        tilgung_percent_basis_points: None,
        apr_basis_points: Some(400),
        loan_start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        first_payment_date: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
        recurring_extras: vec![],
        scheduled_extras: vec![],
        payments: vec![],
    }
}

#[test]
fn monthly_euro_tilgung_total_equals_interest_plus_principal() {
    let loan = base_loan();
    let interest = periodic_interest_minor(100_000_00, 400, PaymentFrequency::Monthly);
    let principal = periodic_principal_minor(&loan);
    assert_eq!(periodic_total_minor(&loan), interest + principal);
}

#[test]
fn monthly_percent_tilgung_from_original_principal() {
    let mut loan = base_loan();
    loan.payment_type = PaymentType::TilgungPercent;
    loan.tilgung_euro_minor = None;
    loan.tilgung_percent_basis_points = Some(200);
    assert_eq!(periodic_principal_minor(&loan), 166_67);
}

#[test]
fn due_dates_respect_first_payment_not_loan_start() {
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let first = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
    let anchor = payment_due_anchor(Some(first), start);
    let through = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
    let dates = due_regular_payment_dates(anchor, PaymentFrequency::Monthly, None, through);
    assert_eq!(dates, vec![first, NaiveDate::from_ymd_opt(2024, 4, 1).unwrap()]);
}

#[test]
fn split_regular_last_payment_zero_balance() {
    let loan = base_loan();
    let split = split_regular_payment(&loan, 100_00);
    assert_eq!(split.balance_after_minor, 0);
    assert_eq!(split.principal_portion_minor, 100_00);
}

#[test]
fn projection_payoff_after_many_periods() {
    let loan = base_loan();
    let result = project_payoff(&loan, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    assert!(result.projected_payoff_date.is_some());
    assert!(result.periodic_payment_minor > 500_00);
}

#[test]
fn yearly_euro_tilgung_one_installment_per_year() {
    let mut loan = base_loan();
    loan.payment_frequency = PaymentFrequency::Yearly;
    let first = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    loan.first_payment_date = first;
    let through = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
    let dates = due_regular_payment_dates(first, PaymentFrequency::Yearly, None, through);
    assert_eq!(dates.len(), 2);
}

#[test]
fn amortization_schedule_balances_decrease_to_zero() {
    let loan = base_loan();
    let as_of = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
    let rows = compute_amortization_schedule(&loan, as_of);

    assert!(!rows.is_empty(), "schedule must have rows");

    // Balance decreases monotonically.
    for w in rows.windows(2) {
        assert!(
            w[1].balance_minor <= w[0].balance_minor,
            "balance should not increase: {} -> {}",
            w[0].balance_minor,
            w[1].balance_minor
        );
    }

    // Final balance is zero.
    assert_eq!(rows.last().unwrap().balance_minor, 0);

    // Each payment = interest + scheduled Tilgung (500 €).
    let first = &rows[0];
    let interest = periodic_interest_minor(loan.remaining_balance_minor, 400, PaymentFrequency::Monthly);
    assert_eq!(first.interest_minor, interest);
    assert_eq!(first.payment_minor, interest + 500_00);
}

#[test]
fn amortization_schedule_zero_balance_returns_empty() {
    let mut loan = base_loan();
    loan.remaining_balance_minor = 0;
    let as_of = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
    let rows = compute_amortization_schedule(&loan, as_of);
    assert!(rows.is_empty());
}

// ── Regression tests for Bug 1: advance_period stall on end-of-month days ──────────────────

/// Loan paying on day 31. Monthly advance must NOT stall: Jan 31 → Feb 28/29, not Jan 31 again.
#[test]
fn amortization_end_of_month_31_no_repeated_dates() {
    let mut loan = base_loan();
    // Payment anchor on the 31st.
    loan.first_payment_date = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
    loan.loan_start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

    let as_of = NaiveDate::from_ymd_opt(2024, 1, 30).unwrap();
    let rows = compute_amortization_schedule(&loan, as_of);

    // Must have more than one row (would stall at MAX_PERIODS with the same date if broken).
    assert!(rows.len() > 1, "schedule has only {} rows; stall suspected", rows.len());

    // All dates must be strictly ascending.
    for w in rows.windows(2) {
        assert!(
            w[1].date > w[0].date,
            "dates not strictly ascending: {} -> {}",
            w[0].date,
            w[1].date
        );
    }
}

/// Yearly loan with anchor on Feb 29 (leap year). Next year must advance to Feb 28.
#[test]
fn yearly_loan_feb29_advances_correctly() {
    let mut loan = base_loan();
    loan.payment_frequency = PaymentFrequency::Yearly;
    loan.tilgung_euro_minor = Some(10_000_00);
    loan.remaining_balance_minor = 20_000_00;
    loan.first_payment_date = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap(); // leap
    loan.loan_start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

    let as_of = NaiveDate::from_ymd_opt(2024, 2, 28).unwrap();
    let rows = compute_amortization_schedule(&loan, as_of);

    // Should produce 2 yearly rows (20 000 / 10 000 each).
    assert!(rows.len() >= 2, "expected at least 2 yearly rows, got {}", rows.len());

    // Dates must be strictly ascending — non-leap year must not repeat Feb 29.
    for w in rows.windows(2) {
        assert!(w[1].date > w[0].date, "date stall: {} -> {}", w[0].date, w[1].date);
    }
}

/// project_payoff must not exceed MAX_PERIODS when the payment day is 31.
#[test]
fn project_payoff_end_of_month_31_completes() {
    let mut loan = base_loan();
    loan.first_payment_date = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();
    loan.loan_start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

    let as_of = NaiveDate::from_ymd_opt(2024, 3, 30).unwrap();
    let result = project_payoff(&loan, as_of);

    assert!(result.projected_payoff_date.is_some());
    // Should be far in the future, not stuck at the first date.
    let payoff = result.projected_payoff_date.unwrap();
    assert!(payoff > NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
}
