use serde::Deserialize;
use uuid::Uuid;
use validator_derive::Validate;

#[derive(Deserialize, Validate)]
pub struct RequestCreateBranch {
    #[validate(length(min = 4, max = 24))]
    pub name: String,
    pub reference_id: Uuid,
    pub user_id: Uuid,
}
