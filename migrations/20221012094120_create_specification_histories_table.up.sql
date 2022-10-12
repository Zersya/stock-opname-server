-- Add up migration script here
CREATE TABLE specification_histories (
    id uuid DEFAULT uuid_generate_v4(),
    flow_type VARCHAR(50) NOT NULL,  -- flow_type of the specification history | e.g. "IN" or "OUT"
    specification_id uuid NOT NULL,
    created_by uuid NOT NULL,
    quantity INTEGER NOT NULL,
    transaction_item_id uuid,
    note VARCHAR(255),
    price INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (transaction_item_id) REFERENCES transaction_items(id) ON DELETE CASCADE,
    FOREIGN KEY (specification_id) REFERENCES specifications(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
);