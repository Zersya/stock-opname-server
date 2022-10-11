use serde::Deserialize;
use uuid::Uuid;
use validator_derive::Validate;

#[derive(Deserialize, Validate)]
pub struct RequestFormSpecification {
    pub name: String,
    pub unit: String,
    pub amount: i32,
}

#[derive(Deserialize, Validate)]
pub struct RequestFormSpecificationHistory {
    pub created_by: Uuid,
    pub note: String,
    #[validate(range(min = 1))]
    pub amount: i32,
    #[validate(range(min = 10))]
    pub price: i32,
}