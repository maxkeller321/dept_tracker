use crate::types::{PaymentFrequency, PaymentType};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ValidationError {
    #[error("loan label is required")]
    MissingLabel,
    #[error("remaining balance must be positive")]
    NonPositiveBalance,
    #[error("interest rate (APR) is required")]
    MissingApr,
    #[error("APR cannot be negative")]
    InvalidApr,
    #[error("prozentuale Tilgung (%) is required")]
    MissingTilgungPercent,
    #[error("Tilgung percent cannot be negative")]
    InvalidTilgungPercent,
    #[error("Tilgung (€) per period is required")]
    MissingTilgungEuro,
    #[error("Tilgung (€) must be positive")]
    InvalidTilgungEuro,
    #[error("invalid payment type")]
    InvalidPaymentType,
}

#[derive(Debug, Clone)]
pub struct CreateLoanValidation {
    pub label: String,
    pub remaining_balance_minor: i64,
    pub payment_frequency: PaymentFrequency,
    pub payment_type: PaymentType,
    pub tilgung_euro_minor: Option<i64>,
    pub tilgung_percent_basis_points: Option<i32>,
    pub apr_basis_points: Option<i32>,
}

pub fn validate_create_loan(input: &CreateLoanValidation) -> Result<(), ValidationError> {
    if input.label.trim().is_empty() {
        return Err(ValidationError::MissingLabel);
    }
    if input.remaining_balance_minor <= 0 {
        return Err(ValidationError::NonPositiveBalance);
    }
    match input.apr_basis_points {
        None => return Err(ValidationError::MissingApr),
        Some(a) if a < 0 => return Err(ValidationError::InvalidApr),
        Some(_) => {}
    }
    match input.payment_type {
        PaymentType::TilgungPercent => match input.tilgung_percent_basis_points {
            None => return Err(ValidationError::MissingTilgungPercent),
            Some(p) if p < 0 => return Err(ValidationError::InvalidTilgungPercent),
            Some(_) => {}
        },
        PaymentType::TilgungEuro => match input.tilgung_euro_minor {
            None => return Err(ValidationError::MissingTilgungEuro),
            Some(e) if e <= 0 => return Err(ValidationError::InvalidTilgungEuro),
            Some(_) => {}
        },
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_percent_tilgung_with_apr() {
        let input = CreateLoanValidation {
            label: "Test".into(),
            remaining_balance_minor: 100_000,
            payment_frequency: PaymentFrequency::Monthly,
            payment_type: PaymentType::TilgungPercent,
            tilgung_euro_minor: None,
            tilgung_percent_basis_points: Some(200),
            apr_basis_points: Some(350),
        };
        assert!(validate_create_loan(&input).is_ok());
    }

    #[test]
    fn accepts_euro_tilgung_with_apr() {
        let input = CreateLoanValidation {
            label: "Test".into(),
            remaining_balance_minor: 100_000,
            payment_frequency: PaymentFrequency::Monthly,
            payment_type: PaymentType::TilgungEuro,
            tilgung_euro_minor: Some(500),
            tilgung_percent_basis_points: None,
            apr_basis_points: Some(350),
        };
        assert!(validate_create_loan(&input).is_ok());
    }

    #[test]
    fn rejects_missing_tilgung_percent() {
        let input = CreateLoanValidation {
            label: "Test".into(),
            remaining_balance_minor: 100_000,
            payment_frequency: PaymentFrequency::Monthly,
            payment_type: PaymentType::TilgungPercent,
            tilgung_percent_basis_points: None,
            tilgung_euro_minor: None,
            apr_basis_points: Some(350),
        };
        assert_eq!(
            validate_create_loan(&input),
            Err(ValidationError::MissingTilgungPercent)
        );
    }
}
