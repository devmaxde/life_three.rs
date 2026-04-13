# Source Reference — Implementation Details Missing from Other Docs
> Fills all gaps identified in the rust_port documentation analysis.
> With this file, the rust_port folder is self-contained for rebuilding.

---

## 1. Missing API Endpoint: POST /api/suggest-fields

Auto-suggests node fields (icon, color, description, etc.) when creating a new node.

```
POST /api/suggest-fields
Request:  { name: string, parentName?: string, siblingNames?: string[] }
Response: { icon?: string, color?: string, description?: string, why?: string, timeRange?: string, badge?: string | null }
```

**Implementation:**
- Uses Claude Haiku via OpenRouter (`anthropic/claude-haiku-4.5`)
- System prompt asks for pure JSON output with fields: icon (emoji), color, description (1 sentence), why (1 sentence), timeRange, badge
- Extracts JSON from response via regex `/{[\s\S]*}/`
- Returns empty `{}` on any error (non-blocking)

---

## 2. Unlock Animation Listener System

Global pub/sub for triggering unlock animations when a node is completed.

```typescript
// store/useTreeStore.ts — module-level (outside the Zustand store)
type UnlockListener = (nodeId: string) => void;
const unlockListeners = new Set<UnlockListener>();

export function onNodeUnlocked(listener: UnlockListener): () => void {
  unlockListeners.add(listener);
  return () => unlockListeners.delete(listener);  // returns unsubscribe fn
}
```

**Fired in** `updateNodeAsync()` when `updates.done === true && !original.done`:
```typescript
for (const listener of unlockListeners) {
  listener(id);
}
```

**Consumed by** `MapCanvasGL` to trigger visual unlock effects:
```typescript
useEffect(() => {
  return onNodeUnlocked((nodeId) => {
    const mn = nodesRef.current.find(n => n.node.id === nodeId);
    if (mn) {
      const isBoss = mn.node.badge === 'boss-level' || mn.node.badge === 'milestone';
      triggerUnlockAnimation(nodeId, mn.x, mn.y, mn.sectorColor, isBoss);
    }
  });
}, []);
```

**Rust equivalent:** Use a `tokio::sync::broadcast` channel or a simple `RwLock<Vec<Callback>>`.

---

## 3. MapCanvasGL — Complete Implementation Details

### 3.1 It's Canvas 2D, NOT WebGL

Despite the name "MapCanvasGL", the actual implementation uses **Canvas 2D** (`canvas.getContext('2d')`), not WebGL. All rendering is done with `CanvasRenderingContext2D` methods (arcs, gradients, fillRect, etc.). No shaders, no GL buffers.

**For Rust:** Use `web-sys::CanvasRenderingContext2d`, not `WebGlRenderingContext`.

### 3.2 Zoom Thresholds (Compass Transition)

The map transitions smoothly into a DOM-based compass (HubCanvas) at high zoom:

```
COMPASS_FADE_START = 3.4   // DOM compass starts appearing
COMPASS_FADE_END   = 3.8   // DOM compass fully visible, takes pointer events
MAP_FADE_START     = 3.0   // Map layers start fading (used inside layers)
MAP_FADE_END       = 3.4   // Map layers fully gone
```

**Map opacity formula** (used in every layer):
```
mapOpacity = clamp((3.4 - zoom) / 0.4, 0, 1)
```

**Compass opacity formula:**
```
compassOpacity = clamp((zoom - 3.4) / (3.8 - 3.4), 0, 1)
```

**Pointer events:** When `compassOpacity > 0.5`, the DOM compass overlay gets `pointer-events: auto` and the canvas stops handling clicks.

### 3.3 Zoom/Pan State (Ref-Based for rAF)

All mutable state lives in refs (not React state) so the render loop can read them without causing re-renders:

```
zoomRef    = 1           // current zoom level (0.15 to 5.0)
panRef     = {x:0, y:0} // pixel offset
sizeRef    = {w:0, h:0} // container dimensions
nodesRef   = MapNode[]   // current layout
draggingRef = false      // is user panning
lastPosRef = {x:0, y:0} // last mouse position during drag
dragDistRef = 0          // total drag distance (for click vs drag detection)
```

### 3.4 Auto-Fit on First Load

On first load only (tracked by `initialFitDone` ref), auto-fits all nodes into view:
```
fitZoom = min(width * 0.85 / contentWidth, height * 0.85 / contentHeight, 1)
zoom = max(0.15, fitZoom)
pan = { x: width/2 - centerX * zoom, y: height/2 - centerY * zoom }
```

### 3.5 Wheel Zoom (Zoom-to-Cursor)

Zoom preserves the world point under the cursor:
```
delta = -event.deltaY * 0.001
next = clamp(prev + delta, 0.15, 5.0)
scale = next / prev
pan.x = mouseX - (mouseX - pan.x) * scale
pan.y = mouseY - (mouseY - pan.y) * scale
```

### 3.6 Render Loop

Uses `requestAnimationFrame`. Runs continuously. Each frame:
1. Clear canvas (DPR-aware: `canvas.width = containerWidth * dpr`)
2. Fill with `#000000`
3. Render layers in order: Nebula → Connections → Nodes → Compass → Particles → Effects

On cleanup: `cancelAnimationFrame` + reset caches for nebula, connections, particles.

### 3.7 Click Detection

Click fires only if `dragDistRef < 5` (not a pan gesture) and `zoom < MAP_FADE_END` (not in compass mode).

Hit test converts screen→world coords, then checks distance to each node:
```
worldX = (screenX - pan.x) / zoom
worldY = (screenY - pan.y) / zoom
// iterate nodes back-to-front, find closest within radius + 6px padding
```

Click actions:
- Locked nodes → ignore
- Nodes with children + `onDrillDown` callback → open SubtreeView overlay
- Nodes with children (no callback) → navigate to `/tree/{slug}`
- Leaf nodes → `openDetailPanel(node.id)`

### 3.8 Right-Click Context Menu

Shows for active nodes only. Options:
1. Pin/Unpin focus (`updateNodeAsync(id, { pinned: !pinned })`)
2. Open detail panel
3. Open in Tree view (navigate)
4. Open in Notion (external link: `https://notion.so/{id-without-dashes}`)

### 3.9 DOM Overlays on Canvas

These are positioned absolutely over the canvas (not drawn on it):
- **Zoom indicator** (top-right): `{Math.round(zoom * 100)}%`
- **NebulaRadar** (mini-map, top-right below zoom): visible at zoom >= 0.4, opacity fades in from 0.4 to 0.5
- **Tooltip** (bottom-left): shows hovered node name/status/description
- **Context menu** (fixed position at click coords)
- **Zoom buttons** (bottom-right): +, −, reset (⌂)
- **HubCanvas** (compass DOM overlay): fades in at zoom 3.4–3.8

---

## 4. Canvas Rendering Layers — Complete Specifications

### 4.0 Shared Types

```typescript
type LayerRenderFn = (ctx, t, zoom, pan, nodes, size) => void

ACCENT_HEX = { purple: '#A78BFA', blue: '#60A5FA', green: '#34D399',
               orange: '#F59E0B', pink: '#F472B6', teal: '#2DD4BF' }

Node radii:  ROOT=36, NODE=28, BOSS=32, MILESTONE=30
```

### 4.1 Layer 0: Nebula

Multi-layered radial gradients per sector. Uses OffscreenCanvas cache (invalidated when zoom/pan changes beyond quantized thresholds).

**Per sector (4 sub-layers):**
1. **Ambient cloud**: radius `(300 + nodeCount * 18) * zoom`, alpha `0.06 + completionRatio * 0.10`
2. **Core cloud**: radius `(150 + nodeCount * 10) * zoom`, alpha `0.08 + completionRatio * 0.14`
3. **Sub-clouds** (up to 8): scattered around node positions via seeded RNG, radius `(60 + random * 80) * zoom`
4. **Bright spots**: on completed nodes, radius `25 * zoom`, alpha `0.05 + completionRatio * 0.06`

**Sector labels**: visible at zoom 0.2–0.8, font "600 {48*zoom}px Outfit", alpha 0.08 with fade-in/fade-out

**Cache key**: quantized `(w, h, round(zoom*20), round(pan.x/2), round(pan.y/2), nodeCount)`

### 4.2 Layer 1: Connections

Builds edge list from parent→child and dependency relationships. Cached (rebuilt when node count changes).

**Edge styling by status:**
- Both completed: opacity 0.6, lineWidth 2
- Active (from=completed, to=active): opacity 0.7, + flowing dash animation (`lineDashOffset = -t * 0.03`)
- Locked (to=locked): opacity 0.1, dashed `[4*zoom, 3*zoom]`
- Other: opacity 0.15, lineWidth 1

**Cross-sector edges**: use gray `#888888` instead of sector color.

**Curve**: cubic bezier with control points at `(x1 + dx*0.3, y1 + dy*0.1)` and `(x1 + dx*0.7, y2 - dy*0.1)`.

**Viewport culling**: skip if both endpoints off-screen (margin 100px).

### 4.3 Layer 2: Nodes

**Breathing animation:**
- Completed: ±5%, 4s cycle (`1 + 0.05 * sin(t * 0.0015 + charCode)`)
- Active: ±10%, 2s cycle (`1 + 0.10 * sin(t * 0.003 + charCode)`)

**Per-node rendering order:**
1. Glow (radial gradient, active: radius 2.5x, completed: 1.8x)
2. Boss: gold outer ring with rotating dash (`#F59E0B`, dash offset `-t * 0.02`)
3. Milestone: dashed ring (sector color)
4. Pinned: pulsing glow ring (purple, period ~2.5s)
5. Active (not pinned): subtle pulse ring
6. Locked: fog cloud (dark radial gradient, radius 3x)
7. Main circle (filled: completed=solid hex, pinned=33% alpha, active=13% alpha, locked=3% white)
8. Stroke (varies by state)
9. Emoji icon (cached to OffscreenCanvas per emoji+size)
10. Pin indicator (📌 at top-left of node)
11. Label (rounded rect background + text, truncated at 18 chars)
12. Badge icon (⭐ for boss, 🏅 for milestone)
13. Subtree indicator (small circle with descendant count at bottom-right)

**Visibility thresholds:**
- Labels: zoom > 0.7
- Icons: zoom > 0.4
- Roots and hovered nodes: always show labels+icons

### 4.4 Layer 3: Compass Center Dot

A glowing dot at world origin (0,0) that grows with zoom. Fades out at zoom >= 3.6 (DOM compass takes over).

```
avatarR = max(6, 40 * zoom)
pulse = 1 + 0.04 * sin(t * 0.002)
```

Renders: outer glow (purple→blue gradient), main circle (purple→blue), border, tree emoji (🌳).

### 4.5 Layer 4: Particles

Pool of max 80 particles, emitting from max 8 active nodes in viewport (pinned preferred).

**Emission**: every ~100ms, 1 particle per emitter. 2px fillRect, random angle, speed 0.2–0.5, life 2000–4000ms. Slight upward drift (`vy -= 0.002` per frame). Fades out linearly with life.

### 4.6 Layer 5: Effects (Unlock Animations)

Triggered by `triggerUnlockAnimation(nodeId, x, y, color, isBoss)`.

**5-phase sequence (2500ms, boss: 3000ms):**
1. **Flash** (0–80ms): white-hot core expanding to 60px + color halo
2. **Core Burst** (50–300ms): expanding filled gradient, easeOutCubic
3. **Shockwave 1** (100–600ms): expanding ring, thick→thin
4. **Shockwave 2** (250–900ms, boss only): second larger ring
5. **Particle Shower** (200–1500ms): 16 sparkles (24 for boss) in expanding ring with wobble
6. **Long Glow** (400–2500ms): soft lingering radial gradient

**Boss extras:**
- Screen flash (0–150ms): full-screen color overlay at 8% alpha
- Confetti (60 pieces): gravity physics, air resistance, rotation, fade after 70% life

---

## 5. NebulaRadar (Mini-Map)

120×120px canvas, circular clip, renders at 60fps via rAF.

1. Compute world bounds from all nodes (with 100px margin)
2. Scale to fit radar: `radarScale = 110 / max(worldW, worldH)`
3. Draw sector nebulae as blurred dots (radial gradients, radius `8 + count * 0.8`)
4. Draw nodes as tiny dots (completed: 1.5px, active: 1.2px, other: 0.6px)
5. Draw viewport rectangle (red, `rgba(255,80,80,0.7)`)
6. Draw center dot (purple, 2px)
7. Circle border (white 10% opacity)

---

## 6. SubtreeView (Drill-Down Overlay)

Modal overlay with breadcrumb navigation and recursive card tree.

**Auto-expand depth** based on tree size:
- ≤15 descendants: expand 3 levels
- ≤40 descendants: expand 2 levels
- >40 descendants: expand 1 level

**Card scaling**: each depth level shrinks by factor 0.12 (min scale 0.7). Affects min-width, padding, font sizes.

**Measured connectors**: uses `MutationObserver` + `getBoundingClientRect` to measure horizontal bar width between sibling cards. Vertical stubs connect bar to each card.

**Framer Motion** animations: expand/collapse children with `AnimatePresence`.

---

## 7. Client-Side SSE Streaming (Jarvis Chat)

The chat client in `jarvis-client.tsx` implements SSE parsing manually (not using EventSource):

```typescript
// 1. POST to /api/chat with full body
const res = await fetch('/api/chat', { method: 'POST', body: JSON.stringify({
  messages,           // chat history [{role, content}]
  existingNodes,      // all non-archived nodes (compact format)
  parentId,           // focused subtree root (or null)
  workingDraft,       // current draft nodes (or null)
  webSearchResults,   // optional Tavily results
}), signal: controller.signal });

// 2. Read response as streaming text
const reader = res.body.getReader();
const decoder = new TextDecoder();
let buffer = '';

while (true) {
  const { done, value } = await reader.read();
  if (done) break;
  buffer += decoder.decode(value, { stream: true });
  const lines = buffer.split('\n');
  buffer = lines.pop();  // keep incomplete line in buffer
  
  for (const line of lines) {
    if (!line.startsWith('data: ')) continue;
    const event = JSON.parse(line.slice(6));
    
    switch (event.type) {
      case 'text':      // append to streaming message
      case 'tool_text': // text from tool use (shown after stream)
      case 'done':      // { suggestedNodes, suggestedParentId, nodeEdits, truncated }
      case 'error':     // { message }
    }
  }
}
```

**SSE Event Types (from server):**
```
data: {"type":"text","content":"..."}         // streaming text delta
data: {"type":"tool_text","content":"..."}    // message from tool_use block
data: {"type":"done","suggestedNodes":[...],"suggestedParentId":"...","nodeEdits":[...],"truncated":false}
data: {"type":"error","message":"..."}
```

**Client features:**
- AbortController for cancel support
- Loading status rotation: ["Thinking...", "Brainstorming...", "Cooking...", "Vibing...", "Sketching nodes...", "Building skilltree..."] every 3s
- Optional web search: if enabled, calls `/api/web-search` first, passes results to chat API
- Chat persistence: `localStorage` keyed by `lat-chat-{nodeId}` (or `lat-chat-root`)
- Draft persistence: `localStorage` keyed by `lat-draft-{nodeId}`
- Selected node persistence: `localStorage` key `lat-selected-node`
- Tree collapsed state: `localStorage` key `lat-tree-collapsed`

---

## 8. Node Schema — AI Tool Generation Functions

### makeNodeJsonSchema(maxDepth)

Generates recursive JSON schema for the `suggest_nodes` tool. Key fields:
- name (required), icon, description, why, criteria, color (enum), badge (enum), timeRange, due
- `_ref`: short reference ID for cross-referencing between new nodes
- `dependsOnIds`: array of strings (can mix real IDs + `_ref` values)
- `resources`: array of `{ title (required), url?, type? (enum) }`
- `children`: recursive (up to maxDepth levels deep, called with maxDepth=4)

### EDIT_NODES_TOOL

Tool for modifying existing nodes. Input schema:
```json
{
  "message": "string (response to user)",
  "edits": [{
    "nodeId": "string",
    "updates": {
      "name?", "description?", "why?", "criteria?",
      "color?" (enum), "badge?" (enum or "" to remove),
      "timeRange?", "due?" (YYYY-MM-DD or "" to remove),
      "parentId?" (ID or "" to make root),
      "dependsOnIds?" (array, overwrites entirely),
      "done?" (boolean), "pinned?" (boolean)
    }
  }]
}
```

### SUGGEST_NODES_TOOL

Tool for proposing new nodes. Input schema:
```json
{
  "message": "string (response to user)",
  "parentId?": "string (existing node ID to nest under)",
  "nodes": [makeNodeJsonSchema(4)]  // recursive tree up to 4 levels
}
```

### _ref Resolution (Server-Side)

After receiving tool output, the server:
1. Assigns `tempId = "suggested-{timestamp}-{counter}"` to each node
2. Builds `refToTempId` map from `_ref` values to tempIds
3. Replaces all `_ref` references in `dependsOnIds` with resolved tempIds
4. Sends resolved nodes to client

---

## 9. Toast with Undo Action

```typescript
interface Toast {
  id: string;              // "toast-{counter}"
  message: string;
  undoAction?: () => void; // callback for undo button
  timeout: ReturnType<typeof setTimeout>;  // 5000ms auto-dismiss
}
```

Used by `archiveNode`: passes `() => unarchiveNode(id)` as undoAction.

---

## 10. Notion CRUD — Complete Property Mapping

### fetchAllNodes()
Cursor-based pagination, 100 pages per request:
```typescript
do {
  const response = await notion.dataSources.query({
    data_source_id: DATABASE_ID,
    start_cursor: cursor,
    page_size: 100,
  });
  // process results
  cursor = response.has_more ? response.next_cursor : undefined;
} while (cursor);
```

### pageToNode() — Field Extraction

| Schema Field | Notion Source | Extraction |
|-------------|---------------|------------|
| id | `page.id` | Direct |
| name | `props.Name` | `extractText` (checks `.title` then `.rich_text`) |
| icon | `page.icon` | `page.icon.type === 'emoji' ? page.icon.emoji : null` (NOT a property) |
| description | `props.Description` | `extractText` (rich_text) |
| why | `props.Why` | `extractText` (rich_text) |
| criteria | `props.Criteria` | `extractText` (rich_text) |
| pinned | `props.Pinned` | `extractCheckbox` |
| parentId | `props.Parent` | `extractRelation()[0]` (first relation or null) |
| dependsOnIds | `props['Depends on']` | `extractRelation()` (all relations) |
| done | `props.Done` | `extractCheckbox` |
| badge | `props.Badge` | `extractSelect` cast to enum |
| color | `props.Color` | `extractSelect` cast to enum |
| archived | `props.Archived` | `extractSelect` cast to enum |
| due | `props.Due` | `extractDate` (`.date?.start`) |
| timeRange | `props['Time Range']` | `extractText` or null if empty |
| resources | `props.Resources` | `JSON.parse(extractText())` with try/catch → `[]` on error |
| createdTime | `page.created_time` | Direct |

### createNode() — Property Building

Sets properties only if values are provided (skips null/empty):
- Name → `{ title: [{ text: { content } }] }`
- Description/Why/Criteria/TimeRange → `{ rich_text: [{ text: { content } }] }`
- Parent → `{ relation: [{ id }] }`
- Depends on → `{ relation: ids.map(id => ({ id })) }`
- Color/Badge/Archived → `{ select: { name } }` or `{ select: null }`
- Done/Pinned → `{ checkbox: value }`
- Due → `{ date: { start: value } }` or `{ date: null }`
- Resources → `{ rich_text: [{ text: { content: JSON.stringify(resources) } }] }`
- Icon → `page.icon: { emoji: value }` (set on page object, not properties)

### updateNode() — Partial Updates

Same property format as create, but only includes fields present in the update object. Null values clear fields:
- `archived: null` → `{ select: null }`
- `badge: null` → `{ select: null }`
- `due: null` → `{ date: null }`
- `timeRange: null` → `{ rich_text: [] }`
- `parentId: null` → `{ relation: [] }`

### fetchPageContent() — Blocks to Markdown

Fetches all blocks (paginated), converts each to markdown:
- paragraph → `text\n\n`
- heading_1/2/3 → `# / ## / ### text\n\n`
- bulleted_list_item → `- text\n`
- numbered_list_item → `1. text\n`
- to_do → `- [x] text\n` or `- [ ] text\n`
- quote/callout → `> text\n\n`
- code → `` ```lang\ntext\n``` ``
- divider → `---\n\n`
- bookmark → `[label](url)\n\n`
- image → `![alt](url)\n\n`
- Rich text annotations: bold (`**`), italic (`*`), code (`` ` ``), strikethrough (`~~`), href (`[text](url)`)

---

## 11. Oura Integration — Complete Details

### Token Storage
File-based at `data/oura-tokens.json`:
```json
{ "access_token": "...", "refresh_token": "...", "expires_at": 1718000000000 }
```

`expires_at` is calculated as `Date.now() + expires_in * 1000`.

### Token Refresh
**Single-use refresh tokens** — Oura returns a NEW refresh_token on each refresh. Must save both tokens after refresh.

### Data Fetching
Three parallel API calls:
1. `GET /v2/usercollection/daily_readiness?start_date={today}` → readiness score
2. `GET /v2/usercollection/daily_sleep?start_date={weekAgo}` → sleep scores
3. `GET /v2/usercollection/sleep?start_date={weekAgo}` → sleep sessions (duration + bedtime)

### Sleep Duration Calculation
Groups `total_sleep_duration` by day. Takes today's value, falls back to yesterday's. Converts seconds → hours (1 decimal).

### Bedtime Regularity
Standard deviation of bedtimes from sleep sessions:
```
1. Extract bedtime_start as minutes since midnight
2. After-midnight correction: if hours < 12, add 24*60 (treats 1am as 25:00)
3. Calculate mean, then std deviation
4. regularity = stdDev < 30 ? 'good' : stdDev < 60 ? 'medium' : 'poor'
5. Needs >= 3 data points, otherwise defaults to 'good'
```

### Cache
`data/oura-cache.json` with 1-hour TTL: `{ data: OuraData, timestamp: number }`.

---

## 12. computeLayout — Complete Edge Cases

Constants:
```
NODE_WIDTH  = 180
NODE_HEIGHT = 120
H_GAP       = 80
V_GAP       = 40
GROUP_GAP   = 60
```

### Zone-Band Allocation

Prevents nodes from different parent groups from interleaving on the Y axis:

1. Compute `getGroup(node)` = `nodeGroupMap?.get(node.id) ?? node.parentId ?? ''`
2. For each group, find max node count in any single depth column → that's the band height
3. Sort groups by minimum depth (groups starting earlier go on top)
4. Stack bands vertically: `bandTop += bandHeight + GROUP_GAP`
5. Ungrouped nodes go after all groups

### Initial Placement

For each depth column, split nodes by zone. Place each zone's nodes starting from their band top, stacking vertically with `NODE_HEIGHT + V_GAP` spacing.

### Centering Pass (3 Iterations, Bidirectional)

**Sweep 1 (downstream, deep→shallow):** Center each parent on the Y midpoint of its dependents.

**Sweep 2 (upstream, shallow→deep):** Center nodes on their dependencies, BUT only if the node is the **sole dependent** of ALL its parent nodes. This prevents zigzag in branching chains.

**Overlap resolution** (per iteration): Within each depth column, process each zone independently. Sort nodes by Y, push apart to maintain `NODE_HEIGHT + V_GAP` minimum spacing.

### Global Group Separation (After Centering)

Compute the global Y extent of each group across ALL depth columns. Sort groups by minY. Push overlapping groups apart by `GROUP_GAP`.

### Final Steps

1. **Normalize**: shift all Y positions so minimum is 0
2. **Apply overrides**: manual position overrides from user
3. **Write back**: set `node.position` for each visible node

---

## 13. MapNode — Complete Interface

```typescript
interface MapNode {
  node: ComputedNode;
  x: number;
  y: number;
  ring: number;
  sectorId: string;
  sectorColor: string;
  sectorAngleStart: number;   // radians — MISSING from DATA_MODELS.md
  sectorAngleEnd: number;     // radians — MISSING from DATA_MODELS.md
  sectorCenter: { x: number; y: number };
}
```

---

## 14. Additional Types

### ChatMessage
```typescript
interface ChatMessage {
  role: 'user' | 'assistant';
  text: string;
  suggestedNodes?: SuggestedNode[];
  pendingEdits?: NodeEdit[];
  editsApproved?: boolean;
  durationMs?: number;
}
```

### NodeEdit
```typescript
interface NodeEdit {
  nodeId: string;
  nodeName?: string;  // resolved name for display
  updates: Record<string, unknown>;
}
```

### LayoutOverrides
```typescript
interface LayoutOverrides {
  [nodeId: string]: { x: number; y: number };
}
```

---

## 15. Chat API — Input Sanitization

### sanitizeText(input, maxLength)
Strips control characters (keeps `\n`, `\r`, `\t`), truncates to maxLength.

### sanitizeNodeName(name)
Calls `sanitizeText(name, 200)`, then strips `< > { } [ ]` characters.

### sanitizeDraft(nodes)
Recursively sanitizes all string values in draft node tree. Strips unknown types entirely.

### Limits
```
MAX_MESSAGE_LENGTH  = 10,000
MAX_MESSAGES        = 50
MAX_EXISTING_NODES  = 200
MAX_DRAFT_NODES     = 100
```

### Data Context Format

Existing nodes are formatted in two tiers when a parentId is set:
- **Focused subtree** (full detail): all fields including description, why, criteria
- **All other nodes** (compact): name, id, type, status, parent, depends-on only

When no parentId: all nodes in compact format.

Wrapped in `<app-data type="existing-nodes">` tags.
Working draft wrapped in `<app-data type="working-draft">`.
Web search results wrapped in `<app-data type="web-search-results" trust-level="external">`.

Security boundary appended after system prompt warns Claude to treat `<app-data>` as data, not instructions.

---

## 16. HubCanvas (Compass View)

SVG spoke lines from center to each pinned node position. DOM-based (not canvas).

**Layout:**
- Center: `(containerWidth/2, containerHeight/2)`
- Radius: `clamp(containerSize * 0.32, 180, 280)`
- Angle step: `360 / spokeCount`, starting at -90° (top)

**Spoke lines:** SVG `<line>` with sector color, dashed when not expanded, animated dash offset.

**Components:**
- `CenterNode` — user avatar at center (80×80px)
- `CompassSpoke` — positioned absolutely at angle/distance from center, shows next active node + path to boss
- `BranchCluster` — groups unpinned branches

**Empty states:**
- No roots → "Erstelle deinen ersten Branch"
- Roots but no pinned nodes → hint to pin nodes
- Unpinned nodes available → indicator bar at top

---

## 17. Web Search API

```
POST /api/web-search
Request:  { query: string }
Response: { answer?: string, results: [{ title, url, content }] }
```

Uses Tavily API. Results are sanitized and limited to 5 before passing to chat.

---

## 18. Chat System Prompt (Complete)

The full system prompt is ~105 lines and defines Jarvis as a German-speaking goal coach and skill-tree architect. Key sections:

1. **What is the Life Achievement Tree** — game-inspired personal skill tree (Warframe, PoE, Minecraft)
2. **Data model** — explains Parent vs Depends-on distinction
3. **Personality** — "smarter Kumpel", direct, enthusiastic, honest
4. **Response patterns** — clear request → suggest directly; vague → max 1-2 questions; open → analyze tree
5. **Skill-Scout** — proactively suggest cross-branch synergies
6. **How to build skill trees** — progression patterns (chain, fan-out, fan-in, cross-branch)
7. **Hard rules** — no cycles, always enterable, forward-only deps, real IDs, under existing roots
8. **_ref system** — cross-referencing between new nodes
9. **Insert between** — pattern for inserting nodes in existing chains
10. **Node quality** — binary, no emojis in name, criteria + timeRange always set
11. **edit_nodes** — overwrites depIds completely, parentId="" makes root, edits need approval
12. **Style** — German, markdown, 2-3 variants for goals, resources only if known

The prompt is stored as a const string in the chat route handler.
