use crate::errors::FieldValidator;
use crate::models::branch::Branch;
use crate::models::requests::specification::RequestFormSpecification;
use crate::models::responses::DefaultResponse;
use crate::models::specification::Specification;

use axum::extract::Path;
use axum::response::{Response, IntoResponse};
use axum::{extract::State, response::Json};
use reqwest::StatusCode;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    State(db): State<PgPool>,
    Path((branch_id,)): Path<(Uuid,)>,
    Json(payload): Json<RequestFormSpecification>,
) -> Response {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        let body = DefaultResponse::error("Branch not found", Some("Branch ID not found".to_string())).into_json();
        return (StatusCode::BAD_REQUEST, body).into_response();
    }

    let mut extractor = FieldValidator::validate(&payload);

    let name = extractor.extract("name", Some(payload.name));
    let unit = extractor.extract("unit", Some(payload.unit));
    let unit_name = extractor.extract("unit_name", Some(payload.unit_name));
    let smallest_unit = extractor.extract("smallest_unit", Some(payload.smallest_unit));
    let raw_price = extractor.extract("raw_price", Some(payload.raw_price));
    extractor.check();

    let lowest_price = match (Decimal::from(raw_price) / Decimal::from(smallest_unit)) 
        .round_dp(2)
        .to_f64() {
            Some(val) => val,
            None => {
                let body = DefaultResponse::error("Failed to generate lowest price", Some("Unabel to convert raw price / smallest unit into lowest price".to_string())).into_json();
                return (StatusCode::BAD_REQUEST, body).into_response();
            }
        };

    let specification = Specification::create(
        &db,
        branch_id,
        name,
        smallest_unit,
        unit_name,
        unit,
        lowest_price,
        raw_price,
    )
    .await
    .unwrap();

    let body = DefaultResponse::created("Create specification successfully")
        .with_data(json!(specification)).into_json();

    (StatusCode::CREATED, body).into_response()
}

pub async fn get_by_branch_id(
    State(db): State<PgPool>,
    Path((branch_id,)): Path<(Uuid,)>,
) -> Response {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        let body = DefaultResponse::error("Branch not found", Some("Branch ID not found".to_string())).into_json();
        return (StatusCode::BAD_REQUEST, body).into_response();
    }

    let specifications = Specification::get_by_branch_id_with_product(&db, branch_id)
        .await
        .unwrap();

    let body = DefaultResponse::ok("Get all specifications successfully")
        .with_data(json!(specifications)).into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn delete(
    State(db): State<PgPool>,
    Path((branch_id, specification_id)): Path<(Uuid, Uuid)>,
) -> Response {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        let body = DefaultResponse::error("Branch not found", Some("Branch ID not found".to_string())).into_json();
        return (StatusCode::BAD_REQUEST, body).into_response();
    }

    let result = Specification::delete(&db, specification_id).await;

    if result.is_err() {
        let body = DefaultResponse::error("Specification not found", Some(result.err().unwrap().to_string())).into_json();
        return (StatusCode::BAD_REQUEST, body).into_response();
    }

    let body = DefaultResponse::ok("delete specification successfully").into_json();

    (StatusCode::OK, body).into_response()
}
