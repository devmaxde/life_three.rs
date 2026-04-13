# Life Achievement Tree — Rust/Leptos Port
> Reference folder for porting from Next.js/React to Rust/Leptos + Axum

---

## Contents

| File | Purpose |
|------|---------|
| `ANALYSIS.md` | Complete technical analysis of the existing Next.js codebase — routes, components, state, APIs, algorithms, dependencies |
| `ARCHITECTURE.md` | Proposed Rust/Leptos architecture — workspace layout, crate structure, Axum router, Leptos state, WebGL approach, build system |
| `DATA_MODELS.md` | TypeScript → Rust type translation for every interface, enum, and request/response type |
| `ALGORITHMS.md` | Pseudocode + Rust sketches for all graph algorithms from `lib/graph.ts` |
| `PORTING_PLAN.md` | 12-phase implementation plan with detailed task lists, risk register, and dependency reference |
| `SOURCE_REFERENCE.md` | Complete implementation details for all gaps: canvas layers, SSE streaming, Notion CRUD, Oura, schema generation, unlock animations |

---

## Quick Start for the Port

1. Read `ANALYSIS.md` to understand what you're porting
2. Read `ARCHITECTURE.md` to understand where everything goes
3. Read `DATA_MODELS.md` before writing any `struct` definitions
4. Read `ALGORITHMS.md` before touching `graph.rs`
5. Read `SOURCE_REFERENCE.md` for all implementation details (canvas layers, SSE, Notion, etc.)
6. Follow `PORTING_PLAN.md` phase by phase

---

## Key Decisions Made

- **Leptos 0.7 CSR-first** — no SSR initially; simplifies WASM build; add SSR in Phase 12+ if needed
- **Axum 0.7** — async, ergonomic, well-maintained; integrates with `leptos_axum`
- **cargo-leptos** — unified build tool for WASM + native in one workspace
- **Keep Notion as database** — no migration; Rust Notion client via `reqwest`
- **Keep Tailwind CSS** — just update `content` to scan `.rs` files; CSS vars work identically
- **WebGL via raw `web-sys`** — full control, mirrors existing layer architecture exactly
- **No user authentication** — app remains single-user; all API keys in `.env`
- **Two-pass tree build** — solve Rust borrow conflicts in graph construction by computing flat then assembling recursive

---

## Stack Reference

```
life-tree-rust/
├── Cargo.toml              (workspace)
├── crates/
│   ├── core/               (life-tree-core: data models + graph algorithms)
│   ├── backend/            (Axum HTTP server)
│   └── frontend/           (Leptos WASM app)
├── assets/
│   ├── app.css             (Tailwind entry)
│   └── glass.css           (design system variables — copied verbatim)
└── .env                    (same vars as .env.local)
```

## External Services (unchanged)
- **Notion** — primary database
- **Anthropic/Claude** via OpenRouter — AI coaching
- **Oura Ring** — health data (OAuth 2.0)
- **Tavily** — web search for resources
