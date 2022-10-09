use stock_opname_server;

#[tokio::main]
async fn main() {
    stock_opname_server::axum().await;
}
