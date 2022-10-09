use crate::errors::{Errors, FieldValidator};
use crate::models::requests::login::RequestLogin;
use crate::models::responses::DefaultResponse;
use crate::models::user::User;

use argon2::{self, Config};
use argon2::{ThreadMode, Variant, Version};

use axum::{extract::State, response::Json};
use serde_json::{json, Value};
use sqlx::PgPool;

pub async fn login(
    State(db): State<PgPool>,
    Json(payload): Json<RequestLogin>,
) -> Result<Json<Value>, Errors> {
    let mut extractor = FieldValidator::validate(&payload);

    let email = extractor.extract("email", Some(payload.email));
    let password = extractor.extract("password", Some(payload.password));
    extractor.check()?;

    let salt = std::env::var("APPKEY").unwrap();
    let config = Config {
        variant: Variant::Argon2i,
        version: Version::Version13,
        mem_cost: 512,
        time_cost: 2,
        lanes: 1,
        thread_mode: ThreadMode::Parallel,
        secret: &[],
        ad: &[],
        hash_length: 32,
    };

    let email = email.trim().to_string().to_lowercase();
    let password = password.trim().to_string().to_lowercase();
    let hash = argon2::hash_encoded(password.as_bytes(), salt.as_bytes(), &config).unwrap();

    let matches = argon2::verify_encoded(&hash, password.as_bytes()).unwrap();
    assert!(matches);

    let user = match User::login(&db, email, hash).await {
        Ok(user) => user,
        Err(_) => return Err(Errors::new(&[("email or password", "is invalid")])),
    };

    let body = DefaultResponse::new("ok", "login successfully".to_string()).with_data(json!(user));

    Ok(body.into_response())
}
