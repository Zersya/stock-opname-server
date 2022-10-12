use crate::errors::Errors;
use crate::models::product::Product;
use crate::models::product_specification::ProductSpecification;
use crate::models::responses::DefaultResponse;
use crate::models::specification::Specification;

use axum::extract::Path;
use axum::{extract::State, response::Json};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

