use std::str::FromStr;

use crate::errors::{Errors, FieldValidator};
use crate::models::branch::Branch;
use crate::models::product::Product;
use crate::models::requests::branch::RequestFormBranch;
use crate::models::responses::DefaultResponse;

use axum::extract::Path;
use axum::{extract::State, response::Json};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    State(db): State<PgPool>,
    Json(payload): Json<RequestFormBranch>,
) -> Result<Json<Value>, Errors> {
    let branch = Branch::get_by_reference_id(&db, payload.reference_id).await;

    if branch.is_ok() {
        return Err(Errors::new(&[("reference_id", "already exists")]));
    }

    let maresto_url = std::env::var("MARESTO_URL").unwrap();

    let response = reqwest::get(&format!(
        "{}/customer/v1/branch?id={}",
        maresto_url, payload.reference_id
    ))
    .await;

    if response.as_ref().unwrap().status() != 200 {
        return Err(Errors::new(&[("reference_id", "not found at maresto")]));
    }

    let mut extractor = FieldValidator::validate(&payload);

    let name = extractor.extract("name", Some(payload.name));
    extractor.check()?;

    let branch = Branch::create(&db, payload.user_id, name, payload.reference_id)
        .await
        .unwrap();

    let body = DefaultResponse::new("ok", "create branch successfully".to_string())
        .with_data(json!(branch));

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

    Ok(body.into_response())
}

pub async fn update(
    State(db): State<PgPool>,
    Path((branch_id,)): Path<(Uuid,)>,
    Json(payload): Json<RequestFormBranch>,
) -> Result<Json<Value>, Errors> {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        return Err(Errors::new(&[("branch_id", "not found")]));
    }

    let mut extractor = FieldValidator::validate(&payload);

    let name = extractor.extract("name", Some(payload.name));
    extractor.check()?;

    let branch = Branch::update(&db, branch_id, name, payload.reference_id)
        .await
        .unwrap();

    let body = DefaultResponse::new("ok", "update branch successfully".to_string())
        .with_data(json!(branch));

    Ok(body.into_response())
}

pub async fn get_by_id(
    State(db): State<PgPool>,
    Path((branch_id,)): Path<(Uuid,)>,
) -> Result<Json<Value>, Errors> {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        return Err(Errors::new(&[("branch_id", "not found")]));
    }

    let body = DefaultResponse::new("ok", "get branch successfully".to_string())
        .with_data(json!(branch.unwrap()));

    Ok(body.into_response())
}

pub async fn sync(
    State(db): State<PgPool>,
    Path(branch_id): Path<Uuid>,
) -> Result<Json<Value>, Errors> {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        return Err(Errors::new(&[("branch_id", "not found")]));
    }

    let maresto_url = std::env::var("MARESTO_URL").unwrap();

    let response = reqwest::get(&format!(
        "{}/customer/v1/branch?id={}",
        maresto_url,
        branch.as_ref().unwrap().reference_id
    ))
    .await;

    if response.as_ref().unwrap().status() != 200 {
        return Err(Errors::new(&[("reference_id", "not found at maresto")]));
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

    let body = DefaultResponse::new("ok", "update branch successfully".to_string())
        .with_data(json!(branch));

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

    Ok(body.into_response())
}
