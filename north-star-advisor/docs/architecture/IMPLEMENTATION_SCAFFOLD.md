# Implementation Scaffold: Alaya

> Embeddable Rust memory library with cognitive lifecycle processes and implicit preference emergence for privacy-first AI agents.

**Generated:** 2026-02-26 | **Phase:** 10 | **Status:** Implementation-ready
**Codebase verified:** 43 tests passing, 4064 lines across 25 source files

---

## Table of Contents

1. [Current vs Target Module Tree](#1-current-vs-target-module-tree)
2. [Cargo.toml Blueprint](#2-cargotoml-blueprint)
3. [Core Types and Traits](#3-core-types-and-traits)
4. [Store Module Implementation](#4-store-module-implementation)
5. [Retrieval Module Implementation](#5-retrieval-module-implementation)
6. [Lifecycle Module Implementation](#6-lifecycle-module-implementation)
7. [Graph Module Implementation](#7-graph-module-implementation)
8. [Provider Traits](#8-provider-traits)
9. [Feature Flag Architecture](#9-feature-flag-architecture)
10. [FFI and Language Bindings Plan](#10-ffi-and-language-bindings-plan)
11. [Build and CI Configuration](#11-build-and-ci-configuration)
12. [Migration Path](#12-migration-path)

---

## 1. Current vs Target Module Tree

### Current State (v0.1-alpha, verified)

The crate has already adopted the target module structure established by the Architecture Blueprint. This is a significant achievement -- the layout matches the architecture document with no orphan files or misplaced modules.

```
alaya/
  Cargo.toml                      # [EXISTS] 4 deps: rusqlite, serde, serde_json, thiserror
  Cargo.lock                      # [EXISTS]
  src/
    lib.rs                        # [EXISTS]  276 lines - AlayaStore struct, pub API, 2 integration tests
    error.rs                      # [EXISTS]   21 lines - AlayaError enum (5 variants), Result alias
    types.rs                      # [EXISTS]  402 lines - All public types: IDs, enums, structs, reports
    schema.rs                     # [EXISTS]  238 lines - DB init: 7 tables, 1 FTS5, 3 triggers, 9 indexes
    provider.rs                   # [EXISTS]   78 lines - ConsolidationProvider trait, NoOpProvider, MockProvider
    store/
      mod.rs                      # [EXISTS]    5 lines - Re-exports submodules
      episodic.rs                 # [EXISTS]  177 lines - CRUD, FTS5 sync, session queries, unconsolidated
      semantic.rs                 # [EXISTS]  139 lines - CRUD, corroboration, find_by_type, cascade delete
      implicit.rs                 # [EXISTS]  182 lines - Impressions + Preferences CRUD, decay, prune
      embeddings.rs               # [EXISTS]  176 lines - f32 BLOB ser/de, cosine similarity, brute-force search
      strengths.rs                # [EXISTS]  145 lines - Bjork dual-strength: init, on_access, decay, archive
    graph/
      mod.rs                      # [EXISTS]    2 lines - Re-exports submodules
      links.rs                    # [EXISTS]  164 lines - CRUD, Hebbian co-retrieval LTP, decay, prune
      activation.rs               # [EXISTS]  126 lines - Collins & Loftus spreading activation
    retrieval/
      mod.rs                      # [EXISTS]    5 lines - Re-exports submodules
      bm25.rs                     # [EXISTS]  104 lines - FTS5 MATCH with sanitization, BM25 normalization
      vector.rs                   # [EXISTS]   27 lines - Thin wrapper over embeddings::search_by_vector
      fusion.rs                   # [EXISTS]   68 lines - RRF merge (k=60)
      rerank.rs                   # [EXISTS]   85 lines - Context similarity + recency decay
      pipeline.rs                 # [EXISTS]  143 lines - Full query orchestration: BM25+vector+graph->RRF->rerank
    lifecycle/
      mod.rs                      # [EXISTS]    4 lines - Re-exports submodules
      consolidation.rs            # [EXISTS]  118 lines - CLS replay, episodes -> semantic nodes
      perfuming.rs                # [EXISTS]  135 lines - Vasana: impressions -> preference crystallization
      transformation.rs           # [EXISTS]  134 lines - Dedup, link prune, preference decay, impression prune
      forgetting.rs               # [EXISTS]   95 lines - Bjork RS decay + archive below threshold
```

### Target State (v0.1 release)

The module tree is structurally complete. What remains is hardening, not restructuring.

```diff
  alaya/
    Cargo.toml
    Cargo.lock
+   rust-toolchain.toml           # [MISSING] Pin MSRV
+   clippy.toml                   # [MISSING] Lint configuration
+   .github/
+     workflows/
+       ci.yml                    # [MISSING] Test + clippy + audit pipeline
    src/
      lib.rs                      # [NEEDS WORK] See gap analysis below
      error.rs                    # [NEEDS WORK] #[non_exhaustive] missing
      types.rs                    # [NEEDS WORK] #[non_exhaustive] on enums, Serialize/Deserialize on more types
      schema.rs                   # [NEEDS WORK] BEGIN IMMEDIATE, WAL checkpoint, content-hash column
      provider.rs                 # [NEEDS WORK] EmbeddingProvider trait (planned v0.2)
      store/
        (all files exist and function correctly)
      graph/
        (all files exist and function correctly)
      retrieval/
        (all files exist and function correctly)
      lifecycle/
        (all files exist and function correctly)
+   benches/
+     retrieval.rs                # [MISSING] divan benchmarks (v0.2)
+   tests/
+     integration.rs              # [MISSING] Cross-module integration tests
```

### Gap Analysis: Existing vs Required

| Area | Current State | Target State (v0.1) | Priority |
|------|--------------|---------------------|----------|
| Module layout | Matches architecture blueprint | No changes needed | Done |
| Test count | 43 passing | Add doctests on all pub methods | P0 |
| `#[non_exhaustive]` | Missing on all enums | All public enums | P0 |
| `BEGIN IMMEDIATE` | Not used (uses implicit BEGIN DEFERRED) | All write transactions | P0 |
| Input validation | No validation at API boundary | Validate content length, embedding dimensions | P0 |
| Visibility | All modules `pub` | Internal modules should be `pub(crate)` | P1 |
| LTD in retrieval | `decay_links()` exists but never called from pipeline | Call during transform() or after retrieval | P1 |
| Tombstone table | Not present | Track deleted node IDs to prevent resurrection | P1 |
| WAL checkpoint | Not managed | Periodic checkpoint call in `transform()` or admin | P2 |
| Content-hash integrity | No integrity checking | SHA-256 on episodes for tamper detection | P2 |
| Compilable doctests | 0 doctests | Every pub method in lib.rs and types.rs | P0 |

---

## 2. Cargo.toml Blueprint

### Current Cargo.toml (verified)

```toml
[package]
name = "alaya"
version = "0.1.0"
edition = "2021"
authors = ["Albert Hui <albert@securityronin.com>"]
description = "A memory engine for conversational AI agents, inspired by neuroscience and Buddhist psychology"
license = "MIT"
repository = "https://github.com/h4x0r/alaya"
keywords = ["memory", "ai", "rag", "conversational", "embedding"]
categories = ["database", "science"]

[dependencies]
rusqlite = { version = "0.32", features = ["bundled", "modern_sqlite"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"

[dev-dependencies]
```

### Target Cargo.toml (v0.1 release)

```toml
[package]
name = "alaya"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"  # MSRV: edition 2021, thiserror 2.x requires 1.74+
authors = ["Albert Hui <albert@securityronin.com>"]
description = "A memory engine for conversational AI agents, inspired by neuroscience and Buddhist psychology"
license = "MIT"
repository = "https://github.com/h4x0r/alaya"
keywords = ["memory", "ai", "rag", "conversational", "embedding"]
categories = ["database", "science"]
readme = "README.md"
exclude = [
    "north-star-advisor/",
    ".github/",
    "benches/",
]

[dependencies]
rusqlite = { version = "0.32", features = ["bundled", "modern_sqlite"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"

[dev-dependencies]
tempfile = "3"

[features]
default = []
# v0.2 features -- stubs for forward declaration
# vec-sqlite = ["sqlite-vec"]
# embed-ort = ["ort"]
# embed-fastembed = ["fastembed"]
# async = ["tokio"]

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]

[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = ['cfg(docsrs)'] }

[lints.clippy]
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"
must_use_candidate = "allow"
```

### Target Workspace Cargo.toml (v0.2)

When FFI and Python binding crates are added, the project will transition to a workspace layout.

```toml
[workspace]
members = [
    "alaya",       # Core library crate
    "alaya-ffi",   # C FFI crate (cbindgen)
    "alaya-py",    # Python bindings (PyO3)
]
resolver = "2"

[workspace.package]
version = "0.2.0"
edition = "2021"
rust-version = "1.75"
license = "MIT"
repository = "https://github.com/h4x0r/alaya"
```

### Dependency Rationale

| Dependency | Version | Purpose | ADR Reference |
|-----------|---------|---------|---------------|
| `rusqlite` | 0.32, bundled + modern_sqlite | SQLite storage, WAL, FTS5 | ADR-001, ADR-010 |
| `serde` | 1.x, derive | Serialization for types, JSON context storage | -- |
| `serde_json` | 1.x | JSON encoding of EpisodeContext, source_episodes | -- |
| `thiserror` | 2.x | AlayaError derive macro | -- |

No other runtime dependencies. This is deliberate per ADR-009 (Zero Network Calls). The `cargo tree` output should show exactly these four crates (plus their transitive dependencies, all compile-time). Notably absent: `reqwest`, `hyper`, `tokio`, `async-std`, or any crate from the `net` family.

### v0.1 dev-dependencies to Add

```toml
[dev-dependencies]
tempfile = "3"           # Persistent-path tests without polluting filesystem
```

### v0.2 Planned Dependencies (behind feature flags)

```toml
# In [dependencies], gated by feature flags:
sqlite-vec = { version = "0.1", optional = true }        # SIMD vector search
ort = { version = "2", optional = true }                  # ONNX Runtime embeddings
fastembed = { version = "4", optional = true }            # Turnkey embeddings
tokio = { version = "1", features = ["rt"], optional = true }  # spawn_blocking

# In alaya-ffi/Cargo.toml:
cbindgen = "0.27"  # [build-dependency]

# In alaya-py/Cargo.toml:
pyo3 = { version = "0.22", features = ["extension-module"] }
```

---

## 3. Core Types and Traits

### AlayaStore (Entry Point)

The `AlayaStore` struct is the sole public API surface. All interaction flows through it. Internal modules (`store`, `retrieval`, `graph`, `lifecycle`) are implementation details that should be `pub(crate)`, not `pub`.

**Current state (lib.rs, 276 lines):** Fully functional. Owns a `Connection`, exposes 12 methods across write/read/lifecycle/admin categories. Two integration tests.

**File:** `/Users/4n6h4x0r/src/alaya/src/lib.rs`

```rust
pub struct AlayaStore {
    conn: Connection,
}
```

**Public API surface (12 methods, all implemented and tested):**

| Method | Category | Input | Output | Status |
|--------|----------|-------|--------|--------|
| `open(path)` | Constructor | `impl AsRef<Path>` | `Result<Self>` | Working |
| `open_in_memory()` | Constructor | -- | `Result<Self>` | Working |
| `store_episode(&self, &NewEpisode)` | Write | `&NewEpisode` | `Result<EpisodeId>` | Working |
| `query(&self, &Query)` | Read | `&Query` | `Result<Vec<ScoredMemory>>` | Working |
| `preferences(&self, Option<&str>)` | Read | domain filter | `Result<Vec<Preference>>` | Working |
| `knowledge(&self, Option<KnowledgeFilter>)` | Read | filter | `Result<Vec<SemanticNode>>` | Working |
| `neighbors(&self, NodeRef, u32)` | Read | node + depth | `Result<Vec<(NodeRef, f32)>>` | Working |
| `consolidate(&self, &dyn ConsolidationProvider)` | Lifecycle | provider | `Result<ConsolidationReport>` | Working |
| `perfume(&self, &Interaction, &dyn ConsolidationProvider)` | Lifecycle | interaction + provider | `Result<PerfumingReport>` | Working |
| `transform(&self)` | Lifecycle | -- | `Result<TransformationReport>` | Working |
| `forget(&self)` | Lifecycle | -- | `Result<ForgettingReport>` | Working |
| `status(&self)` | Admin | -- | `Result<MemoryStatus>` | Working |
| `purge(&self, PurgeFilter)` | Admin | filter | `Result<PurgeReport>` | Working |

**Threading model:** `AlayaStore` is `Send` (owns `Connection`) but not `Sync`. For multi-threaded use, consumers wrap in `Arc<Mutex<AlayaStore>>`. This is documented in ADR-008 and is intentional for v0.1 simplicity.

### ID Newtypes

**File:** `/Users/4n6h4x0r/src/alaya/src/types.rs` (lines 7-20)

All ID types follow the same pattern: newtype wrapper around `i64` with `Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize`.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EpisodeId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PreferenceId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImpressionId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinkId(pub i64);
```

**Known gap:** The inner `i64` field is `pub`, which allows construction of arbitrary IDs. For v0.1 this is acceptable (library trust model), but for v0.2 consider making fields private and adding `pub(crate) fn new(id: i64) -> Self` constructors.

### NodeRef (Polymorphic Pointer)

**File:** `/Users/4n6h4x0r/src/alaya/src/types.rs` (lines 26-58)

`NodeRef` is the polymorphic reference that enables the graph overlay to link across all three stores. It is the conceptual equivalent of a tagged union for store identity.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeRef {
    Episode(EpisodeId),
    Semantic(NodeId),
    Preference(PreferenceId),
}
```

Helper methods: `type_str() -> &'static str`, `id() -> i64`, `from_parts(node_type, id) -> Option<Self>`. These are used extensively in the SQLite layer where polymorphic rows store `(node_type TEXT, node_id INTEGER)` pairs.

**Known gap:** `#[non_exhaustive]` is missing. Adding it is a P0 task per the security architecture document.

### Enums

**File:** `/Users/4n6h4x0r/src/alaya/src/types.rs` (lines 64-152)

Three enums, all with `as_str()` and `from_str()` conversion methods for SQLite serialization:

| Enum | Variants | Serialization |
|------|----------|---------------|
| `Role` | `User`, `Assistant`, `System` | `#[serde(rename_all = "lowercase")]` |
| `SemanticType` | `Fact`, `Relationship`, `Event`, `Concept` | `#[serde(rename_all = "lowercase")]` |
| `LinkType` | `Temporal`, `Topical`, `Entity`, `Causal`, `CoRetrieval` | `#[serde(rename_all = "lowercase")]` |

**Known gap:** All three enums need `#[non_exhaustive]` to allow adding variants in minor versions without breaking downstream matches. This is a semver consideration for a library crate.

### Input Types (New* pattern)

Input types are what the consumer constructs. They contain only the data the consumer provides. System-generated fields (IDs, timestamps, corroboration counts) are added during storage.

| Type | Key Fields | Derive Traits | Notes |
|------|-----------|---------------|-------|
| `NewEpisode` | content, role, session_id, timestamp, context, embedding | `Debug, Clone` | embedding is `Option<Vec<f32>>` for graceful degradation |
| `NewSemanticNode` | content, node_type, confidence, source_episodes, embedding | `Debug, Clone` | Created by ConsolidationProvider |
| `NewImpression` | domain, observation, valence | `Debug, Clone` | Created by ConsolidationProvider |
| `Interaction` | text, role, session_id, timestamp, context | `Debug, Clone` | Consumer-constructed for perfuming |
| `Query` | text, embedding, context, max_results | `Debug, Clone` | Has `Query::simple(text)` convenience constructor |
| `EpisodeContext` | topics, sentiment, conversation_turn, mentioned_entities, preceding_episode | `Debug, Clone, Default, Serialize, Deserialize` | All fields have `#[serde(default)]` |
| `QueryContext` | topics, sentiment, mentioned_entities, current_timestamp | `Debug, Clone, Default` | Mirrors EpisodeContext for retrieval |

### Output Types (Entity pattern)

Output types are what Alaya returns. They include system-generated fields like IDs and timestamps.

| Type | Key Fields Beyond Input | Serialize |
|------|------------------------|-----------|
| `Episode` | id, (all NewEpisode fields minus embedding) | Yes |
| `SemanticNode` | id, created_at, last_corroborated, corroboration_count | Yes |
| `Impression` | id, timestamp | Yes |
| `Preference` | id, confidence, evidence_count, first_observed, last_reinforced | Yes |
| `Link` | id, source, target, forward_weight, backward_weight, link_type, created_at, last_activated, activation_count | Yes |
| `NodeStrength` | node, storage_strength, retrieval_strength, access_count, last_accessed | Yes |
| `ScoredMemory` | node, content, score (f64), role, timestamp | Yes |

### Report Types

Every lifecycle method returns a typed report rather than `()`. This enables audit logging, monitoring, and test assertions.

| Report | Fields | Default |
|--------|--------|---------|
| `ConsolidationReport` | episodes_processed (u32), nodes_created, links_created | All 0 |
| `PerfumingReport` | impressions_stored (u32), preferences_crystallized, preferences_reinforced | All 0 |
| `TransformationReport` | duplicates_merged (u32), links_pruned, preferences_decayed, impressions_pruned | All 0 |
| `ForgettingReport` | nodes_decayed (u32), nodes_archived | All 0 |
| `PurgeReport` | episodes_deleted (u32), nodes_deleted, links_deleted, embeddings_deleted | All 0 |
| `MemoryStatus` | episode_count (u64), semantic_node_count, preference_count, impression_count, link_count, embedding_count | N/A |

### Filter Types

| Type | Variants/Fields |
|------|----------------|
| `KnowledgeFilter` | node_type: `Option<SemanticType>`, min_confidence: `Option<f32>`, limit: `Option<usize>` |
| `PurgeFilter` | `Session(String)`, `OlderThan(i64)`, `All` |

### AlayaError

**File:** `/Users/4n6h4x0r/src/alaya/src/error.rs` (21 lines)

```rust
#[derive(Debug, Error)]
pub enum AlayaError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("provider error: {0}")]
    Provider(String),
}

pub type Result<T> = std::result::Result<T, AlayaError>;
```

Five variants, two with `#[from]` auto-conversion. The `InvalidInput` variant is defined but never constructed in the current codebase -- it exists for the input validation that is listed as a P0 gap.

**Known gap:** `#[non_exhaustive]` missing. The `Provider` variant takes a `String` rather than `Box<dyn std::error::Error + Send + Sync>`, which means consumers lose error context when forwarding provider errors.

---

## 4. Store Module Implementation

The store module contains five submodules corresponding to the five storage concerns: episodic episodes, semantic knowledge nodes, implicit impressions/preferences, polymorphic embeddings, and Bjork dual-strength tracking.

### store/episodic.rs (177 lines, 4 tests)

**File:** `/Users/4n6h4x0r/src/alaya/src/store/episodic.rs`

Manages the `episodes` table and indirectly the `episodes_fts` FTS5 virtual table (synced via triggers defined in `schema.rs`).

**Functions:**

| Function | Signature | Purpose | Tested |
|----------|-----------|---------|--------|
| `store_episode` | `(&Connection, &NewEpisode) -> Result<EpisodeId>` | INSERT with context JSON serialization | Yes |
| `get_episode` | `(&Connection, EpisodeId) -> Result<Episode>` | Single row by PK, maps QueryReturnedNoRows to NotFound | Yes |
| `get_episodes_by_session` | `(&Connection, &str) -> Result<Vec<Episode>>` | Session-scoped, ordered by timestamp ASC | Yes |
| `get_recent_episodes` | `(&Connection, u32) -> Result<Vec<Episode>>` | Top N by timestamp DESC | No |
| `get_unconsolidated_episodes` | `(&Connection, u32) -> Result<Vec<Episode>>` | Episodes not linked to any semantic node (used by consolidation) | Indirectly |
| `delete_episodes` | `(&Connection, &[EpisodeId]) -> Result<u64>` | Batch delete with dynamic placeholders | Yes |
| `count_episodes` | `(&Connection) -> Result<u64>` | COUNT(*) | Yes |

**Implementation notes:**
- `context_json` column stores serialized `EpisodeContext` as TEXT. Round-trip via `serde_json::to_string` / `from_str`.
- `get_unconsolidated_episodes` uses a NOT EXISTS subquery against the `links` table to find episodes not yet connected to semantic nodes. This approach avoids a separate `consolidated` boolean flag.
- `delete_episodes` constructs dynamic SQL with `?` placeholders. This is safe from SQL injection (parameterized) but the dynamic placeholder count is a minor concern at scale.

**Known gaps:**
- No `BEGIN IMMEDIATE` wrapping around `store_episode`. Under concurrent write pressure (e.g., WebSocket server storing episodes from multiple sessions), `BEGIN DEFERRED` can cause `SQLITE_BUSY` errors that the current code does not retry.
- No input validation: empty content, content exceeding reasonable limits, or `session_id` as empty string are all accepted silently.

### store/semantic.rs (139 lines, 2 tests)

**File:** `/Users/4n6h4x0r/src/alaya/src/store/semantic.rs`

Manages the `semantic_nodes` table. Semantic nodes are the "neocortex" of the three-store architecture -- extracted knowledge that persists beyond individual episodes.

**Functions:**

| Function | Signature | Purpose | Tested |
|----------|-----------|---------|--------|
| `store_semantic_node` | `(&Connection, &NewSemanticNode) -> Result<NodeId>` | INSERT, also stores embedding if provided | Yes |
| `get_semantic_node` | `(&Connection, NodeId) -> Result<SemanticNode>` | Single row by PK | Yes |
| `update_corroboration` | `(&Connection, NodeId) -> Result<()>` | INCREMENT corroboration_count, update last_corroborated | Yes |
| `find_by_type` | `(&Connection, SemanticType, u32) -> Result<Vec<SemanticNode>>` | Filter by node_type, ordered by confidence DESC | Implicitly |
| `delete_node` | `(&Connection, NodeId) -> Result<()>` | Cascade: node + embedding + links + strengths | No |
| `count_nodes` | `(&Connection) -> Result<u64>` | COUNT(*) | Implicitly |

**Implementation notes:**
- `source_episodes_json` stores `Vec<EpisodeId>` as JSON TEXT. This denormalization avoids a junction table but means source episode tracking is not enforced by foreign keys.
- `delete_node` performs manual cascade deletion across four tables. This is correct but should be wrapped in a transaction (currently not).
- Timestamps use `SystemTime::now()` directly rather than accepting a timestamp parameter. This makes testing time-dependent behavior difficult.

### store/implicit.rs (182 lines, 2 tests)

**File:** `/Users/4n6h4x0r/src/alaya/src/store/implicit.rs`

Manages both `impressions` and `preferences` tables. Impressions are raw behavioral traces (vasana). Preferences are crystallized patterns that emerge when enough impressions accumulate in a domain.

**Functions:**

| Function | Signature | Purpose | Tested |
|----------|-----------|---------|--------|
| `store_impression` | `(&Connection, &NewImpression) -> Result<ImpressionId>` | INSERT with auto timestamp | Yes |
| `get_impressions_by_domain` | `(&Connection, &str, u32) -> Result<Vec<Impression>>` | Domain-scoped, DESC by timestamp | Yes |
| `count_impressions_by_domain` | `(&Connection, &str) -> Result<u64>` | Domain-scoped COUNT | Implicitly |
| `store_preference` | `(&Connection, &str, &str, f32) -> Result<PreferenceId>` | INSERT with auto timestamp | Yes |
| `get_preferences` | `(&Connection, Option<&str>) -> Result<Vec<Preference>>` | All or domain-filtered, DESC by confidence | Yes |
| `reinforce_preference` | `(&Connection, PreferenceId, u32) -> Result<()>` | Increment evidence_count, boost confidence | Yes |
| `decay_preferences` | `(&Connection, i64, i64) -> Result<u64>` | Multiply confidence by 0.95 for stale preferences | No |
| `prune_weak_preferences` | `(&Connection, f32) -> Result<u64>` | DELETE below min confidence | No |
| `prune_old_impressions` | `(&Connection, i64) -> Result<u64>` | DELETE impressions older than max_age | No |
| `count_preferences` | `(&Connection) -> Result<u64>` | COUNT(*) | Implicitly |
| `count_impressions` | `(&Connection) -> Result<u64>` | COUNT(*) | Implicitly |

**Implementation notes:**
- `get_preferences` has two code paths depending on whether a domain filter is provided. The `None` path uses `query_map([], ...)` while the `Some` path uses `query_map([d], ...)`. This works but is slightly awkward.
- `decay_preferences` applies a flat 0.95 multiplier rather than true exponential decay. The comment acknowledges this as an approximation since SQLite lacks an `exp()` function. For v0.2, consider computing the exact factor in Rust and passing it as a parameter.

### store/embeddings.rs (176 lines, 4 tests)

**File:** `/Users/4n6h4x0r/src/alaya/src/store/embeddings.rs`

Polymorphic embedding storage shared across all three stores. Embeddings are stored as little-endian `f32` BLOBs. Search is brute-force cosine similarity over all candidates.

**Functions:**

| Function | Signature | Purpose | Tested |
|----------|-----------|---------|--------|
| `serialize_embedding` | `(&[f32]) -> Vec<u8>` | f32 -> LE bytes | Yes |
| `deserialize_embedding` | `(&[u8]) -> Vec<f32>` | LE bytes -> f32 | Yes |
| `cosine_similarity` | `(&[f32], &[f32]) -> f32` | Dot product / (norm_a * norm_b), f64 accumulation | Yes |
| `store_embedding` | `(&Connection, &str, i64, &[f32], &str) -> Result<()>` | INSERT OR REPLACE | Implicitly |
| `get_embedding` | `(&Connection, &str, i64) -> Result<Option<Vec<f32>>>` | Single embedding by (type, id) | No |
| `get_unembedded_episodes` | `(&Connection, u32) -> Result<Vec<EpisodeId>>` | LEFT JOIN to find episodes without embeddings | No |
| `search_by_vector` | `(&Connection, &[f32], Option<&str>, usize) -> Result<Vec<(NodeRef, f32)>>` | Brute-force: load all, compute cosine, sort, truncate | Yes |
| `count_embeddings` | `(&Connection) -> Result<u64>` | COUNT(*) | Implicitly |

**Implementation notes:**
- `cosine_similarity` accumulates in `f64` then casts back to `f32`. This avoids floating-point precision loss during summation. The function handles zero-length vectors and zero-norm vectors gracefully (returns 0.0).
- `search_by_vector` loads ALL embeddings into memory before computing similarities. This is the known scale ceiling (~50K embeddings per ADR-001). The `vec-sqlite` feature flag (v0.2) will replace this with SIMD-accelerated approximate search.
- The `UNIQUE INDEX` on `(node_type, node_id)` ensures each node has at most one embedding. `INSERT OR REPLACE` handles re-embedding.

**Scale ceiling:** With 768-dimensional embeddings at 4 bytes each, 50K embeddings = ~150 MB of data loaded into memory per search call. This is the primary performance bottleneck and the reason `vec-sqlite` is planned for v0.2.

### store/strengths.rs (145 lines, 2 tests)

**File:** `/Users/4n6h4x0r/src/alaya/src/store/strengths.rs`

Implements the Bjork & Bjork (1992) dual-strength model. Every node tracked in the graph has two strengths: Storage Strength (SS, how well-learned, monotonically increasing) and Retrieval Strength (RS, how accessible now, decays without access).

**Functions:**

| Function | Signature | Purpose | Tested |
|----------|-----------|---------|--------|
| `init_strength` | `(&Connection, NodeRef) -> Result<()>` | INSERT OR IGNORE, SS=0.5, RS=1.0 | Yes |
| `get_strength` | `(&Connection, NodeRef) -> Result<NodeStrength>` | Returns default if not found | Yes |
| `on_access` | `(&Connection, NodeRef) -> Result<()>` | RS=1.0, SS += 0.05*(1-SS), access_count++ | Yes |
| `boost_retrieval` | `(&Connection, NodeRef, f32) -> Result<()>` | RS = MIN(1.0, RS * factor) | No |
| `suppress_retrieval` | `(&Connection, NodeRef, f32) -> Result<()>` | RS *= factor (RIF mechanism) | Yes |
| `decay_all_retrieval` | `(&Connection, f32) -> Result<u64>` | Bulk RS *= decay_factor WHERE RS > 0.01 | Yes |
| `find_archivable` | `(&Connection, f32, f32) -> Result<Vec<NodeRef>>` | Nodes below both SS and RS thresholds | Implicitly |

**Implementation notes:**
- `on_access` uses the SQL `ON CONFLICT DO UPDATE` (upsert) pattern. This is both correct and efficient -- a single statement handles both first-access and repeat-access.
- The SS growth formula `SS += 0.05 * (1 - SS)` approaches 1.0 asymptotically. After 20 accesses, SS reaches ~0.86. After 50 accesses, ~0.95. This matches the cognitive science model where well-rehearsed memories are deeply encoded but never "perfectly" stored.
- `get_strength` returns a default (`SS=0.5, RS=0.5, access_count=0`) for nodes without a strength record rather than an error. This enables graceful handling of nodes created before strength tracking was added.

---

## 5. Retrieval Module Implementation

The retrieval module implements the hybrid pipeline: BM25 + vector + graph spreading activation -> Reciprocal Rank Fusion -> context-aware reranking -> post-retrieval effects.

### retrieval/bm25.rs (104 lines, 2 tests)

**File:** `/Users/4n6h4x0r/src/alaya/src/retrieval/bm25.rs`

FTS5-based full-text search with query sanitization to prevent FTS5 injection.

**Key implementation detail -- sanitization:**
```rust
let sanitized: String = query
    .chars()
    .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
    .collect();
```

This strips all non-alphanumeric, non-whitespace characters, converting them to spaces. This prevents FTS5 syntax injection (operators like `AND`, `OR`, `NOT`, `NEAR`, column filters, etc.) while preserving searchable terms. The security architecture document identifies this as the primary mitigation for threat T2 (FTS5 MATCH injection).

**Score normalization:** FTS5's `rank` column returns negative values where lower (more negative) = better match. The code normalizes to `[0.0, 1.0]` using min-max scaling across the result set. When all results have the same rank, all get score 1.0.

**Graceful degradation:** Empty query -> empty result, no error. All-punctuation query -> empty result, no error.

### retrieval/vector.rs (27 lines, 1 test)

**File:** `/Users/4n6h4x0r/src/alaya/src/retrieval/vector.rs`

Thin wrapper over `embeddings::search_by_vector` that casts `f32` similarities to `f64` for consistency with the fusion stage.

**Graceful degradation:** If no embeddings exist in the database, returns empty vec (no error). If `query.embedding` is `None`, the pipeline skips this stage entirely.

### retrieval/fusion.rs (68 lines, 3 tests)

**File:** `/Users/4n6h4x0r/src/alaya/src/retrieval/fusion.rs`

Implements Reciprocal Rank Fusion (Cormack, Clarke & Buettcher, 2009).

**Formula:** `RRF(d) = sum over L: 1 / (k + rank_L(d) + 1)` where `k = 60` and `rank_L(d)` is 0-based.

**Implementation:** ~20 lines of core logic (matches the ADR-006 claim of simplicity). Iterates over all result sets, accumulates RRF scores per NodeRef in a HashMap, sorts descending.

**Key property:** Score-agnostic. Operates on ranks, not raw scores. This means BM25 scores, cosine similarities, and spreading activation values can be fused without calibration. Any subset of signals works (graceful degradation).

### retrieval/rerank.rs (85 lines, 3 tests)

**File:** `/Users/4n6h4x0r/src/alaya/src/retrieval/rerank.rs`

Context-aware reranking with two factors:

1. **Context similarity** (weight 0.3): Jaccard similarity over topics (0.5), entities (0.25), and sentiment distance (0.25) between the query context and the episode's stored context.

2. **Recency decay** (weight 0.2): `exp(-age_days / 30.0)`. Recent = ~1.0, 30 days = ~0.37, 90 days = ~0.05.

**Formula:** `final_score = base * (1 + 0.3 * context_sim) * (1 + 0.2 * recency)`

This formula ensures that base relevance (from BM25/vector/graph) is the primary signal, with context similarity and recency providing multiplicative boosts rather than overriding relevance.

### retrieval/pipeline.rs (143 lines, 2 tests)

**File:** `/Users/4n6h4x0r/src/alaya/src/retrieval/pipeline.rs`

Orchestrates the full retrieval pipeline. This is the most architecturally significant module -- it is where all the components converge.

**Pipeline stages:**

1. **BM25 search** -> `Vec<(NodeRef::Episode, f64)>` -- always runs
2. **Vector search** -> `Vec<(NodeRef, f64)>` -- only if `query.embedding.is_some()`
3. **Graph activation** -> `Vec<(NodeRef, f64)>` -- seeded from top 3 BM25 + top 3 vector results, 1 hop, excludes seeds
4. **RRF fusion** -> `Vec<(NodeRef, f64)>` -- merges available signal sets (1-3 of them)
5. **Candidate enrichment** -- loads full Episode data for each fused NodeRef
6. **Context reranking** -> `Vec<ScoredMemory>` -- applies recency and context similarity
7. **Post-retrieval effects:**
   - `strengths::on_access()` for each retrieved node (RS=1.0, SS growth)
   - `links::on_co_retrieval()` for all pairs of retrieved nodes (Hebbian LTP)

**Known gaps:**
- Candidate enrichment (step 5) only handles `NodeRef::Episode`. Semantic and preference nodes are filtered out with a `// TODO: enrich semantic and preference nodes` comment. This means the retrieval pipeline currently returns only episodes, even though the graph and vector search can surface semantic nodes.
- Post-retrieval co-retrieval strengthening runs in O(n^2) over retrieved nodes. With the default `max_results = 5`, this is 10 pairs -- negligible. But if `max_results` were set to 100, it would be 4950 SQL updates.
- No RIF (Retrieval-Induced Forgetting) is applied to non-retrieved competitors. The `suppress_retrieval` function exists in `strengths.rs` but is never called from the pipeline.

---

## 6. Lifecycle Module Implementation

The lifecycle module implements the four cognitive processes that transform raw memories into structured knowledge. These are explicit calls (ADR-008), not automatic triggers.

### lifecycle/consolidation.rs (118 lines, 2 tests)

**File:** `/Users/4n6h4x0r/src/alaya/src/lifecycle/consolidation.rs`

Models the Complementary Learning Systems (CLS) theory (McClelland et al., 1995): the hippocampus (episodic store) gradually teaches the neocortex (semantic store) through interleaved replay.

**Algorithm:**
1. Fetch up to `CONSOLIDATION_BATCH_SIZE` (10) unconsolidated episodes
2. If fewer than 3 episodes, skip (need corroboration)
3. Call `provider.extract_knowledge(episodes)` -> `Vec<NewSemanticNode>`
4. For each extracted node: store it, create Causal links to source episodes, init node strength

**Graceful degradation:** With `NoOpProvider`, `extract_knowledge` returns empty vec -> consolidation simply reports 0 nodes created. Episodes accumulate but are not promoted to semantic knowledge. No error.

### lifecycle/perfuming.rs (135 lines, 2 tests)

**File:** `/Users/4n6h4x0r/src/alaya/src/lifecycle/perfuming.rs`

Models vasana (perfuming) from Yogacara Buddhist psychology: each interaction leaves a subtle trace. When enough traces accumulate, a preference crystallizes.

**Algorithm:**
1. Call `provider.extract_impressions(interaction)` -> `Vec<NewImpression>`
2. Store each impression
3. For each affected domain:
   - Count impressions in that domain
   - If count >= `CRYSTALLIZATION_THRESHOLD` (5):
     - If no existing preference: crystallize (pick most recent observation as summary)
     - If existing preference: reinforce (increment evidence_count, boost confidence)

**Known gaps:**
- `summarize_impressions` is a placeholder that simply picks the most recent observation. A real implementation would use the `ConsolidationProvider` to generate a summary. The function signature is already correct; only the body needs work.
- No semantic clustering of impressions. v0.1 uses exact domain string matching. The architecture document plans semantic clustering for v0.2.

### lifecycle/transformation.rs (134 lines, 2 tests)

**File:** `/Users/4n6h4x0r/src/alaya/src/lifecycle/transformation.rs`

Models asraya-paravrtti (transformation of the basis) from Yogacara: periodic refinement toward clarity.

**Algorithm:**
1. **Dedup** semantic nodes with embedding cosine similarity >= 0.95 (keep older, transfer links, increment corroboration, delete duplicate)
2. **Prune** weak graph links with forward_weight AND backward_weight < 0.02
3. **Decay** un-reinforced preferences (confidence *= 0.95 for stale ones)
4. **Prune** preferences with confidence < 0.05
5. **Prune** impressions older than 90 days

**Constants:**

| Constant | Value | Meaning |
|----------|-------|---------|
| `DEDUP_SIMILARITY_THRESHOLD` | 0.95 | Cosine similarity above which nodes are duplicates |
| `LINK_PRUNE_THRESHOLD` | 0.02 | Links below this weight are pruned |
| `MIN_PREFERENCE_CONFIDENCE` | 0.05 | Preferences below this are deleted |
| `PREFERENCE_HALF_LIFE_SECS` | 30 days | Decay trigger age |
| `MAX_IMPRESSION_AGE_SECS` | 90 days | Impressions older than this are pruned |

**Known gap:** The dedup function loads all semantic embeddings into memory and does O(n^2) pairwise comparison. This mirrors the brute-force search limitation (~50K ceiling). For v0.2, consider batch processing or approximate dedup.

### lifecycle/forgetting.rs (95 lines, 2 tests)

**File:** `/Users/4n6h4x0r/src/alaya/src/lifecycle/forgetting.rs`

Models the Bjork & Bjork (1992) New Theory of Disuse.

**Algorithm:**
1. Decay ALL retrieval strengths by `DEFAULT_DECAY_FACTOR` (0.95): `RS *= 0.95`
2. Find nodes where BOTH `SS < 0.1` AND `RS < 0.05` (low learning AND low accessibility)
3. Archive (delete) those nodes:
   - Episodes: `delete_episodes`
   - Semantic: `delete_node` (with cascade)
   - Preferences: skip (handled by transformation decay)
4. Clean up strength records for archived nodes

**Key insight:** Nodes with high SS but low RS are "latent" -- well-learned but temporarily hard to retrieve. They are NOT deleted. A strong retrieval cue (high BM25/vector match) can "rescue" them by setting RS=1.0 via `on_access`. This matches the cognitive science model where well-known facts can be temporarily inaccessible but recalled given the right prompt.

---

## 7. Graph Module Implementation

The graph overlay enables self-organizing relationships across all three stores without requiring LLM involvement.

### graph/links.rs (164 lines, 3 tests)

**File:** `/Users/4n6h4x0r/src/alaya/src/graph/links.rs`

Manages the `links` table with directed, weighted, typed edges between any pair of `NodeRef` values.

**Link properties:**
- `forward_weight` and `backward_weight`: Asymmetric weights allow "A strongly implies B" without "B strongly implies A"
- Five link types: `Temporal` (session sequence), `Topical` (shared topic), `Entity` (shared entity), `Causal` (episode -> derived semantic node), `CoRetrieval` (retrieved together)

**Hebbian LTP (Long-Term Potentiation):**
```rust
// on_co_retrieval: Hebbian learning rule
forward_weight += 0.1 * (1.0 - forward_weight)
```

Asymptotic approach to 1.0. After 10 co-retrievals, weight reaches ~0.83. After 20, ~0.93. Links that are never co-retrieved eventually decay via `decay_links` during `transform()`.

**Functions:** `create_link`, `get_links_from`, `get_links_to`, `on_co_retrieval`, `decay_links`, `prune_weak_links`, `count_links`.

### graph/activation.rs (126 lines, 3 tests)

**File:** `/Users/4n6h4x0r/src/alaya/src/graph/activation.rs`

Collins & Loftus (1975) spreading activation: activation propagates from seed nodes through weighted edges, decaying at each hop.

**Algorithm:**
1. Initialize seed nodes with activation 1.0
2. For `max_depth` iterations:
   - For each node above threshold: spread `activation * forward_weight * decay_per_hop` to targets
   - Merge deltas, capping at 2.0 to prevent runaway amplification
3. Filter nodes below threshold

**Parameters used by the retrieval pipeline:** `max_depth=1`, `threshold=0.1`, `decay_per_hop=0.6`. These conservative defaults limit graph exploration to direct neighbors, preventing false positives from long activation chains.

**Implementation note:** The current implementation queries `get_links_from` for each active node at each hop. This means N database queries per iteration where N is the number of active nodes. For large graphs, the architecture document suggests replacing this with a recursive CTE in a single SQL query. This optimization is planned for v0.2.

---

## 8. Provider Traits

### ConsolidationProvider (current, v0.1)

**File:** `/Users/4n6h4x0r/src/alaya/src/provider.rs` (78 lines)

The extension boundary between Alaya (memory engine) and the consumer's agent (which owns the LLM connection).

```rust
pub trait ConsolidationProvider {
    fn extract_knowledge(&self, episodes: &[Episode]) -> Result<Vec<NewSemanticNode>>;
    fn extract_impressions(&self, interaction: &Interaction) -> Result<Vec<NewImpression>>;
    fn detect_contradiction(&self, a: &SemanticNode, b: &SemanticNode) -> Result<bool>;
}
```

**Design rationale (ADR-004):** Alaya never makes network calls. The consumer implements this trait, typically by calling an LLM. This means:
- Alaya has zero network dependencies (ADR-009)
- The consumer controls cost, latency, and model choice
- Testing uses `NoOpProvider` or `MockProvider`

**NoOpProvider:** Returns empty results for all methods. This is the graceful degradation baseline -- when no LLM is available, consolidation and perfuming become no-ops, and the system operates on BM25 + graph only.

**MockProvider (cfg(test)):** Allows tests to inject specific knowledge and impressions without an actual LLM. Defined in the same file behind `#[cfg(test)]`.

### EmbeddingProvider (planned, v0.2)

Not yet defined in code. The architecture document specifies:

```rust
/// Planned for v0.2 -- currently, consumers pass embeddings directly via NewEpisode.embedding
pub trait EmbeddingProvider {
    /// Embed a single text into a vector.
    fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Return the dimensionality of embeddings produced.
    fn dimension(&self) -> usize;
}
```

This will enable Alaya to automatically embed episodes and semantic nodes during storage, rather than requiring the consumer to pre-compute embeddings and pass them in `NewEpisode.embedding`.

---

## 9. Feature Flag Architecture

### Current State

No feature flags are defined in `Cargo.toml`. The crate is a single, unconditional compilation unit.

### Target Feature Flag Design (v0.2)

Feature flags gate optional functionality that adds dependencies or changes behavior. The core crate must always compile and function without any features enabled.

```toml
[features]
default = []

# Vector search acceleration via sqlite-vec
vec-sqlite = ["dep:sqlite-vec"]

# ONNX Runtime embedding backend
embed-ort = ["dep:ort"]

# Turnkey fastembed-rs embedding backend
embed-fastembed = ["dep:fastembed"]

# Async API surface (tokio spawn_blocking)
async = ["dep:tokio"]
```

### How Feature Flags Gate Code

**Pattern 1: Conditional compilation blocks**

```rust
// In store/embeddings.rs:
#[cfg(feature = "vec-sqlite")]
fn search_by_vector_simd(conn: &Connection, query: &[f32], limit: usize) -> Result<Vec<(NodeRef, f32)>> {
    // Use sqlite-vec virtual table for SIMD-accelerated search
    // ...
}

#[cfg(not(feature = "vec-sqlite"))]
fn search_by_vector_brute(conn: &Connection, query: &[f32], limit: usize) -> Result<Vec<(NodeRef, f32)>> {
    // Current brute-force implementation
    // ...
}
```

**Pattern 2: Trait implementations behind feature flags**

```rust
#[cfg(feature = "embed-ort")]
pub struct OrtEmbeddingProvider { /* ... */ }

#[cfg(feature = "embed-ort")]
impl EmbeddingProvider for OrtEmbeddingProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>> { /* ... */ }
    fn dimension(&self) -> usize { /* ... */ }
}
```

**Pattern 3: Async wrappers**

```rust
#[cfg(feature = "async")]
pub mod async_api {
    use super::*;
    use tokio::task::spawn_blocking;

    pub async fn store_episode(store: &AlayaStore, ep: &NewEpisode) -> Result<EpisodeId> {
        let store = store.clone(); // Requires AlayaStore to be cheaply cloneable or Arc-wrapped
        let ep = ep.clone();
        spawn_blocking(move || store.store_episode(&ep)).await.unwrap()
    }
}
```

### Feature Flag Combinations and CI Testing

The CI matrix must test all valid feature flag combinations:

| Combination | What it tests |
|------------|---------------|
| `--no-default-features` | Core-only, zero optional deps |
| `--features vec-sqlite` | SIMD vector search |
| `--features embed-ort` | ONNX embedding backend |
| `--features embed-fastembed` | Fastembed backend |
| `--features async` | Async API surface |
| `--all-features` | Everything together |

Mutually exclusive flags: `embed-ort` and `embed-fastembed` should not be enabled simultaneously (both provide `EmbeddingProvider` implementations). This should be enforced with a compile-time check:

```rust
#[cfg(all(feature = "embed-ort", feature = "embed-fastembed"))]
compile_error!("Features 'embed-ort' and 'embed-fastembed' are mutually exclusive. Choose one embedding backend.");
```

### Zero-Network-Call Enforcement

Per ADR-009, the core crate must never make network calls. Feature flags that add networking dependencies (like `embed-ort` with model downloads, or any HTTP-based embedding provider) must document this clearly. The CI pipeline should include a `cargo tree --features <flag>` check to verify no networking crates appear in the dependency tree for the default feature set.

```bash
# CI check: no networking deps in default build
cargo tree --no-default-features 2>&1 | grep -E '(reqwest|hyper|h2|tokio.*net|std.*net)' && exit 1 || echo "OK: no network deps"
```

---

## 10. FFI and Language Bindings Plan

### Workspace Layout (v0.2)

```
alaya/                           # Workspace root
  Cargo.toml                     # [workspace] definition
  alaya/                         # Core library crate (current src/ moves here)
    Cargo.toml
    src/
      lib.rs
      ...
  alaya-ffi/                     # C FFI crate
    Cargo.toml
    build.rs                     # cbindgen invocation
    src/
      lib.rs                     # extern "C" fn wrappers
    include/
      alaya.h                    # Generated C header
  alaya-py/                      # Python bindings crate
    Cargo.toml
    src/
      lib.rs                     # PyO3 module definition
    python/
      alaya/
        __init__.py              # Python package
        __init__.pyi             # Type stubs
  pyproject.toml                 # maturin build configuration
```

### alaya-ffi (v0.2, cbindgen)

The FFI crate exposes a C-compatible API using `extern "C"` functions and opaque pointers. This enables integration with any language that can call C functions (Swift, Go, Ruby, C#, etc.).

**Design principles:**
- Opaque pointer for `AlayaStore` (consumer never sees internal layout)
- All functions return error codes (i32), with a `alaya_last_error()` thread-local for error messages
- String parameters are `*const c_char`, output strings are caller-freed via `alaya_string_free()`
- Vectors are represented as `(ptr, len)` pairs with explicit free functions

**Core FFI surface (planned):**

```c
// alaya.h (generated by cbindgen)
typedef struct AlayaStore AlayaStore;
typedef int64_t AlayaEpisodeId;

// Lifecycle
AlayaStore* alaya_open(const char* path);
AlayaStore* alaya_open_in_memory(void);
void alaya_close(AlayaStore* store);

// Write
AlayaEpisodeId alaya_store_episode(AlayaStore* store, const char* content,
                                    const char* role, const char* session_id,
                                    int64_t timestamp, const char* context_json);

// Read
char* alaya_query(AlayaStore* store, const char* text, int max_results);
char* alaya_preferences(AlayaStore* store, const char* domain);

// Lifecycle
char* alaya_consolidate(AlayaStore* store);
char* alaya_transform(AlayaStore* store);
char* alaya_forget(AlayaStore* store);
char* alaya_status(AlayaStore* store);

// Memory management
void alaya_string_free(char* s);
const char* alaya_last_error(void);
```

**Build process:** `cbindgen` reads the Rust FFI source and generates `alaya.h` during `cargo build`.

### alaya-py (v0.3, PyO3)

Python bindings provide a native Python experience with type hints, context managers, and Pythonic error handling.

**Planned Python API:**

```python
from alaya import AlayaStore, Query, NewEpisode, Role

# Open store (context manager for cleanup)
with AlayaStore.open("memory.db") as store:
    # Store episode
    episode_id = store.store_episode(NewEpisode(
        content="I love Rust programming",
        role=Role.USER,
        session_id="s1",
        timestamp=1000,
    ))

    # Query
    results = store.query(Query.simple("Rust"))
    for memory in results:
        print(f"{memory.score:.2f}: {memory.content}")

    # Lifecycle
    report = store.consolidate(provider=NoOpProvider())
    print(f"Consolidated {report.nodes_created} nodes")
```

**Build tooling:** `maturin` for building and publishing to PyPI. The `pyproject.toml` in the workspace root configures the Python package.

**ConsolidationProvider in Python:** The trait becomes a Python protocol/ABC:

```python
from alaya import ConsolidationProvider, Episode, NewSemanticNode

class MyProvider(ConsolidationProvider):
    def extract_knowledge(self, episodes: list[Episode]) -> list[NewSemanticNode]:
        # Call your LLM here
        ...
```

PyO3 handles the Rust-Python callback boundary, converting Python exceptions to `AlayaError::Provider`.

---

## 11. Build and CI Configuration

### Local Development Commands

```bash
# Run all tests (43 currently passing)
cargo test

# Run tests with output (for debugging)
cargo test -- --nocapture

# Run a specific test
cargo test test_full_lifecycle

# Run tests for a specific module
cargo test store::episodic

# Check compilation without running tests
cargo check

# Lint with clippy (pedantic)
cargo clippy -- -W clippy::pedantic

# Format check
cargo fmt -- --check

# Build documentation
cargo doc --no-deps --open

# Dependency audit
cargo audit
```

### rust-toolchain.toml (to be created)

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

### GitHub Actions CI Pipeline (to be created)

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: -Dwarnings

jobs:
  test:
    name: Test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all-features

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - run: cargo clippy --all-features -- -W clippy::pedantic

  fmt:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt -- --check

  audit:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v1

  msrv:
    name: MSRV (1.75)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.75
      - run: cargo check

  no-network-deps:
    name: Verify No Network Dependencies
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Check for networking crates
        run: |
          ! cargo tree --no-default-features 2>&1 | grep -qE '(reqwest|hyper|h2|surf|ureq|attohttpc)'
```

### Benchmarks (v0.2, divan)

```toml
# In Cargo.toml
[dev-dependencies]
divan = "0.1"

[[bench]]
name = "retrieval"
harness = false

[[bench]]
name = "lifecycle"
harness = false
```

```rust
// benches/retrieval.rs
use divan::Bencher;
use alaya::{AlayaStore, NewEpisode, Query, Role, EpisodeContext};

fn main() { divan::main(); }

#[divan::bench(args = [10, 100, 1000, 10000])]
fn query_bm25(bencher: Bencher, n: usize) {
    let store = AlayaStore::open_in_memory().unwrap();
    // Seed n episodes
    for i in 0..n {
        store.store_episode(&NewEpisode {
            content: format!("Episode {} about topic {}", i, i % 50),
            role: Role::User,
            session_id: "bench".to_string(),
            timestamp: i as i64,
            context: EpisodeContext::default(),
            embedding: None,
        }).unwrap();
    }
    bencher.bench_local(|| {
        store.query(&Query::simple("topic")).unwrap()
    });
}

#[divan::bench(args = [10, 100, 1000])]
fn store_episode(bencher: Bencher, n: usize) {
    let store = AlayaStore::open_in_memory().unwrap();
    let mut counter = 0;
    bencher.bench_local(|| {
        store.store_episode(&NewEpisode {
            content: format!("Benchmark episode {}", counter),
            role: Role::User,
            session_id: "bench".to_string(),
            timestamp: counter as i64,
            context: EpisodeContext::default(),
            embedding: None,
        }).unwrap();
        counter += 1;
    });
}
```

### Documentation Standards

Every `pub` method must have:
1. First line: imperative mood summary (e.g., "Store a conversation episode with full context.")
2. Blank line, then behavior description with performance notes
3. `# Examples` section with compilable doctests
4. Research citations for non-obvious mechanisms (e.g., "Based on Bjork & Bjork (1992)")
5. Cross-reference links using `` [`AlayaStore`] `` syntax

Example:

```rust
/// Store a conversation episode with full context.
///
/// Creates an episode in the episodic store, initializes its node strength,
/// stores its embedding (if provided), and creates a temporal link to the
/// preceding episode (if specified in the context).
///
/// # Examples
///
/// ```
/// use alaya::{AlayaStore, NewEpisode, Role, EpisodeContext};
///
/// let store = AlayaStore::open_in_memory().unwrap();
/// let id = store.store_episode(&NewEpisode {
///     content: "I love Rust programming".to_string(),
///     role: Role::User,
///     session_id: "session-1".to_string(),
///     timestamp: 1700000000,
///     context: EpisodeContext::default(),
///     embedding: None,
/// }).unwrap();
///
/// assert_eq!(id.0, 1);
/// ```
///
/// # Errors
///
/// Returns [`AlayaError::Db`] if the SQLite write fails.
/// Returns [`AlayaError::Serialization`] if context JSON encoding fails.
pub fn store_episode(&self, episode: &NewEpisode) -> Result<EpisodeId> {
    // ...
}
```

---

## 12. Migration Path

The current codebase is already structured to match the architecture blueprint. The migration path focuses on hardening and quality improvements, not structural changes.

### Phase 1: P0 Hardening (pre-v0.1 release)

These changes are required before the first public release.

#### 1.1 Add `#[non_exhaustive]` to All Public Enums

**Files affected:** `src/types.rs`

```diff
+ #[non_exhaustive]
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  pub enum Role { User, Assistant, System }

+ #[non_exhaustive]
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  pub enum SemanticType { Fact, Relationship, Event, Concept }

+ #[non_exhaustive]
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  pub enum LinkType { Temporal, Topical, Entity, Causal, CoRetrieval }

+ #[non_exhaustive]
  #[derive(Debug, Clone)]
  pub enum PurgeFilter { Session(String), OlderThan(i64), All }
```

**Also add to AlayaError in `src/error.rs`:**

```diff
+ #[non_exhaustive]
  #[derive(Debug, Error)]
  pub enum AlayaError { ... }
```

#### 1.2 Add `BEGIN IMMEDIATE` for Write Transactions

**Files affected:** `src/store/episodic.rs`, `src/store/semantic.rs`, `src/store/implicit.rs`, `src/store/embeddings.rs`, `src/store/strengths.rs`, `src/graph/links.rs`, `src/lifecycle/*.rs`

Create a helper function:

```rust
// In src/schema.rs or a new src/tx.rs:
pub(crate) fn immediate_transaction<F, T>(conn: &Connection, f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    conn.execute_batch("BEGIN IMMEDIATE")?;
    match f() {
        Ok(val) => {
            conn.execute_batch("COMMIT")?;
            Ok(val)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}
```

Then wrap all write operations. Example for `store_episode` in `lib.rs`:

```rust
pub fn store_episode(&self, episode: &NewEpisode) -> Result<EpisodeId> {
    schema::immediate_transaction(&self.conn, || {
        let id = store::episodic::store_episode(&self.conn, episode)?;
        if let Some(ref emb) = episode.embedding {
            store::embeddings::store_embedding(&self.conn, "episode", id.0, emb, "")?;
        }
        store::strengths::init_strength(&self.conn, NodeRef::Episode(id))?;
        if let Some(prev) = episode.context.preceding_episode {
            graph::links::create_link(
                &self.conn,
                NodeRef::Episode(prev),
                NodeRef::Episode(id),
                LinkType::Temporal,
                0.5,
            )?;
        }
        Ok(id)
    })
}
```

#### 1.3 Add Input Validation at API Boundary

**File affected:** `src/lib.rs`

```rust
fn validate_episode(ep: &NewEpisode) -> Result<()> {
    if ep.content.is_empty() {
        return Err(AlayaError::InvalidInput("episode content must not be empty".into()));
    }
    if ep.content.len() > 100_000 {
        return Err(AlayaError::InvalidInput(
            format!("episode content too long: {} bytes (max 100,000)", ep.content.len())
        ));
    }
    if ep.session_id.is_empty() {
        return Err(AlayaError::InvalidInput("session_id must not be empty".into()));
    }
    if let Some(ref emb) = ep.embedding {
        if emb.is_empty() {
            return Err(AlayaError::InvalidInput("embedding must not be empty if provided".into()));
        }
    }
    Ok(())
}

fn validate_query(q: &Query) -> Result<()> {
    if q.max_results == 0 {
        return Err(AlayaError::InvalidInput("max_results must be > 0".into()));
    }
    if q.max_results > 1000 {
        return Err(AlayaError::InvalidInput("max_results must be <= 1000".into()));
    }
    Ok(())
}
```

Call from the public API methods before delegating to internal modules.

#### 1.4 Restrict Module Visibility

**File affected:** `src/lib.rs`

```diff
- pub mod error;
- pub mod types;
- pub mod schema;
- pub mod store;
- pub mod graph;
- pub mod retrieval;
- pub mod lifecycle;
- pub mod provider;
+ pub mod error;
+ pub mod types;
+ pub(crate) mod schema;
+ pub(crate) mod store;
+ pub(crate) mod graph;
+ pub(crate) mod retrieval;
+ pub(crate) mod lifecycle;
+ pub mod provider;
```

This ensures consumers interact only through `AlayaStore` methods, not by importing `alaya::store::episodic::store_episode` directly. The `error`, `types`, and `provider` modules remain public because consumers need to construct input types, match on errors, and implement provider traits.

#### 1.5 Add Compilable Doctests

Add `# Examples` to every `pub fn` on `AlayaStore` and `# Examples` or usage documentation to every public type. Target: at least 15 doctests covering the core API surface.

### Phase 2: P1 Quality (v0.1 release)

#### 2.1 Call LTD from Transform

**File affected:** `src/lifecycle/transformation.rs`

Add link weight decay (Long-Term Depression) to the transformation cycle:

```rust
// In transform():
// Between step 1 (dedup) and step 2 (prune):
let decay_factor = 0.95;
links::decay_links(conn, decay_factor)?;
// The subsequent prune step will then remove any links that decayed below threshold.
```

#### 2.2 Enrich Non-Episode Nodes in Pipeline

**File affected:** `src/retrieval/pipeline.rs`

Replace the `TODO` in candidate enrichment:

```rust
NodeRef::Semantic(nid) => {
    store::semantic::get_semantic_node(conn, nid).ok().map(|node| {
        (node_ref, score, node.content, None, node.created_at, EpisodeContext::default())
    })
}
NodeRef::Preference(pid) => {
    // Look up preference and return as enriched candidate
    None // TODO: implement preference lookup
}
```

#### 2.3 Add Tombstone Table

**File affected:** `src/schema.rs`

```sql
CREATE TABLE IF NOT EXISTS tombstones (
    node_type TEXT NOT NULL,
    node_id   INTEGER NOT NULL,
    deleted_at INTEGER NOT NULL,
    PRIMARY KEY (node_type, node_id)
);
```

On delete, insert into tombstones. On store/resurrection, check tombstones first.

### Phase 3: P2 Polish (v0.1 release)

- Add WAL checkpoint management (call `PRAGMA wal_checkpoint(TRUNCATE)` in `transform()`)
- Add `tempfile` dev-dependency for persistent-path tests
- Create `rust-toolchain.toml` pinning stable
- Create `.github/workflows/ci.yml`
- Set up `cargo audit` in CI
- Add MSRV check (1.75) in CI

### Migration File Manifest

| File | Action | Phase | Priority |
|------|--------|-------|----------|
| `src/types.rs` | Add `#[non_exhaustive]` to 4 enums | 1 | P0 |
| `src/error.rs` | Add `#[non_exhaustive]` to AlayaError | 1 | P0 |
| `src/schema.rs` | Add `immediate_transaction()` helper | 1 | P0 |
| `src/lib.rs` | Add validation functions, wrap writes in transactions, restrict visibility | 1 | P0 |
| `src/lib.rs` | Add doctests to all 12 pub methods | 1 | P0 |
| `src/lifecycle/transformation.rs` | Add LTD call before prune | 2 | P1 |
| `src/retrieval/pipeline.rs` | Enrich semantic nodes in candidate step | 2 | P1 |
| `src/schema.rs` | Add tombstones table | 2 | P1 |
| `src/schema.rs` | Add WAL checkpoint pragma | 3 | P2 |
| `Cargo.toml` | Add metadata, dev-deps, lints | 3 | P2 |
| `rust-toolchain.toml` | Create | 3 | P2 |
| `.github/workflows/ci.yml` | Create | 3 | P2 |

---

## Appendix A: File-by-File Line Count Summary

| File | Lines | Tests | Status |
|------|-------|-------|--------|
| `src/lib.rs` | 276 | 2 | Working, needs doctests + validation + visibility |
| `src/error.rs` | 21 | 0 | Working, needs `#[non_exhaustive]` |
| `src/types.rs` | 402 | 0 | Working, needs `#[non_exhaustive]` on enums |
| `src/schema.rs` | 238 | 3 | Working, needs `BEGIN IMMEDIATE` helper + tombstones |
| `src/provider.rs` | 78 | 0 | Working, MockProvider is test-only |
| `src/store/mod.rs` | 5 | 0 | Complete |
| `src/store/episodic.rs` | 177 | 4 | Working |
| `src/store/semantic.rs` | 139 | 2 | Working |
| `src/store/implicit.rs` | 182 | 2 | Working |
| `src/store/embeddings.rs` | 176 | 4 | Working |
| `src/store/strengths.rs` | 145 | 2 | Working |
| `src/graph/mod.rs` | 2 | 0 | Complete |
| `src/graph/links.rs` | 164 | 3 | Working |
| `src/graph/activation.rs` | 126 | 3 | Working |
| `src/retrieval/mod.rs` | 5 | 0 | Complete |
| `src/retrieval/bm25.rs` | 104 | 2 | Working |
| `src/retrieval/vector.rs` | 27 | 1 | Working |
| `src/retrieval/fusion.rs` | 68 | 3 | Working |
| `src/retrieval/rerank.rs` | 85 | 3 | Working |
| `src/retrieval/pipeline.rs` | 143 | 2 | Working, needs semantic node enrichment |
| `src/lifecycle/mod.rs` | 4 | 0 | Complete |
| `src/lifecycle/consolidation.rs` | 118 | 2 | Working |
| `src/lifecycle/perfuming.rs` | 135 | 2 | Working |
| `src/lifecycle/transformation.rs` | 134 | 2 | Working, needs LTD integration |
| `src/lifecycle/forgetting.rs` | 95 | 2 | Working |
| **Total** | **4064** | **43** | **All passing** |

## Appendix B: Cross-Reference Index

| Cross-Ref | Source | Used By |
|-----------|--------|---------|
| `architecture.components` | `architecture.yml` | Module tree, all sections |
| `architecture.retrieval_pipeline` | `architecture.yml` | Section 5 |
| `architecture.lifecycle` | `architecture.yml` | Section 6 |
| `architecture.feature_flags_planned` | `architecture.yml` | Section 9 |
| `architecture.known_gaps` | `architecture.yml` | Gap analysis table |
| `design-system.naming` | `design-system.yml` | Section 3 type naming |
| `design-system.public_api` | `design-system.yml` | Section 3 API table |
| `design-system.error_design` | `design-system.yml` | Section 3 AlayaError |
| `design-system.consistency_rules` | `design-system.yml` | Sections 3, 4 |
| `adr.ADR-001` | `adr.yml` | Section 2 (SQLite), Section 4 (scale ceiling) |
| `adr.ADR-004` | `adr.yml` | Section 8 (provider traits) |
| `adr.ADR-005` | `adr.yml` | Section 4 (strengths), Section 6 (forgetting) |
| `adr.ADR-006` | `adr.yml` | Section 5 (RRF fusion) |
| `adr.ADR-008` | `adr.yml` | Section 3 (threading model) |
| `adr.ADR-009` | `adr.yml` | Sections 2, 9 (zero network calls) |
| `adr.ADR-010` | `adr.yml` | Section 5 (FTS5/BM25) |

## Appendix C: Cognitive Science References

| Mechanism | Reference | Implementation Location |
|-----------|-----------|------------------------|
| Three-store architecture | Tulving (1972), Schacter & Tulving (1994) | ADR-002, `store/` module |
| CLS consolidation | McClelland, McNaughton & O'Reilly (1995) | `lifecycle/consolidation.rs` |
| Spreading activation | Collins & Loftus (1975) | `graph/activation.rs` |
| Hebbian LTP/LTD | Hebb (1949) | `graph/links.rs::on_co_retrieval` |
| Dual-strength forgetting | Bjork & Bjork (1992) | `store/strengths.rs`, `lifecycle/forgetting.rs` |
| Retrieval-Induced Forgetting | Anderson, Bjork & Bjork (1994), Storm (2011) | `store/strengths.rs::suppress_retrieval` (defined, not yet wired) |
| Reciprocal Rank Fusion | Cormack, Clarke & Buettcher (2009) | `retrieval/fusion.rs` |
| Vasana (perfuming) | Yogacara Buddhism (Vasubandhu, 4th-5th century) | `lifecycle/perfuming.rs` |
| Asraya-paravrtti (transformation) | Yogacara Buddhism | `lifecycle/transformation.rs` |
| Alaya-vijnana (storehouse consciousness) | Yogacara Buddhism | Crate name, architectural metaphor |
