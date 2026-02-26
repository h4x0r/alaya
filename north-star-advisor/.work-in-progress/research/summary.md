# Research Summary

## Generated
2026-02-26T12:50:00+08:00

## Project Context
- **Name:** Alaya
- **Type:** Embeddable Rust memory library for AI agents
- **Users:** Agent developers building privacy-first, relationship-heavy agents
- **Preferred Stack:** Rust, SQLite (embedded), zero external dependencies

---

## Technology Stack

### Recommended / Validated
| Layer | Recommendation | Rationale |
|-------|---------------|-----------|
| Language | Rust (validated) | Zero GC, FFI-embeddable, single binary |
| Database | rusqlite 0.38 with `bundled`, `cache`, `vtab` | Sync-first, minimal deps, upgrade from 0.32 needed |
| Full-text search | FTS5 external content tables | Already correct pattern; add porter stemmer |
| Vector search | Tiered: brute-force (default) -> sqlite-vec (feature) -> HNSW (trait) | Zero-dep default with escape hatch |
| Error handling | thiserror 2.x (validated) | Add `#[non_exhaustive]` to AlayaError |
| Benchmarking | Divan | Attribute-based API, allocation profiling |
| Serialization | serde + serde_json (validated) | Already minimal |
| FFI | cbindgen (Tier 1) + UniFFI (Tier 2) + PyO3 (Tier 3) | Separate crates, not in core lib |

### Key Libraries
- `rusqlite` 0.38: SQLite bindings (upgrade needed from 0.32)
- `sqlite-vec`: Optional SIMD vector search via feature flag
- `divan`: Benchmarking framework
- `cbindgen`: C header generation for FFI
- `ort`: Optional ONNX embedding backend
- `fastembed-rs`: Optional turnkey embedding solution
- `cargo-semver-checks`: CI semver validation

### Best Practices
- Builder pattern for `AlayaStore` configuration (`AlayaConfig::builder()`)
- `#[non_exhaustive]` on all public enums and structs
- `pub(crate)` for internal modules
- Integration tests in `tests/` directory testing the public API
- `cargo-semver-checks` in CI
- Feature flags only for genuinely optional heavy deps (4-6 max)
- Stay sync-first; async via `spawn_blocking` behind feature flag later

---

## Features & UX

### Expected Features
Users of AI agent memory libraries typically expect:
1. **Three-tier memory taxonomy** (episodic, semantic, behavioral) -- Alaya covers this
2. **User/session/agent scoping** -- Alaya needs `user_id` and `agent_id` additions
3. **Temporal awareness** -- `valid_at`/`invalid_at` on semantic nodes
4. **Graph relationships** -- Alaya's Hebbian overlay is a differentiator
5. **Hybrid retrieval** (BM25 + vector + graph) -- Alaya has this via RRF
6. **MCP server integration** -- Highest-priority integration to build
7. **Standard CRUD operations** -- Missing: `get_episode()`, `delete_episode()`, `session_history()`
8. **Benchmarkable** -- LoCoMo and LongMemEval are industry-standard benchmarks

### UX Patterns
- **Under 5 minutes to working example:** Alaya's `open()` + `store_episode()` + `query()` achieves this
- **Builder constructors:** Add `NewEpisode::quick()`, `Query::with_embedding()`
- **Debug mode:** `QueryExplanation` returning per-stage scores (BM25, vector, graph, RRF)
- **Convenience lifecycle:** `dream()` method chaining consolidate + perfume + transform + forget
- **Doc examples:** Every public method needs compilable doctests

### Accessibility Requirements
- Clear error messages with actionable context
- Zero-config sensible defaults with builder for advanced users
- Examples directory with graduated complexity (basic -> lifecycle -> custom provider -> graph)
- `#[deny(missing_docs)]` enforced in lib.rs

---

## Architecture

### Recommended Pattern
**Three-store + Hebbian graph + hybrid retrieval + cognitive lifecycle** (validated as novel in the field)

The architecture is validated as the most complete single-system approach. No competitor combines all four elements with zero external dependencies.

### Core Architecture Layers
1. **Storage layer:** SQLite with WAL mode, FTS5, embeddings table
2. **Graph layer:** Adjacency list with recursive CTEs, LTP/LTD dynamics
3. **Retrieval layer:** BM25 + vector + graph activation + RRF fusion
4. **Lifecycle layer:** Consolidation, perfuming, transformation, forgetting
5. **Provider layer:** Traits for LLM, embeddings, and extension points

### Data Flow
```
Input (episodes) -> Storage (SQLite)
                 -> FTS5 index
                 -> Embeddings (optional)
                 -> Graph links (co-occurrence)

Query -> BM25 (FTS5) ─┐
      -> Vector (cosine) ─┤── RRF fusion -> Reranking -> Spreading activation -> RIF -> Results
      -> Graph (neighbors) ─┘

Lifecycle -> Consolidation (episodes -> semantic nodes)
          -> Perfuming (impressions -> preferences)
          -> Transformation (dedup, resolve contradictions)
          -> Forgetting (Bjork dual-strength decay)
```

### Key Implementation Details
- **Hebbian LTP/LTD:** Add LTD (currently missing) -- multiplicative decay on disuse
- **Bjork forgetting:** RS decays inversely proportional to SS; reference FSRS for validated formulas
- **Spreading activation:** Recursive CTEs with depth limits (2-3 hops), decay factor (0.5-0.7)
- **RRF:** k=60 standard, merge 3 signal types
- **Vector search:** Brute-force to ~10K, sqlite-vec to ~50K, HNSW beyond

### Scalability Considerations
- Vector brute-force viable to ~10K vectors; design swappable index for scale
- Graph traversal viable to ~100K edges with depth limits
- SQLite WAL handles concurrent reads; single writer sufficient for embedded use
- FTS5 requires periodic `automerge` and `optimize` maintenance
- Consolidation should be event-driven (batch threshold), not time-based

---

## Pitfalls to Avoid

### Common Mistakes
1. **Deferred transaction upgrade trap** -> Prevention: ALWAYS use `BEGIN IMMEDIATE` for writes
2. **FTS5 query injection** -> Prevention: Sanitize MATCH input, wrap in double quotes
3. **Memory resurrection after deletion** -> Prevention: Tombstone table, hard deletes, cascade across all stores
4. **Over-aggressive forgetting** -> Prevention: Differential decay rates by importance, cold storage before deletion
5. **Context flooding ("Dumb RAG")** -> Prevention: Relevance-gated retrieval, configurable limits

### Security Concerns
- **Memory poisoning (OWASP ASI06):** Provide content validation hooks, quarantine API, content-hash integrity
- **FTS5 injection:** Safe `search()` API that handles escaping internally
- **Data leakage:** Mandatory `context_id`/`session_id` scoping on all queries
- **Embedding poisoning:** Metadata tracking, re-generation capability
- **PII exposure:** PII scrubbing hooks, field-level encryption option, document all persistence locations
- **GDPR compliance:** Crypto-shredding, surrogate keys, `forget(entity_id)` API, VACUUM after deletion

### Performance Gotchas
- **WAL unbounded growth:** Periodic `PRAGMA wal_checkpoint(PASSIVE)`, set `journal_size_limit`
- **Vector search degradation:** Design swappable index, document O(N) curve
- **Graph explosion:** Max edges per node, weight-threshold pruning, depth limits
- **FTS5 bloat:** Automerge config, incremental merges during idle, UNINDEXED for non-searchable columns
- **Compile time:** Feature-gate heavy deps, minimize monomorphization, track with `cargo build --timings`

---

## Generation Guidance

These findings should inform:
- **Phase 1 (BRAND_GUIDELINES):** Position around "memory as process" and neuroscience-grounded design. The research narrative (Yogacara + CLS + Bjork) is unique in the field.
- **Phase 2 (NORTHSTAR):** North Star metric should combine retrieval quality (LoCoMo/LongMemEval scores) with developer adoption signals.
- **Phase 3 (COMPETITIVE_LANDSCAPE):** Key competitors are Mem0 (cloud, LLM-dependent), Zep/Graphiti (temporal KG), Letta (LLM OS model), Engram (simple local). Alaya's unique position: complete cognitive lifecycle + zero dependencies + implicit preference emergence.
- **Phase 4 (AXIOMS):** Core axioms should include: memory is process not database, forgetting is feature, preferences emerge, graceful degradation, zero ops.
- **Phase 6 (ARCHITECTURE_BLUEPRINT):** Use tiered vector search strategy, recursive CTE spreading activation, `BEGIN IMMEDIATE` for all writes, `EmbeddingProvider` trait, `AlayaConfig` builder. Address all critical SQLite pitfalls.
- **Phase 7 (AGENT_PROMPTS):** Not applicable (Alaya is a library, not an agent). Reframe as "consumer integration patterns" -- MCP server, direct Rust API, FFI.
- **Phase 8 (SECURITY_ARCHITECTURE):** Address memory poisoning (OWASP ASI06), FTS5 injection, GDPR compliance with crypto-shredding, PII scrubbing hooks. Memory resurrection is a critical unique threat.
- **Phase 10 (OPS_RUNBOOK):** WAL checkpoint management, FTS5 maintenance, graph pruning, consolidation scheduling.
- **Phase 12 (ROADMAP):** v0.1 (core API polish + missing operations), v0.2 (MCP server + builder config + benchmarks), v0.3 (LoCoMo benchmarks + async + Python bindings).
