-- Add up migration script here
CREATE TABLE specifications (
    id uuid DEFAULT uuid_generate_v4(),
    branch_id uuid NOT NULL,
    name VARCHAR(255) NOT NULL,
    smallest_unit INTEGER NOT NULL,
    unit_name VARCHAR(255) NOT NULL,
    unit VARCHAR(255) NOT NULL,
    lowest_price FLOAT NOT NULL,
    raw_price INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE CASCADE
);