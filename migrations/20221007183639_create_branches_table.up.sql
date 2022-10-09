-- Add up migration script here
CREATE TABLE branches (
    id uuid DEFAULT uuid_generate_v4(),
    user_id uuid NOT NULL,
    name VARCHAR(255) NOT NULL,
    reference_id uuid NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);