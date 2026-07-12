use crate::types::{LoanCalcInput, PaymentFrequency, PaymentType};

/// Periodic interest rate from annual APR basis points.
pub fn periodic_rate(apr_basis_points: i32, frequency: PaymentFrequency) -> f64 {
    let annual = apr_basis_points as f64 / 10_000.0;
    match frequency {
        PaymentFrequency::Monthly => annual / 12.0,
        PaymentFrequency::Yearly => annual,
    }
}

/// Interest portion for one period on the current balance.
pub fn periodic_interest_minor(
    balance_minor: i64,
    apr_basis_points: i32,
    frequency: PaymentFrequency,
) -> i64 {
    let r = periodic_rate(apr_basis_points, frequency);
    let balance = balance_minor as f64 / 100.0;
    (balance * r * 100.0).round() as i64
}

/// Principal (Tilgung) for one period — not including interest.
pub fn periodic_principal_minor(loan: &LoanCalcInput) -> i64 {
    match loan.payment_type {
        PaymentType::TilgungEuro => loan.tilgung_euro_minor.unwrap_or(0).max(0),
        PaymentType::TilgungPercent => {
            let pct = loan.tilgung_percent_basis_points.unwrap_or(0).max(0);
            let base = loan
                .original_principal_minor
                .unwrap_or(loan.remaining_balance_minor);
            let annual_principal = (base as f64 * pct as f64 / 10_000.0).round() as i64;
            match loan.payment_frequency {
                PaymentFrequency::Monthly => (annual_principal as f64 / 12.0).round() as i64,
                PaymentFrequency::Yearly => annual_principal,
            }
        }
    }
}

/// Total installment = interest on remaining balance + Tilgung (principal).
pub fn periodic_total_minor(loan: &LoanCalcInput) -> i64 {
    let apr = loan.apr_basis_points.unwrap_or(0);
    let interest =
        periodic_interest_minor(loan.remaining_balance_minor, apr, loan.payment_frequency);
    let principal = periodic_principal_minor(loan);
    let total = interest + principal;
    let max_pay = loan.remaining_balance_minor + interest;
    total.min(max_pay).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{LoanStatus, PaymentType};
    use chrono::NaiveDate;

    fn sample_loan() -> LoanCalcInput {
        LoanCalcInput {
            id: "1".into(),
            label: "Test".into(),
            status: LoanStatus::Active,
            remaining_balance_minor: 100_000_00,
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
    fn percent_tilgung_principal() {
        let loan = sample_loan();
        // 2% p.a. of 100k = 2000/year -> ~166.67/month
        let p = periodic_principal_minor(&loan);
        assert!(p >= 166_00 && p <= 167_00);
    }

    #[test]
    fn euro_tilgung_principal() {
        let mut loan = sample_loan();
        loan.payment_type = PaymentType::TilgungEuro;
        loan.tilgung_euro_minor = Some(500_00);
        assert_eq!(periodic_principal_minor(&loan), 500_00);
    }

    #[test]
    fn total_includes_interest_and_principal() {
        let loan = sample_loan();
        let total = periodic_total_minor(&loan);
        assert!(total > periodic_principal_minor(&loan));
    }

    #[test]
    fn yearly_percent_tilgung_uses_full_annual_amount() {
        let mut loan = sample_loan();
        loan.payment_frequency = PaymentFrequency::Yearly;
        loan.tilgung_percent_basis_points = Some(200);
        assert_eq!(periodic_principal_minor(&loan), 2_000_00);
    }

    #[test]
    fn yearly_euro_tilgung() {
        let mut loan = sample_loan();
        loan.payment_type = PaymentType::TilgungEuro;
        loan.tilgung_euro_minor = Some(1_200_00);
        loan.payment_frequency = PaymentFrequency::Yearly;
        assert_eq!(periodic_principal_minor(&loan), 1_200_00);
    }

    #[test]
    fn last_payment_capped_at_balance_plus_interest() {
        let mut loan = sample_loan();
        loan.payment_type = PaymentType::TilgungEuro;
        loan.tilgung_euro_minor = Some(500_00);
        loan.remaining_balance_minor = 200_00;
        let total = periodic_total_minor(&loan);
        let interest = periodic_interest_minor(200_00, 400, PaymentFrequency::Monthly);
        assert_eq!(total, 200_00 + interest);
    }
}
