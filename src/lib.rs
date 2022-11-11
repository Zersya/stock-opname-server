use std::net::SocketAddr;

use axum::{routing::{get, post, put, patch, delete}, Router};

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;

mod config;
mod databases;
mod handlers;
mod models;
mod errors;
mod logger;

pub async fn axum() {
    dotenv().ok();

    let config = config::Config::from_env().unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(config.pg.as_ref().unwrap().poolmaxsize)
        .connect(config.database_url().as_ref())
        .await
        .expect("Failed to create pool database connection");

    sqlx::migrate!().run(&pool).await.expect("Failed to migrate the database");

    let app = Router::with_state(pool)
        .route("/", get(handlers::user::hello_world))
        .route("/register", post(handlers::register::register))
        .route("/login", post(handlers::login::login))
        .route("/users", get(handlers::user::user_list))
        .route("/branch", post(handlers::branch::create))
        .route("/branches/:id", get(handlers::branch::get_by_id))
        .route("/branches/:id", patch(handlers::branch::update))
        .route("/branches/:id/import-product-specifications", post(handlers::import::product_specifications))
        .route("/branches/:id/bulk-transaction", post(handlers::transaction::bulk_create))
        .route("/branches/:id/transaction", post(handlers::transaction::create))
        .route("/branches/:id/sync", get(handlers::branch::sync))
        .route("/branches/:id/specification", post(handlers::specification::create))
        .route("/branches/:id/specification/:id", delete(handlers::specification::delete))
        .route("/branches/:id/specification/:id/purchase", post(handlers::specification_history::create))
        .route("/branches/:id/specifications", get(handlers::specification::get_by_branch_id))
        .route("/branches/:id/products", get(handlers::product::get_all))
        .route("/branches/:id/set-product-specification", put(handlers::product::set_product_specification));
        
    let host = &config.server.as_ref().unwrap().host;
    let port = &config.server.as_ref().unwrap().port;
    let addr = format!("{}:{}", host, port).parse::<SocketAddr>().unwrap();

    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
