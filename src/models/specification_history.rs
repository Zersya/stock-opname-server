use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct SpecificationHistory {
    pub id: Uuid,
    pub flow_type: String,
    pub specification_id: Uuid,
    pub created_by: Uuid,
    pub quantity: i32,
    pub transaction_item_id: Option<Uuid>,
    pub note: Option<String>,
    pub price: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl SpecificationHistory {
    pub async fn create(
        db: &sqlx::PgPool,
        specification_id: Uuid,
        transaction_item_id: Option<Uuid>,
        created_by: Uuid,
        note: String,
        flow_type: String,
        quantity: i32,
        price: i32,
    ) -> Result<SpecificationHistory, sqlx::Error> {
        let specification_history = sqlx::query_as!(
            SpecificationHistory,
            r#"
            INSERT INTO specification_histories (flow_type, specification_id, created_by, quantity, transaction_item_id, note, price)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            flow_type,
            specification_id,
            created_by,
            quantity,
            transaction_item_id,
            note,
            price
        )
        .fetch_one(db)
        .await?;

        Ok(specification_history)
    }

    pub async fn get_all(db: &sqlx::PgPool) -> Result<Vec<SpecificationHistory>, sqlx::Error> {
        let specification_histories = sqlx::query_as!(
            SpecificationHistory,
            r#"
            SELECT * FROM specification_histories
            "#,
        )
        .fetch_all(db)
        .await?;

        Ok(specification_histories)
    }

    pub async fn get_by_id(
        db: &sqlx::PgPool,
        id: Uuid,
    ) -> Result<SpecificationHistory, sqlx::Error> {
        let specification_history = sqlx::query_as!(
            SpecificationHistory,
            r#"
            SELECT * FROM specification_histories
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(db)
        .await?;

        Ok(specification_history)
    }
}