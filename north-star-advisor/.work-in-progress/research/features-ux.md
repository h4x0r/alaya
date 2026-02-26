# AI Agent Memory Library: Features & UX Patterns Research

## Generated
2026-02-26T12:50:00+08:00

## 1. Features Agent Developers Expect

### Memory Taxonomy
Three memory types are the baseline. Alaya's three-store model (episodic, semantic, implicit) covers the critical taxonomy. Consider adding a summary/compression layer (Mem0 claims 80% token reduction).

### Memory Scoping
Mem0's user/session/agent scoping is de facto standard. Alaya needs `user_id` and optional `agent_id` on `NewEpisode` -- high priority.

### Temporal Awareness
Zep's temporal KG with `valid_at`/`invalid_at` dates is state of the art. Consider adding temporal validity semantics to `SemanticNode`.

### Graph Relationships
Alaya's Hebbian graph overlay with spreading activation is already a differentiator. `LinkType` enum covers major relationship types.

### Retrieval Quality
Hybrid BM25 + vector + graph with RRF is competitive. Main gaps: temporal-aware retrieval and configurable reranking strategies.

### Lifecycle Management
Alaya's lifecycle (consolidate, perfume, transform, forget) is more complete than most competitors. Consider adding a `dream()` convenience method chaining all stages.

## 2. API Patterns

### Missing Operations
Add: `get_episode(id)`, `get_node(id)`, `update_episode(id, ...)`, `delete_episode(id)`, `session_history(session_id)`.

### Builder Pattern
Add `AlayaStoreBuilder` for configuration (decay rates, retrieval weights, defaults).

### Async vs Sync
Stay sync-first. Add optional `async` feature flag later using `spawn_blocking`.

### Provider Trait
Expand to cover: fact extraction, impression extraction, summary generation, embedding generation.

## 3. Integration Patterns

### MCP Server (Highest Priority)
MCP is the universal integration standard. Build `alaya-mcp` as standalone binary. Single highest-leverage integration.

### Language Bindings
Prioritize: (1) Rust API, (2) MCP server, (3) Python bindings via PyO3, (4) everything else.

## 4. Testing & Benchmarks

### Standard Benchmarks
- **LoCoMo** (ICLR 2025): ~2,000 QA pairs, used by Mem0, Letta, MemMachine
- **LongMemEval** (ICLR 2025): 500 questions, state-of-the-art achieves only 30-70%

### Additional Tests Needed
- Retrieval quality tests with expected rankings
- Lifecycle round-trip tests
- Concurrent access tests
- Edge cases (Unicode, long content, malformed embeddings)
- Property-based testing with `proptest`

## 5. Priority Recommendations

### Immediate (v0.1)
1. Add `user_id` scoping
2. Convenience constructors (`NewEpisode::quick()`)
3. Add `get_episode`, `session_history`, `delete_episode` methods
4. Doc examples on all public methods
5. Add examples/ directory

### Short-term (v0.2)
6. `AlayaStoreBuilder`
7. `QueryExplanation` debugging mode
8. `dream()` convenience method
9. Build `alaya-mcp` crate
10. Criterion/Divan benchmarks

### Medium-term (v0.3+)
11. LoCoMo benchmark suite
12. `async` feature flag
13. Python bindings
14. Memory compression lifecycle
15. Temporal validity on semantic nodes

## Sources

- Mem0 GitHub, Official Site, TechCrunch
- Zep: Temporal KG Architecture (arXiv), Official Site
- Letta Official Site, Blog (benchmarking, agent memory)
- Supermemory GitHub, Research
- Engram (multiple variants)
- Memvid V2
- MCP (Wikipedia, Anthropic announcement, Linux Foundation)
- LoCoMo Benchmark (Snap Research)
- LongMemEval (ICLR 2025)
- Rust API Guidelines, Cargo Features Documentation
