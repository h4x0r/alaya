# Alaya API Design System

Alaya is a Rust crate. The API surface is the developer's user interface. This document defines the naming conventions, type design, method patterns, documentation standards, and diagnostic output that make Alaya's API consistent, predictable, and self-documenting.

Every rule below is grounded in the actual codebase. Code examples compile against the current `AlayaStore` API.

**Cross-references:** [Brand Guidelines](../BRAND_GUIDELINES.md) | [North Star](../NORTHSTAR.md) | [North Star Extract](../NORTHSTAR_EXTRACT.md) | [Developer Journeys](USER_JOURNEYS.md) | [Competitive Landscape](../COMPETITIVE_LANDSCAPE.md)

---

## Table of Contents

1. [API Design Tokens](#1-api-design-tokens)
2. [Type Design System](#2-type-design-system)
3. [API Patterns](#3-api-patterns)
4. [Documentation Patterns](#4-documentation-patterns)
5. [Code Example Progression](#5-code-example-progression)
6. [Diagnostic Output Design](#6-diagnostic-output-design)
7. [Consistency Rules](#7-consistency-rules)
8. [Error Message Design](#8-error-message-design)
9. [API Evolution Guidelines](#9-api-evolution-guidelines)

---

## 1. API Design Tokens

Design tokens are the atomic naming and structural conventions that unify the entire API surface. They are to a Rust library what color tokens are to a design system: the lowest-level primitives that everything else builds on.

### Naming Conventions

| Category | Convention | Examples |
|----------|-----------|----------|
| Functions and methods | `snake_case` | `store_episode`, `query`, `consolidate`, `open_in_memory` |
| Types and structs | `CamelCase` | `AlayaStore`, `NewEpisode`, `ScoredMemory`, `ConsolidationReport` |
| Enum variants | `CamelCase` | `Role::User`, `LinkType::Temporal`, `PurgeFilter::All` |
| Constants | `SCREAMING_SNAKE_CASE` | `CONSOLIDATION_BATCH_SIZE`, `DEFAULT_DECAY_FACTOR` |
| Module names | `snake_case`, singular | `store`, `graph`, `retrieval`, `lifecycle`, `provider` |
| ID newtypes | `CamelCase` + `Id` suffix | `EpisodeId`, `NodeId`, `PreferenceId`, `ImpressionId`, `LinkId` |
| Report types | `CamelCase` + `Report` suffix | `ConsolidationReport`, `ForgettingReport`, `TransformationReport` |
| Input types | `New` + entity name | `NewEpisode`, `NewSemanticNode`, `NewImpression` |
| Filter types | entity + `Filter` | `KnowledgeFilter`, `PurgeFilter` |

### Module Structure

```
alaya::
  lib.rs              -- AlayaStore, re-exports
  error.rs            -- AlayaError, Result<T>
  types.rs            -- All public types (IDs, enums, structs)
  schema.rs           -- Database initialization (pub(crate))
  provider.rs         -- ConsolidationProvider trait, NoOpProvider
  store/
    mod.rs            -- Sub-module declarations
    episodic.rs       -- Episode CRUD (pub(crate))
    semantic.rs       -- Semantic node CRUD (pub(crate))
    implicit.rs       -- Impression + Preference CRUD (pub(crate))
    embeddings.rs     -- Embedding storage + vector search (pub(crate))
    strengths.rs      -- Bjork dual-strength model (pub(crate))
  retrieval/
    mod.rs            -- Sub-module declarations
    pipeline.rs       -- Full hybrid retrieval pipeline (pub(crate))
    bm25.rs           -- FTS5 BM25 search (pub(crate))
    vector.rs         -- Cosine similarity search (pub(crate))
    fusion.rs         -- Reciprocal Rank Fusion (pub(crate))
    rerank.rs         -- Context-weighted reranking (pub(crate))
  graph/
    mod.rs            -- Sub-module declarations
    links.rs          -- Hebbian link CRUD (pub(crate))
    activation.rs     -- Spreading activation (pub(crate))
  lifecycle/
    mod.rs            -- Sub-module declarations
    consolidation.rs  -- CLS episodic->semantic (pub(crate))
    perfuming.rs      -- Vasana impression->preference (pub(crate))
    transformation.rs -- Dedup, prune, decay (pub(crate))
    forgetting.rs     -- Bjork dual-strength forgetting (pub(crate))
```

### Visibility Rules

| Visibility | Usage | Rationale |
|-----------|-------|-----------|
| `pub` | `AlayaStore` methods, types in `types.rs`, `AlayaError`, `ConsolidationProvider`, `NoOpProvider` | The public API surface. Changes require semver consideration |
| `pub(crate)` | All store/retrieval/graph/lifecycle/schema functions | Internal implementation. Called only by `AlayaStore` methods. Allows refactoring without breaking the public API |
| `pub` (on `types.rs` struct fields) | All fields of public structs | Developers need to construct `NewEpisode`, read `ScoredMemory.score`, etc. No opaque wrappers on data types |
| Private | Helper functions within modules | Implementation details: `sanitize_fts5`, `map_link`, `jaccard`, `recency_decay` |

**Principle:** The public API is `AlayaStore` + the types it accepts and returns + `ConsolidationProvider`. Everything else is `pub(crate)` or private. Developers interact with one struct and one trait.

### Feature Flag Naming (Future)

When feature flags are introduced, they follow these conventions:

| Pattern | Example | What It Gates |
|---------|---------|---------------|
| `embed-{backend}` | `embed-fastembed` | Embedding backend integration |
| `vec-{backend}` | `vec-sqlite-vec` | Vector search acceleration |
| `async` | `async` | Async versions of lifecycle methods |
| `ffi` | `ffi` | C-ABI compatible exports for FFI consumers |

Maximum 4-6 feature flags. Default features provide a complete working system (BM25 retrieval, brute-force vector search, synchronous lifecycle).

### Re-exports from Crate Root

The crate root (`lib.rs`) re-exports commonly used types so developers write:

```rust
use alaya::{AlayaStore, NewEpisode, Role, EpisodeContext, Query};
```

Not:

```rust
use alaya::store::AlayaStore;
use alaya::types::NewEpisode;
use alaya::types::Role;
```

**Rule:** Every type that appears in a public method signature is re-exported from the crate root. Types used only internally are not.

Currently re-exported:

| Re-export | Source |
|-----------|--------|
| `AlayaError`, `Result` | `error.rs` |
| `ConsolidationProvider`, `NoOpProvider` | `provider.rs` |
| All types via `pub use types::*` | `types.rs` |

---

## 2. Type Design System

Types are the vocabulary of the API. Alaya organizes types into five categories: IDs, inputs, outputs, enums, and reports.

### ID Types (Newtypes)

Every entity has a newtype wrapper around `i64`. This prevents accidental ID confusion (passing an `EpisodeId` where a `NodeId` is expected).

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

**ID conventions:**
- Inner field is `pub` for construction and pattern matching
- All derive `Copy` (they are just `i64` under the hood)
- All derive `Hash` for use as `HashMap` keys
- All derive `Serialize, Deserialize` for MCP/JSON interop

### Polymorphic Node Reference

`NodeRef` is the universal pointer into any store. It appears in graph links, retrieval results, and strength tracking.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeRef {
    Episode(EpisodeId),
    Semantic(NodeId),
    Preference(PreferenceId),
}
```

**Usage pattern:** Every graph operation, every retrieval result, and every strength record uses `NodeRef` rather than bare `i64` values. This makes the polymorphic nature of the graph explicit in the type system.

Helper methods on `NodeRef`:
- `type_str() -> &'static str` -- returns `"episode"`, `"semantic"`, or `"preference"` for SQLite storage
- `id() -> i64` -- extracts the inner ID for database queries
- `from_parts(node_type: &str, id: i64) -> Option<Self>` -- reconstructs from database columns

### Input Types (New* Pattern)

Every entity that can be created has a corresponding `New*` struct. Input types contain the fields the caller provides; output types add system-generated fields.

| Input Type | Output Type | System-Added Fields |
|-----------|-------------|---------------------|
| `NewEpisode` | `Episode` | `id: EpisodeId` |
| `NewSemanticNode` | `SemanticNode` | `id: NodeId`, `created_at`, `last_corroborated`, `corroboration_count` |
| `NewImpression` | `Impression` | `id: ImpressionId`, `timestamp` |
| `Interaction` | (consumed by perfuming, not stored directly) | N/A |

**Example -- NewEpisode vs Episode:**

```rust
// Input: what the developer provides
pub struct NewEpisode {
    pub content: String,
    pub role: Role,
    pub session_id: String,
    pub timestamp: i64,
    pub context: EpisodeContext,
    pub embedding: Option<Vec<f32>>,
}

// Output: what the database returns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: EpisodeId,
    pub content: String,
    pub role: Role,
    pub session_id: String,
    pub timestamp: i64,
    pub context: EpisodeContext,
}
```

**Design rule:** Input types do not derive `Serialize, Deserialize` unless needed for MCP tool input. Output types always derive both for serialization to JSON, logging, and cross-system transport.

### Context Types

`EpisodeContext` provides optional metadata that enriches retrieval. All fields have defaults so the developer can start with `EpisodeContext::default()` and add fields incrementally.

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EpisodeContext {
    pub topics: Vec<String>,           // default: empty
    pub sentiment: f32,                // default: 0.0
    pub conversation_turn: u32,        // default: 0
    pub mentioned_entities: Vec<String>, // default: empty
    pub preceding_episode: Option<EpisodeId>, // default: None
}
```

`QueryContext` mirrors `EpisodeContext` for the query side, enabling context-weighted reranking:

```rust
#[derive(Debug, Clone, Default)]
pub struct QueryContext {
    pub topics: Vec<String>,
    pub sentiment: f32,
    pub mentioned_entities: Vec<String>,
    pub current_timestamp: Option<i64>,
}
```

### Query Type

`Query` provides both a convenience constructor and full control:

```rust
pub struct Query {
    pub text: String,
    pub embedding: Option<Vec<f32>>,
    pub context: QueryContext,
    pub max_results: usize,
}

impl Query {
    /// Create a query with sensible defaults. BM25-only, 5 results.
    pub fn simple(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            embedding: None,
            context: QueryContext::default(),
            max_results: 5,
        }
    }
}
```

**Two-level API:** `Query::simple("text")` for quickstart, manual struct construction for full control. No builder needed because the struct has only four fields.

### Retrieval Output

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredMemory {
    pub node: NodeRef,       // Which store and which ID
    pub content: String,     // The text content
    pub score: f64,          // RRF + reranking score (higher = more relevant)
    pub role: Option<Role>,  // Original speaker (episodes only)
    pub timestamp: i64,      // When the memory was created
}
```

**Design rule:** Retrieval results always include the content string so the caller does not need a follow-up query. The score is `f64` (not `f32`) because RRF fusion accumulates small reciprocal values where precision matters.

### Report Types

Every lifecycle process returns a typed report. Reports use `Default` so the process can start with an empty report and increment counters.

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsolidationReport {
    pub episodes_processed: u32,
    pub nodes_created: u32,
    pub links_created: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerfumingReport {
    pub impressions_stored: u32,
    pub preferences_crystallized: u32,
    pub preferences_reinforced: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransformationReport {
    pub duplicates_merged: u32,
    pub links_pruned: u32,
    pub preferences_decayed: u32,
    pub impressions_pruned: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForgettingReport {
    pub nodes_decayed: u32,
    pub nodes_archived: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PurgeReport {
    pub episodes_deleted: u32,
    pub nodes_deleted: u32,
    pub links_deleted: u32,
    pub embeddings_deleted: u32,
}
```

**Design rule:** Report fields are `u32` counters, not `usize`. Reports serialize to JSON for MCP, logging, and monitoring. Every lifecycle method returns a report, never `()`.

### Status Type

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatus {
    pub episode_count: u64,
    pub semantic_node_count: u64,
    pub preference_count: u64,
    pub impression_count: u64,
    pub link_count: u64,
    pub embedding_count: u64,
}
```

**Design rule:** Status counts are `u64` because they represent database row counts that could theoretically grow large. They are not `usize` because they must be platform-independent for serialization.

### Enum Design

All public enums use `CamelCase` variants, derive `Serialize, Deserialize` with `#[serde(rename_all = "lowercase")]` for JSON interop, and provide `as_str()` and `from_str()` methods for SQLite storage.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SemanticType {
    Fact,
    Relationship,
    Event,
    Concept,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkType {
    Temporal,
    Topical,
    Entity,
    Causal,
    CoRetrieval,
}
```

**Pending:** All public enums should carry `#[non_exhaustive]` per the Extract's "always" rules. This allows adding variants in minor versions without breaking downstream `match` arms.

### Dual-Strength Model Type

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStrength {
    pub node: NodeRef,
    pub storage_strength: f32,     // [0.0, 1.0] -- monotonically increases with access
    pub retrieval_strength: f32,   // [0.0, 1.0] -- decays over time without access
    pub access_count: u32,
    pub last_accessed: i64,
}
```

This type encodes the Bjork & Bjork (1992) dual-strength model. Storage strength represents how well-learned a memory is (increases on every access, never decreases). Retrieval strength represents how accessible it is right now (decays without use, resets to 1.0 on access).

### Complete Type Hierarchy

```
AlayaStore
  |
  +-- Write path
  |     NewEpisode --> EpisodeId
  |       EpisodeContext
  |       Role
  |       Option<Vec<f32>> (embedding)
  |
  +-- Read path
  |     Query --> Vec<ScoredMemory>
  |       QueryContext
  |     Option<&str> (domain) --> Vec<Preference>
  |     Option<KnowledgeFilter> --> Vec<SemanticNode>
  |     (NodeRef, u32) --> Vec<(NodeRef, f32)>
  |
  +-- Lifecycle path
  |     &dyn ConsolidationProvider --> ConsolidationReport
  |     (&Interaction, &dyn ConsolidationProvider) --> PerfumingReport
  |     () --> TransformationReport
  |     () --> ForgettingReport
  |
  +-- Admin path
  |     () --> MemoryStatus
  |     PurgeFilter --> PurgeReport
  |
  +-- Supporting types
        NodeRef (Episode | Semantic | Preference)
        NodeStrength
        Link, LinkType
        SemanticType
        Impression, NewImpression
        NewSemanticNode, SemanticNode
        Episode
```

---

## 3. API Patterns

### Pattern 1: Single Entry Point

All interaction goes through `AlayaStore`. The developer never imports or constructs internal types like `Connection`, `episodic::store_episode`, or `bm25::search_bm25`.

```rust
// Correct: one entry point
let store = AlayaStore::open("memory.db")?;
store.store_episode(&episode)?;
let results = store.query(&Query::simple("search text"))?;

// Wrong: reaching into internals
let conn = schema::open_db("memory.db")?;  // pub(crate), not accessible
```

**Why:** This gives Alaya freedom to restructure internals without breaking downstream code. The Extract's "never" list says: "Expose SQLite internals through public API."

### Pattern 2: Open Variants

Two constructors, one for persistence and one for testing:

```rust
// Persistent: creates or opens a SQLite file
let store = AlayaStore::open("memory.db")?;

// Ephemeral: in-memory database for unit tests
let store = AlayaStore::open_in_memory()?;
```

**Why:** `open_in_memory()` enables fast, isolated tests without filesystem cleanup. It uses the same code paths as `open()` so tests are representative.

### Pattern 3: Store-and-Get Symmetry

The write path accepts a `New*` struct and returns an ID. The corresponding read path accepts the ID and returns the full entity.

```rust
// Write: NewEpisode -> EpisodeId
let id = store.store_episode(&NewEpisode {
    content: "I prefer Vim keybindings".to_string(),
    role: Role::User,
    session_id: "session-1".to_string(),
    timestamp: 1709000000,
    context: EpisodeContext::default(),
    embedding: None,
})?;

// Read (via query, not get_episode -- episodes are retrieved through the pipeline)
let results = store.query(&Query::simple("keybindings"))?;
```

**Note:** The current API exposes `store_episode()` for writes and `query()` for reads. Individual entity getters (`get_episode`, `get_semantic_node`) are `pub(crate)` because direct ID-based access bypasses the retrieval pipeline and its strength-tracking side effects. This is intentional: the pipeline is the product, not the database.

### Pattern 4: Reference by Borrow

All `AlayaStore` methods that accept data take `&self` and borrow their arguments:

```rust
pub fn store_episode(&self, episode: &NewEpisode) -> Result<EpisodeId>
pub fn query(&self, q: &Query) -> Result<Vec<ScoredMemory>>
pub fn consolidate(&self, provider: &dyn ConsolidationProvider) -> Result<ConsolidationReport>
pub fn perfume(&self, interaction: &Interaction, provider: &dyn ConsolidationProvider) -> Result<PerfumingReport>
```

**Why:** Borrowing is idiomatic Rust for read-heavy operations. The caller retains ownership. `&self` (not `&mut self`) because SQLite handles concurrency through WAL mode, and all mutations go through `Connection::execute` which takes `&self`.

### Pattern 5: Trait Extension

Alaya defines behavior through traits that the developer implements with their own LLM. The core library never calls an LLM directly.

```rust
pub trait ConsolidationProvider {
    /// Extract semantic knowledge from a batch of episodes.
    fn extract_knowledge(&self, episodes: &[Episode]) -> Result<Vec<NewSemanticNode>>;

    /// Extract behavioral impressions from an interaction.
    fn extract_impressions(&self, interaction: &Interaction) -> Result<Vec<NewImpression>>;

    /// Detect whether two semantic nodes contradict each other.
    fn detect_contradiction(&self, a: &SemanticNode, b: &SemanticNode) -> Result<bool>;
}
```

**The NoOpProvider fallback:**

```rust
pub struct NoOpProvider;

impl ConsolidationProvider for NoOpProvider {
    fn extract_knowledge(&self, _episodes: &[Episode]) -> Result<Vec<NewSemanticNode>> {
        Ok(vec![])
    }

    fn extract_impressions(&self, _interaction: &Interaction) -> Result<Vec<NewImpression>> {
        Ok(vec![])
    }

    fn detect_contradiction(&self, _a: &SemanticNode, _b: &SemanticNode) -> Result<bool> {
        Ok(false)
    }
}
```

**Graceful degradation chain:**
1. Developer provides a full `ConsolidationProvider` -- consolidation extracts knowledge, perfuming crystallizes preferences
2. Developer uses `NoOpProvider` -- consolidation produces no nodes, but episodes still accumulate. Forgetting and transformation still run. BM25 retrieval works independently
3. No lifecycle calls at all -- Alaya is a pure store-and-query system with BM25 + optional vector retrieval

### Pattern 6: Result Everywhere

Every fallible operation returns `Result<T, AlayaError>`. The crate defines a type alias for convenience:

```rust
pub type Result<T> = std::result::Result<T, AlayaError>;
```

**No panics in public API.** Methods that could fail return `Result`. Methods that cannot fail (e.g., `NodeRef::type_str()`) return values directly.

### Pattern 7: Lifecycle as Explicit Methods

Lifecycle processes are explicit method calls, not background jobs or automatic triggers. The developer controls when and how often each process runs.

```rust
// The "dream cycle" pattern -- developer orchestrates timing
let consolidation = store.consolidate(&provider)?;
let forgetting = store.forget()?;
let transformation = store.transform()?;

// Perfuming runs per-interaction, not in batch
let perfuming = store.perfume(&interaction, &provider)?;
```

**Why:** Per the Extract, the lifecycle is opt-in. The developer decides scheduling: after every N conversations, on a timer, on application idle, or never. Alaya provides the mechanisms, not the policy.

### Pattern 8: Admin Operations

Status inspection and data purging:

```rust
// Inspect
let status = store.status()?;
println!("Episodes: {}, Semantic: {}, Preferences: {}",
    status.episode_count, status.semantic_node_count, status.preference_count);

// Purge by session
store.purge(PurgeFilter::Session("session-1".to_string()))?;

// Purge by age
store.purge(PurgeFilter::OlderThan(cutoff_timestamp))?;

// Nuclear option
store.purge(PurgeFilter::All)?;
```

**Design rule:** `purge()` is an explicit, hard delete. It is distinct from `forget()`, which decays retrieval strength and only archives nodes that fall below both strength thresholds. The naming distinction is deliberate: forgetting is a cognitive process; purging is a data operation.

### Pattern 9: Interaction Type for Perfuming

```rust
pub struct Interaction {
    pub text: String,
    pub role: Role,
    pub session_id: String,
    pub timestamp: i64,
    pub context: EpisodeContext,
}
```

`Interaction` is the bridge between the agent's conversation format and Alaya's perfuming process. The agent constructs it from whatever message format they use (Signal, Discord, HTTP, etc.) and passes it to `store.perfume()`.

---

## 4. Documentation Patterns

### Doc Comment Style

Every public item follows this structure:

```rust
/// One-line summary in imperative mood.
///
/// Longer description if needed. Explains what the method does,
/// when to use it, and any important behavior (side effects,
/// performance characteristics, relationship to other methods).
///
/// # Arguments
///
/// * `param` - What this parameter controls
///
/// # Returns
///
/// Description of the return value and what it means.
///
/// # Errors
///
/// When and why this method returns an error.
///
/// # Examples
///
/// ```rust
/// # use alaya::*;
/// # fn main() -> alaya::Result<()> {
/// let store = AlayaStore::open_in_memory()?;
/// // ... example code
/// # Ok(())
/// # }
/// ```
pub fn method_name(&self) -> Result<ReturnType> { ... }
```

**Rules:**
- First line is a summary: imperative mood ("Store a conversation episode"), not descriptive ("Stores a conversation episode")
- Blank line between summary and description
- Use `# Examples` with compilable doctests for every `pub` method
- Use `# fn main() -> alaya::Result<()>` wrapper in doctests so `?` works
- Use `# use alaya::*;` as a hidden import in doctests

### Module-Level Documentation

Each module explains its role in the cognitive lifecycle:

```rust
//! # Lifecycle: Consolidation
//!
//! Consolidation models the Complementary Learning Systems (CLS) theory:
//! the hippocampus (episodic store) gradually teaches the neocortex (semantic
//! store) through interleaved replay, avoiding catastrophic interference.
//!
//! The agent triggers consolidation by calling [`AlayaStore::consolidate()`]
//! with a [`ConsolidationProvider`] that extracts knowledge from episodes.
```

**Rules:**
- Use `//!` for module docs
- First paragraph explains the neuroscience or Buddhist psychology foundation
- Second paragraph explains the API entry point with cross-reference links
- Do not repeat implementation details; point to the relevant `AlayaStore` method

### Cross-Reference Linking

Use Rustdoc link syntax to connect related items:

```rust
/// Run a consolidation cycle.
///
/// See also: [`AlayaStore::forget()`], [`AlayaStore::transform()`],
/// [`ConsolidationProvider`].
```

**Rules:**
- Link to every related public type or method
- Use backtick-bracket syntax: `[`TypeName`]`
- Link to the `AlayaStore` method, not the internal function

### Error Documentation

Every `AlayaError` variant documents what went wrong and how to fix it:

```rust
#[derive(Debug, Error)]
pub enum AlayaError {
    /// Database operation failed.
    ///
    /// This wraps a SQLite error. Common causes:
    /// - File permissions on the database path
    /// - Disk full
    /// - Corrupt database file (run `.integrity_check`)
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    /// Requested entity does not exist.
    ///
    /// The ID was valid but no row matched. This can happen if:
    /// - The entity was deleted by `purge()` or `forget()`
    /// - The ID came from a different database file
    #[error("not found: {0}")]
    NotFound(String),

    /// Input validation failed.
    ///
    /// Check the error message for which field was invalid and why.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// JSON serialization or deserialization failed.
    ///
    /// This usually indicates corrupt `context_json` in the database
    /// or a schema version mismatch.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// The developer's ConsolidationProvider returned an error.
    ///
    /// The wrapped message comes from your provider implementation.
    /// Alaya caught the error during lifecycle processing.
    /// Your data is safe -- the lifecycle step was skipped for this batch.
    #[error("provider error: {0}")]
    Provider(String),
}
```

### Research Citation Pattern

Non-obvious mechanisms include a brief citation in the doc comment:

```rust
/// Spread activation from seed nodes through the graph.
///
/// Models the Collins & Loftus (1975) spreading activation theory:
/// activation propagates from seed nodes through weighted edges,
/// decaying at each hop, and splitting proportionally at branching points.
```

```rust
/// Run a forgetting sweep.
///
/// Models the Bjork & Bjork (1992) "New Theory of Disuse":
/// - Storage strength (how well-learned) monotonically increases with access
/// - Retrieval strength (how accessible now) decays over time
```

**Rules:**
- Include author(s) and year
- One sentence explaining what the theory predicts
- Map the theory to the specific Alaya mechanism

---

## 5. Code Example Progression

Examples progress from zero-config quickstart to production deployment. Each level builds on the previous one.

### Level 1: Quickstart (10 lines)

Goal: store an episode, query it back. Instant gratification. Under 2 minutes.

```rust
use alaya::{AlayaStore, NewEpisode, Role, EpisodeContext, Query};

fn main() -> alaya::Result<()> {
    let store = AlayaStore::open("memory.db")?;

    store.store_episode(&NewEpisode {
        content: "I prefer dark mode and Vim keybindings".to_string(),
        role: Role::User,
        session_id: "session-1".to_string(),
        timestamp: 1709000000,
        context: EpisodeContext::default(),
        embedding: None,
    })?;

    let results = store.query(&Query::simple("editor preferences"))?;
    for r in &results {
        println!("[{:.2}] {}", r.score, r.content);
    }

    Ok(())
}
```

### Level 2: Lifecycle (Dream Cycle)

Goal: show that memory transforms through use. Episodes become knowledge and preferences.

```rust
use alaya::{AlayaStore, NewEpisode, Role, EpisodeContext, Query, NoOpProvider};

fn main() -> alaya::Result<()> {
    let store = AlayaStore::open("memory.db")?;
    let provider = NoOpProvider;

    // Store multiple episodes across sessions
    for i in 0..10 {
        store.store_episode(&NewEpisode {
            content: format!("Discussion about Rust async patterns, part {}", i),
            role: Role::User,
            session_id: format!("session-{}", i / 3),
            timestamp: 1709000000 + i * 3600,
            context: EpisodeContext::default(),
            embedding: None,
        })?;
    }

    // Run the dream cycle: consolidate -> forget -> transform
    let consolidation = store.consolidate(&provider)?;
    let forgetting = store.forget()?;
    let transformation = store.transform()?;

    println!("Consolidation: {} nodes created", consolidation.nodes_created);
    println!("Forgetting: {} nodes decayed", forgetting.nodes_decayed);
    println!("Transformation: {} duplicates merged", transformation.duplicates_merged);

    // Inspect the memory system state
    let status = store.status()?;
    println!("Episodes: {}, Semantic nodes: {}, Preferences: {}",
        status.episode_count, status.semantic_node_count, status.preference_count);

    Ok(())
}
```

### Level 3: Custom Provider

Goal: implement `ConsolidationProvider` with the developer's own LLM.

```rust
use alaya::*;

struct MyProvider {
    // Your LLM client here
}

impl ConsolidationProvider for MyProvider {
    fn extract_knowledge(
        &self,
        episodes: &[Episode],
    ) -> alaya::Result<Vec<NewSemanticNode>> {
        // Send episodes to your LLM for structured extraction.
        // Return facts, relationships, events, concepts.
        //
        // Example: if episodes discuss "Rust async" repeatedly,
        // extract SemanticType::Fact("User is learning Rust async").
        let combined: String = episodes.iter()
            .map(|e| e.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        // ... call your LLM here ...
        // For demonstration, return a hardcoded node:
        Ok(vec![NewSemanticNode {
            content: format!("User discussed: {}", &combined[..combined.len().min(100)]),
            node_type: SemanticType::Fact,
            confidence: 0.7,
            source_episodes: episodes.iter().map(|e| e.id).collect(),
            embedding: None,
        }])
    }

    fn extract_impressions(
        &self,
        interaction: &Interaction,
    ) -> alaya::Result<Vec<NewImpression>> {
        // Analyze interaction for implicit behavioral signals.
        // "User asked for concise answers three times" -> impression in "communication" domain.
        Ok(vec![])
    }

    fn detect_contradiction(
        &self,
        _a: &SemanticNode,
        _b: &SemanticNode,
    ) -> alaya::Result<bool> {
        Ok(false)
    }
}
```

### Level 4: Advanced Retrieval

Goal: tune retrieval with context, embeddings, and graph exploration.

```rust
use alaya::*;

fn advanced_query(store: &AlayaStore) -> alaya::Result<()> {
    // Full query with context for better reranking
    let results = store.query(&Query {
        text: "async programming patterns".to_string(),
        embedding: None, // Add Vec<f32> from your embedding model for hybrid search
        context: QueryContext {
            topics: vec!["rust".to_string(), "async".to_string()],
            sentiment: 0.0,
            mentioned_entities: vec!["tokio".to_string()],
            current_timestamp: Some(1709100000),
        },
        max_results: 10,
    })?;

    // Explore the graph around the top result
    if let Some(top) = results.first() {
        let neighbors = store.neighbors(top.node, 2)?;
        println!("Top result: [{}] {}", top.score, top.content);
        println!("Graph neighbors ({} found):", neighbors.len());
        for (node, activation) in &neighbors {
            println!("  {:?} (activation: {:.3})", node, activation);
        }
    }

    // Get crystallized preferences
    let prefs = store.preferences(None)?;
    for p in &prefs {
        println!("Preference [{}]: {} (confidence: {:.2}, evidence: {})",
            p.domain, p.preference, p.confidence, p.evidence_count);
    }

    // Get structured knowledge
    let knowledge = store.knowledge(Some(KnowledgeFilter {
        node_type: Some(SemanticType::Fact),
        min_confidence: Some(0.5),
        limit: Some(20),
    }))?;
    for node in &knowledge {
        println!("Fact: {} (confidence: {:.2}, corroborated: {} times)",
            node.content, node.confidence, node.corroboration_count);
    }

    Ok(())
}
```

### Level 5: Production

Goal: deployment considerations, backup, monitoring.

```rust
use alaya::*;
use std::sync::Arc;

fn production_setup() -> alaya::Result<()> {
    // Use a stable path for the database
    let db_path = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("my-agent")
        .join("memory.db");

    // Ensure parent directory exists
    std::fs::create_dir_all(db_path.parent().unwrap())?;

    let store = AlayaStore::open(&db_path)?;

    // Monitoring: check store health on startup
    let status = store.status()?;
    println!("Memory loaded: {} episodes, {} semantic nodes, {} preferences",
        status.episode_count, status.semantic_node_count, status.preference_count);

    // Backup: single file copy (SQLite WAL is checkpointed on close)
    // Back up memory.db and memory.db-wal if it exists
    let backup_path = db_path.with_extension("db.bak");
    std::fs::copy(&db_path, &backup_path)?;

    // Thread safety: wrap in Arc for shared access
    // SQLite WAL mode supports concurrent reads with a single writer
    let store = Arc::new(store);

    // Periodic dream cycle (run on a timer or between conversations)
    let store_clone = Arc::clone(&store);
    std::thread::spawn(move || {
        let provider = NoOpProvider;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
            let _ = store_clone.consolidate(&provider);
            let _ = store_clone.forget();
            let _ = store_clone.transform();
        }
    });

    // Data deletion path for privacy compliance
    // Option 1: Targeted purge
    store.purge(PurgeFilter::Session("old-session".to_string()))?;
    // Option 2: Age-based purge
    store.purge(PurgeFilter::OlderThan(1700000000))?;
    // Option 3: Complete wipe (or just delete the file)
    // store.purge(PurgeFilter::All)?;

    Ok(())
}
```

---

## 6. Diagnostic Output Design

### MemoryStatus

The primary diagnostic output. Answers "what is in the memory system right now?"

```rust
let status = store.status()?;
// MemoryStatus {
//     episode_count: 147,
//     semantic_node_count: 23,
//     preference_count: 5,
//     impression_count: 89,
//     link_count: 312,
//     embedding_count: 42,
// }
```

**Design rule:** Every count is a separate field, not a HashMap. This enables compile-time checking and IDE autocompletion. If a new entity type is added, adding a field to `MemoryStatus` is a breaking change (mitigated by future `#[non_exhaustive]`).

### Lifecycle Reports

Each lifecycle process returns a report showing what changed. Reports serve three purposes:
1. **Operational visibility** -- the developer knows the lifecycle ran and what it did
2. **Debugging** -- unexpected zeros indicate the process did not find work to do
3. **Monitoring** -- serialize to JSON and ship to logging/metrics

```
Consolidation: 10 episodes processed -> 3 nodes created, 15 links created
Forgetting:    42 nodes had retrieval strength decayed, 2 nodes archived
Transformation: 1 duplicate merged, 3 weak links pruned, 0 preferences decayed
Perfuming:     2 impressions stored, 0 preferences crystallized, 1 preference reinforced
```

### Retrieval Scoring

`ScoredMemory.score` is the final ranking score after the full pipeline:

1. **BM25 stage:** FTS5 rank, normalized to [0.0, 1.0]
2. **Vector stage:** Cosine similarity [0.0, 1.0] (only when embedding provided)
3. **Graph stage:** Spreading activation from seed nodes
4. **RRF fusion:** Reciprocal Rank Fusion with k=60 merges all three signals
5. **Reranking:** Context similarity (topic Jaccard, entity Jaccard, sentiment distance) and recency decay (exponential, 30-day half-life) adjust the fused score

The score is not a probability. It is a relative ranking value. Higher is more relevant. Scores from different queries are not comparable.

**Future (v0.2): QueryExplanation type**

```rust
// Planned diagnostic type for debugging empty or poor results
pub struct QueryExplanation {
    pub bm25_matches: Vec<(NodeRef, f64)>,        // What BM25 found
    pub vector_matches: Vec<(NodeRef, f64)>,       // What vector search found
    pub graph_activated: Vec<(NodeRef, f32)>,       // What spreading activation found
    pub fused: Vec<(NodeRef, f64)>,                // After RRF
    pub reranked: Vec<ScoredMemory>,               // After context reranking
    pub tokens_matched: Vec<String>,               // BM25 tokens that overlapped
    pub tokens_missed: Vec<String>,                // Query tokens with no FTS5 match
}
```

This type addresses the highest-risk failure mode from the Developer Journeys: empty query results where the developer thinks the library is broken, but the real cause is lexical mismatch between query and stored content.

### Debug Logging

Alaya uses `tracing` integration (future) for structured diagnostic output:

```
TRACE alaya::retrieval::bm25    query="Rust patterns" matches=3 top_score=0.87
TRACE alaya::retrieval::vector  query_dim=384 matches=5 top_sim=0.92
TRACE alaya::retrieval::fusion  bm25_count=3 vec_count=5 graph_count=2 fused_count=7
TRACE alaya::retrieval::rerank  input=7 output=5 recency_boost=0.15
DEBUG alaya::lifecycle::forget   decayed=42 archived=2 threshold=(storage<0.1, retrieval<0.05)
```

### Error Message Design

Every error message answers three questions:

1. **What happened?** -- the immediate failure
2. **Why?** -- the likely cause
3. **What to do?** -- the recovery action

**Examples:**

```
database error: unable to open database file
  Path: /var/data/memory.db
  Likely cause: directory does not exist or insufficient permissions
  Recovery: ensure the parent directory exists and is writable

not found: episode 42
  The episode may have been removed by purge() or forget()
  Recovery: query by content rather than by ID, or check store.status()

provider error: timeout waiting for LLM response
  This error came from your ConsolidationProvider implementation
  Your data is safe -- consolidation skipped this batch
  Recovery: check your LLM connection; episodes will be retried next cycle

invalid input: episode content is empty
  Episodes must contain at least one character of content
  Recovery: check for empty strings before calling store_episode()
```

---

## 7. Consistency Rules

These rules apply across the entire codebase. Violations are considered bugs.

### Data Type Rules

| Rule | Convention | Rationale |
|------|-----------|-----------|
| Timestamps | `i64` (Unix seconds) | SQLite stores integers efficiently. Chrono is a user-space concern, not a storage concern |
| IDs | Newtype around `i64` | SQLite rowid is `i64`. Newtypes prevent cross-entity confusion |
| Text content | `String` in public API | Owned strings avoid lifetime complexity. `&str` only in internal `pub(crate)` functions |
| Optional fields | `Option<T>` | Never sentinel values (-1, empty string, NaN) |
| Collections | `Vec<T>` in return types | Never iterators in public returns. Collect before returning |
| Floating-point scores | `f64` for retrieval scores, `f32` for weights and strengths | Retrieval needs precision for RRF accumulation; weights are SQLite `REAL` (f64 in storage, f32 in API for memory efficiency) |
| Counts in reports | `u32` | Large enough for any realistic count; serializes cleanly |
| Counts in status | `u64` | Matches SQLite `count(*)` return type |

### Derive Rules

| Type Category | Required Derives |
|--------------|-----------------|
| ID newtypes | `Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize` |
| Input types (`New*`) | `Debug, Clone` |
| Output types | `Debug, Clone, Serialize, Deserialize` |
| Report types | `Debug, Clone, Default, Serialize, Deserialize` |
| Enums | `Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize` |
| Context types | `Debug, Clone, Default, Serialize, Deserialize` |
| Error type | `Debug, Error` (via thiserror) |

### Naming Rules

| Rule | Example | Counter-Example |
|------|---------|-----------------|
| Methods on `AlayaStore` use verb-first naming | `store_episode`, `query`, `consolidate` | `episode_store`, `do_query` |
| Boolean-returning methods use `is_` or `has_` prefix | `is_empty()`, `has_embeddings()` | `empty()`, `check_embeddings()` |
| Conversion methods use `as_` (borrowed) or `into_` (owned) | `as_str()`, `into_string()` | `to_str()` (unless `ToOwned`) |
| Constructor methods use `new`, `open`, `from_*`, or `simple` | `AlayaStore::open()`, `Query::simple()`, `NodeRef::from_parts()` | `AlayaStore::create()` |
| Report field names match the process they report on | `nodes_created`, `links_pruned` | `created`, `pruned` (too generic) |

### SQLite Interaction Rules

| Rule | Implementation | Rationale |
|------|---------------|-----------|
| WAL mode always enabled | `PRAGMA journal_mode = WAL` in `init_db()` | Concurrent reads, crash safety |
| Foreign keys enabled | `PRAGMA foreign_keys = ON` | Referential integrity |
| FTS5 input sanitized | Strip non-alphanumeric characters before `MATCH` | Prevent syntax injection |
| Indexes on query paths | All `WHERE` columns indexed | Predictable query performance |
| `INSERT OR IGNORE` for idempotent writes | Links, embeddings | Prevents duplicate errors on retry |
| `INSERT OR REPLACE` for upsert | Embeddings | New embedding replaces old for same node |
| Cascading cleanup on delete | Delete episode removes FTS5 entry, embedding, links, strengths | No orphaned data |

### Semver Rules

| Change | Version Bump |
|--------|-------------|
| New method on `AlayaStore` | Minor (additive) |
| New field on `#[non_exhaustive]` struct | Minor (additive) |
| New variant on `#[non_exhaustive]` enum | Minor (additive) |
| Change return type of existing method | Major (breaking) |
| Remove public method | Major (breaking) |
| Change method signature | Major (breaking) |
| Change SQLite schema | Minor + migration path |
| New feature flag | Minor (additive) |

---

## 8. Error Message Design

### Error Hierarchy

```
AlayaError
  |
  +-- Db(rusqlite::Error)          -- SQLite layer failure
  |     Cause: file permissions, disk full, corrupt DB, schema error
  |     Recovery: check path, check disk, run integrity check
  |
  +-- NotFound(String)             -- Entity lookup miss
  |     Cause: ID from old session, entity was forgotten/purged
  |     Recovery: query by content, check status
  |
  +-- InvalidInput(String)         -- Validation failure at API boundary
  |     Cause: empty content, zero-length embedding, invalid filter
  |     Recovery: check input before calling
  |
  +-- Serialization(serde_json)    -- JSON round-trip failure
  |     Cause: corrupt context_json, schema version mismatch
  |     Recovery: check DB version, rebuild from episodes
  |
  +-- Provider(String)             -- Developer's provider errored
        Cause: LLM timeout, malformed response, API key expired
        Recovery: check provider code; data is safe, batch skipped
```

### Error Attribution

Every error clearly indicates its origin:

| Origin | How Developer Identifies It |
|--------|---------------------------|
| Alaya internals | `AlayaError::Db`, `AlayaError::Serialization` |
| Developer's input | `AlayaError::InvalidInput` with field name |
| Developer's provider | `AlayaError::Provider` with provider's error message |
| SQLite | `AlayaError::Db` with SQLite error code |
| Data state | `AlayaError::NotFound` with entity type and ID |

### Error Design Principles

1. **Compilation errors over runtime errors.** The type system prevents passing an `EpisodeId` where a `NodeId` is expected. Enums prevent invalid string values. Required fields prevent incomplete structs.

2. **Every error is recoverable.** No error variant represents an unrecoverable state. Even `Db` errors from corrupt databases have a recovery path (backup, rebuild).

3. **Provider errors are isolated.** A failing `ConsolidationProvider` does not corrupt data or crash the process. The lifecycle step is skipped, and a `Provider` error is returned. Episodes remain safe.

4. **Silent failures are worse than loud errors.** `query()` returning an empty `Vec` is not an error, but it is a developer experience problem. The future `QueryExplanation` type turns silent emptiness into actionable diagnostics.

---

## 9. API Evolution Guidelines

### Adding New Entity Types

When adding a new entity type (e.g., `Skill`, `Summary`):

1. Create ID newtype: `pub struct SkillId(pub i64)` with standard derives
2. Add `NodeRef` variant: `NodeRef::Skill(SkillId)` (requires `#[non_exhaustive]` to avoid breaking change)
3. Create `NewSkill` (input) and `Skill` (output) structs
4. Add `store_skill()` and related methods to `AlayaStore`
5. Add count to `MemoryStatus`
6. Add table and indexes in `schema.rs`
7. Add cascade cleanup in delete paths

### Adding New Lifecycle Processes

When adding a new lifecycle process (e.g., `reconsolidate`, `rehearse`):

1. Create a new report type: `ReconsolidationReport`
2. Create the module: `lifecycle/reconsolidation.rs`
3. Add the method to `AlayaStore`
4. Method returns `Result<ReconsolidationReport>`
5. Document the neuroscience foundation in module docs

### Adding New Provider Traits

When Alaya needs new developer-provided intelligence (e.g., `EmbeddingProvider`):

1. Define the trait in `provider.rs`
2. Provide a `NoOp` implementation
3. Existing code continues to work without the new provider (graceful degradation)
4. Re-export from crate root

### Breaking Change Checklist

Before any breaking change:

- [ ] Is there an additive alternative? (Prefer new method over changing existing one)
- [ ] Can `#[non_exhaustive]` absorb the change?
- [ ] Is migration provided? (Schema changes)
- [ ] Is the changelog entry written?
- [ ] Does the major version bump?

---

## Appendix: API Surface Summary

### AlayaStore Methods (Complete)

| Method | Category | Signature | Returns |
|--------|----------|-----------|---------|
| `open(path)` | Constructor | `impl AsRef<Path> -> Result<Self>` | New or existing store |
| `open_in_memory()` | Constructor | `() -> Result<Self>` | Ephemeral test store |
| `store_episode(&ep)` | Write | `&NewEpisode -> Result<EpisodeId>` | Stored episode ID |
| `query(&q)` | Read | `&Query -> Result<Vec<ScoredMemory>>` | Ranked retrieval results |
| `preferences(domain)` | Read | `Option<&str> -> Result<Vec<Preference>>` | Crystallized preferences |
| `knowledge(filter)` | Read | `Option<KnowledgeFilter> -> Result<Vec<SemanticNode>>` | Semantic knowledge |
| `neighbors(node, depth)` | Read | `(NodeRef, u32) -> Result<Vec<(NodeRef, f32)>>` | Graph neighborhood |
| `consolidate(&provider)` | Lifecycle | `&dyn ConsolidationProvider -> Result<ConsolidationReport>` | CLS replay results |
| `perfume(&interaction, &provider)` | Lifecycle | `(&Interaction, &dyn ConsolidationProvider) -> Result<PerfumingReport>` | Vasana results |
| `transform()` | Lifecycle | `() -> Result<TransformationReport>` | Cleanup results |
| `forget()` | Lifecycle | `() -> Result<ForgettingReport>` | Decay results |
| `status()` | Admin | `() -> Result<MemoryStatus>` | Store counts |
| `purge(filter)` | Admin | `PurgeFilter -> Result<PurgeReport>` | Deletion counts |

### ConsolidationProvider Methods (Complete)

| Method | Input | Output |
|--------|-------|--------|
| `extract_knowledge` | `&[Episode]` | `Result<Vec<NewSemanticNode>>` |
| `extract_impressions` | `&Interaction` | `Result<Vec<NewImpression>>` |
| `detect_contradiction` | `(&SemanticNode, &SemanticNode)` | `Result<bool>` |

### Retrieval Pipeline Stages

```
Query
  |
  +-- BM25 (FTS5, sanitized input, normalized to [0,1])
  |
  +-- Vector (cosine similarity, optional, requires embedding)
  |
  +-- Graph (spreading activation from top BM25+vector seeds, 1 hop)
  |
  v
RRF Fusion (k=60, merges all available signals)
  |
  v
Context Reranking (topic Jaccard, entity Jaccard, sentiment, recency decay)
  |
  v
Post-Retrieval Effects
  +-- Strength boost (on_access: retrieval_strength = 1.0, storage_strength += 0.05)
  +-- Co-retrieval Hebbian strengthening (LTP on all result pairs)
  |
  v
Vec<ScoredMemory> (top max_results)
```

---

*Generated: 2026-02-26 | Phase: 5b | Cross-references: Brand Guidelines (Phase 1), North Star (Phase 2), Competitive Landscape (Phase 3), North Star Extract (Phase 4), Developer Journeys (Phase 5a)*
