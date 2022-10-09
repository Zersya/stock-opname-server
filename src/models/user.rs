use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl User {
    pub async fn login(
        db: &sqlx::PgPool,
        email: String,
        password: String,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users
            WHERE email = $1 AND password = $2
            "#,
            email,
            password
        )
        .fetch_one(db)
        .await?;

        Ok(user)
    }

    pub async fn create(
        db: &sqlx::PgPool,
        name: String,
        email: String,
        password: String,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (name, email, password)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            name,
            email,
            password
        )
        .fetch_one(db)
        .await?;

        Ok(user)
    }

    pub async fn get_all(db: sqlx::PgPool) -> Result<Vec<User>, sqlx::Error> {
        let users = sqlx::query_as!(User, r"SELECT * FROM users")
            .fetch_all(&db)
            .await?;

        Ok(users)
    }

    pub async fn find_by_email(
        db: &sqlx::PgPool,
        email: String,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_one(db)
        .await?;

        Ok(user)
    }
}