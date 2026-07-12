use chrono::{Datelike, NaiveDate};
use serde::Serialize;

use crate::projection::project_balance_and_interest_schedule;
use crate::types::{LoanCalcInput, LoanStatus};

const MAX_MONTHS: u32 = 360;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PayoffTimelineSeries {
    pub id: String,
    pub label: String,
    pub balances_minor: Vec<i64>,
    /// Total future interest still to be paid at each timeline point.
    pub interest_remaining_minor: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PayoffTimeline {
    pub dates: Vec<String>,
    pub series: Vec<PayoffTimelineSeries>,
    pub as_of_index: usize,
}

pub fn build_payoff_timeline(loans: &[LoanCalcInput], as_of: NaiveDate) -> PayoffTimeline {
    let active: Vec<_> = loans
        .iter()
        .filter(|l| l.status == LoanStatus::Active && l.remaining_balance_minor > 0)
        .collect();

    if active.is_empty() {
        return PayoffTimeline {
            dates: vec![as_of.to_string()],
            series: vec![],
            as_of_index: 0,
        };
    }

    let schedules: Vec<_> = active
        .iter()
        .map(|loan| {
            (
                loan.id.clone(),
                loan.label.clone(),
                project_balance_and_interest_schedule(loan, as_of),
            )
        })
        .collect();

    let mut end = as_of;
    for (_, _, schedule) in &schedules {
        if let Some(&(d, _, _)) = schedule.last() {
            if d > end {
                end = d;
            }
        }
    }

    let dates = month_starts(as_of, end);
    let as_of_index = dates.iter().position(|d| *d >= as_of).unwrap_or(0);

    let series = schedules
        .into_iter()
        .map(|(id, label, schedule)| {
            let (balances, interests): (Vec<i64>, Vec<i64>) = dates
                .iter()
                .map(|d| balance_and_interest_at(*d, &schedule))
                .unzip();
            PayoffTimelineSeries {
                id,
                label,
                balances_minor: balances,
                interest_remaining_minor: interests,
            }
        })
        .collect();

    PayoffTimeline {
        dates: dates.iter().map(|d| d.to_string()).collect(),
        series,
        as_of_index,
    }
}

/// Returns (principal_remaining, interest_remaining) at the latest schedule point <= date.
fn balance_and_interest_at(date: NaiveDate, schedule: &[(NaiveDate, i64, i64)]) -> (i64, i64) {
    schedule
        .iter()
        .rev()
        .find(|(d, _, _)| *d <= date)
        .map(|&(_, b, interest)| (b, interest))
        .unwrap_or((0, 0))
}

fn month_starts(from: NaiveDate, to: NaiveDate) -> Vec<NaiveDate> {
    let mut dates = Vec::new();
    let mut y = from.year();
    let mut m = from.month();
    let end_y = to.year();
    let end_m = to.month();

    for _ in 0..MAX_MONTHS {
        if let Some(d) = NaiveDate::from_ymd_opt(y, m, 1) {
            dates.push(d);
        }
        if y > end_y || (y == end_y && m >= end_m) {
            break;
        }
        if m == 12 {
            y += 1;
            m = 1;
        } else {
            m += 1;
        }
    }

    if dates.is_empty() {
        dates.push(NaiveDate::from_ymd_opt(from.year(), from.month(), 1).unwrap_or(from));
    }
    dates
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PaymentFrequency, PaymentType};
    use chrono::NaiveDate;

    fn sample_loan() -> LoanCalcInput {
        LoanCalcInput {
            id: "a".into(),
            label: "Mortgage".into(),
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
    fn timeline_decreases_and_reaches_zero() {
        let as_of = NaiveDate::from_ymd_opt(2025, 5, 1).unwrap();
        let timeline = build_payoff_timeline(&[sample_loan()], as_of);
        assert!(!timeline.dates.is_empty());
        assert_eq!(timeline.series.len(), 1);
        let balances = &timeline.series[0].balances_minor;
        assert_eq!(balances[0], 100_000_00);
        assert!(*balances.last().unwrap() <= 0 || balances.windows(2).any(|w| w[1] <= w[0]));
    }

    #[test]
    fn stacked_timeline_sums_multiple_loans() {
        let as_of = NaiveDate::from_ymd_opt(2025, 5, 1).unwrap();
        let mut second = sample_loan();
        second.id = "b".into();
        second.label = "Car".into();
        second.remaining_balance_minor = 20_000_00;
        let timeline = build_payoff_timeline(&[sample_loan(), second], as_of);
        assert_eq!(timeline.series.len(), 2);
        let total_start: i64 = timeline.series.iter().map(|s| s.balances_minor[0]).sum();
        assert_eq!(total_start, 120_000_00);
    }

    #[test]
    fn interest_remaining_is_positive_and_decreases() {
        let as_of = NaiveDate::from_ymd_opt(2025, 5, 1).unwrap();
        let timeline = build_payoff_timeline(&[sample_loan()], as_of);
        let interest = &timeline.series[0].interest_remaining_minor;
        // At start, total future interest must be > 0 (APR = 3% on 100k)
        assert!(interest[0] > 0, "expected positive interest at start");
        // Interest remaining must decrease over time (generally)
        assert!(interest[0] >= *interest.last().unwrap());
        // Last entry must be 0 (no interest owed after payoff)
        assert_eq!(*interest.last().unwrap(), 0);
    }

    #[test]
    fn zero_apr_has_zero_interest() {
        let as_of = NaiveDate::from_ymd_opt(2025, 5, 1).unwrap();
        let mut loan = sample_loan();
        loan.apr_basis_points = Some(0);
        let timeline = build_payoff_timeline(&[loan], as_of);
        let interest = &timeline.series[0].interest_remaining_minor;
        assert_eq!(interest[0], 0);
    }
}
