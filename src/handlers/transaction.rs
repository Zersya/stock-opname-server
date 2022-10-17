use crate::errors::Errors;
use crate::models::branch::Branch;
use crate::models::product::Product;
use crate::models::requests::transaction::RequestCreateTransaction;
use crate::models::responses::DefaultResponse;
use crate::models::specification_history::SpecificationHistory;
use crate::models::transaction::{SimplifyTransaction, Transaction, TransactionItem};
use crate::models::user::User;

use axum::{extract::Path, extract::State, response::Json};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    State(db): State<PgPool>,
    Path((branch_id,)): Path<(Uuid,)>,
    Json(payload): Json<RequestCreateTransaction>,
) -> Result<Json<Value>, Errors> {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        return Err(Errors::new(&[("branch_id", "branch not found")]));
    }

    if payload.created_by.is_some() {
        let user = User::get_by_id(&db, payload.created_by.unwrap()).await;

        if user.is_err() {
            return Err(Errors::new(&[("created_by", "user not found")]));
        }
    }

    let mut db_transaction = db.begin().await.unwrap();

    let transaction =
        Transaction::create(&mut db_transaction, payload.created_by, payload.note).await;

    if transaction.is_err() {
        return Err(Errors::new(&[(
            "transaction",
            "failed to create transaction",
        )]));
    }

    for item in payload.items {
        let result =
            Product::get_by_reference_id_with_specification(&db, item.product_reference_id).await;

        if result.is_err() {
            return Err(Errors::new(&[(
                "product_id",
                "product with ref id not found",
            )]));
        }

        let product = result.unwrap();

        let result_transaction_item = TransactionItem::create(
            &mut db_transaction,
            transaction.as_ref().unwrap().id,
            product.id,
            product.name,
            item.product_reference_id,
            item.product_quantity,
        )
        .await;

        if result_transaction_item.is_err() {
            return Err(Errors::new(&[(
                "transaction_item",
                "failed to create transaction item",
            )]));
        }

        let transaction_item = result_transaction_item.unwrap();

        for specification in product.specifications.unwrap() {
            let product_spec_price = specification.product_specification_price.unwrap();
            let product_spec_quantity = specification.product_specification_quantity.unwrap();
            let spec_unit_price = specification.unit_price.unwrap();
            let transaction_item_spec_quantity =
                product_spec_quantity * transaction_item.product_quantity;

            SpecificationHistory::create(
                &mut db_transaction,
                specification.id.unwrap(),
                Some(transaction_item.id),
                Uuid::parse_str("9f175978-100f-431e-97ad-d4f1ab54ba76").unwrap(),
                None,
                String::from("OUT"),
                transaction_item_spec_quantity,
                transaction_item_spec_quantity as f64 * product_spec_price,
                spec_unit_price,
            )
            .await
            .unwrap();
        }
    }

    let commit = db_transaction.commit().await;

    if commit.is_err() {
        return Err(Errors::new(&[(
            "commit_db_transaction",
            "failed to commit db_transaction",
        )]));
    }

    let transaction = SimplifyTransaction::get_by_id_with_items(&db, transaction.unwrap().id)
        .await
        .unwrap();

    let body = DefaultResponse::new("ok", "create transaction successfully".to_string())
        .with_data(json!(transaction));

    Ok(body.into_response())
}
