use chrono::{Datelike, Months, NaiveDate};

use crate::types::PaymentFrequency;

/// Anchor for installment due dates: explicit first payment date, else loan start.
/// All scheduling uses `NaiveDate` only — no UTC/timezone conversion.
pub fn payment_due_anchor(first_payment: Option<NaiveDate>, loan_start: NaiveDate) -> NaiveDate {
    first_payment.unwrap_or(loan_start)
}

/// Calendar due dates for regular installments from the payment anchor through `through`.
/// Skips dates on or before `last_regular` (already recorded).
pub fn due_regular_payment_dates(
    first_due: NaiveDate,
    frequency: PaymentFrequency,
    last_regular: Option<NaiveDate>,
    through: NaiveDate,
) -> Vec<NaiveDate> {
    let mut dates = Vec::new();
    let mut due = match last_regular {
        Some(last) => advance_due(last, frequency),
        None => first_due,
    };
    while due <= through {
        dates.push(due);
        due = advance_due(due, frequency);
    }
    dates
}

/// First installment due date strictly after `as_of` on the schedule from `anchor`.
pub fn first_due_after(
    anchor: NaiveDate,
    frequency: PaymentFrequency,
    as_of: NaiveDate,
) -> NaiveDate {
    let mut due = anchor;
    while due <= as_of {
        due = advance_due(due, frequency);
    }
    due
}

pub fn advance_due(date: NaiveDate, frequency: PaymentFrequency) -> NaiveDate {
    match frequency {
        PaymentFrequency::Monthly => date
            .checked_add_months(Months::new(1))
            .unwrap_or_else(|| end_of_month(date.year(), date.month())),
        PaymentFrequency::Yearly => date
            .checked_add_months(Months::new(12))
            .unwrap_or(date),
    }
}

fn end_of_month(year: i32, month: u32) -> NaiveDate {
    let (y, m) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(y, m, 1)
        .unwrap()
        .pred_opt()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_monthly_dates_from_first_due() {
        let first = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let through = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let dates = due_regular_payment_dates(first, PaymentFrequency::Monthly, None, through);
        assert_eq!(dates.len(), 3);
        assert_eq!(dates[0], first);
        assert_eq!(dates[1], NaiveDate::from_ymd_opt(2024, 2, 15).unwrap());
        assert_eq!(dates[2], NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
    }

    #[test]
    fn resumes_after_last_regular() {
        let first = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let last = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        let through = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
        let dates =
            due_regular_payment_dates(first, PaymentFrequency::Monthly, Some(last), through);
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2024, 3, 1).unwrap());
        assert_eq!(dates[1], NaiveDate::from_ymd_opt(2024, 4, 1).unwrap());
    }

    #[test]
    fn first_due_after_skips_paid_period() {
        let anchor = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let as_of = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        assert_eq!(
            first_due_after(anchor, PaymentFrequency::Monthly, as_of),
            NaiveDate::from_ymd_opt(2024, 4, 1).unwrap()
        );
    }

    #[test]
    fn no_dues_before_first_payment_date() {
        let loan_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let first_payment = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        let anchor = payment_due_anchor(Some(first_payment), loan_start);
        let through = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let dates = due_regular_payment_dates(anchor, PaymentFrequency::Monthly, None, through);
        assert!(dates.is_empty());
    }

    #[test]
    fn anchor_falls_back_to_loan_start() {
        let start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        assert_eq!(payment_due_anchor(None, start), start);
    }

    #[test]
    fn yearly_schedule_from_anchor() {
        let first = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let through = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        let dates = due_regular_payment_dates(first, PaymentFrequency::Yearly, None, through);
        assert_eq!(dates.len(), 3);
        assert_eq!(dates[2], NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());
    }
}
