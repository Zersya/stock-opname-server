use std::str::FromStr;

use crate::errors::{FieldValidator};
use crate::models::branch::Branch;
use crate::models::product::Product;
use crate::models::requests::branch::RequestFormBranch;
use crate::models::responses::DefaultResponse;

use axum::extract::Path;
use axum::Extension;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, response::Json};
use reqwest::StatusCode;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<RequestFormBranch>,
) -> Response {
    let branch = Branch::get_by_reference_id(&db, payload.reference_id).await;

    if branch.is_ok() {
        let body = DefaultResponse::error("Reference already exists", Some("reference_id is duplicate".to_string())).into_json();
        return (StatusCode::BAD_REQUEST, body).into_response();
    }

    let maresto_url = std::env::var("MARESTO_URL").unwrap();

    let response = reqwest::get(&format!(
        "{}/customer/v1/branch?id={}",
        maresto_url, payload.reference_id
    ))
    .await;

    if response.as_ref().unwrap().status() != 200 {
        let body = DefaultResponse::error("Reference not found at Maresto", Some("reference_id not exists at Maresto".to_string())).into_json();
        return (StatusCode::BAD_REQUEST, body).into_response();
    }

    let mut extractor = FieldValidator::validate(&payload);

    let name = extractor.extract("name", Some(payload.name));
    extractor.check();

    let branch = Branch::create(&db, user_id, name, payload.reference_id)
        .await
        .unwrap();

    let body = DefaultResponse::created("Create branch successfully")
        .with_data(json!(branch)).into_json();

    tokio::spawn(async move {
        let json = response.unwrap().json::<Value>().await.unwrap();
        let map_branch = json["data"].as_object().unwrap();
        let branch_product_categories = map_branch["branch_product_categories"].as_array().unwrap();

        for category in branch_product_categories {
            let products = category["products"].as_array().unwrap();

            for product in products {
                Product::create(
                    &db,
                    branch.id,
                    product["name"].to_string().replace("\"", ""),
                    Uuid::parse_str(product["id"].as_str().unwrap()).unwrap(),
                )
                .await
                .unwrap();
            }
        }
    });

    (StatusCode::CREATED, body).into_response()
}

pub async fn get_all(State(db): State<PgPool>) -> Response {
    let branches = Branch::get_all(&db).await.unwrap();

    let body = DefaultResponse::ok("Get branches successfully")
        .with_data(json!(branches)).into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn update(
    State(db): State<PgPool>,
    Path((branch_id,)): Path<(Uuid,)>,
    Json(payload): Json<RequestFormBranch>,
) -> Response {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        let body = DefaultResponse::error("Branch not found", Some("branch_id is not exist".to_string())).into_json();
        return (StatusCode::BAD_REQUEST, body).into_response();
    }

    let mut extractor = FieldValidator::validate(&payload);

    let name = extractor.extract("name", Some(payload.name));
    extractor.check();

    let branch = Branch::update(&db, branch_id, name, payload.reference_id)
        .await
        .unwrap();

    let body = DefaultResponse::ok("Update branch successfully")
        .with_data(json!(branch)).into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn get_by_id(
    State(db): State<PgPool>,
    Path((branch_id,)): Path<(Uuid,)>,
) -> Response {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        let body = DefaultResponse::error("Branch not found", Some("branch_id is not exist".to_string())).into_json();
        return (StatusCode::BAD_REQUEST, body).into_response();
    }

    let body = DefaultResponse::ok("get branch successfully")
        .with_data(json!(branch.unwrap())).into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn get_by_user_id(
    State(db): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
) -> Response {
    let branch = Branch::get_by_user_id(&db, user_id).await;

    let body = DefaultResponse::ok("get branch successfully")
        .with_data(json!(branch.unwrap())).into_json();

    (StatusCode::OK, body).into_response()
}

pub async fn sync(
    State(db): State<PgPool>,
    Path(branch_id): Path<Uuid>,
) -> Response {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        let body = DefaultResponse::error("Branch not found", Some("branch_id is not exist".to_string())).into_json();
        return (StatusCode::BAD_REQUEST, body).into_response();
    }

    let maresto_url = std::env::var("MARESTO_URL").unwrap();

    let response = reqwest::get(&format!(
        "{}/customer/v1/branch?id={}",
        maresto_url,
        branch.as_ref().unwrap().reference_id
    ))
    .await;

    if response.as_ref().unwrap().status() != 200 {
        let body = DefaultResponse::error("reference not found at maresto", Some("reference_id not exists at Maresto".to_string())).into_json();
        return (StatusCode::BAD_REQUEST, body).into_response();
    }

    let json = response.unwrap().json::<Value>().await.unwrap();
    let map_branch = json["data"].as_object().unwrap();

    let branch = Branch::update(
        &db,
        branch_id,
        map_branch["name"].to_string().replace("\"", ""),
        branch.as_ref().unwrap().reference_id,
    )
    .await
    .unwrap();

    let body = DefaultResponse::ok("update branch successfully")
        .with_data(json!(branch)).into_json();

    tokio::spawn(async move {
        let map_branch = json["data"].as_object().unwrap();

        let branch_product_categories = map_branch["branch_product_categories"].as_array().unwrap();

        for category in branch_product_categories {
            let products = category["products"].as_array().unwrap();

            for product in products {
                let result = Product::update_by_reference_id(
                    &db,
                    Uuid::from_str(product["id"].as_str().unwrap()).unwrap(),
                    product["name"].to_string().replace("\"", ""),
                )
                .await;

                if result.is_err() {
                    Product::create(
                        &db,
                        branch.id,
                        product["name"].to_string().replace("\"", ""),
                        Uuid::parse_str(product["id"].as_str().unwrap()).unwrap(),
                    )
                    .await
                    .unwrap();
                }
            }
        }
    });

    (StatusCode::OK, body).into_response()
}
