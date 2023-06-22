use crate::errors::FieldValidator;
use crate::models::product::Product;
use crate::models::product_specification::ProductSpecification;
use crate::models::requests::product::RequestCreateProductSpecification;
use crate::models::responses::DefaultResponse;
use crate::models::specification::Specification;

use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, response::Json};
use reqwest::{StatusCode};
use serde_json::{json};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_all(State(db): State<PgPool>, Path((branch_id,)): Path<(Uuid,)>) -> Response {
    let result = Product::get_all_with_specifications(&db, branch_id).await;

    let products = match result {
        Ok(products) => products,
        Err(e) => {
            println!("{}", e);
            let body = DefaultResponse::error("Something went wrong", None).into_json();
            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("Get all product successfully").with_data(json!(products)).into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn set_product_specification(
    State(db): State<PgPool>,
    Path((_,)): Path<(Uuid,)>,
    Json(payload): Json<RequestCreateProductSpecification>,
) -> Response {
    let mut extractor = FieldValidator::validate(&payload);

    let quantity = extractor.extract("quantity", Some(payload.quantity));
    extractor.check();

    let product = Product::get_by_id(&db, payload.product_id).await;

    if product.is_err() {
        let body = DefaultResponse::error("Product not found", Some("product_id is not exist".to_string())).into_json();
            return (StatusCode::BAD_REQUEST, body).into_response();
    }

    let specification = Specification::get_by_id(&db, payload.specification_id).await;

    if specification.is_err() {
        let body = DefaultResponse::error("Specification not found", Some("specification_id is not exist".to_string())).into_json();
        return (StatusCode::BAD_REQUEST, body).into_response();
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

        let body = DefaultResponse::ok("Update product specification successfully")
            .with_data(json!(result)).into_json();

        return (StatusCode::OK, body).into_response()
    }

    let result =
        ProductSpecification::create(&db, payload.product_id, payload.specification_id, quantity)
            .await
            .unwrap();

    let body = DefaultResponse::created("Create product specification successfully")
        .with_data(json!(result)).into_json();

    (StatusCode::CREATED, body).into_response()
}
