# Life Achievement Tree — Complete Technical Analysis
> Generated 2026-04-13 for Rust/Leptos port planning

---

## Project Summary

**Life Achievement Tree** is a personal goal-management web app structured as a tree of achievements, skills, and milestones. The user navigates goals through three views (Compass Hub, WebGL Map, Tree Browser), edits nodes inline, and interacts with an AI coach ("Jarvis") powered by Claude. All data is persisted in a Notion database. Health metrics from an Oura Ring are shown as context.

It is a **single-user** personal tool — no multi-tenancy, no user accounts.

---

## 1. Project Structure

```
life-tree/
├── app/                        # Next.js App Router
│   ├── page.tsx                # / → HubCanvas (compass view)
│   ├── map/page.tsx            # /map → WebGL map
│   ├── jarvis/page.tsx         # /jarvis → AI coach
│   ├── tree/[...path]/page.tsx # /tree/* → tree browser
│   ├── layout.tsx              # Root layout (fonts, store init)
│   └── api/                   # API routes (server-side)
│       ├── chat/route.ts       # POST — Claude SSE streaming
│       ├── nodes/route.ts      # GET/POST — node CRUD
│       ├── nodes/[id]/route.ts # PATCH — update node
│       ├── nodes/[id]/content/route.ts  # GET — Notion blocks → MD
│       ├── oura/auth/route.ts  # GET — OAuth initiate
│       ├── oura/callback/route.ts # GET — OAuth code exchange
│       ├── oura/data/route.ts  # GET — health metrics
│       └── web-search/route.ts # POST — Tavily search
├── components/                 # React UI (~33 files)
│   ├── hub/                   # Compass spoke visualization
│   ├── map/                   # WebGL canvas + layers
│   ├── tree/                  # Hierarchical tree browser
│   ├── detail/                # Node detail panel
│   ├── jarvis/                # Chat + node preview
│   └── ui/                    # Shared primitives
├── lib/                       # Business logic
│   ├── graph.ts               # 868 lines — core algorithms
│   ├── node-schema.ts         # Zod schemas + TypeScript types
│   ├── notion.ts              # 394 lines — Notion client
│   ├── oura.ts                # 187 lines — Oura OAuth + data
│   └── types.ts               # ComputedNode, MapNode, etc.
├── store/
│   └── useTreeStore.ts        # 343 lines — Zustand store
├── styles/
│   └── glass.css              # Design system variables
├── data/                      # Runtime (gitignored)
│   ├── oura-tokens.json       # OAuth tokens (file-based)
│   └── oura-cache.json        # Response cache (1h TTL)
└── scripts/                   # Notion seed utilities
```

---

## 2. All Routes / Pages

| Route | Method | Purpose |
|-------|--------|---------|
| `/` | GET | Compass Hub — radial spoke view of active goals |
| `/map` | GET | WebGL canvas — full tree in radial layout, zoom/pan |
| `/jarvis` | GET | AI Coach — tree browser + streaming chat with Claude |
| `/tree/[...path]` | GET | Hierarchical Tree Browser — navigable subtree |
| `/api/chat` | POST | Claude AI coaching via SSE streaming |
| `/api/nodes` | GET | Fetch all nodes from Notion |
| `/api/nodes` | POST | Create new node in Notion |
| `/api/nodes/[id]` | PATCH | Update node properties |
| `/api/nodes/[id]/content` | GET | Fetch Notion blocks → Markdown |
| `/api/oura/auth` | GET | Initiate Oura OAuth flow |
| `/api/oura/callback` | GET | Exchange OAuth code for tokens |
| `/api/oura/data` | GET | Fetch sleep/readiness data |
| `/api/web-search` | POST | Tavily web search |
| `/api/suggest-fields` | POST | AI field suggestion for new nodes (Claude Haiku) |

---

## 3. React Components (33 files)

### Hub Compass
| File | Lines | Purpose |
|------|-------|---------|
| `HubCanvas.tsx` | ~112 | Radial spoke visualization, expand/collapse |
| `CenterNode.tsx` | — | Central hub node render |
| `CompassSpoke.tsx` | — | Individual spoke to next active node |
| `BranchCluster.tsx` | — | Cluster of unpinned branches |

### Map / WebGL
| File | Lines | Purpose |
|------|-------|---------|
| `MapCanvasGL.tsx` | 400+ | WebGL renderer, zoom/pan, hit detection |
| `MapCanvas.tsx` | — | Wrapper/fallback |
| `NebulaRadar.tsx` | — | Background nebula effect |
| `SubtreeView.tsx` | — | Drill-down subtree view |
| `layers/nodes.ts` | — | Node circles (color, status) |
| `layers/connections.ts` | — | Edge lines between dependent nodes |
| `layers/compass.ts` | — | Compass overlay at high zoom |
| `layers/nebula.ts` | — | Animated nebula background |
| `layers/particles.ts` | — | Particle effect pool |
| `layers/effects.ts` | — | Unlock/completion animations |
| `layers/hit-detection.ts` | — | Ray-casting mouse interaction |

### Tree Browser
| File | Lines | Purpose |
|------|-------|---------|
| `TreeCanvas.tsx` | — | Layout-based tree with computed positions |
| `TreeBrowser.tsx` | — | Jarvis sidebar for node selection |
| `NodeCard.tsx` | — | Visual card (icon, name, progress ring, status) |
| `Breadcrumb.tsx` | — | Navigation breadcrumb |
| `AddNodeButton.tsx` | — | + button for new children |
| `ConnectionLine.tsx` | — | Visual connector lines |
| `UnlockBurst.tsx` | — | Unlock animation effect |

### Detail Panel
| File | Lines | Purpose |
|------|-------|---------|
| `NodeDetailPanel.tsx` | 500+ | Full node details, inline editing, markdown |

### AI Chat
| File | Lines | Purpose |
|------|-------|---------|
| `AIChatPanel.tsx` | 300+ | Chat UI, streams responses, suggested nodes |
| `NodeTreePreview.tsx` | 500+ | Preview + inline edit of suggested nodes |
| `AddNodeModal.tsx` | — | Quick-add node dialog |

### Shared UI
| File | Purpose |
|------|---------|
| `ProgressRing.tsx` | Circular progress indicator |
| `GlassCard.tsx` | Glassmorphism card wrapper |
| `ToastContainer.tsx` | Toast notification system |
| `StatsBar.tsx` | Oura sleep/readiness display |

---

## 4. State Management (Zustand)

**Store**: `store/useTreeStore.ts` (343 lines)

### State shape
```typescript
nodes: ComputedNode[]          // Full tree with computed fields
rootNodes: ComputedNode[]      // Only roots
rawNodes: NotionNode[]         // Raw Notion data
selectedNodeId: string | null  // Currently selected node
detailPanelOpen: boolean
loading: boolean
error: string | null
ouraData: OuraData | null
showArchived: boolean
toasts: Toast[]
addNodeModalOpen: boolean
addNodeParentId: string | null
addNodePreDeps: string[]
addNodeInsertBeforeId: string | null
```

### Key actions
- `rebuildTree()` — Recomputes entire ComputedNode tree from rawNodes
- `createNode(data)` — Optimistic create: temp ID → Notion → swap real ID
- `updateNodeAsync(id, updates)` — PATCH + store update
- `archiveNode(id, reason)` — Archive operation
- Unlock listener callbacks for animation triggers

### Chat history persistence
Chat messages are stored in `localStorage` keyed by `lat-chat-{nodeId}`.

---

## 5. Data Models

### NotionNode (persisted in Notion)
```typescript
id: string                      // Notion page ID
name: string                    // Short name (max 5–6 words)
icon: string | null             // Single emoji
description: string             // What is this goal?
why: string                     // Motivation
criteria: string                // Success checklist (binary)
parentId: string | null         // Parent node (0–1 relationship)
dependsOnIds: string[]          // Prerequisites (cross-branch allowed)
done: boolean                   // Completed flag
archived: 'abgebrochen' | 'pausiert' | 'erledigt' | null
pinned: boolean
badge: 'milestone' | 'boss-level' | null
color: 'purple'|'blue'|'green'|'orange'|'pink'|'teal' | null
due: string | null              // YYYY-MM-DD
timeRange: string | null        // e.g. "2–4 Wochen"
resources: Resource[]           // [{ title, url?, type? }]
createdTime: string             // ISO timestamp
```

### ComputedNode (runtime extensions)
```typescript
nodeType: 'root' | 'container' | 'leaf'
status: 'locked' | 'active' | 'completed' | 'archived'
progress: number                // 0–1
depth: number                   // from root
children: ComputedNode[]
dependents: ComputedNode[]
position: { x: number, y: number }
isCycleMember: boolean
```

### Notion Property Mapping
| Field | Notion Type | Notes |
|-------|-------------|-------|
| name | title | Required |
| icon | page icon | Not a property |
| description, why, criteria, timeRange | rich_text | Plain text |
| resources | rich_text | JSON-encoded array |
| parentId, dependsOnIds | relation | Self-referencing |
| done, pinned | checkbox | |
| badge, color, archived | select | Fixed options |
| due | date | Single date |

### OuraData
```typescript
readiness: number      // 0–100
sleepScore: number     // 0–100
sleepDuration: number  // hours
bedtimeRegularity: 'good' | 'medium' | 'poor'
lastUpdated: string    // ISO timestamp
```

### AI Suggested Node (chat output)
Same fields as NotionNode but:
- All fields optional except `name`
- Includes `tempId` (client UUID) and `_ref` (cross-node dep reference)
- Recursive `children?: SuggestedNode[]`

---

## 6. API Routes (Server Details)

### POST /api/chat (488 lines)
- **Input**: `{ messages[], existingNodes[], parentId?, workingDraft?, webSearchResults? }`
- **Processing**: Sanitizes all inputs; calls Claude via OpenRouter with tools
- **Output**: SSE stream of text deltas, then final JSON `{ suggestedNodes, suggestedParentId, nodeEdits, truncated }`
- **Tools**: `suggest_nodes` (tree), `edit_nodes` (modify existing)
- **System prompt**: ~350 lines defining Jarvis personality and rules
- **Model**: `anthropic/claude-sonnet-4` via OpenRouter

### GET /api/nodes
- Returns all `NotionNode[]` via Notion `queryDatabase` (paginated, 100/page)

### POST /api/nodes
- Creates a Notion page, returns `NotionNode`

### PATCH /api/nodes/[id]
- Accepts partial update; calls `updateNode()` or `archiveNode()`

### GET /api/nodes/[id]/content
- Fetches Notion blocks, converts to Markdown
- Uses `blockToMarkdown()` helper

### POST /api/web-search
- Calls Tavily API; sanitizes results; returns `{ answer?, results[] }`

### GET /api/oura/auth + /api/oura/callback
- Standard OAuth 2.0 flow; stores tokens in `/data/oura-tokens.json`

### GET /api/oura/data
- Reads Oura readiness + sleep data; caches 1h in `/data/oura-cache.json`

---

## 7. External Integrations

| Service | SDK/Client | Auth | Purpose |
|---------|-----------|------|---------|
| Notion | `@notionhq/client` | Integration token | Primary database |
| Anthropic/Claude | `@anthropic-ai/sdk` via OpenRouter | API key | AI coaching |
| Oura Ring | Raw HTTP | OAuth 2.0 | Health metrics |
| Tavily | Raw HTTP | API key | Web search for resources |

---

## 8. Styling

- **Tailwind CSS v4** — utility classes throughout
- **CSS Variables** — `styles/glass.css` defines 100+ variables (surfaces, borders, branch colors, status colors)
- **Glassmorphism** — `rgba(255,255,255,0.12)` + `backdrop-filter: blur(20px)`
- **Dark theme primary** with light theme fallback via `prefers-color-scheme`
- **Inline styles** heavily used for dynamic CSS variable consumption
- **Framer Motion** for layout animations (springs, presence transitions)
- **Google Fonts**: Outfit (main) + Playfair Display (accent)

---

## 9. Real-time Features

- **SSE streaming** from `/api/chat` — text deltas from Claude
- **Optimistic updates** — node creation/editing reflected instantly in store
- **No WebSockets** — HTTP POST → SSE response only
- **No polling** — Oura data fetched once on page load

---

## 10. Authentication

- **No user auth** — single-user personal tool
- **API keys** in `.env.local` (server-only, never exposed to client)
- **Oura OAuth** — user connects via browser redirect; tokens stored in local file
- **Input sanitization** in chat API: `sanitizeText()`, `sanitizeNodeName()`, `sanitizeDraft()`

---

## 11. Core Algorithms (lib/graph.ts — 868 lines)

### buildTree(nodes)
Flat Notion array → ComputedNode tree:
1. Initialize all nodes with defaults
2. Build parent-child relationships from `parentId`
3. Build `dependents` (reverse of `dependsOnIds`)
4. Classify `nodeType` (root/container/leaf)
5. Cycle detection via topoSort
6. Compute statuses
7. Compute progress

### topoSort(nodes)
Kahn's algorithm respecting `dependsOnIds`. Unsorted nodes = cycles.

### computeStatuses(allNodes)
Per-node status logic:
- `archived` if `node.archived != null`
- `archived` if any ancestor is archived
- `completed` if leaf `done === true`, or if all non-archived children completed
- `locked` if any unmet dependency (dep not completed)
- else `active`
- Root nodes always `active`

### computeProgress(node)
- Leaf: `done ? 100 : 0`
- Container: `(completed_children / non_archived_children) * 100`

### computeLayout(children, overrides)
Positions nodes for tree browser view:
1. Group by dependency depth within subtree
2. Layout in columns (x = depth × spacing)
3. Zone-band allocation (group nodes by parent to prevent Y-interleaving)
4. Centering pass (parents center on children, 3 iterations)
5. Overlap resolution (push overlapping nodes apart)
6. Normalize Y to 0 at top

### getCompassSpokes(allNodes)
Per-root: finds active leaf nodes → builds path to boss-level milestone. Returns pinned + unpinned spokes.

### computeRadialMapLayout(allNodes)
Positions all nodes for WebGL map view:
1. Roots at ring 1 of a radial layout
2. Each root gets a sector of 360°
3. Dependencies spread across rings 2+
4. Minimum arc distance enforced per ring
5. Returns `MapNode[]` with x, y, ring, sectorId, sectorColor, sectorCenter

### getBreadcrumbPath(node, allNodes)
Walks `parentId` chain up → returns path from root to node (reversed).

### getNextSteps(allNodes)
Returns active leaf nodes sorted by due date, then branch.

---

## 12. Dependencies

```json
{
  "@anthropic-ai/sdk": "^0.82.0",
  "@notionhq/client": "^5.15.0",
  "framer-motion": "^12.38.0",
  "next": "16.2.1",
  "react": "19.2.4",
  "react-dom": "19.2.4",
  "react-markdown": "^10.1.0",
  "zustand": "^5.0.12",
  "tailwindcss": "^4"
}
```

---

## 13. Environment Variables

```
NOTION_TOKEN=<notion integration token>
NOTION_DATABASE_ID=<database uuid>
OPENROUTER_API_KEY=<openrouter api key>
ANTHROPIC_API_KEY=<optional direct anthropic key>
TAVILY_API_KEY=<tavily search key>
OURA_CLIENT_ID=<oura oauth client id>
OURA_CLIENT_SECRET=<oura oauth client secret>
```

Runtime files (gitignored):
- `/data/oura-tokens.json` — OAuth tokens after user connects Oura
- `/data/oura-cache.json` — Oura response cache (1h TTL)

---

## Complexity Assessment

| Area | Complexity | Notes |
|------|-----------|-------|
| WebGL map canvas | Very High | Layer rendering, hit detection, zoom/pan, animations |
| Graph algorithms | High | Layout math, overlap resolution, radial placement |
| Chat streaming + tool use | Medium-High | Async SSE, Claude tool_use JSON parsing |
| Detail panel inline editing | Medium | Many fields, conditional visibility, form state |
| Hub compass visualization | Medium | SVG/canvas math, spoke layout |
| Tree browser rendering | Medium | Recursive components, computed positions |
| Notion client | Medium | Property mapping, pagination, rich_text ↔ string |
| Oura integration | Low-Medium | OAuth flow + 3 API endpoints |
| Core data models | Low | Simple structs, well-defined |
| Toast / modals / UI helpers | Low | Standard patterns |
