use crate::errors::Errors;
use crate::logger::Logger;
use crate::models::branch::Branch;
use crate::models::product::Product;
use crate::models::requests::transaction::{
    RequestCreateTransaction, RequestCreateTransactionItem,
};
use crate::models::responses::DefaultResponse;
use crate::models::specification_history::SpecificationHistory;
use crate::models::transaction::{Transaction, TransactionItem};
use crate::models::user::User;

use axum::response::{IntoResponse, Response};
use axum::{extract::Path, extract::State, response::Json};
use reqwest::StatusCode;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    State(db): State<PgPool>,
    Path((branch_id,)): Path<(Uuid,)>,
    Json(payload): Json<RequestCreateTransaction>,
) -> Response {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        let body =
            DefaultResponse::error("Branch not found", Some("Branch ID not found".to_string())).into_json();
        return (StatusCode::BAD_REQUEST, body).into_response();
    }

    if payload.created_by.is_some() {
        let user = User::get_by_id(&db, payload.created_by.unwrap()).await;

        if user.is_err() {
            let body =
                DefaultResponse::error("User not found", Some("Created by not found".to_string())).into_json();
            return (StatusCode::BAD_REQUEST, body).into_response();
        }
    }

    let mut db_transaction = db.begin().await.unwrap();

    let transaction_id = match process_create(
        &db,
        &mut db_transaction,
        &branch_id,
        payload.created_by,
        &payload.note,
        &payload.items,
    )
    .await
    {
        Ok(transaction) => transaction,
        Err(err) => {
            let body = DefaultResponse::error("Something went wrong", Some("Something went wrong".to_string()))
                .into_json();
            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let commit = db_transaction.commit().await;

    if commit.is_err() {
        let body = DefaultResponse::error(
            "Something went wrong",
            Some("Failed to commit db_transaction".to_string()),
        )
        .into_json();
        return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
    }

    let body = DefaultResponse::created("create transaction successfully")
        .with_data(json!(transaction_id))
        .into_json();

    (StatusCode::CREATED, body).into_response()
}

pub async fn bulk_create(
    State(db): State<PgPool>,
    Path((branch_id,)): Path<(Uuid,)>,
    Json(payload): Json<Vec<RequestCreateTransaction>>,
) -> Response {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        let body =
            DefaultResponse::error("Branch not found", Some("Branch ID not found".to_string())).into_json();
        return (StatusCode::BAD_REQUEST, body).into_response();
    }

    let mut db_transaction = db.begin().await.unwrap();

    let mut transaction_ids = Vec::<String>::new();

    for transaction in payload.iter() {
        if transaction.created_by.is_some() {
            let user = User::get_by_id(&db, transaction.created_by.unwrap()).await;

            if user.is_err() {
                let body = DefaultResponse::error("User not found", Some("Created by not found".to_string()))
                    .into_json();
                return (StatusCode::BAD_REQUEST, body).into_response();
            }
        }

        let transaction_id = match process_create(
            &db,
            &mut db_transaction,
            &branch_id,
            transaction.created_by,
            &transaction.note,
            &transaction.items,
        )
        .await
        {
            Ok(transaction) => transaction,
            Err(err) => {
                Logger::new(format!("{:?}", err)).log();

                db_transaction
                    .rollback()
                    .await
                    .expect("Failed to rollback transaction");

                let body =
                    DefaultResponse::error("Something went wrong", Some("Something went wrong".to_string()))
                        .into_json();
                return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
            }
        };

        transaction_ids.push(transaction_id.to_string());
    }

    match db_transaction.commit().await {
        Ok(_) => {
            let body = DefaultResponse::created("create transaction successfully")
                .with_data(json!(transaction_ids))
                .into_json();

            (StatusCode::CREATED, body).into_response()
        }
        Err(_) => {
            let body = DefaultResponse::error("Something went wrong", Some("Something went wrong".to_string()))
                .into_json();
            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    }
}

pub async fn process_create(
    db: &PgPool,
    db_transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    branch_id: &Uuid,
    created_by: Option<Uuid>,
    note: &Option<String>,
    items: &Vec<RequestCreateTransactionItem>,
) -> Result<String, Errors> {
    let transaction =
        match Transaction::create(db_transaction, branch_id, created_by, note.to_owned()).await {
            Ok(transaction) => transaction,
            Err(err) => {
                Logger::new(format!("{:?}", err)).log();

                return Err(Errors::new(&[(
                    "transaction",
                    "failed to create transaction",
                )]));
            }
        };

    for item in items {
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
            db_transaction,
            transaction.id,
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
            let product_spec_price = specification
                .product_specification_price
                .expect("product_spec_price not found");
            let product_spec_quantity = specification
                .product_specification_quantity
                .expect("product_spec_quantity not found");
            let spec_unit_price = specification.unit_price.expect("unit_price not found");
            let transaction_item_spec_quantity =
                product_spec_quantity * transaction_item.product_quantity;

            let decimal_price = Decimal::from(transaction_item_spec_quantity);
            let decimal_product_spec_price = Decimal::from_f64(product_spec_price)
                .expect("failed to convert product_spec_price to decimal");

            let decimal_result = decimal_price * decimal_product_spec_price;
            let price = decimal_result
                .round_dp(2)
                .to_f64()
                .expect("failed to convert decimal to f64");

            SpecificationHistory::create(
                db_transaction,
                specification.id.unwrap(),
                Some(transaction_item.id),
                Uuid::parse_str("9f175978-100f-431e-97ad-d4f1ab54ba76").unwrap(),
                None,
                String::from("OUT"),
                transaction_item_spec_quantity,
                price,
                spec_unit_price,
            )
            .await
            .unwrap();
        }
    }

    Ok(transaction.id.to_string())
}
