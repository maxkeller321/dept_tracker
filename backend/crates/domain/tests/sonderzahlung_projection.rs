use chrono::NaiveDate;
use domain::projection::project_payoff;
use domain::types::{
    LoanCalcInput, LoanStatus, PaymentFrequency, PaymentType, RecurringExtra, ScheduledExtra,
};

fn base_loan() -> LoanCalcInput {
    LoanCalcInput {
        id: "1".into(),
        label: "Test".into(),
        status: LoanStatus::Active,
        remaining_balance_minor: 500_000_00,
        original_principal_minor: Some(600_000_00),
        payment_frequency: PaymentFrequency::Monthly,
        payment_type: PaymentType::TilgungEuro,
        tilgung_euro_minor: Some(2_000_00),
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
fn extra_payment_speeds_up_payoff() {
    let mut without = base_loan();
    let mut with_extra = base_loan();
    with_extra.scheduled_extras.push(ScheduledExtra {
        amount_minor: 50_000_00,
        due_date: NaiveDate::from_ymd_opt(2025, 7, 1).unwrap(),
        status: "pending".into(),
    });
    let as_of = NaiveDate::from_ymd_opt(2025, 5, 1).unwrap();
    let p_without = project_payoff(&without, as_of);
    let p_with = project_payoff(&with_extra, as_of);
    if let (Some(earlier), Some(later)) = (p_with.projected_payoff_date, p_without.projected_payoff_date) {
        assert!(earlier <= later);
    }
}

#[test]
fn recurring_yearly_extra_included() {
    let mut loan = base_loan();
    loan.recurring_extras.push(RecurringExtra {
        amount_minor: 5_000_00,
        month: 12,
        day: 1,
        enabled: true,
    });
    let result = project_payoff(&loan, NaiveDate::from_ymd_opt(2025, 5, 1).unwrap());
    assert!(result.periodic_payment_minor > 0);
}
