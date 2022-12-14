use serde::Deserialize;
use validator_derive::Validate;

#[derive(Deserialize, Validate)]
pub struct RequestLogin {
    #[validate(length(min = 4, max = 24), email)]
    pub email: String,
    #[validate(length(min = 4))]
    pub password: String,
}
