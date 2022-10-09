-- Add up migration script here
CREATE TABLE product_specifications (
    id uuid DEFAULT uuid_generate_v4(),
    product_id uuid NOT NULL,
    specification_id uuid NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    FOREIGN KEY (specification_id) REFERENCES specifications(id) ON DELETE CASCADE
);