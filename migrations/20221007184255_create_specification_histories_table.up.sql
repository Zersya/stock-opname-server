-- Add up migration script here
CREATE TABLE specification_histories (
    id uuid DEFAULT uuid_generate_v4(),
    specification_id uuid NOT NULL,
    created_by uuid NOT NULL,
    note VARCHAR(255) NOT NULL,
    amount INTEGER NOT NULL,
    price INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (specification_id) REFERENCES specifications(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
);