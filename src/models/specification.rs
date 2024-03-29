use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{product::SimplifyProduct, specification_history::SimplifySpecificationHistory};

#[derive(Serialize, Deserialize, Debug)]
pub struct Specification {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub name: String,
    pub smallest_unit: i32,
    pub unit_name: String,
    pub unit: String,
    pub lowest_price: f64,
    pub raw_price: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpecificationWithProduct {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub name: String,
    pub smallest_unit: i32,
    pub unit_name: String,
    pub unit: String,
    pub lowest_price: f64,
    pub raw_price: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,

    pub products: Option<Vec<SimplifyProduct>>,
    pub specification_histories: Option<Vec<SimplifySpecificationHistory>>,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
pub struct SimplifySpecification {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_specification_quantity: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_specification_price: Option<f64>,
}

impl Specification {
    pub async fn create(
        db: &sqlx::PgPool,
        branch_id: Uuid,
        name: String,
        smallest_unit: i32,
        unit_name: String,
        unit: String,
        lowest_price: f64,
        raw_price: i32,
    ) -> Result<Specification, sqlx::Error> {
        let specification = sqlx::query_as!(
            Specification,
            r#"
            INSERT INTO specifications (branch_id, name, smallest_unit, unit_name, unit, lowest_price, raw_price)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            branch_id,
            name,
            smallest_unit,
            unit_name,
            unit,
            lowest_price,
            raw_price
        )
        .fetch_one(db)
        .await?;

        Ok(specification)
    }

    pub async fn create_with_db_trx(
        db_trx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        branch_id: Uuid,
        name: String,
        smallest_unit: i32,
        unit_name: String,
        unit: String,
        lowest_price: f64,
        raw_price: i32,
    ) -> Result<Specification, sqlx::Error> {
        let specification = sqlx::query_as!(
            Specification,
            r#"
            INSERT INTO specifications (branch_id, name, smallest_unit, unit_name, unit, lowest_price, raw_price)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            branch_id,
            name,
            smallest_unit,
            unit_name,
            unit,
            lowest_price,
            raw_price
        )
        .fetch_one(db_trx)
        .await?;

        Ok(specification)
    }

    pub async fn update_with_db_trx_by_name_and_branch_id(
        db_trx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        branch_id: &Uuid,
        name: &String,
        smallest_unit: &i32,
        unit_name: &String,
        unit: &String,
        lowest_price: &f64,
        raw_price: &i32,
    ) -> Result<Specification, sqlx::Error> {
        let specification = sqlx::query_as!(
            Specification,
            r#"
            UPDATE specifications
            SET smallest_unit = $1, unit_name = $2, unit = $3, lowest_price = $4, raw_price = $5
            WHERE branch_id = $6 AND name = $7
            RETURNING *
            "#,
            smallest_unit,
            unit_name,
            unit,
            lowest_price,
            raw_price,
            branch_id,
            name
        )
        .fetch_one(db_trx)
        .await?;

        Ok(specification)
    }

    //    pub async fn get_all(db: &sqlx::PgPool) -> Result<Vec<Specification>, sqlx::Error> {
    //        let specifications = sqlx::query_as!(
    //            Specification,
    //            r#"
    //            SELECT * FROM specifications
    //            "#,
    //        )
    //        .fetch_all(db)
    //        .await?;
    //
    //        Ok(specifications)
    //    }

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

    pub async fn get_by_name_and_branch_id(
        db: &sqlx::PgPool,
        name: &String,
        branch_id: &Uuid,
    ) -> Result<Specification, sqlx::Error> {
        let specification = sqlx::query_as!(
            Specification,
            r#"
            SELECT * FROM specifications
            WHERE name = $1 AND branch_id = $2
            "#,
            name,
            branch_id
        )
        .fetch_one(db)
        .await?;

        Ok(specification)
    }

    pub async fn delete(db: &sqlx::PgPool, id: Uuid) -> Result<Specification, sqlx::Error> {
        let specification = sqlx::query_as!(
            Specification,
            r#"
            UPDATE specifications
            SET deleted_at = now()
            WHERE id = $1
            RETURNING *
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
            SELECT
                s.id,
                s.branch_id,
                s.name,
                s.smallest_unit,
                s.unit_name,
                s.unit,
                s.lowest_price,
                s.raw_price,
                s.created_at,
                s.updated_at,
                coalesce(array_agg(DISTINCT (p.id, p.name, ps.quantity, p.updated_at)) FILTER (WHERE p.id IS NOT NULL
                    AND p.deleted_at IS NULL), '{}') AS "products: Vec<SimplifyProduct>",
                coalesce(array_agg((sh.id, sh.flow_type, sh.note, sh.quantity, sh.price, sh.unit_price, sh.created_at)
                ORDER BY
                    sh.created_at DESC) FILTER (WHERE sh.id IS NOT NULL
                    AND sh.created_at >= now() - interval '7 day'), '{}') AS "specification_histories: Vec<SimplifySpecificationHistory>"
            FROM
                specifications s
                LEFT JOIN product_specifications ps ON ps.specification_id = s.id
                INNER JOIN products p ON p.id = ps.product_id AND s.id = ps.specification_id
                LEFT JOIN specification_histories sh ON sh.specification_id = s.id
            WHERE
                s.branch_id = $1
                AND s.deleted_at IS NULL
            GROUP BY
                s.id
            ORDER BY
                s.created_at DESC
            "#,
            branch_id
        )
        .fetch_all(db)
        .await?;

        Ok(specifications)
    }
}
