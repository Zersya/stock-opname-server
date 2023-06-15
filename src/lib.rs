use std::net::SocketAddr;

use axum::{
    routing::{delete, get, patch, post, put},
    Router, http::HeaderValue,
};

use dotenvy::dotenv;
use reqwest::Method;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;

mod config;
mod errors;
mod handlers;
mod logger;
mod models;
mod middlewares;

pub async fn axum() {
    dotenv().ok();

    let config = config::Config::from_env().unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(config.pg.as_ref().unwrap().poolmaxsize)
        .connect(config.database_url().as_ref())
        .await
        .expect("Failed to create pool database connection");

        let auth_middleware = axum::middleware::from_fn_with_state(
            pool.clone(),
            middlewares::authentication::check_authentication,
        );
        
        let check_headers = axum::middleware::from_fn(middlewares::headers::check_headers);

    let origins = [
        "http://localhost:3000".parse::<HeaderValue>().unwrap(),
        "https://new-web.maresto.id".parse::<HeaderValue>().unwrap(),
        "https://maresto-menu-web.vercel.app".parse::<HeaderValue>().unwrap(),
    ];

    let app = Router::with_state(pool)
        .route("/branches/:id/specification/:id",delete(handlers::specification::delete))
        .route("/branches/:id/specification/:id/purchase",post(handlers::specification_history::create))
        .route("/branches/:id/specifications",get(handlers::specification::get_by_branch_id).post(handlers::specification::create))
        .route("/branches/:id/products", get(handlers::product::get_all))
        .route("/branches/:id/set-product-specification",put(handlers::product::set_product_specification))
        .route("/branches/:id/import-product-specifications",post(handlers::import::product_specifications))
        .route("/branches/:id/bulk-transaction", post(handlers::transaction::bulk_create))
        .route("/branches/:id/transaction",post(handlers::transaction::create))
        .route("/branches/:id/sync", get(handlers::branch::sync))
        .route("/branches/:id", get(handlers::branch::get_by_id).patch(handlers::branch::update))
        .route("/users", get(handlers::user::user_list))
        .route("/branch", post(handlers::branch::create))
        .route("/branches", get(handlers::branch::get_all))
        .route_layer(auth_middleware)
        .route("/register", post(handlers::register::register))
        .route("/login", post(handlers::login::login))
        .route_layer(check_headers)
        .route("/", get(handlers::user::hello_world))
        .layer(
            CorsLayer::new()
                .allow_origin(origins)
                .allow_headers(tower_http::cors::Any)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ]),
        );

    let host = &config.server.as_ref().unwrap().host;
    let port = &config.server.as_ref().unwrap().port;
    let addr = format!("{}:{}", host, port).parse::<SocketAddr>().unwrap();

    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
