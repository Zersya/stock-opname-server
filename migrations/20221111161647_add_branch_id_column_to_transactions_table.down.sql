-- Add down migration script here
ALTER TABLE transactions DROP COLUMN branch_id;
