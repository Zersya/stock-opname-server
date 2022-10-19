use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Branch {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub reference_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl Branch {
    pub async fn create(
        db: &sqlx::PgPool,
        user_id: Uuid,
        name: String,
        reference_id: Uuid,
    ) -> Result<Branch, sqlx::Error> {
        let branch = sqlx::query_as!(
            Branch,
            r#"
            INSERT INTO branches (user_id, name, reference_id)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            user_id,
            name,
            reference_id
        )
        .fetch_one(db)
        .await?;

        Ok(branch)
    }

//    pub async fn get_all(db: &sqlx::PgPool) -> Result<Vec<Branch>, sqlx::Error> {
//        let branches = sqlx::query_as!(
//            Branch,
//            r#"
//            SELECT * FROM branches
//            "#,
//        )
//        .fetch_all(db)
//        .await?;
//
//        Ok(branches)
//    }

    pub async fn get_by_id(db: &sqlx::PgPool, id: Uuid) -> Result<Branch, sqlx::Error> {
        let branch = sqlx::query_as!(
            Branch,
            r#"
            SELECT * FROM branches
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(db)
        .await?;

        Ok(branch)
    }

//    pub async fn get_by_user_id(
//        db: &sqlx::PgPool,
//        user_id: Uuid,
//    ) -> Result<Vec<Branch>, sqlx::Error> {
//        let branches = sqlx::query_as!(
//            Branch,
//            r#"
//            SELECT * FROM branches
//            WHERE user_id = $1
//            "#,
//            user_id
//        )
//        .fetch_all(db)
//        .await?;
//
//        Ok(branches)
//    }

    pub async fn get_by_reference_id(
        db: &sqlx::PgPool,
        reference_id: Uuid,
    ) -> Result<Branch, sqlx::Error> {
        let branch = sqlx::query_as!(
            Branch,
            r#"
            SELECT * FROM branches
            WHERE reference_id = $1
            "#,
            reference_id
        )
        .fetch_one(db)
        .await?;

        Ok(branch)
    }

    pub async fn update(
        db: &sqlx::PgPool,
        id: Uuid,
        name: String,
        reference_id: Uuid,
    ) -> Result<Branch, sqlx::Error> {
        let branch = sqlx::query_as!(
            Branch,
            r#"
            UPDATE branches
            SET name = $1, reference_id = $2
            WHERE id = $3
            RETURNING *
            "#,
            name,
            reference_id,
            id
        )
        .fetch_one(db)
        .await?;

        Ok(branch)
    }

//    pub async fn delete(db: &sqlx::PgPool, id: Uuid) -> Result<Branch, sqlx::Error> {
//        let branch = sqlx::query_as!(
//            Branch,
//            r#"
//            DELETE FROM branches
//            WHERE id = $1
//            RETURNING *
//            "#,
//            id
//        )
//        .fetch_one(db)
//        .await?;
//
//        Ok(branch)
//    }
}
