use crate::errors::Errors;
use crate::models::product::Product;
use crate::models::responses::DefaultResponse;

use axum::extract::Path;
use axum::{extract::State, response::Json};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_all(
    State(db): State<PgPool>,
    Path((_,)): Path<(Uuid,)>,
) -> Result<Json<Value>, Errors> {
    let result = Product::get_all_with_specifications(&db).await;

    let products = match result {
        Ok(products) => products,
        Err(e) => {
            println!("{}", e);
            return Err(Errors::new(&[("product", "something wrong")]));
        }
    };

    let body = DefaultResponse::new("ok", "get all product successfully".to_string())
        .with_data(json!(products));

    Ok(body.into_response())
}
