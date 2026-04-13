mod handlers;
mod services;

use axum::{response::Html, routing::{get, post}, Router};
use std::net::SocketAddr;
use handlers::schema::validate_database_schema;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/api/schema/validate/:database_id", post(validate_database_schema));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Life Tree Backend listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

async fn hello_world() -> Html<&'static str> {
    Html("<h1>Life Tree Backend - Ready for Notion Integration</h1>")
}
