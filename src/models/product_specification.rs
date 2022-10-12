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
    
}