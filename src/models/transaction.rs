use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub id: Uuid,
    pub created_by: Option<Uuid>,
    pub note: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionItem {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,
    pub product_reference_id: Uuid,
    pub product_quantity: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SimplifyTransaction {
    pub id: Uuid,
    pub created_by: Option<Uuid>,
    pub note: Option<String>,
    pub created_at: NaiveDateTime,

    pub items: Option<Vec<SimplifyTransactionItem>>,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
pub struct SimplifyTransactionItem {
    pub id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,
    pub product_reference_id: Uuid,
    pub product_quantity: i32,
    pub created_at: NaiveDateTime,
}

impl Transaction {
    pub async fn create(
        db_trx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        created_by: Option<Uuid>,
        note: Option<String>,
    ) -> Result<Transaction, sqlx::Error> {
        let transaction = sqlx::query_as!(
            Transaction,
            r#"
            INSERT INTO transactions (created_by, note)
            VALUES ($1, $2)
            RETURNING *
            "#,
            created_by,
            note
        )
        .fetch_one(db_trx)
        .await?;

        Ok(transaction)
    }
}

impl TransactionItem {
    pub async fn create(
        db_trx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        transaction_id: Uuid,
        product_id: Uuid,
        product_name: String,
        product_reference_id: Uuid,
        product_quantity: i32,
    ) -> Result<TransactionItem, sqlx::Error> {
        let transaction_item = sqlx::query_as!(
            TransactionItem,
            r#"
            INSERT INTO transaction_items (transaction_id, product_id, product_name, product_reference_id, product_quantity)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            transaction_id,
            product_id,
            product_name,
            product_reference_id,
            product_quantity
        )
        .fetch_one(db_trx)
        .await?;

        Ok(transaction_item)
    }
}

impl SimplifyTransaction {
    pub async fn get_by_id_with_items(db: &sqlx::PgPool, id: Uuid) -> Result<Self, sqlx::Error> {
        let transaction = sqlx::query_as!(
            SimplifyTransaction,
            r#"
                SELECT
                    t.id,
                    t.created_by,
                    t.note,
                    coalesce(array_agg((ti.id, ti.product_id, ti.product_name, ti.product_reference_id, ti.product_quantity, ti.created_at)) FILTER (WHERE ti.id IS NOT NULL AND ti.deleted_at IS NULL), '{}') AS "items: Vec<SimplifyTransactionItem>",
                    t.created_at
                FROM
                    transactions t
                    LEFT JOIN transaction_items ti ON ti.transaction_id = t.id
                WHERE
                    t.id = $1
                GROUP BY
                    t.id
            "#,
            id
        )
        .fetch_one(db)
        .await?;

        Ok(transaction)
    }

//    pub async fn get_all_with_items(db: &sqlx::PgPool) -> Result<Vec<Self>, sqlx::Error> {
//        let transactions = sqlx::query_as!(
//            SimplifyTransaction,
//            r#"
//                SELECT
//                    t.id,
//                    t.created_by,
//                    t.note,
//                    coalesce(array_agg((ti.id, ti.product_id, ti.product_name, ti.product_reference_id, ti.product_quantity, ti.created_at)) FILTER (WHERE ti.id IS NOT NULL AND ti.deleted_at IS NULL), '{}') AS "items: Vec<SimplifyTransactionItem>",
//                    t.created_at
//                FROM
//                    transactions t
//                    LEFT JOIN transaction_items ti ON ti.transaction_id = t.id
//                GROUP BY
//                    t.id
//            "#
//        )
//        .fetch_all(db)
//        .await?;
//
//        Ok(transactions)
//    }
}
