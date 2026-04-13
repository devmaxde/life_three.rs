mod handlers;
mod services;
mod state;

use axum::{
    extract::DefaultBodyLimit,
    response::Html,
    routing::{delete, get, patch, post},
    Router,
};
use std::net::SocketAddr;
use state::AppState;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize app state
    let app_state = AppState::from_env().expect("Failed to initialize app state");

    // Build router with all routes
    let app = Router::new()
        // Health & info
        .route("/", get(hello_world))
        .route("/health", get(handlers::nodes::health))

        // Schema validation
        .route(
            "/api/schema/validate/:database_id",
            post(handlers::schema::validate_database_schema),
        )

        // Node API
        .route("/api/nodes", get(handlers::nodes::get_nodes))
        .route("/api/nodes", post(handlers::nodes::create_node))
        .route("/api/nodes/:id", patch(handlers::nodes::update_node))
        .route("/api/nodes/:id", delete(handlers::nodes::delete_node))

        // Limit request body size to 10MB
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))

        // Share app state
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Life Tree Backend listening on {}", addr);
    println!("  GET /health - Health check");
    println!("  GET /api/nodes - Fetch all nodes");
    println!("  POST /api/nodes - Create a node");
    println!("  PATCH /api/nodes/:id - Update a node");
    println!("  DELETE /api/nodes/:id - Archive a node");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

async fn hello_world() -> Html<&'static str> {
    Html(
        r#"
        <html>
            <head><title>Life Tree Backend</title></head>
            <body>
                <h1>🌳 Life Tree Backend</h1>
                <p>Notion-powered life achievement tree</p>
                <h2>API Endpoints:</h2>
                <ul>
                    <li>GET /health - Health check</li>
                    <li>GET /api/nodes - Fetch all nodes</li>
                    <li>POST /api/nodes - Create a node</li>
                    <li>PATCH /api/nodes/:id - Update a node</li>
                    <li>DELETE /api/nodes/:id - Archive a node</li>
                </ul>
            </body>
        </html>
        "#,
    )
}
