use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct SpecificationHistory {
    pub id: Uuid,
    pub specification_id: Uuid,
    pub created_by: Uuid,
    pub note: String,
    pub amount: i32,
    pub price: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl SpecificationHistory {
    pub async fn create(
        db: &sqlx::PgPool,
        specification_id: Uuid,
        created_by: Uuid,
        note: String,
        amount: i32,
        price: i32,
    ) -> Result<SpecificationHistory, sqlx::Error> {
        let specification_history = sqlx::query_as!(
            SpecificationHistory,
            r#"
            INSERT INTO specification_histories (specification_id, created_by, note, amount, price)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            specification_id,
            created_by,
            note,
            amount,
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