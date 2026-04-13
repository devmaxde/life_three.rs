# Porting Plan — Life Achievement Tree → Rust/Leptos
> Ordered implementation roadmap with full task breakdown

---

## Summary

Porting this app to Rust/Leptos is a significant but well-bounded project. The app has clear separation between backend logic (API routes + external service calls), data/graph algorithms, and frontend UI. The Rust port follows the same architecture but compiled to WASM (frontend) + native binary (backend).

**Stack**:
- Frontend: **Leptos 0.7** (WASM, CSR-first with optional SSR)
- Backend: **Axum 0.7** (async HTTP, SSE, OAuth)
- Shared: **life-tree-core** crate (data models + graph algorithms)
- Build tool: **cargo-leptos**
- CSS: **Tailwind CSS 4** (unchanged, scanned from `.rs` files)
- Database: **Notion API** (unchanged)
- AI: **Anthropic via OpenRouter** (unchanged)

---

## Prerequisites & Toolchain

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install cargo-leptos
cargo install cargo-leptos

# Install trunk (alternative WASM bundler, used by cargo-leptos)
cargo install trunk

# Install wasm-bindgen-cli
cargo install wasm-bindgen-cli

# Tailwind CLI (for CSS bundling)
npm install -g tailwindcss
```

---

## Phase 0: Project Scaffold

**Goal**: Empty but compiling workspace with correct crate structure.

### Tasks

- [ ] Create workspace `Cargo.toml` with three crates
- [ ] Create `crates/core/` with empty `lib.rs`
- [ ] Create `crates/backend/` with empty `main.rs` (axum hello-world)
- [ ] Create `crates/frontend/` with empty Leptos app
- [ ] Configure `cargo-leptos` in root `Cargo.toml`
- [ ] Copy `styles/glass.css` → `assets/glass.css`
- [ ] Create `assets/app.css` (Tailwind entry, imports glass.css)
- [ ] Create `tailwind.config.mjs` scanning `crates/frontend/src/**/*.rs`
- [ ] Copy `.env.local` → `.env` (same vars)
- [ ] Verify `cargo leptos watch` starts without errors

### Deliverable
`http://localhost:3000` returns Leptos placeholder page with glass CSS loaded.

---

## Phase 1: Core Crate — Data Models

**Goal**: All types defined, serializable, tested.

### Tasks

- [ ] `types.rs`: Define all enums (`ArchiveReason`, `NodeBadge`, `NodeColor`, `NodeType`, `NodeStatus`, `BedtimeRegularity`, `ToastType`)
- [ ] `types.rs`: Define `Resource`, `NotionNode`, `ComputedNode`, `MapNode`, `OuraData`, `Toast`, `Position`
- [ ] `types.rs`: Implement `ComputedNode::from_notion(n: NotionNode) -> ComputedNode`
- [ ] `api.rs`: Define all request/response types (`CreateNodeRequest`, `UpdateNodeRequest`, `ChatRequest`, `ChatMessage`, `ChatEvent`, `SuggestedNode`, `NodeEdit`)
- [ ] `oura.rs`: Define `OuraTokens` with `is_expired()`
- [ ] `sanitize.rs`: Port `sanitizeText()`, `sanitizeNodeName()`, `sanitizeDraft()` from TypeScript

**Verification**: `cargo test -p life-tree-core` — all types compile, serde round-trips.

---

## Phase 2: Core Crate — Graph Algorithms

**Goal**: `graph.rs` passes all algorithm tests.

### Tasks

- [ ] `graph.rs`: Implement `topo_sort(nodes: &[NotionNode]) -> Vec<NotionNode>`
- [ ] `graph.rs`: Implement `detect_cycles(nodes: &[NotionNode]) -> HashSet<String>`
- [ ] `graph.rs`: Implement `is_ancestor_archived()` helper
- [ ] `graph.rs`: Implement `is_node_completed()` helper
- [ ] `graph.rs`: Implement `compute_statuses(nodes: &mut HashMap<String, ComputedNode>)`
- [ ] `graph.rs`: Implement `compute_progress(id: &str, nodes: &HashMap<String, ComputedNode>) -> f64`
- [ ] `graph.rs`: Implement `build_tree(nodes: Vec<NotionNode>) -> Vec<ComputedNode>`
  - Two-pass strategy: flat computation then recursive tree assembly
- [ ] `graph.rs`: Implement `compute_layout(children: &[ComputedNode], overrides: Option<&LayoutOverrides>) -> HashMap<String, Position>`
  - Port zone-band allocation, centering pass (3 iterations), overlap resolution, Y normalize
- [ ] `graph.rs`: Implement `compute_radial_map_layout(all_nodes: &[ComputedNode]) -> Vec<MapNode>`
- [ ] `graph.rs`: Implement `get_compass_spokes(all_nodes: &[ComputedNode]) -> (Vec<CompassSpoke>, Vec<CompassSpoke>)`
- [ ] `graph.rs`: Implement `get_breadcrumb_path<'a>(node: &'a ComputedNode, all_nodes: &'a [ComputedNode]) -> Vec<&'a ComputedNode>`
- [ ] `graph.rs`: Implement `get_next_steps(all_nodes: &[ComputedNode]) -> Vec<&ComputedNode>`
- [ ] Write unit tests for all algorithms (port from TypeScript Vitest tests)

**Verification**: `cargo test -p life-tree-core` — all algorithm tests pass.

---

## Phase 3: Backend — Notion Client

**Goal**: Can fetch, create, update, archive nodes via Notion API.

### Tasks

- [ ] `state.rs`: Define `AppState` with `reqwest::Client` and all env vars
- [ ] `state.rs`: Load env vars from `.env` using `dotenvy`
- [ ] `services/notion.rs`: Implement property extraction helpers (text, relation, select, checkbox, date)
- [ ] `services/notion.rs`: Implement `fetch_all_nodes(state: &AppState) -> Vec<NotionNode>`
  - Handle Notion pagination (cursor-based, 100 per page)
  - Map Notion response JSON → `NotionNode`
- [ ] `services/notion.rs`: Implement `create_node(state: &AppState, req: CreateNodeRequest) -> NotionNode`
  - Build Notion page create request body
  - Set page icon (emoji) via `icon.emoji` field
  - Set `parent.database_id`
- [ ] `services/notion.rs`: Implement `update_node(state: &AppState, id: &str, req: UpdateNodeRequest) -> NotionNode`
  - Build partial PATCH body from `Option` fields
- [ ] `services/notion.rs`: Implement `archive_node(state: &AppState, id: &str, reason: ArchiveReason)`
  - Set `archived` select property
- [ ] `services/notion.rs`: Implement `fetch_node_content(state: &AppState, id: &str) -> String`
  - Fetch Notion blocks, convert to Markdown
  - Port `blockToMarkdown()` and `richTextToMarkdown()` helpers
- [ ] `handlers/nodes.rs`: GET `/api/nodes` → call `fetch_all_nodes`, return JSON
- [ ] `handlers/nodes.rs`: POST `/api/nodes` → call `create_node`, return JSON
- [ ] `handlers/nodes.rs`: PATCH `/api/nodes/:id` → call `update_node` or `archive_node`
- [ ] `handlers/content.rs`: GET `/api/nodes/:id/content` → call `fetch_node_content`

**Verification**: Hit all node endpoints with curl; compare responses to Next.js version.

---

## Phase 4: Backend — Claude Chat Endpoint (SSE)

**Goal**: `/api/chat` streams Claude responses identically to Next.js.

### Tasks

- [ ] `services/claude.rs`: Build OpenRouter HTTP client
- [ ] `services/claude.rs`: Define Claude request/response types (streaming)
- [ ] `services/claude.rs`: Define tool schemas for `suggest_nodes` and `edit_nodes` (port JSON schema from TypeScript)
- [ ] `services/claude.rs`: Port system prompt verbatim (~105 lines) from `/api/chat/route.ts` (see `SOURCE_REFERENCE.md` §18)
- [ ] `services/claude.rs`: Implement `stream_chat(state: &AppState, req: ChatRequest) -> impl Stream<Item = ChatEvent>`
  - Call OpenRouter with streaming
  - Parse SSE text deltas from response
  - Parse `tool_use` events, extract tool calls
  - Handle `_ref` resolution for inter-node dependencies (port from TypeScript)
  - Emit `ChatEvent::Text`, `ChatEvent::ToolText`, `ChatEvent::Done`
- [ ] `handlers/chat.rs`: POST `/api/chat` → SSE response
  - Accept `ChatRequest` JSON body
  - Sanitize all inputs (port sanitize functions from core crate)
  - Call `stream_chat`, pipe events to SSE response
  - Headers: `Content-Type: text/event-stream`, `Cache-Control: no-cache`
- [ ] `handlers/suggest_fields.rs`: POST `/api/suggest-fields` → AI field suggestions
  - Uses Claude Haiku via OpenRouter (`anthropic/claude-haiku-4.5`)
  - Input: `{ name, parentName?, siblingNames? }`
  - Output: `{ icon?, color?, description?, why?, timeRange?, badge? }`
  - Returns `{}` on any error (see `SOURCE_REFERENCE.md` §1)

**Verification**: Test with curl; verify SSE stream produces text deltas, then done event with suggestedNodes.

---

## Phase 5: Backend — Oura & Search

**Goal**: Oura OAuth and Tavily search working.

### Tasks

- [ ] `services/oura.rs`: Implement token file I/O (`save_tokens`, `load_tokens`)
- [ ] `services/oura.rs`: Implement `get_valid_token(state: &AppState)` with refresh token rotation
- [ ] `services/oura.rs`: Implement `fetch_oura_data(state: &AppState) -> Option<OuraData>`
  - Fetch readiness endpoint
  - Fetch daily_sleep + sleep (sessions) endpoints
  - Parse scores, duration, bedtime regularity
  - Cache to `/data/oura-cache.json` with 1h TTL
- [ ] `handlers/oura.rs`: GET `/api/oura/auth` → redirect to Oura OAuth page
- [ ] `handlers/oura.rs`: GET `/api/oura/callback?code=...` → exchange code, save tokens, redirect
- [ ] `handlers/oura.rs`: GET `/api/oura/data` → return `OuraData` or `{ connected: false }`
- [ ] `services/tavily.rs`: Implement `web_search(state: &AppState, query: &str) -> SearchResult`
  - POST to Tavily API; return top 5 results
  - Sanitize all result strings
- [ ] `handlers/search.rs`: POST `/api/web-search` → call `web_search`, return JSON

**Verification**: OAuth flow works end-to-end; Oura data appears correctly.

---

## Phase 6: Frontend — Global State & Data Fetching

**Goal**: Leptos app loads, fetches nodes, stores in signals.

### Tasks

- [ ] `lib.rs`: Set up Leptos app root with `provide_context(AppState::new())`
- [ ] `state.rs`: Define `AppState` with all `RwSignal<...>` fields
- [ ] `state.rs`: Implement `AppState::rebuild_tree()` — calls `build_tree()` from core, sets `nodes` signal
- [ ] `state.rs`: Implement `AppState::create_node(req)` — optimistic: temp ID → fetch → real ID swap
- [ ] `state.rs`: Implement `AppState::update_node(id, req)` — optimistic update + async fetch
- [ ] `api/nodes.rs`: `fetch_all_nodes() -> Vec<NotionNode>` using `gloo-net`
- [ ] `api/nodes.rs`: `create_node(req) -> NotionNode`
- [ ] `api/nodes.rs`: `update_node(id, req) -> NotionNode`
- [ ] `api/oura.rs`: `fetch_oura_data() -> Option<OuraData>`
- [ ] `lib.rs`: On app mount: fetch nodes → set raw_nodes → rebuild_tree → fetch oura_data
- [ ] `lib.rs`: Leptos router setup:
  - `/` → `HubPage`
  - `/map` → `MapPage`
  - `/jarvis` → `JarvisPage`
  - `/tree/*path` → `TreePage`

**Verification**: App loads, nodes appear in browser dev console via signal inspection.

---

## Phase 7: Frontend — Tree Browser Page

**Goal**: `/tree/*` route shows navigable hierarchical tree.

### Tasks

- [ ] `components/ui/glass_card.rs`: `GlassCard` component (CSS class wrapper)
- [ ] `components/ui/progress_ring.rs`: `ProgressRing` component (SVG circle)
- [ ] `components/ui/toast.rs`: `ToastContainer` component
- [ ] `components/tree/node_card.rs`: `NodeCard` (icon, name, status ring, color band)
- [ ] `components/tree/connection_line.rs`: `ConnectionLine` (SVG connector)
- [ ] `components/tree/breadcrumb.rs`: `Breadcrumb` (path navigation)
- [ ] `components/tree/add_node_button.rs`: `AddNodeButton`
- [ ] `components/tree/tree_canvas.rs`: `TreeCanvas` — layout-based tree view
  - Consume computed positions from `compute_layout()`
  - Render `NodeCard` at each position
  - Render `ConnectionLine` between nodes
  - Handle click → select node → open detail panel
- [ ] `pages/tree.rs`: `TreePage` — extract path from URL, find root, render `TreeCanvas`

**Verification**: Navigate to `/tree/some-node-name`, tree renders correctly, click navigates.

---

## Phase 8: Frontend — Detail Panel

**Goal**: Clicking a node opens the side panel with all fields editable.

### Tasks

- [ ] `components/detail/node_detail_panel.rs`: `NodeDetailPanel` (500+ line component)
  - Read `selected_node_id` signal
  - Display all node fields
  - Inline editing for: name, icon, description, why, criteria, color, badge, due, timeRange, resources
  - Toggle done checkbox
  - Archive controls with reason select
  - Dependency list with resolved names
  - Content section (fetch `/api/nodes/:id/content` → render markdown)
  - Collapsible sections
  - Bottom action bar (archive, delete)
- [ ] Markdown rendering: Use `pulldown-cmark` + custom `view!` rendering or `leptos-markdown` crate
- [ ] Auto-save on field blur (debounced `update_node` call)

**Verification**: Click any node → detail panel opens → edit name → blur → see Notion updated.

---

## Phase 9: Frontend — Hub Compass Page

**Goal**: `/` shows the radial spoke compass view.

### Tasks

- [ ] `components/hub/center_node.rs`: `CenterNode` (central circle)
- [ ] `components/hub/compass_spoke.rs`: `CompassSpoke` (spoke with path info)
- [ ] `components/hub/branch_cluster.rs`: `BranchCluster` (unpinned branches)
- [ ] `components/hub/hub_canvas.rs`: `HubCanvas`
  - Call `get_compass_spokes()` from core
  - SVG/HTML layout of spokes in radial arrangement
  - Click spoke → navigate to tree view
  - Collapse/expand state per spoke
- [ ] `components/ui/stats_bar.rs`: `StatsBar` (Oura data display)
- [ ] `pages/hub.rs`: `HubPage`

**Verification**: Hub shows active goal spokes; Oura stats appear in bar.

---

## Phase 10: Frontend — WebGL Map Page

**Goal**: `/map` renders the full tree in radial WebGL canvas. This is the hardest component.

### Tasks

- [ ] Set up WebGL context via `web-sys`
- [ ] Write GLSL shaders (vertex + fragment) for:
  - `nodes` layer (colored circles with glow)
  - `connections` layer (lines between dependent nodes)
  - `nebula` layer (background nebula effect — animated)
  - `compass` layer (compass overlay at high zoom)
  - `particles` layer (particle effects)
  - `effects` layer (unlock burst animation)
- [ ] Implement vertex buffer management for each layer
- [ ] Implement zoom/pan state (scale, offset) managed by Leptos signals
- [ ] Implement mouse event handlers (wheel zoom, drag pan, click hit detection)
- [ ] Implement hit detection via ray-casting (`hit-detection.ts` → Rust)
- [ ] `compute_radial_map_layout()` output consumed to populate buffers
- [ ] Render loop via `requestAnimationFrame` (web-sys `window().request_animation_frame()`)
- [ ] SubtreeView: click node → drill-down overlay for subtree
- [ ] `components/map/nebula_radar.rs`: Background nebula effect
- [ ] `pages/map.rs`: `MapPage`

> **Note**: Consider using a `canvas` element with `wasm-bindgen` callbacks instead of fighting Leptos reactivity inside a render loop. The render loop should be purely imperative; only use Leptos signals to trigger re-initialization when node data changes.

**Verification**: Map renders all nodes; zoom/pan works; click selects node and opens detail panel.

---

## Phase 11: Frontend — Jarvis AI Coach Page

**Goal**: `/jarvis` shows tree browser + streaming chat with Claude.

### Tasks

- [ ] `api/chat.rs`: SSE streaming client
  - POST to `/api/chat`
  - Read response as `ReadableStream` via `web-sys`
  - Parse `data: {...}\n\n` SSE lines
  - Emit parsed `ChatEvent` values
- [ ] `components/chat/chat_panel.rs`: `AIChatPanel`
  - Chat message list
  - Input box + send button
  - Streaming text delta rendering (append to last message)
  - "Thinking..." indicator during stream
  - Tool text (tree suggestions) display
  - Web search toggle
- [ ] `components/chat/node_tree_preview.rs`: `NodeTreePreview` (500+ lines)
  - Renders `SuggestedNode[]` as interactive tree
  - Inline editing of suggested nodes
  - Apply button (calls create_node for each in order)
  - Progress bar during apply
- [ ] `components/chat/add_node_modal.rs`: `AddNodeModal`
- [ ] `components/jarvis/tree_browser.rs`: `TreeBrowser` (sidebar node selector)
- [ ] Chat history persistence: `gloo-storage` localStorage with key `lat-chat-{nodeId}`
- [ ] `pages/jarvis.rs`: `JarvisPage` — split layout: tree browser | chat panel

**Verification**: Full chat flow: type message → stream response → see suggested nodes → apply → see in tree.

---

## Phase 12: Polish & Production

### Tasks

- [ ] Error boundaries in Leptos (`<ErrorBoundary>`)
- [ ] Loading skeletons (shimmer placeholders while nodes load)
- [ ] Animations: CSS transitions for node cards, panel open/close, toast slide-in
  - Replace Framer Motion with CSS animations + Leptos `AnimatedShow`
- [ ] Unlock animation: `UnlockBurst` — CSS keyframe animation triggered by status change
- [ ] Light mode support via `prefers-color-scheme` (CSS variables already handle this)
- [ ] 404 page for unknown routes
- [ ] `robots.txt`, `favicon.ico` in `assets/`
- [ ] Dockerfile (multi-stage: build → slim runtime image)
- [ ] `.env.example` with all required vars documented
- [ ] Verify all Notion property names match exactly
- [ ] Test with production Notion database

---

## Risk Register

| Risk | Severity | Mitigation |
|------|----------|-----------|
| WebGL map complexity | High | Port layer by layer; skip effects initially; get basic rendering first |
| Recursive tree borrow conflicts | High | Use two-pass strategy (compute flat, then assemble) |
| Claude SSE streaming in Rust | Medium | Use `reqwest` streaming + `async-stream` crate; test independently |
| Leptos reactivity inside render loop | Medium | Keep WebGL render loop fully imperative; only use signals as triggers |
| Notion pagination edge cases | Low | Test with > 100 nodes; verify cursor handling |
| Oura OAuth token rotation | Low | Port exact logic; test token refresh path |

---

## Dependency Reference

### Backend (`crates/backend/Cargo.toml`)
```toml
[dependencies]
life-tree-core = { path = "../core" }
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tower-http = { version = "0.5", features = ["cors", "fs", "trace"] }
tokio-stream = "0.1"
async-stream = "0.3"
futures-util = "0.3"
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenvy = "0.15"
leptos = { version = "0.7", features = ["ssr"] }
leptos_axum = "0.7"
leptos_meta = "0.7"
leptos_router = "0.7"
```

### Frontend (`crates/frontend/Cargo.toml`)
```toml
[dependencies]
life-tree-core = { path = "../core" }
leptos = { version = "0.7", features = ["csr"] }
leptos_router = "0.7"
leptos_meta = "0.7"
leptos-use = "0.13"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
web-sys = { version = "0.3", features = [
    "WebGlRenderingContext", "WebGlBuffer", "WebGlProgram", "WebGlShader",
    "HtmlCanvasElement", "CanvasRenderingContext2d",
    "EventSource", "MessageEvent",
    "Window", "Document", "Element", "HtmlElement",
    "MouseEvent", "WheelEvent", "KeyboardEvent",
    "Storage", "console",
] }
js-sys = "0.3"
gloo-net = { version = "0.6", features = ["http"] }
gloo-storage = "0.3"
gloo-timers = "0.3"
pulldown-cmark = "0.12"
chrono = { version = "0.4", features = ["serde", "wasmbind"] }
uuid = { version = "1", features = ["v4", "js"] }
```

### Core (`crates/core/Cargo.toml`)
```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
chrono = { version = "0.4", features = ["serde", "wasmbind"] }
```

---

## Estimated Effort by Phase

| Phase | Effort | Notes |
|-------|--------|-------|
| 0: Scaffold | 0.5 day | Boilerplate |
| 1: Data Models | 1 day | Mechanical translation |
| 2: Graph Algorithms | 2–3 days | Complex, needs testing |
| 3: Notion Client | 1.5 days | Many edge cases in property mapping |
| 4: Chat Endpoint | 2 days | SSE + tool_use parsing |
| 5: Oura & Search | 1 day | Straightforward OAuth |
| 6: Frontend State | 1 day | Leptos context setup |
| 7: Tree Browser | 2 days | Many components |
| 8: Detail Panel | 2 days | Largest single component |
| 9: Hub Compass | 1.5 days | SVG math |
| 10: WebGL Map | 4–5 days | Hardest component |
| 11: Jarvis Chat | 2.5 days | SSE client + preview |
| 12: Polish | 1.5 days | Animations, errors, prod |
| **Total** | **~22–25 days** | Solo developer estimate |

---

## Order of Operations (Recommended)

Start with the **backend** (Phases 0–5) since it's pure Rust with no WASM complexity. Get all API endpoints working and verified against the existing Next.js behavior. Then build the **frontend** from simplest to most complex (tree browser → detail panel → hub → map → Jarvis). This lets you run the app end-to-end as early as Phase 7.

```
Phase 0 → 1 → 2 → 3 → 4 → 5     (backend + core: ~8 days)
       ↓
Phase 6 → 7 → 8 → 9 → 10 → 11   (frontend: ~14 days)
       ↓
Phase 12                           (polish: ~2 days)
```
