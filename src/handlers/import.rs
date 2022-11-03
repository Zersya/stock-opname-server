use std::io::Write;

use calamine::{open_workbook, Error, RangeDeserializerBuilder, Reader, Xlsx};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::logger::Logger;
use crate::models::branch::Branch;
use crate::models::specification::Specification;
use crate::{errors::Errors, models::responses::DefaultResponse};

use axum::{extract::Multipart, extract::Path, extract::State, response::Json};
use serde_json::Value;

use sqlx::PgPool;

pub async fn product_specifications(
    State(db): State<PgPool>,
    Path((branch_id,)): Path<(Uuid,)>,
    mut multipart: Multipart,
) -> Result<Json<Value>, Errors> {
    let branch = Branch::get_by_id(&db, branch_id).await;

    if branch.is_err() {
        return Err(Errors::new(&[("branch_id", "branch not found")]));
    }

    let field = multipart.next_field().await.unwrap().unwrap();

    let mime_type = field.file_name().unwrap().to_string();
    let file_extension = mime_type.split('.').last().unwrap();

    let data = field.bytes().await.unwrap();
    let data_length = data.len();

    if data_length > 2097152 {
        return Err(Errors::new(&[("file", "file size must be less than 2mb")]));
    }

    let file_name = Uuid::new_v4().to_string();
    let path = format!("storage/temp/{}.{}", file_name, file_extension);
    let mut file = std::fs::File::create(&path).unwrap();

    file.write_all(&data).unwrap();

    let mut workbook: Xlsx<_> = open_workbook(path).unwrap();
    let range = match workbook
        .worksheet_range("Specifications")
        .ok_or(Error::Msg("Cannot find 'Specifications'"))
    {
        Ok(range) => range,
        Err(e) => {
            Logger::new(e.to_string()).log();
            return Err(Errors::new(&[("worksheet", "cannot find worksheet")]));
        }
    };

    let range = match range {
        Ok(range) => range,
        Err(e) => {
            Logger::new(e.to_string()).log();
            return Err(Errors::new(&[("file", "file is not valid")]));
        }
    };

    let mut iter = RangeDeserializerBuilder::new().from_range(&range).unwrap();

    let mut db_transaction = db.begin().await.unwrap();

    while let Some(row) = iter.next() {
        let (_, name, unit, smallest_unit, unit_name, raw_price): (
            String,
            String,
            String,
            i32,
            String,
            i32,
        ) = row.unwrap();

        let lowest_price = (Decimal::from(raw_price) / Decimal::from(smallest_unit))
            .round_dp(2)
            .to_f64()
            .expect("failed to convert to f64");

        match Specification::create_with_db_trx(
            &mut db_transaction,
            branch_id,
            name.to_lowercase(),
            smallest_unit,
            unit_name.to_lowercase(),
            unit,
            lowest_price,
            raw_price,
        )
        .await
        {
            Ok(_) => (),
            Err(e) => {
                Logger::new(e.to_string()).log();

                db_transaction.rollback().await.unwrap();

                return Err(Errors::new(&[(
                    "specification",
                    "failed to create specification",
                )]));
            }
        }
    }

    let commit = db_transaction.commit().await;

    if commit.is_err() {
        return Err(Errors::new(&[(
            "commit_db_transaction",
            "failed to commit db_transaction",
        )]));
    }

    // if let Some(result) = iter.next() {
    //     let (no, name, unit, minimum_unit, unit_name, lowest_price, price): (
    //         String,
    //         String,
    //         String,
    //         String,
    //         String,
    //         String,
    //         String,
    //     ) = match result {
    //         Ok((no, name, unit, minimum_unit, unit_name, lowest_price, price)) => {
    //             (no, name, unit, minimum_unit, unit_name, lowest_price, price)
    //         }
    //         Err(e) => {
    //             Logger::new(e.to_string()).log();

    //             return Err(Errors::new(&[("file", "invalid file")]));
    //         }
    //     };

    // } else {
    //     return Err(Errors::new(&[("result", "error")]));
    // }

    let body = DefaultResponse::new("ok", "success to import product specifications".to_string());
    Ok(body.into_response())
}
