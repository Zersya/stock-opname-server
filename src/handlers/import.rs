use std::io::Write;

use axum::response::{IntoResponse, Response};
use calamine::{open_workbook, Error, RangeDeserializerBuilder, Reader, Xlsx};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::logger::Logger;
use crate::models::branch::Branch;
use crate::models::product::Product;
use crate::models::product_specification::ProductSpecification;
use crate::models::specification::Specification;
use crate::{errors::Errors, models::responses::DefaultResponse};

use axum::{extract::Multipart, extract::Path, extract::State};
use reqwest::StatusCode;

use sqlx::PgPool;

async fn process_specifications(
    db: &PgPool,
    db_transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    row: (String, String, String, i32, String, i32),
    branch_id: Uuid,
) -> Result<(), Errors> {
    let (_, name, unit, smallest_unit, unit_name, raw_price): (
        String,
        String,
        String,
        i32,
        String,
        i32,
    ) = row;

    let name = name.to_lowercase();
    let unit_name = unit_name.to_lowercase();

    let lowest_price = (Decimal::from(raw_price) / Decimal::from(smallest_unit))
        .round_dp(2)
        .to_f64()
        .expect("failed to convert to f64");

    let specification = Specification::get_by_name_and_branch_id(&db, &name, &branch_id).await;

    if specification.is_ok() {
        let spec = Specification::update_with_db_trx_by_name_and_branch_id(
            db_transaction,
            &branch_id,
            &name,
            &smallest_unit,
            &unit_name,
            &unit,
            &lowest_price,
            &raw_price,
        )
        .await;

        if spec.is_err() {
            return Err(Errors::new(&[(
                "specification",
                "failed to update specification",
            )]));
        }
    } else {
        match Specification::create_with_db_trx(
            db_transaction,
            branch_id,
            name,
            smallest_unit,
            unit_name,
            unit,
            lowest_price,
            raw_price,
        )
        .await
        {
            Ok(_) => (),
            Err(e) => {
                Logger::new(e.to_string()).log();

                return Err(Errors::new(&[(
                    "specification",
                    "failed to create specification",
                )]));
            }
        }
    }

    Ok(())
}

async fn process_product_specifications(
    db: &PgPool,
    db_transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    product_name: &String,
    specification_name: String,
    measure: i32,

    branch_id: Uuid,
) -> Result<(), Errors> {
    let product_name = product_name.to_lowercase();
    let specification_name = specification_name.to_lowercase();

    let specification =
        Specification::get_by_name_and_branch_id(&db, &specification_name, &branch_id).await;

    if specification.is_err() {
        return Ok(());
    }

    let products =
        match Product::get_by_contain_name_and_branch_id(&db, &product_name, &branch_id).await {
            Ok(products) => products,
            Err(e) => {
                Logger::new(e.to_string()).log();

                return Err(Errors::new(&[("product", "failed to get product")]));
            }
        };

    if products.is_empty() {
        return Ok(());
    }

    for product in products {
        let product_specification = ProductSpecification::get_by_product_id_and_specification_id(
            &db,
            product.id,
            specification.as_ref().unwrap().id,
        )
        .await;

        if product_specification.is_ok() {
            let product_specification = ProductSpecification::update_with_db_trx(
                db_transaction,
                product_specification.unwrap().id,
                product.id,
                specification.as_ref().unwrap().id,
                measure,
            )
            .await;

            if product_specification.is_err() {
                return Err(Errors::new(&[(
                    "product_specification",
                    "failed to update product specification",
                )]));
            }
        } else {
            let product_specification = ProductSpecification::create_with_db_trx(
                db_transaction,
                product.id,
                specification.as_ref().unwrap().id,
                measure,
            )
            .await;

            if product_specification.is_err() {
                return Err(Errors::new(&[(
                    "product_specification",
                    "failed to create product specification",
                )]));
            }
        }
    }

    Ok(())
}

pub async fn product_specifications(
    State(db): State<PgPool>,
    Path((branch_id,)): Path<(Uuid,)>,
    mut multipart: Multipart,
) -> Response {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        let body = DefaultResponse::error("Branch not found", Some("branch_id is not exist".to_string())).into_json();
        return (StatusCode::BAD_REQUEST, body).into_response();
    }

    let field = multipart.next_field().await.unwrap().unwrap();

    let mime_type = field.file_name().unwrap().to_string();
    let file_extension = mime_type.split('.').last().unwrap();

    let data = field.bytes().await.unwrap();
    let data_length = data.len();

    if data_length > 2097152 {
        let body = DefaultResponse::error("File size must be less than 2mb", None).into_json();
        return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
    }

    let file_name = Uuid::new_v4().to_string();
    let path = format!("storage/temp/{}.{}", file_name, file_extension);
    let mut file = std::fs::File::create(&path).unwrap();

    file.write_all(&data).unwrap();

    let mut workbook: Xlsx<_> = open_workbook(path).unwrap();
    let range_specifications = match workbook
        .worksheet_range("Specifications")
        .ok_or(Error::Msg("Cannot find 'Specifications'"))
    {
        Ok(range) => match range {
            Ok(range) => range,
            Err(e) => {
                Logger::new(e.to_string()).log();
                let body = DefaultResponse::error("File is not valid", None).into_json();
                return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
            }
        },
        Err(e) => {
            Logger::new(e.to_string()).log();
            let body = DefaultResponse::error("Worksheet cannot be found", None).into_json();
            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let mut iter = RangeDeserializerBuilder::new()
        .from_range(&range_specifications)
        .expect("failed to create iterator for specifications");

    let mut db_transaction = db.begin().await.expect("Failed to begin transaction");

    while let Some(row) = iter.next() {
        match process_specifications(&db, &mut db_transaction, row.unwrap(), branch_id).await {
            Ok(_) => (),
            Err(e) => {
                Logger::new(format!("{:?}", e)).log();
                db_transaction
                    .rollback()
                    .await
                    .expect("Failed to rollback transaction");
                let body =
                    DefaultResponse::error("Failed to import specification", None).into_json();
                return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
            }
        }
    }

    match db_transaction.commit().await {
        Ok(_) => (),
        Err(e) => {
            Logger::new(e.to_string()).log();
            let body = DefaultResponse::error("Failed to commit transaction", None).into_json();
            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let mut db_transaction = db.begin().await.expect("Failed to begin transaction");

    let range_product_specifications = match workbook
        .worksheet_range("Product Specifications")
        .ok_or(Error::Msg("Cannot find 'Product Specifications'"))
    {
        Ok(range) => match range {
            Ok(range) => range,
            Err(e) => {
                Logger::new(e.to_string()).log();
                let body = DefaultResponse::error("File is not valid", None).into_json();
                return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
            }
        },
        Err(e) => {
            Logger::new(e.to_string()).log();
            let body = DefaultResponse::error("Worksheet cannot be found", None).into_json();
            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let mut iter = RangeDeserializerBuilder::new()
        .from_range(&range_product_specifications)
        .expect("failed to create iterator for product specifications");

    let mut active_product_name = String::new();

    while let Some(row) = iter.next() {
        let (product_name, specification_name, measure, _): (String, String, i32, String) =
            row.unwrap();

        if !product_name.is_empty() {
            active_product_name = product_name;
        }

        match process_product_specifications(
            &db,
            &mut db_transaction,
            &active_product_name,
            specification_name,
            measure,
            branch_id,
        )
        .await
        {
            Ok(_) => (),
            Err(e) => {
                Logger::new(format!("{:?}", e)).log();
                db_transaction
                    .rollback()
                    .await
                    .expect("Failed to rollback transaction");
                let body =
                    DefaultResponse::error("Failed to import specification", None).into_json();
                return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
            }
        }
    }

    match db_transaction.commit().await {
        Ok(_) => (),
        Err(e) => {
            Logger::new(e.to_string()).log();
            let body = DefaultResponse::error("Failed to commit transaction", None).into_json();
            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }
    };

    let body = DefaultResponse::ok("Success to import product specifications").into_json();
    (StatusCode::OK, body).into_response()
}
