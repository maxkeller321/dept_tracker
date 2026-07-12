-- Anchor for auto-applied regular installments (date-only, may differ from loan_start_date).
ALTER TABLE loans ADD COLUMN first_payment_date TEXT;

UPDATE loans
SET first_payment_date = loan_start_date
WHERE first_payment_date IS NULL AND loan_start_date IS NOT NULL;
