use crate::errors::{Errors, FieldValidator};
use crate::models::branch::Branch;
use crate::models::requests::specification::{
     RequestFormSpecificationHistory,
};
use crate::models::responses::DefaultResponse;
use crate::models::specification_history::SpecificationHistory;
use crate::models::user::User;

use axum::extract::Path;
use axum::{extract::State, response::Json};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    State(db): State<PgPool>,
    Path((branch_id, specification_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<RequestFormSpecificationHistory>,
) -> Result<Json<Value>, Errors> {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        return Err(Errors::new(&[("branch_id", "branch not found")]));
    }

    let specification = SpecificationHistory::get_by_id(&db, specification_id).await;

    if specification.is_err() {
        return Err(Errors::new(&[(
            "specification_id",
            "specification not found",
        )]));
    }

    let user = User::get_by_id(&db, payload.created_by).await;

    if user.is_err() {
        return Err(Errors::new(&[("created_by", "user not found")]));
    }

    let mut extractor = FieldValidator::validate(&payload);

    let quantity = extractor.extract("quantity", Some(payload.quantity));
    let flow_type = extractor.extract("flow_type", Some(payload.flow_type));
    let price = extractor.extract("price", Some(payload.price));
    extractor.check()?;

    let specification = SpecificationHistory::create(
        &db,
        specification_id,
        payload.transaction_item_id,
        payload.created_by,
        payload.note,
        flow_type,
        quantity,
        price,
    )
    .await
    .unwrap();

    let body = DefaultResponse::new("ok", "create specification successfully".to_string())
        .with_data(json!(specification));

    Ok(body.into_response())
}
