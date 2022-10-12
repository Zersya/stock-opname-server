use std::borrow::Cow;

use serde::Deserialize;
use uuid::Uuid;
use validator_derive::Validate;

#[derive(Deserialize, Validate)]
pub struct RequestFormSpecification {
    pub name: String,
    pub unit: String,
    pub quantity: i32,
}

#[derive(Deserialize, Validate)]
pub struct RequestFormSpecificationHistory {
    pub created_by: Uuid,
    pub transaction_item_id: Option<Uuid>,
    #[validate(custom = "validate_flow_specification_history")]
    pub flow_type: String,
    pub note: String,
    #[validate(range(min = 1))]
    pub quantity: i32,
    #[validate(range(min = 10))]
    pub price: f64,
}


fn validate_flow_specification_history(transaction_type: &str) -> Result<(), validator::ValidationError> {
    if transaction_type != "IN" && transaction_type != "OUT" {
        let err = validator::ValidationError::new("flow type invalid ( must IN or OUT )");

        return Err(err);
    }

    Ok(())
}
