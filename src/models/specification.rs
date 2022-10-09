use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Specification {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub name: String,
    pub amount: i32,
    pub unit: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl Specification {
    pub async fn create(
        db: &sqlx::PgPool,
        branch_id: Uuid,
        name: String,
        amount: i32,
        unit: String,
    ) -> Result<Specification, sqlx::Error> {
        let specification = sqlx::query_as!(
            Specification,
            r#"
            INSERT INTO specifications (branch_id, name, amount, unit)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            branch_id,
            name,
            amount,
            unit
        )
        .fetch_one(db)
        .await?;

        Ok(specification)
    }

    pub async fn get_all(db: &sqlx::PgPool) -> Result<Vec<Specification>, sqlx::Error> {
        let specifications = sqlx::query_as!(
            Specification,
            r#"
            SELECT * FROM specifications
            "#,
        )
        .fetch_all(db)
        .await?;

        Ok(specifications)
    }

    pub async fn get_by_id(db: &sqlx::PgPool, id: Uuid) -> Result<Specification, sqlx::Error> {
        let specification = sqlx::query_as!(
            Specification,
            r#"
            SELECT * FROM specifications
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(db)
        .await?;

        Ok(specification)
    }
}