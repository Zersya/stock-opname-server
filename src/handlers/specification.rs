use crate::errors::{Errors, FieldValidator};
use crate::models::branch::Branch;
use crate::models::requests::specification::RequestFormSpecification;
use crate::models::responses::DefaultResponse;
use crate::models::specification::Specification;

use axum::extract::Path;
use axum::{extract::State, response::Json};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    State(db): State<PgPool>,
    Path((branch_id,)): Path<(Uuid,)>,
    Json(payload): Json<RequestFormSpecification>,
) -> Result<Json<Value>, Errors> {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        return Err(Errors::new(&[("branch_id", "branch not found")]));
    }

    let mut extractor = FieldValidator::validate(&payload);

    let name = extractor.extract("name", Some(payload.name));
    let unit = extractor.extract("unit", Some(payload.unit));
    let quantity = extractor.extract("quantity", Some(payload.quantity));
    extractor.check()?;

    let specification = Specification::create(&db, branch_id, name, quantity, unit)
        .await
        .unwrap();

    let body = DefaultResponse::new("ok", "create specification successfully".to_string())
        .with_data(json!(specification));

    Ok(body.into_response())
}


pub async fn get_by_branch_id(
    State(db): State<PgPool>,
    Path((branch_id,)): Path<(Uuid,)>,
) -> Result<Json<Value>, Errors> {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        return Err(Errors::new(&[("branch_id", "branch not found")]));
    }

    let specifications = Specification::get_by_branch_id_with_product(&db, branch_id).await.unwrap();

    let body = DefaultResponse::new("ok", "get all specifications successfully".to_string())
        .with_data(json!(specifications));

    Ok(body.into_response())
}

pub async fn delete(
    State(db): State<PgPool>,
    Path((branch_id, specification_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Value>, Errors> {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        return Err(Errors::new(&[("branch_id", "branch not found")]));
    }

    let specification = Specification::get_by_id(&db, specification_id).await;

    if specification.is_err() {
        return Err(Errors::new(&[("specification_id", "specification not found")]));
    }

    let result = Specification::delete(&db, specification_id).await;

    if result.is_err() {
        return Err(Errors::new(&[("specification_id", "specification not found")]));
    }

    let body = DefaultResponse::new("ok", "delete specification successfully".to_string());

    Ok(body.into_response())
}