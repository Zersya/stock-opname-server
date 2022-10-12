-- Add up migration script here
CREATE TABLE transaction_items (
    id uuid DEFAULT uuid_generate_v4(),
    transaction_id uuid NOT NULL,
    product_id uuid NOT NULL,
    product_name VARCHAR(255) NOT NULL,
    product_reference_id uuid NOT NULL,
    product_quantity INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE CASCADE,
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
);