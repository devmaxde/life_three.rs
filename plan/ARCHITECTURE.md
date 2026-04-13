# Rust/Leptos Architecture Design
> Life Achievement Tree — Rust Port

---

## Overview

The Rust port uses a **full-stack Rust** approach:
- **Frontend**: Leptos (SSR + CSR hybrid, compiled to WASM)
- **Backend**: Axum (async HTTP server)
- **Shared**: A common `life-tree-core` crate with all data models and algorithms

```
life-tree-rust/
├── Cargo.toml                  # Workspace root
├── crates/
│   ├── core/                   # Shared data models, graph algorithms, types
│   ├── backend/                # Axum HTTP server (API + SSE + OAuth)
│   └── frontend/               # Leptos app (WASM + SSR)
├── assets/                     # Static files (CSS, fonts, images)
└── .env                        # Environment variables
```

---

## Workspace Structure

```toml
# Cargo.toml (workspace)
[workspace]
members = ["crates/core", "crates/backend", "crates/frontend"]
resolver = "2"
```

---

## Crate 1: `life-tree-core`

Compiled for **both native** (backend) and **WASM** (frontend). Zero external service calls.

```
crates/core/
├── src/
│   ├── lib.rs
│   ├── types.rs          # NotionNode, ComputedNode, OuraData, etc.
│   ├── graph.rs          # All graph algorithms from lib/graph.ts
│   ├── schema.rs         # Validation logic from node-schema.ts
│   └── sanitize.rs       # Input sanitization utilities
├── Cargo.toml
```

### Key dependencies
```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
```

---

## Crate 2: `life-tree-backend`

Axum HTTP server. Handles all external service calls (Notion, Claude, Oura, Tavily).

```
crates/backend/
├── src/
│   ├── main.rs             # Server startup, router
│   ├── router.rs           # Route definitions
│   ├── handlers/
│   │   ├── nodes.rs        # GET/POST/PATCH /api/nodes
│   │   ├── chat.rs         # POST /api/chat (SSE)
│   │   ├── oura.rs         # OAuth + data endpoints
│   │   ├── search.rs       # POST /api/web-search
│   │   └── content.rs      # GET /api/nodes/:id/content
│   ├── services/
│   │   ├── notion.rs       # Notion API client
│   │   ├── claude.rs       # Anthropic/OpenRouter streaming client
│   │   ├── oura.rs         # Oura OAuth + API client
│   │   └── tavily.rs       # Tavily search client
│   ├── state.rs            # AppState (HTTP client, env config)
│   └── error.rs            # Error types + Into<Response>
├── Cargo.toml
```

### Key dependencies
```toml
[dependencies]
life-tree-core = { path = "../core" }
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tower-http = { version = "0.5", features = ["cors", "fs"] }
axum-extra = { version = "0.9", features = ["typed-header"] }
tokio-stream = "0.1"
futures-util = "0.3"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
dotenvy = "0.15"
```

### Router
```rust
// router.rs
Router::new()
    // Node CRUD
    .route("/api/nodes", get(handlers::nodes::list).post(handlers::nodes::create))
    .route("/api/nodes/:id", patch(handlers::nodes::update))
    .route("/api/nodes/:id/content", get(handlers::content::get))
    // AI Chat (SSE)
    .route("/api/chat", post(handlers::chat::stream))
    // Oura
    .route("/api/oura/auth", get(handlers::oura::auth))
    .route("/api/oura/callback", get(handlers::oura::callback))
    .route("/api/oura/data", get(handlers::oura::data))
    // Web search
    .route("/api/web-search", post(handlers::search::search))
    // Serve Leptos frontend (catch-all)
    .fallback(leptos_axum::render_app_to_stream(App))
    .with_state(state)
```

### AppState
```rust
#[derive(Clone)]
pub struct AppState {
    pub http: reqwest::Client,
    pub notion_token: String,
    pub notion_db_id: String,
    pub openrouter_key: String,
    pub tavily_key: String,
    pub oura_client_id: String,
    pub oura_client_secret: String,
}
```

---

## Crate 3: `life-tree-frontend`

Leptos app compiled to WASM. Served by the Axum backend via `leptos_axum`.

```
crates/frontend/
├── src/
│   ├── lib.rs              # App root, router, global state
│   ├── state.rs            # Global signals (Leptos context)
│   ├── pages/
│   │   ├── hub.rs          # / — Compass Hub
│   │   ├── map.rs          # /map — WebGL Map
│   │   ├── jarvis.rs       # /jarvis — AI Coach
│   │   └── tree.rs         # /tree/* — Tree Browser
│   ├── components/
│   │   ├── hub/            # HubCanvas, CompassSpoke, CenterNode, BranchCluster
│   │   ├── map/            # MapCanvas (WebGL via web-sys), NebulaRadar
│   │   ├── tree/           # TreeCanvas, NodeCard, Breadcrumb, ConnectionLine
│   │   ├── detail/         # NodeDetailPanel
│   │   ├── chat/           # AIChatPanel, NodeTreePreview, AddNodeModal
│   │   └── ui/             # ProgressRing, GlassCard, Toast, StatsBar
│   └── api/
│       ├── nodes.rs        # fetch wrappers for node CRUD
│       ├── chat.rs         # SSE streaming client
│       └── oura.rs         # Oura data fetch
├── Cargo.toml
├── index.html              # WASM entry point
```

### Key dependencies
```toml
[dependencies]
life-tree-core = { path = "../core" }
leptos = { version = "0.7", features = ["csr", "nightly"] }
leptos_router = "0.7"
leptos_meta = "0.7"
leptos-use = "0.13"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["WebGlRenderingContext", "HtmlCanvasElement", "EventSource", ...] }
gloo-net = "0.6"            # fetch API wrapper
gloo-storage = "0.3"        # localStorage wrapper
chrono = { version = "0.4", features = ["serde", "wasmbind"] }
```

---

## Global State (Leptos Signals)

Replaces Zustand. Provided via Leptos `provide_context` at root level.

```rust
// state.rs
#[derive(Clone)]
pub struct AppState {
    pub nodes: RwSignal<Vec<ComputedNode>>,
    pub raw_nodes: RwSignal<Vec<NotionNode>>,
    pub selected_node_id: RwSignal<Option<String>>,
    pub detail_panel_open: RwSignal<bool>,
    pub loading: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
    pub oura_data: RwSignal<Option<OuraData>>,
    pub show_archived: RwSignal<bool>,
    pub toasts: RwSignal<Vec<Toast>>,
    pub add_node_modal_open: RwSignal<bool>,
    pub add_node_parent_id: RwSignal<Option<String>>,
    pub add_node_pre_deps: RwSignal<Vec<String>>,
    pub add_node_insert_before_id: RwSignal<Option<String>>,
}

impl AppState {
    pub fn rebuild_tree(&self) {
        let raw = self.raw_nodes.get();
        let computed = build_tree(raw);  // from core crate
        self.nodes.set(computed);
    }
}
```

---

## SSE Chat Streaming

The backend returns `text/event-stream`. Frontend reads via `EventSource` or raw `fetch` + `ReadableStream`.

```rust
// backend: handlers/chat.rs
async fn stream(
    State(state): State<AppState>,
    Json(body): Json<ChatRequest>,
) -> impl IntoResponse {
    let stream = async_stream::stream! {
        // Call Claude via OpenRouter with streaming
        // Yield SSE events as bytes
        yield Ok::<_, Infallible>(format!("data: {}\n\n", json).into_bytes());
    };
    
    Response::builder()
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .body(Body::from_stream(stream))
}
```

```rust
// frontend: api/chat.rs
pub async fn stream_chat(req: ChatRequest, on_event: impl Fn(ChatEvent)) {
    // Use gloo-net or web-sys fetch with ReadableStream reader
    // Parse SSE events
    // Call on_event callback per delta
}
```

---

## Map Canvas (web-sys / Canvas 2D)

**IMPORTANT:** Despite the TypeScript component being named "MapCanvasGL", the actual implementation uses **Canvas 2D** (`getContext('2d')`), NOT WebGL. All rendering uses `CanvasRenderingContext2D` — radial gradients, arcs, bezier curves, fillRect, etc. No shaders or GL buffers.

```rust
// components/map/canvas.rs
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

fn MapCanvas() -> impl IntoView {
    let canvas_ref = create_node_ref::<html::Canvas>();
    
    create_effect(move |_| {
        let canvas = canvas_ref.get().unwrap();
        let ctx = canvas.get_context("2d")
            .unwrap().unwrap()
            .dyn_into::<CanvasRenderingContext2d>().unwrap();
        // Initialize render loop with requestAnimationFrame
    });
    
    view! { <canvas node_ref=canvas_ref></canvas> }
}
```

The existing layer architecture (`nodes.ts`, `connections.ts`, `nebula.ts`, etc.) translates directly to Rust modules. Each layer is a pure render function: `fn(ctx, t, zoom, pan, nodes, size)`.

See `SOURCE_REFERENCE.md` §3–4 for complete layer specifications including all zoom thresholds, animation formulas, and caching strategies.

---

## Build System

```toml
# Use cargo-leptos for the full build pipeline
# It handles:
# - WASM compilation via trunk
# - Axum server compilation
# - Asset bundling
# - Hot reload in dev

[package.metadata.leptos]
bin-package = "life-tree-backend"
lib-package = "life-tree-frontend"
style-file = "assets/app.css"  # Tailwind output
```

Build commands:
```bash
# Development (hot reload)
cargo leptos watch

# Production
cargo leptos build --release
```

---

## Tailwind Integration

Tailwind CSS continues to work unchanged. The CSS variable design system from `styles/glass.css` is copied to `assets/` verbatim.

```bash
# tailwind.config.mjs — update content to scan .rs files
content: ["./crates/frontend/src/**/*.rs", "./index.html"]
```

---

## File Storage

Oura tokens currently stored in `/data/oura-tokens.json`. In the Rust port, same approach:

```rust
// services/oura.rs
const TOKEN_FILE: &str = "./data/oura-tokens.json";
const CACHE_FILE: &str = "./data/oura-cache.json";

async fn save_tokens(tokens: &OuraTokens) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(tokens)?;
    tokio::fs::write(TOKEN_FILE, json).await?;
    Ok(())
}
```

---

## Deployment

```dockerfile
# Dockerfile (multi-stage)
FROM rust:1.80 AS builder
RUN cargo install cargo-leptos
COPY . .
RUN cargo leptos build --release

FROM debian:bookworm-slim
COPY --from=builder target/release/life-tree-backend /usr/local/bin/
COPY --from=builder target/site/ /app/site/
COPY --from=builder assets/ /app/assets/
WORKDIR /app
CMD ["life-tree-backend"]
```

---

## What Stays the Same

- **Notion** as database — no migration needed
- **Anthropic/Claude** as AI — same API, different SDK
- **Oura Ring** as health data source — same OAuth + REST
- **Tailwind CSS** design system — copy `glass.css` verbatim
- **CSS variables** — work identically in any framework
- **System prompt** for Jarvis — plain text, copy verbatim
- **Data model** — same shape, now Rust structs
