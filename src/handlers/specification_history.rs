use crate::errors::FieldValidator;
use crate::models::branch::Branch;
use crate::models::requests::specification::RequestFormSpecificationHistory;
use crate::models::responses::DefaultResponse;
use crate::models::specification::Specification;
use crate::models::specification_history::SpecificationHistory;
use crate::models::user::User;

use axum::extract::Path;
use axum::response::{Response, IntoResponse};
use axum::{extract::State, response::Json};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    State(db): State<PgPool>,
    Path((branch_id, specification_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<RequestFormSpecificationHistory>,
) -> Response {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        let body = DefaultResponse::error("Branch not found", Some("Branch ID not found".to_string())).into_json();
        return (StatusCode::BAD_REQUEST, body).into_response();
    }

    let specification = Specification::get_by_id(&db, specification_id).await;

    if specification.is_err() {
        let body = DefaultResponse::error("Specification not found", Some("Specification ID not found".to_string())).into_json();
        return (StatusCode::BAD_REQUEST, body).into_response();
    }

    let user = User::get_by_id(&db, payload.created_by).await;

    if user.is_err() {
        let body = DefaultResponse::error("User not found", Some("Created by not found".to_string())).into_json();
        return (StatusCode::BAD_REQUEST, body).into_response();
    }

    let mut extractor = FieldValidator::validate(&payload);

    let quantity = extractor.extract("quantity", Some(payload.quantity));
    let flow_type = extractor.extract("flow_type", Some(payload.flow_type));
    let price = extractor.extract("price", Some(payload.price));
    extractor.check();

    let mut db_transaction = db.begin().await.unwrap();

    let specification = SpecificationHistory::create(
        &mut db_transaction,
        specification_id,
        payload.transaction_item_id,
        payload.created_by,
        Some(payload.note),
        flow_type,
        quantity,
        price,
        price / quantity as f64,
    )
    .await
    .unwrap();

    let commit = db_transaction.commit().await;

    if commit.is_err() {
        let body = DefaultResponse::error("Something went wrong", Some("Failed to commit db_transaction".to_string())).into_json();
        return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
    }

    let body = DefaultResponse::created("create specification successfully")
        .with_data(json!(specification)).into_json();

    (StatusCode::CREATED, body).into_response()
}
