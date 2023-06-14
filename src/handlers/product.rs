use crate::errors::{Errors, FieldValidator};
use crate::models::product::Product;
use crate::models::product_specification::ProductSpecification;
use crate::models::requests::product::RequestCreateProductSpecification;
use crate::models::responses::DefaultResponse;
use crate::models::specification::Specification;

use axum::extract::Path;
use axum::{extract::State, response::Json};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_all(
    State(db): State<PgPool>,
    Path((branch_id,)): Path<(Uuid,)>,
) -> Result<Json<Value>, Errors> {
    let result = Product::get_all_with_specifications(&db, branch_id).await;

    let products = match result {
        Ok(products) => products,
        Err(e) => {
            println!("{}", e);
            return Err(Errors::new(&[("product", "something wrong")]));
        }
    };

    let body = DefaultResponse::new("ok", "get all product successfully".to_string())
        .with_data(json!(products));

    Ok(body.into_json())
}

pub async fn set_product_specification(
    State(db): State<PgPool>,
    Path((_,)): Path<(Uuid,)>,
    Json(payload): Json<RequestCreateProductSpecification>,
) -> Result<Json<Value>, Errors> {
    let mut extractor = FieldValidator::validate(&payload);

    let quantity = extractor.extract("quantity", Some(payload.quantity));
    extractor.check()?;

    let product = Product::get_by_id(&db, payload.product_id).await;

    if product.is_err() {
        return Err(Errors::new(&[("product_id", "product not found")]));
    }

    let specification = Specification::get_by_id(&db, payload.specification_id).await;

    if specification.is_err() {
        return Err(Errors::new(&[(
            "specification_id",
            "specification not found",
        )]));
    }

    let product_specification = ProductSpecification::get_by_product_and_specification(
        &db,
        payload.product_id,
        payload.specification_id,
    )
    .await;

    if product_specification.is_ok() {
        let result = ProductSpecification::update(
            &db,
            payload.product_id,
            payload.specification_id,
            quantity,
        )
        .await
        .unwrap();

        let body = DefaultResponse::new(
            "ok",
            "update product specification successfully".to_string(),
        )
        .with_data(json!(result));

        return Ok(body.into_json());
    }

    let result =
        ProductSpecification::create(&db, payload.product_id, payload.specification_id, quantity)
            .await
            .unwrap();

    let body = DefaultResponse::new(
        "ok",
        "create product specification successfully".to_string(),
    )
    .with_data(json!(result));

    Ok(body.into_json())
}
