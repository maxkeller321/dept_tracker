pub mod amortization;
pub mod dashboard;
pub mod due_payments;
pub mod interest;
pub mod money;
pub mod payment_split;
pub mod payoff_timeline;
pub mod projection;
pub mod types;
pub mod validation;

pub use dashboard::{build_dashboard, DashboardResponse, HouseholdSummary, LoanSummary};
pub use types::*;
