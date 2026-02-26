# Alaya Architecture Decision Records

> Architecture Decision Records for Alaya v0.1 -- Embeddable Rust Memory Engine

**Document type:** Architecture Decision Records (ADR)
**Version:** 0.1.0
**Last updated:** 2026-02-27
**Status:** Living document, tracks implementation decisions

---

## Purpose

This document captures the significant architecture decisions made during the design and implementation of Alaya, an embeddable Rust memory library with cognitive lifecycle processes and implicit preference emergence for privacy-first AI agents.

Each ADR follows a consistent format: the context that motivated the decision, the decision itself, its consequences (both positive and negative), and the alternatives that were considered and rejected. Decisions are interconnected -- ADR-009 (Zero Network Calls) constrains ADR-004 (Trait-Based Extension), which in turn enables ADR-005 (Bjork Forgetting) to work without LLM involvement. Reading them in sequence reveals the architectural logic.

**Governing axioms** (from Phase 4 North Star Extract):

1. Privacy > Features
2. Process > Storage
3. Correctness > Speed
4. Simplicity > Completeness
5. Honesty > Marketing

**Conflict resolution hierarchy:** Safety > Privacy > Correctness > Simplicity > Performance > Features

---

## Decision Index

| ADR | Title | Status | Primary Axiom |
|-----|-------|--------|---------------|
| [001](#adr-001-sqlite-as-sole-storage-engine) | SQLite as Sole Storage Engine | Accepted | Simplicity > Completeness |
| [002](#adr-002-three-store-memory-architecture) | Three-Store Memory Architecture | Accepted | Process > Storage |
| [003](#adr-003-hebbian-graph-overlay) | Hebbian Graph Overlay | Accepted | Process > Storage |
| [004](#adr-004-trait-based-extension-model) | Trait-Based Extension Model | Accepted | Privacy > Features |
| [005](#adr-005-bjork-dual-strength-forgetting-model) | Bjork Dual-Strength Forgetting Model | Accepted | Correctness > Speed |
| [006](#adr-006-rrf-as-fusion-strategy) | RRF as Fusion Strategy | Accepted | Simplicity > Completeness |
| [007](#adr-007-vasana-preference-emergence) | Vasana Preference Emergence | Accepted | Process > Storage |
| [008](#adr-008-sync-first-api) | Sync-First API | Accepted | Simplicity > Completeness |
| [009](#adr-009-zero-network-calls) | Zero Network Calls | Accepted | Privacy > Features |
| [010](#adr-010-fts5-for-full-text-search) | FTS5 for Full-Text Search | Accepted | Simplicity > Completeness |

---

## ADR-001: SQLite as Sole Storage Engine

**Status:** Accepted
**Date:** 2026-02-27
**Deciders:** Project maintainer

**Context:**

Alaya requires persistent storage for episodes, semantic nodes, impressions, preferences, embeddings, graph links, node strengths, and a full-text search index. The storage engine must support:

- Full-text search (BM25 scoring) for the retrieval pipeline
- BLOB storage for embedding vectors (f32 arrays, little-endian)
- Transactional writes across multiple tables in a single atomic operation
- Zero external dependencies (no separate database process, no network calls)
- Single-file deployment for the consumer (one `.db` file contains all state)
- WAL mode for concurrent read/write access

The choice of storage engine is the most consequential infrastructure decision because it constrains scale ceiling, deployment model, query capability, and dependency graph.

**Decision:**

Use embedded SQLite via `rusqlite 0.32` with the `bundled` feature flag (compiles SQLite from C source into the Rust binary). All persistent state -- 7 tables, 1 FTS5 virtual table, 3 triggers, 9 indexes -- lives in a single SQLite file. WAL mode provides concurrent readers with a single writer. FTS5 provides BM25 scoring. Embeddings are stored as f32 BLOBs with brute-force cosine similarity search at v0.1, with a trait-based escape hatch to `sqlite-vec` or external vector backends at v0.2+.

SQLite pragmas are set at connection open:

```sql
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;
PRAGMA synchronous = NORMAL;
PRAGMA wal_autocheckpoint = 1000;      -- planned v0.1
PRAGMA journal_size_limit = 67108864;  -- planned v0.1
```

All write transactions use `BEGIN IMMEDIATE` to prevent WAL deadlocks (see Security Architecture, threat: transaction-deadlock).

**Consequences:**

- **Single-file invariant:** All state in one `.db` file. Backup is `cp`. Migration is schema versioning. No orchestration, no connection strings, no credentials.
- **Zero-config deployment:** Consumer calls `AlayaStore::open("path/to/memory.db")` and everything works. No database setup, no migration scripts for the consumer.
- **FTS5 built-in:** BM25 scoring without adding tantivy (~30K lines) or a custom inverted index. External content tables keep the FTS index synchronized via triggers.
- **Transactional integrity:** Multi-table writes (e.g., store episode + update FTS + create embedding + update strengths) are atomic. No partial writes, no eventual consistency.
- **WAL concurrency:** Multiple readers concurrent with a single writer. Sufficient for single-process embedded library use.
- **Scale ceiling at ~50K embeddings:** Brute-force cosine search is O(n). At 50K 384-dimensional embeddings, search takes ~10ms. Beyond this, the consumer must provide an `EmbeddingProvider` implementation or enable the `vec-sqlite` feature flag (v0.2). This is an honest, documented limitation.
- **Rust-to-C FFI boundary:** `rusqlite` wraps SQLite's C library via FFI. This is the only `unsafe` code in the dependency chain and is the most-used Rust database crate.
- **No horizontal scaling:** Single-writer WAL mode means one process writes at a time. This is acceptable for an embedded library but precludes multi-process server architectures.

**Alternatives Considered:**

| Alternative | Pros | Cons | Why Rejected |
|-------------|------|------|--------------|
| RocksDB (via rust-rocksdb) | LSM-tree optimized for write-heavy workloads; used by CockroachDB, TiKV | No built-in FTS, no SQL query language, key-value only (would need custom indexing), larger binary, C++ FFI complexity | Requires reimplementing FTS and relational queries; write optimization unnecessary for embedded library workload |
| Sled | Pure Rust (no FFI), embedded key-value store | No FTS, no SQL, no relational queries, known stability issues pre-1.0, API instability | Insufficient query capabilities; stability concerns for a memory system where data loss is unacceptable |
| LMDB (via lmdb-rs) | Memory-mapped, very fast reads, ACID transactions | No FTS, no SQL, key-value only, memory-mapped files complicate deployment on some platforms | Same query limitations as RocksDB; memory mapping adds platform-specific concerns |
| Separate databases per store | Clean isolation between episodic/semantic/implicit | Multiple files to manage, no cross-store transactions, complex backup, breaks single-file invariant | Violates Simplicity > Completeness axiom; single-file invariant is a core value proposition |

**Cross-references:**

- Architecture Blueprint: Section 3 (Storage Layer), Section 5 (SQLite Configuration)
- Security Architecture: Threats T7 (WAL corruption), T8 (transaction deadlock)
- North Star Extract: Axiom "Simplicity > Completeness", constraint "All persistent state in single SQLite file"
- Phase 4 always-list: "BEGIN IMMEDIATE for all write transactions"

---

## ADR-002: Three-Store Memory Architecture

**Status:** Accepted
**Date:** 2026-02-27
**Deciders:** Project maintainer

**Context:**

Human memory research distinguishes between episodic memory (events and experiences), semantic memory (facts and knowledge), and implicit memory (habits, preferences, and skills). Agent memory systems must model at least these three categories to support meaningful conversational continuity. The question is whether to store them in a single undifferentiated table, in separate logical stores sharing a database, or in entirely separate databases.

Alaya must support:

- Raw conversation episodes (who said what, when, in what session)
- Extracted knowledge nodes (facts, entities, relationships derived from episodes)
- Implicit observations and crystallized preferences (behavioral patterns the agent notices)
- Cross-store relationships (an episode that generated a knowledge node that influenced a preference)
- Independent lifecycle processes per store type (consolidation operates on episodes to produce semantic nodes; perfuming operates on impressions to produce preferences)

**Decision:**

Three logically separate stores -- episodic, semantic, and implicit -- sharing a single SQLite file. Each store has its own tables and CRUD operations but participates in a shared graph overlay (ADR-003) and a unified retrieval pipeline (ADR-006).

| Store | Tables | Primary Operations | Lifecycle Role |
|-------|--------|--------------------|----------------|
| Episodic | `episodes`, `episodes_fts` | store, get, list, delete, session_history | Input to consolidation |
| Semantic | `semantic_nodes` | CRUD via consolidation | Output of consolidation, input to transformation |
| Implicit | `impressions`, `preferences` | Impressions via perfuming, preferences crystallized | Output of perfuming |

Shared tables span stores:

- `embeddings` -- polymorphic, keyed by `(node_type, node_id)`
- `links` -- Hebbian graph connecting any `NodeRef` to any other `NodeRef`
- `node_strengths` -- Bjork dual-strength tracking per `(node_type, node_id)`

The `NodeRef` enum (`Episode | Semantic | Preference`) provides type-safe cross-store addressing.

**Consequences:**

- **Clear separation of concerns:** Each store has distinct CRUD semantics, distinct lifecycle roles, and distinct data shapes. Episodic data is append-heavy and session-scoped. Semantic data is consolidation-derived and corroboration-tracked. Implicit data accumulates observations and crystallizes patterns.
- **Independent lifecycle processes:** Consolidation reads episodes and writes semantic nodes. Perfuming reads impressions and writes preferences. Transformation prunes across all stores. Forgetting decays strengths across all stores. Each process is composable and independently testable.
- **Cross-store queries via graph overlay:** The shared `links` table enables spreading activation to traverse from an episode to a related semantic node to a preference. The unified retrieval pipeline (ADR-006) fuses results across all three stores transparently.
- **Cognitive research alignment:** The three-store model maps to established memory taxonomy (Tulving, 1972 for episodic/semantic; Reber, 1967 for implicit). This is not cosmetic -- it determines how consolidation, forgetting, and preference emergence operate.
- **Schema complexity:** Seven core tables plus one FTS5 virtual table, three triggers, and nine indexes. More complex than a single-table design but justified by the distinct lifecycle requirements per store type.
- **Cross-store consistency:** Deleting an episode must cascade to its embeddings, links, strength records, and any derived semantic nodes. This requires careful transaction management (BEGIN IMMEDIATE) and tombstone tracking (planned v0.1).

**Alternatives Considered:**

| Alternative | Pros | Cons | Why Rejected |
|-------------|------|------|--------------|
| Single unified store | Simpler schema, no cross-store consistency concerns, fewer tables | All memory types share one schema; lifecycle processes cannot target specific types; no clear data model boundaries; "content" column becomes catch-all | Violates Process > Storage axiom; lifecycle processes need distinct input/output types |
| Separate SQLite databases | Strongest isolation; independent backup/migration per store | Breaks single-file invariant; no cross-store transactions; complex graph overlay spanning files; consumer manages three files | Violates Simplicity > Completeness; the single-file deployment model is a core differentiator |
| Two stores (episodic + everything else) | Simpler than three; episodes clearly distinct | Conflates semantic knowledge with implicit preferences; consolidation and perfuming outputs share a table; type system becomes ambiguous | Insufficient separation for independent lifecycle processes; preferences and knowledge have fundamentally different schemas and lifecycles |

**Cross-references:**

- Architecture Blueprint: Section 2 (Component Topology), Section 4 (Data Model)
- North Star Extract: Axiom "Process > Storage", pattern "Cognitive Lifecycle Pipeline"
- Competitive Landscape: Differentiator "Complete cognitive lifecycle + zero external dependencies"
- ADR-001: Single SQLite file housing all three stores
- ADR-003: Graph overlay connecting across stores

---

## ADR-003: Hebbian Graph Overlay

**Status:** Accepted
**Date:** 2026-02-27
**Deciders:** Project maintainer

**Context:**

Memories do not exist in isolation. An episode about a user's coffee preference relates to a semantic node about their dietary habits and to an implicit preference for morning routines. These associative relationships are critical for retrieval quality -- a query about "morning habits" should surface the coffee preference even if the text does not match.

The question is how to model these inter-memory relationships:

1. Static knowledge graph (entities and relationships extracted by LLM, as in Zep/Graphiti)
2. Dynamic graph that reshapes through use (Hebbian learning)
3. No graph (rely solely on text and vector similarity)

Alaya's axiom "Process > Storage" favors a graph that changes through interaction over a static graph that requires LLM extraction.

**Decision:**

Adjacency list stored in a `links` table with Hebbian long-term potentiation (LTP) and long-term depression (LTD) dynamics. Links are typed (`LinkType` enum: `CoOccurrence`, `Consolidation`, `Semantic`, `Temporal`, `Preference`), weighted (f32), and directional.

**Hebbian dynamics:**

- **LTP (strengthening):** When two nodes are co-retrieved in the same query, their link weight increases: `w += 0.1 * (1 - w)`. This is bounded and self-limiting -- strong links grow slowly, weak links grow fast.
- **LTD (weakening):** During `transform()`, link weights decay toward zero: `w *= decay_factor`. Links below a prune threshold (0.02) are deleted.
- **Spreading activation:** During retrieval, top BM25 and vector results seed an activation map. Activation spreads along weighted links using Collins & Loftus (1975) semantics, implemented via recursive CTE in SQLite. Activation decays by link weight at each hop (max 3 hops by default).

**Link creation:**

- Episodes in the same session get `Temporal` links
- Consolidation creates `Consolidation` links (episode -> semantic node)
- Perfuming creates `Preference` links (impression -> preference)
- Co-retrieval creates or strengthens `CoOccurrence` links (Hebbian LTP)

**Consequences:**

- **Self-organizing relationships:** The graph reflects actual usage patterns, not LLM-extracted ontology. Frequently co-retrieved memories develop strong links. Rarely accessed paths decay and prune. The graph is a living structure.
- **Retrieval improvement over time:** As the graph accumulates co-retrieval patterns, spreading activation surfaces increasingly relevant results that text/vector search alone would miss. This is the "memory is a process" belief made concrete.
- **No LLM required for graph construction:** Unlike Zep/Graphiti which require LLM entity extraction to build the graph, Alaya's graph emerges from usage patterns. With `NoOpProvider`, the graph still forms through temporal co-occurrence and co-retrieval.
- **Graph maintenance overhead:** LTD and pruning must run periodically via `transform()`. Without maintenance, the graph accumulates stale links that degrade spreading activation quality.
- **Recursive CTE performance:** Spreading activation via recursive CTE is efficient for graphs under 100K links but may need optimization (depth limits, activation thresholds) at scale.
- **Novel approach with limited precedent:** Hebbian dynamics in agent memory systems are novel. SYNAPSE uses spreading activation but with lateral inhibition, not LTP/LTD. This means less established best practices for tuning parameters.

**Alternatives Considered:**

| Alternative | Pros | Cons | Why Rejected |
|-------------|------|------|--------------|
| Static knowledge graph (Neo4j-style) | Well-understood model; rich query language (Cypher); established tooling | Requires LLM for entity extraction (violates zero-dep); graph does not change through use; requires external database or complex embedded graph engine | Violates Privacy > Features (LLM dependency) and Process > Storage (static, not dynamic) |
| No graph (vector + text only) | Simplest implementation; no graph maintenance; fewer tables | Misses associative relationships entirely; no spreading activation; retrieval limited to direct text/vector match; loses key differentiator | Eliminates a core differentiator; retrieval quality ceiling significantly lower without associative paths |
| Embedding-space proximity as implicit graph | No explicit link storage; neighbors in vector space serve as implicit links | Cannot model typed relationships; no temporal links; no strengthening through use; limited to embedding model's semantic space | Does not support Hebbian dynamics; relationship types (temporal, consolidation, preference) are lost; no spreading activation |

**Cross-references:**

- Architecture Blueprint: Section 2.6 (Graph Overlay), Section 6.2 (Spreading Activation)
- Competitive Landscape: Differentiator "Hebbian graph with LTP/LTD -- only dynamic, use-shaped graph without LLM involvement"
- Competitive Landscape: SYNAPSE (closest architecture, spreading activation + lateral inhibition)
- North Star Extract: Axiom "Process > Storage", belief "Memory is a process, not a database"
- ADR-002: Graph spans all three stores via `NodeRef`
- ADR-006: Graph is third retrieval signal in RRF fusion

---

## ADR-004: Trait-Based Extension Model

**Status:** Accepted
**Date:** 2026-02-27
**Deciders:** Project maintainer

**Context:**

Alaya needs LLM capabilities for two operations: consolidation (extracting knowledge from episodes) and embedding generation (converting text to vectors). However, the core axiom "Privacy > Features" prohibits network calls in the core crate (ADR-009), and "Simplicity > Completeness" prohibits bundling LLM inference in the library (which would add hundreds of megabytes of dependencies).

The challenge: how does a library that processes text gain access to LLM capabilities without depending on any specific LLM, making any network calls, or requiring the consumer to use an LLM at all?

**Decision:**

Define Rust traits at the library boundary that the consumer implements. Provide a `NoOpProvider` default that returns empty results, enabling full graceful degradation.

```rust
pub trait ConsolidationProvider {
    fn extract_knowledge(&self, episodes: &[Episode]) -> Result<Vec<ExtractedNode>>;
    fn extract_impressions(&self, episodes: &[Episode]) -> Result<Vec<ExtractedImpression>>;
    fn detect_contradiction(&self, a: &str, b: &str) -> Result<Option<Contradiction>>;
}

pub trait EmbeddingProvider {
    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn dimension(&self) -> usize;
}
```

`NoOpProvider` implements both traits, returning empty vectors and `None` for contradictions. All Alaya methods accept `&dyn ConsolidationProvider` or work with stored embeddings. The consumer decides whether to implement these traits with a local model (e.g., via `ort` or `fastembed-rs`), a remote API (e.g., OpenAI), or not at all.

Feature flags (`embed-ort`, `embed-fastembed`) planned for v0.2 provide turnkey implementations that the consumer can opt into without writing trait implementations.

**Consequences:**

- **Zero-dependency core:** The core crate compiles with only `rusqlite`, `serde`, `serde_json`, and `thiserror`. No HTTP client, no model weights, no inference runtime. `cargo add alaya` adds nothing the consumer does not expect.
- **Graceful degradation chain:** Without `EmbeddingProvider`: BM25 + graph retrieval. Without `ConsolidationProvider`: episodes accumulate but no semantic extraction. Without both: Alaya is a structured episode store with FTS5 search and session history. Every level is useful.
- **Agent controls the LLM connection:** The consumer decides which model, which endpoint, which rate limits, which error handling. Alaya never phones home, never manages API keys, never retries network calls.
- **Testability:** `NoOpProvider` enables all unit and integration tests to run without any LLM. Tests are fast, deterministic, and reproducible.
- **Consumer implementation burden:** The consumer must write a trait impl to get full functionality. This is intentional friction -- it forces the consumer to make an explicit decision about their LLM strategy rather than defaulting to a cloud API.
- **Provider output trust boundary:** The library trusts whatever the provider returns. This creates a security surface (see Security Architecture, threat: provider-output-injection). Input validation on provider outputs is planned for v0.1.

**Alternatives Considered:**

| Alternative | Pros | Cons | Why Rejected |
|-------------|------|------|--------------|
| Built-in LLM client (e.g., reqwest + OpenAI API) | Zero consumer effort for LLM features; "just works" | Adds network dependency to core crate (violates Privacy > Features); ties library to specific API; requires API key management; breaks zero-network-calls guarantee | Fundamentally incompatible with privacy-first positioning and zero-dependency constraint |
| Plugin system (dynamic loading) | Runtime extensibility; consumer ships plugins as separate binaries | Complex FFI boundary; platform-specific dynamic loading; hard to type-check at compile time; security surface (loading untrusted code) | Over-engineered for a library crate; Rust traits provide compile-time safety that plugins cannot |
| Feature-flag-gated implementations only | Consumer enables a flag, gets a working provider | Still adds dependencies to the crate (even if optional); limited to implementations the library author anticipates; no custom provider path | Too restrictive; planned for v0.2 as convenience on top of traits, not as replacement |
| No LLM integration (text-only library) | Simplest possible design; no extension surface | Consolidation impossible without extraction; embedding search impossible; preference emergence requires observation extraction; limits to FTS5-only retrieval | Unacceptably low retrieval quality ceiling; eliminates cognitive lifecycle |

**Cross-references:**

- Architecture Blueprint: Section 2.8 (Provider Traits), Section 8 (Extension Model)
- North Star Extract: Axiom "Privacy > Features", pattern "Trait Extension Pattern", pattern "Graceful Degradation Chain"
- Security Architecture: Threat T9 (provider-output-injection)
- ADR-009: Zero network calls requires trait-based extension
- ADR-005: Forgetting works without provider (purely mathematical)
- ADR-007: Perfuming uses ConsolidationProvider for impression extraction

---

## ADR-005: Bjork Dual-Strength Forgetting Model

**Status:** Accepted
**Date:** 2026-02-27
**Deciders:** Project maintainer

**Context:**

Memory systems that never forget face a fundamental problem: as memories accumulate, retrieval quality degrades. Context windows flood with marginally relevant results. Old, contradicted facts persist alongside current ones. The system drowns in its own history.

Forgetting is not a bug -- it is a feature that improves retrieval quality by reducing noise. The question is what model of forgetting to use:

1. Simple exponential decay (Mem0's approach): decay = e^(-t/half_life)
2. Time-based TTL: delete memories older than N days
3. Research-grounded model: Bjork & Bjork (1992) dual-strength theory
4. No forgetting: accumulate indefinitely

The axiom "Correctness > Speed" demands a research-grounded approach over a convenient heuristic.

**Decision:**

Implement the Bjork dual-strength model with Retrieval-Induced Forgetting (RIF). Every memory node has two independent strength values:

- **Storage Strength (SS):** How deeply encoded the memory is. Increases with each encounter (consolidation, corroboration). Never decays. Monotonically increasing. Range: [0.0, 1.0].
- **Retrieval Strength (RS):** How accessible the memory is right now. Resets to 1.0 on access. Decays over time via `RS *= 0.95` per forgetting cycle. Memories with high SS but low RS are "on the tip of the tongue" -- deeply encoded but temporarily inaccessible.

**Key dynamics:**

- **Access resets RS:** `RS = 1.0` on retrieval. Frequently accessed memories stay retrievable.
- **SS grows on access:** `SS += 0.05 * (1 - SS)` on retrieval. Self-limiting growth.
- **Forgetting decays RS only:** `forget()` multiplies all RS by 0.95. High-SS memories recover quickly when accessed; low-SS memories fade permanently.
- **Archival threshold:** When both `SS < 0.1` and `RS < 0.05`, the memory is archived (excluded from retrieval but not deleted). This is principled "forgetting" -- the memory still exists but is not accessible.
- **RIF (Retrieval-Induced Forgetting):** When memory A is retrieved, competitors of A (memories with similar content but not retrieved) have their RS reduced. This models the psychological finding that retrieving one memory suppresses access to related, non-retrieved memories.

The `forget()` method returns a `ForgettingReport` documenting what was decayed and archived, enabling audit and debugging.

**Consequences:**

- **Research-grounded behavior:** The dual-strength model is well-established in memory research (Bjork & Bjork, 1992; Storm, 2011). It produces more natural forgetting curves than exponential decay: frequently accessed memories persist; deeply encoded but unaccessed memories become temporarily inaccessible rather than permanently deleted.
- **Spaced-repetition-like dynamics:** Memories that are accessed on a spread-out schedule accumulate high SS (deeply encoded) while maintaining high RS (currently accessible). This mirrors the spacing effect in human memory.
- **Competitive differentiator:** No other shipping agent memory system uses dual-strength forgetting. Mem0 uses simple exponential decay. Supermemory uses ad-hoc forgetting. Letta uses crude eviction. This is a genuine novel contribution.
- **Implementation complexity:** Two strength values per node, with different update rules, is more complex than a single decay value. The `node_strengths` table adds a row per memory node. The `forget()` implementation must handle RS decay, archival detection, and RIF application.
- **Tuning difficulty:** The decay factor (0.95), SS growth rate (0.05), and archival thresholds (SS < 0.1, RS < 0.05) are research-informed but not empirically validated for agent memory workloads. These will need tuning based on LoCoMo benchmark results.
- **Memory resurrection risk:** If a "forgotten" (archived) memory is referenced by a consolidation provider that does not check tombstones, it could re-emerge. The Security Architecture identifies this as threat T3 (memory-resurrection) with tombstone-based mitigation planned for v0.1.

**Alternatives Considered:**

| Alternative | Pros | Cons | Why Rejected |
|-------------|------|------|--------------|
| Exponential decay (Mem0-style) | Simple to implement; single decay value; well-understood | Does not distinguish encoding depth from current accessibility; all memories decay at same rate regardless of importance; no theoretical basis for choosing half-life | Violates Correctness > Speed; fails to model the "tip of the tongue" phenomenon; produces flat, uniform forgetting |
| Time-based TTL | Simplest possible; deterministic; easy to reason about | No relationship between importance and persistence; valuable old memories deleted alongside trivial ones; requires consumer to set TTL per type | Too crude; ignores access patterns entirely; produces unnatural cliff-edge forgetting |
| No forgetting | Simplest; no data loss risk; consumer can implement own pruning | Context flooding; retrieval quality degrades with scale; contradicted facts persist; violates "Forgetting is a feature" belief | Fundamentally incompatible with cognitive lifecycle approach; eliminates a core differentiator |
| LRU eviction (Letta/MemGPT style) | Simple to implement; keeps most recently used | No concept of importance; a deeply relevant memory accessed once long ago is evicted while a trivial recent memory persists | Does not model memory dynamics; purely mechanical |

**Cross-references:**

- Architecture Blueprint: Section 7.4 (Forgetting), Section 4 (node_strengths table)
- Security Architecture: Threat T3 (memory-resurrection), mitigation via tombstone mechanism
- Competitive Landscape: Differentiator "Bjork dual-strength forgetting with RIF -- only dual-strength model in any shipping system"
- North Star Extract: Belief "Forgetting is a feature", axiom "Correctness > Speed"
- ADR-002: Strengths tracked per node across all three stores
- ADR-003: RIF suppresses competing graph neighbors

---

## ADR-006: RRF as Fusion Strategy

**Status:** Accepted
**Date:** 2026-02-27
**Deciders:** Project maintainer

**Context:**

Alaya's retrieval pipeline produces three independent signals:

1. **BM25** (from FTS5): text-match scores on the `episodes_fts` table
2. **Vector** (from embeddings): cosine similarity scores from embedding search
3. **Graph** (from spreading activation): activation levels from Hebbian graph traversal

These signals have incompatible score distributions. BM25 scores are unbounded and depend on corpus statistics. Cosine similarity ranges [0, 1] (or [-1, 1] for non-normalized vectors). Graph activation levels are arbitrary floats that depend on link weights and traversal depth. Combining them requires a fusion strategy that handles heterogeneous score scales.

**Decision:**

Reciprocal Rank Fusion (RRF) with k=60, as described by Cormack, Clarke & Buettcher (2009).

For each result, the RRF score is:

```
RRF(d) = sum over all lists L: 1 / (k + rank_L(d))
```

Where `rank_L(d)` is the rank of document `d` in list `L` (1-indexed), and `k = 60` is a smoothing constant. Documents not appearing in a list are assigned infinite rank (contributing 0 to the sum).

After RRF fusion, results pass through a reranking stage:

```
final_score = rrf_score * (1 + 0.3 * context_similarity) * (1 + 0.2 * recency)
```

Where `recency = exp(-age_days / 30)`.

The pipeline operates on ranks, not scores, making it inherently robust to score distribution differences.

**Consequences:**

- **Score-agnostic fusion:** RRF uses only rank positions, not raw scores. This eliminates the need to normalize incompatible score distributions across BM25, vector, and graph signals. A BM25 score of 4.7 and a cosine similarity of 0.82 are not comparable, but "ranked 3rd in BM25" and "ranked 5th in vector" are.
- **Well-studied approach:** RRF has extensive literature and is used in production by Elasticsearch, Azure Cognitive Search, and others. The k=60 constant is the value recommended in the original paper and widely adopted.
- **Simple implementation:** The fusion function is ~20 lines of Rust. No learned weights, no training data, no hyperparameter optimization. This aligns with Simplicity > Completeness.
- **Graceful degradation:** If any signal produces no results (e.g., no embeddings stored, so vector search returns empty), the remaining signals still produce valid RRF scores. Two-signal fusion degrades to the union of two rank lists. Single-signal fusion degrades to the original ranked list.
- **No learned personalization:** RRF treats all signals equally. A learned fusion model could weight BM25 higher for keyword-heavy queries and vector higher for semantic queries. This is a potential v0.3+ optimization.
- **Fixed k parameter:** k=60 is a good default but may not be optimal for all workloads. Making k configurable is low-cost and planned for `AlayaConfig`.

**Alternatives Considered:**

| Alternative | Pros | Cons | Why Rejected |
|-------------|------|------|--------------|
| Weighted linear combination | Intuitive (w1*BM25 + w2*vector + w3*graph); tunable weights | Requires score normalization across heterogeneous signals; weights must be tuned per corpus; fragile to score distribution changes | Score normalization is the hard problem this decision is trying to avoid; weight tuning requires training data we do not have |
| Learned fusion (LTR model) | Optimal weights per query type; adapts to corpus characteristics | Requires training data (labeled relevance judgments); adds ML dependency; opaque; cold-start problem | Violates Simplicity > Completeness; no training data available; adds dependency and complexity disproportionate to benefit |
| CombMNZ | Combines scores and boosts documents appearing in multiple lists | Still requires score normalization; sensitive to score scale differences | Same normalization problem as weighted linear combination |
| Single best signal (cascade) | Pick BM25 first, fall back to vector, then graph | Discards information from non-primary signals; no fusion benefit | Leaves significant retrieval quality on the table; three signals contain more information than one |

**Cross-references:**

- Architecture Blueprint: Section 6.4 (RRF Fusion), retrieval pipeline stages
- North Star Extract: Axiom "Simplicity > Completeness"
- Architecture outputs: `retrieval_pipeline.stages[3]` (RRF Fusion, k=60, reference: Cormack, Clarke & Buettcher 2009)
- ADR-001: FTS5 provides BM25 signal
- ADR-003: Graph provides spreading activation signal
- ADR-010: FTS5 BM25 scoring feeds into RRF as one of three signals

---

## ADR-007: Vasana Preference Emergence

**Status:** Accepted
**Date:** 2026-02-27
**Deciders:** Project maintainer

**Context:**

Users have preferences that influence how an agent should behave -- communication style, topic interests, tool preferences, scheduling habits. The question is how an agent memory system should learn these preferences:

1. **Explicit declaration:** User tells the agent "I prefer morning meetings"
2. **LLM extraction:** After each conversation, an LLM extracts stated preferences (Mem0's approach)
3. **Behavioral emergence:** The system observes patterns across interactions and crystallizes preferences when confidence is sufficient

Most existing systems use approach 2 (LLM extraction), which captures only explicitly stated preferences and requires an LLM for every write. No shipping system implements approach 3.

The axiom "Process > Storage" and the belief "Preferences emerge, they are not declared" point toward behavioral emergence.

**Decision:**

Implement a three-stage preference emergence pipeline inspired by the Yogacara Buddhist concept of *vasana* (perfuming -- how experiences leave residual impressions that shape future perception):

**Stage 1: Impression Accumulation**

During `perfume()`, the `ConsolidationProvider` extracts behavioral observations ("impressions") from recent episodes. Each impression has a `subject` (what domain), `observation` (what was noticed), `valence` (positive/negative/neutral), and `confidence` score. Impressions are raw, unprocessed observations.

With `NoOpProvider`, no impressions are extracted, but the consumer can store impressions directly via lower-level API.

**Stage 2: Pattern Detection**

When 5 or more impressions share a common subject (determined by subject string matching at v0.1, planned semantic clustering at v0.2), the system detects a potential pattern. The crystallization threshold of 5 ensures that one-off observations do not become preferences.

**Stage 3: Preference Crystallization**

Detected patterns are crystallized into `Preference` objects with a `preference_statement` (human-readable), `confidence` (based on impression count and consistency), and `source_impressions` (provenance chain). Preferences have Bjork dual-strength tracking (ADR-005), so unused preferences fade over time.

The `perfume()` method returns a `PerfumingReport` detailing impressions extracted and preferences crystallized.

**Consequences:**

- **Novel contribution:** No other shipping agent memory system learns preferences from behavioral observation without explicit LLM extraction. Mem0 requires the user to state preferences or an LLM to extract them. This is a genuine differentiator.
- **No LLM required for basic operation:** With `NoOpProvider`, no preferences emerge automatically, but the consumer can store impressions manually. With a provider, the full pipeline operates. This follows the graceful degradation pattern.
- **Confidence through repetition:** The 5-impression threshold prevents premature crystallization. A user mentioning coffee once does not create a "prefers coffee" preference. Mentioning it across 5+ sessions does. This produces higher-quality preferences than single-observation extraction.
- **Provenance tracking:** Every preference links back to its source impressions, which link back to source episodes. The consumer can explain why a preference exists and when it formed.
- **Preference decay:** Via Bjork dual-strength (ADR-005), unused preferences lose retrieval strength. A preference formed 6 months ago that has not influenced any interaction fades, reflecting how human preferences evolve.
- **Slow to form:** Requiring 5+ impressions means preferences take multiple sessions to emerge. For applications needing immediate preference capture, the consumer should implement explicit preference storage alongside vasana emergence.
- **Subject matching limitations:** v0.1 uses string matching for subject clustering, which misses semantic equivalence ("coffee" vs "caffeine" vs "morning beverage"). Semantic clustering via embeddings is planned for v0.2.

**Alternatives Considered:**

| Alternative | Pros | Cons | Why Rejected |
|-------------|------|------|--------------|
| LLM-extracted preferences (Mem0) | Immediate capture; works from first mention; consumer does not implement anything | Requires LLM for every write (violates zero-dep); captures only explicitly stated preferences; misses behavioral patterns; no confidence accumulation | Violates Privacy > Features; captures declared preferences only, not emergent behavioral patterns |
| Explicit preference API only | Simplest; consumer has full control; deterministic | No automatic learning; consumer must implement all preference detection; no behavioral emergence; no competitive differentiation | Eliminates a core differentiator; "preferences emerge" is a founding belief |
| No preference tracking | Simplest possible; one less store to maintain | Agent cannot personalize behavior; no preference-aware retrieval; no competitive differentiation in this dimension | Unacceptably limited for an agent memory system; preferences are table-stakes for companion agents |
| Frequency-based heuristics | Count topic mentions; threshold for preference | No valence tracking; no confidence accumulation; misses nuance (mentioning "deadlines" frequently might be stress, not preference) | Too crude; does not model observation quality or behavioral context |

**Cross-references:**

- Architecture Blueprint: Section 2.4 (Implicit Store), Section 7.2 (Perfuming)
- Competitive Landscape: Differentiator "Implicit preference emergence without LLM (vasana/perfuming) -- unique in field"
- Competitive Landscape: Novelty innovation "Vasana preference emergence", research basis "Yogacara Buddhist psychology"
- North Star Extract: Belief "Preferences emerge, they are not declared", axiom "Process > Storage"
- ADR-004: ConsolidationProvider.extract_impressions() enables Stage 1
- ADR-005: Preferences have Bjork dual-strength tracking for decay

---

## ADR-008: Sync-First API

**Status:** Accepted
**Date:** 2026-02-27
**Deciders:** Project maintainer

**Context:**

Alaya is an embedded library, not a network service. Its I/O is exclusively SQLite file operations (via `rusqlite`), which are synchronous system calls. The question is whether to expose a synchronous API, an asynchronous API (requiring a runtime like `tokio`), or both.

The consumer landscape is diverse:

- Some agents are synchronous (simple CLI tools, scripts, single-threaded applications)
- Some agents use `tokio` (web servers, concurrent agent frameworks)
- Some agents use other async runtimes (`async-std`, `smol`)
- Some consumers are not Rust at all (C FFI, Python via PyO3)

**Decision:**

Synchronous API by default. All public methods on `AlayaStore` are synchronous, blocking the calling thread for the duration of the SQLite operation. `AlayaStore` is `Send` but not `Sync` -- it can be moved between threads but not shared.

For multi-threaded use, the consumer wraps in `Arc<Mutex<AlayaStore>>`.

For async use, a feature flag (`async`) is planned for v0.2 that provides an async wrapper using `tokio::task::spawn_blocking`. This keeps the core crate runtime-agnostic while providing convenience for tokio users.

```rust
// Sync (default, v0.1)
let store = AlayaStore::open("memory.db")?;
store.store_episode(&episode)?;

// Multi-threaded sync
let store = Arc::new(Mutex::new(AlayaStore::open("memory.db")?));

// Async (v0.2, feature flag)
let store = AsyncAlayaStore::open("memory.db").await?;
store.store_episode(&episode).await?;
```

**Consequences:**

- **Simple default:** Synchronous API is the simplest possible threading model. No runtime dependency, no future/stream complexity, no pin/unpin concerns. A beginner can use Alaya without understanding async Rust.
- **No runtime dependency:** The core crate does not depend on `tokio`, `async-std`, or any async runtime. This keeps the dependency tree minimal and avoids runtime coupling.
- **FFI-friendly:** Synchronous functions map directly to C FFI (cbindgen) and Python bindings (PyO3). Async functions require runtime bridging that complicates cross-language integration.
- **Consumer bears threading cost:** For concurrent access, the consumer must wrap in `Arc<Mutex<>>`. This is explicit friction that forces the consumer to think about concurrency rather than hiding it behind an async facade.
- **SQLite is inherently synchronous:** `rusqlite` is synchronous. An async API would just wrap `spawn_blocking` around synchronous calls. The async wrapper adds indirection without improving throughput -- SQLite is single-writer regardless.
- **Blocks calling thread:** Long operations (consolidation with many episodes, large retrieval queries) block the calling thread. In async contexts, this requires the consumer to use `spawn_blocking` themselves until the `async` feature flag is available.

**Alternatives Considered:**

| Alternative | Pros | Cons | Why Rejected |
|-------------|------|------|--------------|
| Async-first (tokio) | Familiar to web/agent developers; natural for concurrent frameworks | Requires tokio runtime dependency; forces all consumers into async; complicates FFI; SQLite is synchronous underneath anyway; beginner-unfriendly | Forces a runtime dependency on all consumers; async adds complexity without throughput benefit for single-writer SQLite |
| Both sync and async surfaces | Maximum compatibility; each consumer chooses | Doubles API surface; twice the testing; potential inconsistencies; significant maintenance burden for solo developer | Violates Simplicity > Completeness; maintenance burden unjustified at v0.1; async wrapper planned for v0.2 via feature flag |
| Async-only with sync wrapper | Modern API design; can add sync via `block_on` | Requires runtime even for sync consumers; `block_on` inside `block_on` panics in nested async contexts; overhead for simple use cases | Worse than sync-first; runtime dependency for sync consumers is unnecessary complexity |

**Cross-references:**

- Architecture Blueprint: Section 2.1 (AlayaStore), thread safety model
- Architecture outputs: `ownership: "AlayaStore owns Connection, all methods take &self"`, `thread_safety: "Send but not Sync; caller wraps in Mutex for multi-thread"`
- North Star Extract: Axiom "Simplicity > Completeness", constraint "Sync-first (async via feature flag)"
- Design System: Skill levels "beginner (3 methods) -> intermediate -> advanced -> expert"
- ADR-001: SQLite is inherently single-writer synchronous

---

## ADR-009: Zero Network Calls

**Status:** Accepted
**Date:** 2026-02-27
**Deciders:** Project maintainer

**Context:**

Alaya positions itself as the privacy-first alternative to cloud-dependent memory systems (Mem0, Zep, Letta). The competitive landscape analysis identifies "zero external dependencies" and "privacy by architecture" as core differentiators. The axiom "Privacy > Features" is the highest-priority design axiom after safety.

The question is not whether to limit network calls but how absolutely to enforce the prohibition:

1. No network calls by default, optional via feature flag
2. No network calls in core, allowed in companion crates
3. No network calls in core crate, ever, under any circumstances
4. Optional telemetry for usage analytics

**Decision:**

No network calls in the core `alaya` crate, ever, under any circumstances. This is an architectural guarantee, not a configuration option. The core crate has no dependency on any networking library (`reqwest`, `hyper`, `tokio::net`, `std::net`). There is no HTTP client, no DNS resolver, no socket creation, no telemetry, no analytics, no crash reporting, no update checking.

Network connectivity is exclusively the consumer's responsibility. When the consumer implements `ConsolidationProvider` or `EmbeddingProvider` (ADR-004), their implementation may make network calls (e.g., to an LLM API), but that code lives in the consumer's crate, not in Alaya.

This guarantee is verifiable: `cargo tree` on the core crate shows no networking dependencies. CI can enforce this via dependency auditing.

**Consequences:**

- **Privacy by architecture:** The guarantee is structural, not behavioral. It is impossible for Alaya to exfiltrate data because it has no mechanism to do so. This is a stronger guarantee than "we promise not to send data" -- the code literally cannot make network calls.
- **Compliance simplification:** GDPR, CCPA, SOC2 compliance scoping is dramatically simplified when a component structurally cannot transmit data. Alaya does not need a privacy policy because it cannot communicate with any external system.
- **Edge/air-gapped deployment:** Alaya works in environments with no network access: air-gapped systems, embedded devices, restricted government networks, offline mobile applications.
- **No telemetry for developer:** The maintainer gets zero usage data from the library itself. Adoption must be measured through proxy signals (crates.io downloads, GitHub dependents, issues, community engagement).
- **Consumer must provide all external connectivity:** Embedding generation, LLM consolidation, model downloads -- all external connectivity is the consumer's responsibility. This shifts complexity to the consumer but gives them full control.
- **Feature ceiling without provider:** Without a provider implementation, Alaya cannot generate embeddings or extract knowledge. The library is fully functional (FTS5 search, episode storage, session history, graph from co-occurrence) but operates below its potential.

**Alternatives Considered:**

| Alternative | Pros | Cons | Why Rejected |
|-------------|------|------|--------------|
| Optional telemetry (opt-in) | Usage data for development priorities; adoption measurement; crash reporting | Any telemetry capability means the library can communicate externally; "opt-in" erodes trust; adds networking dependency even if unused by default | Even optional telemetry adds a network dependency to the crate; erodes the "structurally cannot communicate" guarantee |
| Built-in embedding API calls | Turnkey embedding generation; consumer does not need to implement EmbeddingProvider | Requires HTTP client dependency; hardcodes API endpoint; requires API key management; makes network call from library code | Directly violates Privacy > Features; transforms library from privacy-guaranteed to trust-me-I-won't-leak |
| Cloud sync (optional) | Multi-device memory; backup to cloud | Adds network dependency; requires authentication; introduces cloud provider coupling; fundamentally changes the library's nature | On the kill list; contradicts core positioning; Mem0 owns the cloud memory space |
| Network calls in feature-flag-gated code | Core stays clean; optional networking behind explicit opt-in | Feature flag still means the crate contains networking code; supply chain risk; blurs the architectural guarantee | Weakens the "structurally cannot communicate" property; even behind a flag, the code exists in the crate |

**Cross-references:**

- Security Architecture: Property "Zero network calls" (structural guarantee)
- Competitive Landscape: Differentiator "Complete cognitive lifecycle + zero external dependencies"
- North Star Extract: Axiom "Privacy > Features", never-list "Make network calls in core crate"
- Brand Guidelines: Kill list "Not cloud-dependent (zero network calls, privacy by architecture)"
- ADR-004: Trait-based extension is the consequence of this constraint
- ADR-001: SQLite-only storage is consistent with no network access

---

## ADR-010: FTS5 for Full-Text Search

**Status:** Accepted
**Date:** 2026-02-27
**Deciders:** Project maintainer

**Context:**

Text search is the most fundamental retrieval capability. When a user asks "what did we discuss about the project deadline?", the system must find episodes containing relevant text. This requires an inverted index with relevance scoring.

Alaya needs:

- Full-text search with relevance ranking (BM25 scoring)
- Zero additional dependencies (consistent with ADR-001 and ADR-009)
- Integration with SQLite transactions (search results participate in the same transaction as writes)
- Synchronization with the episodic store (new episodes immediately searchable)
- Graceful handling of adversarial input (FTS5 MATCH operator injection)

**Decision:**

Use SQLite FTS5 (Full-Text Search version 5) with external content tables and the porter stemmer tokenizer. The FTS5 virtual table `episodes_fts` is an external content table linked to the `episodes` table. Three SQLite triggers maintain synchronization on INSERT, UPDATE, and DELETE.

```sql
CREATE VIRTUAL TABLE episodes_fts USING fts5(
    content,
    content=episodes,
    content_rowid=id,
    tokenize='porter unicode61'
);
```

BM25 scoring is built into FTS5 via the `bm25()` auxiliary function. The `search_bm25()` method queries FTS5 and returns results ranked by BM25 relevance.

**Input sanitization:** All user-provided search text is sanitized before being passed to FTS5 MATCH:

- Strip all characters that are not alphanumeric, whitespace, or hyphens
- If sanitized result is empty, return `Ok(vec![])` without executing SQL
- No FTS5 operators (AND, OR, NOT, NEAR, ^, *, quotes) pass through to MATCH

This eliminates FTS5 injection entirely at the cost of preventing power users from using FTS5 query syntax. This tradeoff favors safety (Safety > Performance in the conflict resolution hierarchy).

**Consequences:**

- **Zero additional dependencies:** FTS5 is compiled into SQLite, which is bundled via `rusqlite`. No tantivy, no external search engine, no additional binary size.
- **BM25 scoring:** Industry-standard relevance ranking with term frequency, inverse document frequency, and document length normalization. Sufficient for the BM25 signal in the RRF pipeline (ADR-006).
- **Porter stemmer:** "running" matches "run", "connections" matches "connect". Improves recall at the cost of occasional false matches. The `unicode61` tokenizer handles international text.
- **Transactional consistency:** FTS5 updates are synchronized via triggers within the same SQLite transaction. An episode stored via `store_episode()` is immediately searchable -- no eventual consistency, no reindexing delay.
- **Sanitization prevents injection:** The Security Architecture identifies FTS5 MATCH injection as threat T2 (high severity, high likelihood). The sanitization approach eliminates this attack vector entirely. No FTS5 operator can reach the MATCH clause.
- **No advanced query syntax:** The sanitization strips all FTS5 operators, which means users cannot use phrase search ("exact phrase"), proximity search (NEAR), or boolean operators. This is an intentional tradeoff -- the vast majority of agent memory queries are natural language, not structured search syntax.
- **External content table overhead:** FTS5 external content tables require triggers for synchronization. If triggers fail or are bypassed (e.g., direct SQL), the FTS5 index becomes stale. All writes go through the library API, which ensures triggers fire correctly.
- **Scale characteristics:** FTS5 handles millions of documents efficiently. The scale ceiling for Alaya is not FTS5 but rather brute-force vector search (ADR-001). FTS5 will remain performant well beyond the 50K embedding ceiling.

**Alternatives Considered:**

| Alternative | Pros | Cons | Why Rejected |
|-------------|------|------|--------------|
| tantivy (Rust search library) | Full-featured Rust search engine; custom scoring; advanced query syntax; faceted search | ~30K lines of code; separate index files (breaks single-file invariant); adds significant dependency; not transactional with SQLite | Breaks single-file invariant (ADR-001); disproportionate complexity for BM25 signal in a three-signal pipeline |
| Custom inverted index in SQLite | Full control; could be stored in regular SQLite tables; no FTS5 dependency | Reimplementing BM25 scoring, tokenization, and stemming from scratch; significant development effort; likely less efficient than FTS5's optimized C implementation | FTS5 already provides exactly this functionality; reimplementation would be worse by every metric |
| No text search (vector-only) | Simplest; relies entirely on embedding similarity | Cannot retrieve without embeddings (violates graceful degradation); keyword matches lost; no BM25 signal for RRF; requires EmbeddingProvider for any retrieval | Eliminates the most fundamental retrieval mechanism; makes the library useless without an EmbeddingProvider |
| Elasticsearch/Meilisearch | Advanced search features; distributed; battle-tested | External service (violates zero-dep and zero-network); separate process to manage; breaks embedded library model | Fundamentally incompatible with embedded library architecture |

**Cross-references:**

- Architecture Blueprint: Section 6.1 (BM25 Retrieval), Section 5 (SQLite Configuration)
- Security Architecture: Threat T2 (FTS5 MATCH injection), mitigation "search_bm25() strips all non-alphanumeric non-whitespace characters"
- Architecture outputs: `retrieval_pipeline.stages[0]` (BM25, source: "FTS5 MATCH on episodes_fts")
- North Star Extract: Axiom "Simplicity > Completeness", always-list "Sanitize all FTS5 MATCH input"
- ADR-001: FTS5 is built into bundled SQLite
- ADR-006: BM25 is one of three signals in RRF fusion

---

## Decision Dependency Graph

The ten ADRs form an interconnected dependency graph. Understanding these dependencies clarifies why individual decisions cannot be changed in isolation.

```
ADR-009 (Zero Network Calls)
    |
    +---> ADR-004 (Trait-Based Extension)
    |         |
    |         +---> ADR-005 (Bjork Forgetting) [works without provider]
    |         +---> ADR-007 (Vasana Preferences) [uses provider for extraction]
    |
    +---> ADR-001 (SQLite as Storage)
              |
              +---> ADR-010 (FTS5 for Search) [built into SQLite]
              +---> ADR-002 (Three-Store Architecture) [single file, multiple tables]
              |         |
              |         +---> ADR-003 (Hebbian Graph) [spans all three stores]
              |
              +---> ADR-008 (Sync-First API) [SQLite is synchronous]

ADR-006 (RRF Fusion) <--- ADR-010 (BM25 signal)
                      <--- ADR-003 (Graph signal)
                      <--- ADR-001 (Vector signal via embeddings)
```

**Root constraint:** ADR-009 (Zero Network Calls) is the root constraint from which ADR-004 and ADR-001 derive. Relaxing ADR-009 would cascade through the entire decision tree.

**Fusion convergence:** ADR-006 (RRF) is the convergence point where three independent retrieval signals (ADR-010 BM25, ADR-003 graph, ADR-001 vector) merge into a single ranked result.

---

## Decision Traceability Matrix

Each ADR traces back to at least one governing axiom and forward to at least one security consideration.

| ADR | Governing Axiom | Security Implication | Competitive Advantage |
|-----|-----------------|----------------------|-----------------------|
| 001 | Simplicity > Completeness | BEGIN IMMEDIATE prevents deadlocks; WAL checkpoint prevents corruption | Single-file deployment vs. Neo4j/Pinecone infrastructure |
| 002 | Process > Storage | Cross-store cascade deletion; tombstone mechanism | Complete cognitive lifecycle vs. single-store competitors |
| 003 | Process > Storage | Graph link injection (low risk); activation amplification | Self-organizing graph vs. static LLM-extracted graphs |
| 004 | Privacy > Features | Provider output injection; untrusted provider code | Zero-dep core vs. LLM-required competitors |
| 005 | Correctness > Speed | Memory resurrection via forgetting bypass | Only dual-strength forgetting in any shipping system |
| 006 | Simplicity > Completeness | No query injection (operates on ranks, not user input) | Three-signal fusion vs. single-signal competitors |
| 007 | Process > Storage | Impression accumulation reveals behavioral patterns (PII) | Only behavioral preference emergence without LLM |
| 008 | Simplicity > Completeness | No async race conditions in default API | Simplest threading model; FFI-friendly |
| 009 | Privacy > Features | Structural impossibility of data exfiltration | Privacy by architecture vs. privacy by policy |
| 010 | Simplicity > Completeness | FTS5 injection mitigated by input sanitization | Zero-dep text search vs. external search engines |

---

## Revision History

| Date | ADR | Change | Reason |
|------|-----|--------|--------|
| 2026-02-27 | All | Initial creation | Phase 9 document generation |
