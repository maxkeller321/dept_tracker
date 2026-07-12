use chrono::NaiveDate;
use domain::interest::compute_interest_summary;
use domain::types::{LoanCalcInput, LoanStatus, PaymentFrequency, PaymentType};

fn sample_loan() -> LoanCalcInput {
    LoanCalcInput {
        id: "1".into(),
        label: "Test".into(),
        status: LoanStatus::Active,
        remaining_balance_minor: 50_000_00,
        original_principal_minor: Some(100_000_00),
        payment_frequency: PaymentFrequency::Monthly,
        payment_type: PaymentType::TilgungPercent,
        tilgung_euro_minor: None,
        tilgung_percent_basis_points: Some(200),
        apr_basis_points: Some(400),
        loan_start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        first_payment_date: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
        recurring_extras: vec![],
        scheduled_extras: vec![],
        payments: vec![],
    }
}

#[test]
fn interest_summary_computable_with_apr_and_tilgung() {
    let loan = sample_loan();
    let summary = compute_interest_summary(&loan, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
    assert!(summary.computable);
    assert!(summary.interest_remaining_minor >= 0);
}

#[test]
fn interest_summary_not_computable_without_apr() {
    let mut loan = sample_loan();
    loan.apr_basis_points = None;
    let summary = compute_interest_summary(&loan, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
    assert!(!summary.computable);
}
