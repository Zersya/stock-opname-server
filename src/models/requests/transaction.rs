use serde::Deserialize;
use uuid::Uuid;
use validator_derive::Validate;

#[derive(Deserialize, Validate)]
pub struct RequestCreateTransaction {
    pub items: Vec<RequestCreateTransactionItem>,
    pub created_by: Option<Uuid>,
    pub note: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct RequestCreateTransactionItem {
    pub product_reference_id: Uuid,
    pub product_quantity: i32,
}