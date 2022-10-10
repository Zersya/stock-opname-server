use serde::Deserialize;
use uuid::Uuid;
use validator_derive::Validate;

#[derive(Deserialize, Validate)]
pub struct RequestCreateProductSpecification {
    pub product_id: Uuid,
    pub specification_id: Uuid,
}