use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProductSpecification {
    pub id: Uuid,
    pub product_id: Uuid,
    pub specification_id: Uuid,
    pub quantity: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl ProductSpecification {
    pub async fn create(
        db: &sqlx::PgPool,
        product_id: Uuid,
        specification_id: Uuid,
        quantity: i32,
    ) -> Result<ProductSpecification, sqlx::Error> {
        let product_specification = sqlx::query_as!(
            ProductSpecification,
            r#"
            INSERT INTO product_specifications (product_id, specification_id, quantity)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            product_id,
            specification_id,
            quantity
        )
        .fetch_one(db)
        .await?;

        Ok(product_specification)
    }

    pub async fn update(
        db: &sqlx::PgPool,
        product_id: Uuid,
        specification_id: Uuid,
        quantity: i32,
    ) -> Result<ProductSpecification, sqlx::Error> {
        let product_specification = sqlx::query_as!(
            ProductSpecification,
            r#"
            UPDATE product_specifications
            SET quantity = $1
            WHERE product_id = $2 AND specification_id = $3
            RETURNING *
            "#,
            quantity,
            product_id,
            specification_id,
        )
        .fetch_one(db)
        .await?;

        Ok(product_specification)
    }

    pub async fn get_by_product_and_specification(
        db: &sqlx::PgPool,
        product_id: Uuid,
        specification_id: Uuid,
    ) -> Result<ProductSpecification, sqlx::Error> {
        let product_specification = sqlx::query_as!(
            ProductSpecification,
            r#"
            SELECT * FROM product_specifications
            WHERE product_id = $1 AND specification_id = $2
            "#,
            product_id,
            specification_id
        )
        .fetch_one(db)
        .await?;

        Ok(product_specification)
    }

    pub async fn create_with_db_trx(
        db_transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        product_id: Uuid,
        specification_id: Uuid,
        quantity: i32,
    ) -> Result<ProductSpecification, sqlx::Error> {
        let product_specification = sqlx::query_as!(
            ProductSpecification,
            r#"
            INSERT INTO product_specifications (product_id, specification_id, quantity)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            product_id,
            specification_id,
            quantity
        )
        .fetch_one(db_transaction)
        .await?;

        Ok(product_specification)
    }

    pub async fn update_with_db_trx(
        db_transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        product_specification_id: Uuid,
        product_id: Uuid,
        specification_id: Uuid,
        quantity: i32,
    ) -> Result<ProductSpecification, sqlx::Error> {
        let product_specification = sqlx::query_as!(
            ProductSpecification,
            r#"
            UPDATE product_specifications
            SET quantity = $1, product_id = $2, specification_id = $3
            WHERE id = $4
            RETURNING *
            "#,
            quantity,
            product_id,
            specification_id,
            product_specification_id,
        )
        .fetch_one(db_transaction)
        .await?;

        Ok(product_specification)
    }

    pub async fn get_by_product_id_and_specification_id(
        db: &sqlx::PgPool,
        product_id: Uuid,
        specification_id: Uuid,
    ) -> Result<ProductSpecification, sqlx::Error> {
        let product_specification = sqlx::query_as!(
            ProductSpecification,
            r#"
            SELECT * FROM product_specifications
            WHERE product_id = $1 AND specification_id = $2
            "#,
            product_id,
            specification_id
        )
        .fetch_one(db)
        .await?;

        Ok(product_specification)
    }
}
