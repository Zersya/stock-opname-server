-- Add up migration script here
ALTER TABLE transactions ADD COLUMN branch_id uuid NOT NULL REFERENCES branches(id);
