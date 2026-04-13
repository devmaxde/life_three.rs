# Data Models — TypeScript → Rust Translation
> Life Achievement Tree

---

## Core Types (`crates/core/src/types.rs`)

### ArchiveReason

```typescript
// TypeScript
type ArchiveReason = 'abgebrochen' | 'pausiert' | 'erledigt'
```

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveReason {
    Abgebrochen,
    Pausiert,
    Erledigt,
}
```

### NodeBadge

```typescript
type NodeBadge = 'milestone' | 'boss-level'
```

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeBadge {
    Milestone,
    BossLevel,
}
```

### NodeColor

```typescript
type NodeColor = 'purple' | 'blue' | 'green' | 'orange' | 'pink' | 'teal'
```

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeColor {
    Purple,
    Blue,
    Green,
    Orange,
    Pink,
    Teal,
}
```

### Resource

```typescript
interface Resource {
    title: string
    url?: string
    type?: string
}
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub title: String,
    pub url: Option<String>,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
}
```

### NotionNode

```typescript
interface NotionNode {
    id: string
    name: string
    icon: string | null
    description: string
    why: string
    criteria: string
    parentId: string | null
    dependsOnIds: string[]
    done: boolean
    archived: ArchiveReason | null
    pinned: boolean
    badge: NodeBadge | null
    color: NodeColor | null
    due: string | null
    timeRange: string | null
    resources: Resource[]
    createdTime: string
}
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotionNode {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub description: String,
    pub why: String,
    pub criteria: String,
    pub parent_id: Option<String>,
    pub depends_on_ids: Vec<String>,
    pub done: bool,
    pub archived: Option<ArchiveReason>,
    pub pinned: bool,
    pub badge: Option<NodeBadge>,
    pub color: Option<NodeColor>,
    pub due: Option<String>,
    pub time_range: Option<String>,
    pub resources: Vec<Resource>,
    pub created_time: String,
}
```

> Note: Use `#[serde(rename_all = "camelCase")]` to maintain JSON compatibility with the frontend and Notion client.

### NodeType, NodeStatus

```typescript
type NodeType = 'root' | 'container' | 'leaf'
type NodeStatus = 'locked' | 'active' | 'completed' | 'archived'
```

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Root,
    Container,
    Leaf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Locked,
    Active,
    Completed,
    Archived,
}
```

### Position

```typescript
interface Position { x: number, y: number }
```

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}
```

### ComputedNode

```typescript
interface ComputedNode extends NotionNode {
    nodeType: NodeType
    status: NodeStatus
    progress: number          // 0–1
    depth: number
    children: ComputedNode[]
    dependents: ComputedNode[]
    position: Position
    isCycleMember: boolean
}
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputedNode {
    // All NotionNode fields
    #[serde(flatten)]
    pub base: NotionNode,
    
    // Computed fields
    pub node_type: NodeType,
    pub status: NodeStatus,
    pub progress: f64,
    pub depth: u32,
    pub children: Vec<ComputedNode>,
    pub dependents: Vec<ComputedNode>,
    pub position: Position,
    pub is_cycle_member: bool,
}

// Convenience accessors
impl ComputedNode {
    pub fn id(&self) -> &str { &self.base.id }
    pub fn name(&self) -> &str { &self.base.name }
    pub fn parent_id(&self) -> Option<&str> { self.base.parent_id.as_deref() }
    pub fn depends_on_ids(&self) -> &[String] { &self.base.depends_on_ids }
    pub fn done(&self) -> bool { self.base.done }
    pub fn archived(&self) -> Option<&ArchiveReason> { self.base.archived.as_ref() }
}
```

> **Important**: The recursive `children` and `dependents` fields make this a tree structure. In Rust, recursive heap allocation requires `Vec<ComputedNode>` (not `Vec<Box<ComputedNode>>`), which Rust can handle since `Vec` is already heap-allocated.

### MapNode

```typescript
// Actual interface from lib/graph.ts
interface MapNode {
    node: ComputedNode;      // full reference to computed node
    x: number;
    y: number;
    ring: number;            // distance from center (0 = root)
    sectorId: string;        // root node id that owns this sector
    sectorColor: string;     // root's color
    sectorAngleStart: number; // radians
    sectorAngleEnd: number;   // radians
    sectorCenter: { x: number; y: number }; // centroid for nebula
}
```

```rust
#[derive(Debug, Clone)]
pub struct MapNode {
    pub node: ComputedNode,      // full node reference (not just ID)
    pub x: f64,
    pub y: f64,
    pub ring: u32,
    pub sector_id: String,
    pub sector_color: String,
    pub sector_angle_start: f64, // radians
    pub sector_angle_end: f64,   // radians
    pub sector_center: Position,
}
```

> **Note:** `MapNode` holds a full `ComputedNode` reference, not just scalar fields. Rendering layers access `mn.node.status`, `mn.node.badge`, `mn.node.children`, etc.

### OuraData

```typescript
interface OuraData {
    readiness: number
    sleepScore: number
    sleepDuration: number
    bedtimeRegularity: 'good' | 'medium' | 'poor'
    lastUpdated: string
}
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BedtimeRegularity {
    Good,
    Medium,
    Poor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OuraData {
    pub readiness: u8,
    pub sleep_score: u8,
    pub sleep_duration: f64,
    pub bedtime_regularity: BedtimeRegularity,
    pub last_updated: String,
}
```

### Toast

```typescript
interface Toast {
    id: string
    message: string
    type: 'success' | 'error' | 'info'
    duration?: number
}
```

```rust
// Toast — includes optional undo callback (used by archive action)
// Note: no ToastType enum in the actual code — toasts are simple with auto-dismiss
pub struct Toast {
    pub id: String,               // "toast-{counter}"
    pub message: String,
    pub undo_action: Option<Box<dyn Fn()>>,  // callback for undo button
    // timeout handle managed by the store (5000ms auto-dismiss)
}
```

### ChatMessage

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub role: ChatRole,         // User or Assistant
    pub text: String,
    pub suggested_nodes: Option<Vec<SuggestedNode>>,
    pub pending_edits: Option<Vec<NodeEdit>>,
    pub edits_approved: Option<bool>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    User,
    Assistant,
}
```

### NodeEdit

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeEdit {
    pub node_id: String,
    pub node_name: Option<String>,  // resolved name for display
    pub updates: serde_json::Value, // arbitrary field updates
}
```

### LayoutOverrides

```rust
pub type LayoutOverrides = HashMap<String, Position>;
```

---

## API Request/Response Types (`crates/core/src/api.rs`)

### CreateNodeRequest (POST /api/nodes)

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNodeRequest {
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub why: Option<String>,
    pub criteria: Option<String>,
    pub color: Option<NodeColor>,
    pub badge: Option<NodeBadge>,
    pub due: Option<String>,
    pub time_range: Option<String>,
    pub resources: Option<Vec<Resource>>,
    pub parent_id: Option<String>,
    pub depends_on_ids: Option<Vec<String>>,
}
```

### UpdateNodeRequest (PATCH /api/nodes/:id)

```rust
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNodeRequest {
    pub done: Option<bool>,
    pub archived: Option<ArchiveReason>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub why: Option<String>,
    pub criteria: Option<String>,
    pub pinned: Option<bool>,
    pub color: Option<NodeColor>,
    pub badge: Option<NodeBadge>,
    pub due: Option<String>,
    pub time_range: Option<String>,
    pub depends_on_ids: Option<Vec<String>>,
    pub parent_id: Option<String>,
    pub icon: Option<String>,
    pub resources: Option<Vec<Resource>>,
}
```

### ChatRequest (POST /api/chat)

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub role: String,      // "user" | "assistant"
    pub content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub existing_nodes: Vec<NotionNode>,
    pub parent_id: Option<String>,
    pub working_draft: Option<serde_json::Value>,
    pub web_search_results: Option<serde_json::Value>,
}
```

### ChatEvent (SSE events)

```rust
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatEvent {
    Text { content: String },
    ToolText { content: String },
    Done {
        suggested_nodes: Option<Vec<SuggestedNode>>,
        suggested_parent_id: Option<String>,
        node_edits: Option<Vec<NodeEdit>>,
        truncated: bool,
    },
    Error { message: String },
}
```

### SuggestedNode (AI output)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestedNode {
    pub temp_id: String,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub why: Option<String>,
    pub criteria: Option<String>,
    pub color: Option<NodeColor>,
    pub badge: Option<NodeBadge>,
    pub due: Option<String>,
    pub time_range: Option<String>,
    pub resources: Option<Vec<Resource>>,
    pub parent_id: Option<String>,
    pub depends_on_ids: Option<Vec<String>>,  // may contain _ref:tempId
    pub children: Option<Vec<SuggestedNode>>,
}
```

### NodeEdit (AI output)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeEdit {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub why: Option<String>,
    pub criteria: Option<String>,
    pub color: Option<NodeColor>,
    pub badge: Option<NodeBadge>,
    pub due: Option<String>,
    pub time_range: Option<String>,
    pub resources: Option<Vec<Resource>>,
    pub depends_on_ids: Option<Vec<String>>,
}
```

---

## Notion API Mapping (`crates/backend/src/services/notion.rs`)

### Property extraction helpers

```rust
fn extract_text(props: &Map<String, Value>, key: &str) -> String {
    props.get(key)
        .and_then(|v| v["rich_text"].as_array())
        .map(|arr| arr.iter()
            .filter_map(|b| b["plain_text"].as_str())
            .collect::<String>())
        .unwrap_or_default()
}

fn extract_relation(props: &Map<String, Value>, key: &str) -> Vec<String> {
    props.get(key)
        .and_then(|v| v["relation"].as_array())
        .map(|arr| arr.iter()
            .filter_map(|r| r["id"].as_str().map(String::from))
            .collect())
        .unwrap_or_default()
}

fn extract_select(props: &Map<String, Value>, key: &str) -> Option<String> {
    props.get(key)
        .and_then(|v| v["select"]["name"].as_str())
        .map(String::from)
}

fn extract_checkbox(props: &Map<String, Value>, key: &str) -> bool {
    props.get(key)
        .and_then(|v| v["checkbox"].as_bool())
        .unwrap_or(false)
}

fn extract_date(props: &Map<String, Value>, key: &str) -> Option<String> {
    props.get(key)
        .and_then(|v| v["date"]["start"].as_str())
        .map(String::from)
}
```

### Notion database property names
```rust
// Property key constants
const PROP_DESCRIPTION: &str = "Description";
const PROP_WHY: &str = "Why";
const PROP_CRITERIA: &str = "Criteria";
const PROP_TIME_RANGE: &str = "TimeRange";
const PROP_RESOURCES: &str = "Resources";
const PROP_PARENT: &str = "Parent";
const PROP_DEPENDS_ON: &str = "Depends On";
const PROP_DONE: &str = "Done";
const PROP_PINNED: &str = "Pinned";
const PROP_BADGE: &str = "Badge";
const PROP_COLOR: &str = "Color";
const PROP_ARCHIVED: &str = "Archived";
const PROP_DUE: &str = "Due";
```

---

## Oura Token Storage

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct OuraTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub acquired_at: u64,   // Unix timestamp (seconds)
}

impl OuraTokens {
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now >= self.acquired_at + self.expires_in - 60  // 60s buffer
    }
}
```

---

## Serialization Notes

1. **camelCase vs snake_case**: Use `#[serde(rename_all = "camelCase")]` on structs that cross the API boundary (sent to/received from JS frontend or Notion). Internal-only Rust structs can use snake_case naturally.

2. **Flattening**: `ComputedNode` uses `#[serde(flatten)]` to embed `NotionNode` fields while adding computed fields. This produces a flat JSON object identical to the TypeScript interface.

3. **Option vs missing fields**: Use `#[serde(skip_serializing_if = "Option::is_none")]` on optional fields in API responses to omit null values (matches JS behavior).

4. **Recursive types**: `ComputedNode.children: Vec<ComputedNode>` works in Rust because `Vec` is heap-allocated. No `Box` needed.
