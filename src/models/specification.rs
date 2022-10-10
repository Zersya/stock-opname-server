use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::Type;
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

#[derive(Serialize, Deserialize, Debug)]
pub struct SpecificationWithProduct {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub name: String,
    pub amount: i32,
    pub unit: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,

    pub products: Option<Vec<SimplifyProduct>>,
}

#[derive(Serialize, Deserialize, Debug, Type)]
pub struct SimplifyProduct {
    pub id: Uuid,
    pub name: String,
    pub updated_at: NaiveDateTime,
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

    pub async fn get_by_branch_id_with_product(
        db: &sqlx::PgPool,
        branch_id: Uuid,
    ) -> Result<Vec<SpecificationWithProduct>, sqlx::Error> {
        let specifications = sqlx::query_as!(
            SpecificationWithProduct,
            r#"
            SELECT s.id, 
                s.branch_id, 
                s.name, 
                s.amount, 
                s.unit, 
                s.created_at, 
                s.updated_at, 
                coalesce(array_agg((p.id, p.name, p.updated_at)) FILTER (WHERE p.id IS NOT NULL), '{}') AS "products: Vec<SimplifyProduct>"
            FROM specifications s
                LEFT JOIN product_specifications ps ON ps.specification_id = s.id
                LEFT JOIN products p ON p.id = ps.product_id

            WHERE s.branch_id = $1 AND s.deleted_at IS NULL
            GROUP BY
                s.id
            "#,
            branch_id
        )
        .fetch_all(db)
        .await?;

        Ok(specifications)
    }
}