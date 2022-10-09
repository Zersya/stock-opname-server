use crate::errors::{Errors, FieldValidator};
use crate::models::branch::Branch;
use crate::models::requests::specification::RequestCreateSpecification;
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
    Json(payload): Json<RequestCreateSpecification>,
) -> Result<Json<Value>, Errors> {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        return Err(Errors::new(&[("branch_id", "branch not found")]));
    }

    let mut extractor = FieldValidator::validate(&payload);

    let name = extractor.extract("name", Some(payload.name));
    let unit = extractor.extract("unit", Some(payload.unit));
    let amount = extractor.extract("amount", Some(payload.amount));
    extractor.check()?;

    let specification = Specification::create(&db, branch_id, name, amount, unit)
        .await
        .unwrap();

    let body = DefaultResponse::new("ok", "create specification successfully".to_string())
        .with_data(json!(specification));

    Ok(body.into_response())
}
