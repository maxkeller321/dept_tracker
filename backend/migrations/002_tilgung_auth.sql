-- Tilgung payment types + local auth
ALTER TABLE settings ADD COLUMN auth_username TEXT;
ALTER TABLE settings ADD COLUMN auth_password_hash TEXT;
ALTER TABLE settings ADD COLUMN auth_session_token TEXT;
ALTER TABLE settings ADD COLUMN auth_session_expires TEXT;

ALTER TABLE loans ADD COLUMN tilgung_percent_basis_points INTEGER;

UPDATE loans SET payment_type = 'tilgung_euro' WHERE payment_type = 'fixed';
UPDATE loans SET payment_type = 'tilgung_percent' WHERE payment_type = 'apr';
UPDATE loans SET tilgung_percent_basis_points = 200 WHERE payment_type = 'tilgung_percent' AND tilgung_percent_basis_points IS NULL;
