use crate::amortization::{periodic_rate, periodic_total_minor};
use crate::types::LoanCalcInput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentSplit {
    pub interest_portion_minor: i64,
    pub principal_portion_minor: i64,
    pub balance_after_minor: i64,
}

/// Split a scheduled regular installment: interest on current balance + fixed Tilgung.
pub fn split_regular_payment(loan: &LoanCalcInput, current_balance_minor: i64) -> PaymentSplit {
    let amount = periodic_total_minor(&LoanCalcInput {
        remaining_balance_minor: current_balance_minor,
        ..loan.clone()
    });
    split_payment(loan, amount, current_balance_minor)
}

/// Split an arbitrary payment amount: interest first, remainder to principal.
pub fn split_payment(
    loan: &LoanCalcInput,
    amount_minor: i64,
    current_balance_minor: i64,
) -> PaymentSplit {
    let r = loan
        .apr_basis_points
        .map(|a| periodic_rate(a, loan.payment_frequency))
        .unwrap_or(0.0);
    let balance = current_balance_minor as f64 / 100.0;
    let amount = amount_minor as f64 / 100.0;
    let interest = (balance * r).min(amount);
    let principal = (amount - interest).max(0.0).min(balance);
    let balance_after = ((balance - principal) * 100.0).max(0.0).round() as i64;
    PaymentSplit {
        interest_portion_minor: (interest * 100.0).round() as i64,
        principal_portion_minor: (principal * 100.0).round() as i64,
        balance_after_minor: balance_after,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{LoanStatus, PaymentFrequency, PaymentType};
    use chrono::NaiveDate;

    fn euro_loan(balance: i64) -> LoanCalcInput {
        LoanCalcInput {
            id: "1".into(),
            label: "Test".into(),
            status: LoanStatus::Active,
            remaining_balance_minor: balance,
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
    fn regular_split_interest_plus_tilgung_euro() {
        let loan = euro_loan(100_000_00);
        let split = split_regular_payment(&loan, 100_000_00);
        assert_eq!(split.interest_portion_minor, 333_33);
        assert_eq!(split.principal_portion_minor, 500_00);
        assert_eq!(split.balance_after_minor, 99_500_00);
    }

    #[test]
    fn last_payment_caps_principal_at_balance() {
        let loan = euro_loan(300_00);
        let split = split_regular_payment(&loan, 300_00);
        assert_eq!(split.principal_portion_minor, 300_00);
        assert_eq!(split.balance_after_minor, 0);
        assert!(split.interest_portion_minor > 0);
    }

    #[test]
    fn percent_tilgung_uses_original_principal_base() {
        let loan = LoanCalcInput {
            payment_type: PaymentType::TilgungPercent,
            tilgung_euro_minor: None,
            tilgung_percent_basis_points: Some(200),
            remaining_balance_minor: 80_000_00,
            original_principal_minor: Some(100_000_00),
            ..euro_loan(80_000_00)
        };
        let split = split_regular_payment(&loan, 80_000_00);
        assert_eq!(split.principal_portion_minor, 166_67);
    }

    #[test]
    fn zero_balance_yields_zero_split() {
        let loan = euro_loan(0);
        let split = split_regular_payment(&loan, 0);
        assert_eq!(split.interest_portion_minor, 0);
        assert_eq!(split.principal_portion_minor, 0);
        assert_eq!(split.balance_after_minor, 0);
    }
}
