use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{Type};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Product {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub name: String,
    pub reference_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProductWithSpecifications {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub name: String,
    pub reference_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,

    pub specifications: Option<Vec<SimplifySpecification>>,
}
#[derive(Serialize, Deserialize, Debug, Type)]
pub struct SimplifySpecification {
    pub id: Uuid,
    pub name: String,
    pub specification_quantity: i32,
    pub product_specification_quantity: i32,
    pub unit: String,
}


impl Product {
    pub async fn create(
        db: &sqlx::PgPool,
        branch_id: Uuid,
        name: String,
        reference_id: Uuid,
    ) -> Result<Product, sqlx::Error> {
        let product = sqlx::query_as!(
            Product,
            r#"
            INSERT INTO products (branch_id, name, reference_id)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            branch_id,
            name,
            reference_id
        )
        .fetch_one(db)
        .await?;

        Ok(product)
    }

    pub async fn get_all(db: &sqlx::PgPool) -> Result<Vec<Product>, sqlx::Error> {
        let products = sqlx::query_as!(
            Product,
            r#"
            SELECT * FROM products
            "#,
        )
        .fetch_all(db)
        .await?;

        Ok(products)
    }

    // one to many get all product with many specifications
    pub async fn get_all_with_specifications(
        db: &sqlx::PgPool,
        branch_id: Uuid,
    ) -> Result<Vec<ProductWithSpecifications>, sqlx::Error> {
        let products = sqlx::query_as!(
            ProductWithSpecifications,
            r#"
            SELECT
                p.id,
                p.branch_id,
                p.name,
                p.reference_id,
                p.created_at,
                p.updated_at,
                coalesce(array_agg((s.id, s.name, s.quantity, ps.quantity, s.unit)) FILTER (WHERE s.id IS NOT NULL AND s.deleted_at IS NULL), '{}') AS "specifications: Vec<SimplifySpecification>"
            FROM
                products p
                LEFT JOIN product_specifications ps ON ps.product_id = p.id
                LEFT JOIN specifications s ON s.id = ps.specification_id
            WHERE p.branch_id = $1 AND p.deleted_at IS NULL
            GROUP BY
                p.id
            ORDER BY p.created_at DESC
            "#,
            branch_id
        )
        .fetch_all(db)
        .await?;

        Ok(products)
    }

    pub async fn get_by_id(db: &sqlx::PgPool, id: Uuid) -> Result<Product, sqlx::Error> {
        let product = sqlx::query_as!(
            Product,
            r#"
            SELECT * FROM products
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(db)
        .await?;

        Ok(product)
    }

    pub async fn get_by_branch_id(
        db: &sqlx::PgPool,
        branch_id: Uuid,
    ) -> Result<Vec<Product>, sqlx::Error> {
        let products = sqlx::query_as!(
            Product,
            r#"
            SELECT * FROM products
            WHERE branch_id = $1
            "#,
            branch_id
        )
        .fetch_all(db)
        .await?;

        Ok(products)
    }

    pub async fn update(
        db: &sqlx::PgPool,
        id: Uuid,
        branch_id: Uuid,
        name: String,
        reference_id: Uuid,
    ) -> Result<Product, sqlx::Error> {
        let product = sqlx::query_as!(
            Product,
            r#"
            UPDATE products
            SET branch_id = $2, name = $3, reference_id = $4
            WHERE id = $1
            RETURNING *
            "#,
            id,
            branch_id,
            name,
            reference_id
        )
        .fetch_one(db)
        .await?;

        Ok(product)
    }

    pub async fn update_by_reference_id(
        db: &sqlx::PgPool,
        reference_id: Uuid,
        name: String,
    ) -> Result<Product, sqlx::Error> {
        let product = sqlx::query_as!(
            Product,
            r#"
            UPDATE products
            SET name = $2
            WHERE reference_id = $1
            RETURNING *
            "#,
            reference_id,
            name
        )
        .fetch_one(db)
        .await?;

        Ok(product)
    }

    pub async fn delete(db: &sqlx::PgPool, id: Uuid) -> Result<Product, sqlx::Error> {
        let product = sqlx::query_as!(
            Product,
            r#"
            DELETE FROM products
            WHERE id = $1
            RETURNING *
            "#,
            id
        )
        .fetch_one(db)
        .await?;

        Ok(product)
    }
}
