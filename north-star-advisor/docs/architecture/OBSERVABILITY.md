# Observability Architecture

**Library-native observability for an embeddable Rust memory engine.**

Alaya is not a deployed service. It has no APM dashboard, no OpenTelemetry collector, no Prometheus endpoint. It is a Rust library that consumers embed into their own applications. Observability means giving those consumers structured visibility into library internals -- query latency, lifecycle throughput, pipeline stage breakdowns, cache behavior, degradation events -- through the Rust `tracing` ecosystem, without ever leaking private data.

This document specifies the complete observability architecture: span hierarchy, event catalog, structured fields, PII-safe logging policy, performance counters, consumer integration patterns, and the optional feature-flagged metrics export interface.

---

## Table of Contents

1. [Observability Philosophy for Libraries](#1-observability-philosophy-for-libraries)
2. [tracing Crate Integration](#2-tracing-crate-integration)
3. [Span Hierarchy](#3-span-hierarchy)
4. [Structured Fields](#4-structured-fields)
5. [Event Catalog](#5-event-catalog)
6. [PII-Safe Logging Policy](#6-pii-safe-logging-policy)
7. [Performance Counters](#7-performance-counters)
8. [Consumer Integration Guide](#8-consumer-integration-guide)
9. [Feature-Flagged Metrics](#9-feature-flagged-metrics)
10. [Debug Diagnostics](#10-debug-diagnostics)
11. [Implementation Roadmap](#11-implementation-roadmap)
12. [Appendix: Full Span and Event Reference](#appendix-full-span-and-event-reference)

---

## 1. Observability Philosophy for Libraries

### The Consumer Owns the Subscriber

Alaya emits structured spans and events. It never installs a tracing subscriber, never writes to stdout, never opens a network connection, never sends telemetry anywhere. The consumer decides what happens with the data.

This is a direct consequence of Alaya's architectural constraints:

- **Zero network calls** in the core crate (privacy by architecture)
- **No telemetry, analytics, or crash reporting** (security guardrail)
- **Single-file invariant** (all persistent state in one SQLite file, nothing else)

The consumer chooses their subscriber stack:

| Consumer Goal | Subscriber Stack |
|---|---|
| Development debugging | `tracing-subscriber` with `fmt::Layer` to stdout |
| Structured JSON logs | `tracing-subscriber` with `json` feature |
| Performance profiling | `tracing-timing` or `tokio-console` |
| Distributed tracing | `tracing-opentelemetry` with Jaeger/Zipkin exporter |
| Custom metrics pipeline | `tracing-subscriber` with custom `Layer` implementation |
| Production silence | No subscriber installed (zero overhead from `tracing` macros) |

Alaya's only responsibility is to emit the right spans at the right granularity with the right fields -- and to never, under any circumstances, include episode content, user messages, or any data classified as high-sensitivity PII in those emissions.

### Zero-Overhead When Unused

The `tracing` crate is designed for exactly this use case. When no subscriber is installed, `tracing` macros compile to a check against a global atomic and return immediately. The cost is a single branch prediction per span entry/exit. For a library where the hot path (`query()`) involves SQLite I/O measured in microseconds to milliseconds, this overhead is immeasurable.

When `tracing` is added as an optional dependency behind a feature flag, consumers who do not enable the flag pay zero cost -- the macros are replaced with no-ops at compile time. This matches Alaya's principle of graceful degradation: no tracing subscriber installed means no tracing overhead, just as no embedding provided means BM25-only retrieval.

### Three Levels of Observability

Alaya provides observability at three levels, each independently useful:

1. **Typed Reports** (already implemented) -- Every lifecycle method returns a structured report (`ConsolidationReport`, `ForgettingReport`, `TransformationReport`, `PerfumingReport`, `PurgeReport`). These are the primary observability mechanism. They require no subscriber, no feature flag, no configuration. They are return values.

2. **tracing Spans and Events** (this document) -- Structured spans around every public method and internal pipeline stage. Structured events for significant state changes, degradation, and errors. Requires the `tracing` feature flag and a consumer-installed subscriber.

3. **Metrics Export Interface** (planned, v0.2) -- An optional trait that consumers implement to receive periodic counter snapshots (query latency histograms, consolidation throughput, store sizes). Requires the `metrics` feature flag.

The three levels compose. A consumer can use typed reports alone (simplest), add tracing for development debugging, and later wire up the metrics trait for production monitoring -- all without changing their Alaya integration code.

---

## 2. tracing Crate Integration

### Dependency Configuration

The `tracing` dependency is behind a feature flag so that consumers who do not need instrumentation pay zero compile-time and runtime cost.

```toml
# Cargo.toml
[features]
default = []
tracing = ["dep:tracing"]

[dependencies]
tracing = { version = "0.1", optional = true }
```

Inside the library, all tracing usage is gated behind `#[cfg(feature = "tracing")]` with convenience macros that resolve to no-ops when the feature is disabled:

```rust
// src/instrument.rs -- internal convenience macros

/// Emit a tracing span. No-op when `tracing` feature is disabled.
macro_rules! alaya_span {
    ($level:expr, $name:expr, $($field:tt)*) => {
        #[cfg(feature = "tracing")]
        let _span = tracing::span!($level, $name, $($field)*).entered();
    };
}

/// Emit a tracing event. No-op when `tracing` feature is disabled.
macro_rules! alaya_event {
    ($level:expr, $($arg:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing::event!($level, $($arg)*);
    };
}

/// Attribute macro wrapper. When tracing is enabled, uses #[tracing::instrument].
/// When disabled, the function compiles without any instrumentation overhead.
```

For public API methods on `AlayaStore`, the `#[instrument]` attribute macro provides the cleanest integration:

```rust
use tracing::instrument;

impl AlayaStore {
    #[cfg_attr(feature = "tracing", instrument(
        skip(self, episode),
        fields(
            session_id = %episode.session_id,
            role = %episode.role.as_str(),
            content_len = episode.content.len(),
            has_embedding = episode.embedding.is_some(),
        )
    ))]
    pub fn store_episode(&self, episode: &NewEpisode) -> Result<EpisodeId> {
        // ...existing implementation...
    }
}
```

The `skip(self, episode)` is critical: it prevents the `#[instrument]` macro from attempting to `Debug`-format the `NewEpisode` struct, which would include episode content -- a PII violation. Instead, we explicitly list only the safe fields.

### Target Naming Convention

All spans and events use the `alaya` target prefix for filtering:

```
alaya::store_episode
alaya::query
alaya::retrieval::bm25
alaya::retrieval::vector
alaya::retrieval::graph
alaya::retrieval::fusion
alaya::retrieval::rerank
alaya::lifecycle::consolidation
alaya::lifecycle::perfuming
alaya::lifecycle::transformation
alaya::lifecycle::forgetting
alaya::graph::activation
alaya::store::strengths
```

Consumers filter with standard `tracing-subscriber` directives:

```rust
// Show all Alaya spans at DEBUG level, silence everything else
tracing_subscriber::fmt()
    .with_env_filter("alaya=debug")
    .init();

// Show only retrieval pipeline at TRACE level
tracing_subscriber::fmt()
    .with_env_filter("alaya::retrieval=trace")
    .init();

// Show lifecycle processes at INFO, everything else at WARN
tracing_subscriber::fmt()
    .with_env_filter("alaya::lifecycle=info,alaya=warn")
    .init();
```

---

## 3. Span Hierarchy

### Design Principle: One Root Span Per Public Method

Every public method on `AlayaStore` creates a root span. Internal functions called by that method create child spans. This gives consumers a clean tree:

```
alaya::query                          [INFO]  ── root span
  alaya::retrieval::bm25              [DEBUG]   ├── stage 1a
  alaya::retrieval::vector            [DEBUG]   ├── stage 1b
  alaya::retrieval::graph_activation  [DEBUG]   ├── stage 1c
  alaya::retrieval::rrf_fusion        [DEBUG]   ├── stage 2
  alaya::retrieval::enrich            [TRACE]   ├── stage 3
  alaya::retrieval::rerank            [DEBUG]   ├── stage 4
  alaya::retrieval::post_retrieval    [TRACE]   └── stage 5
    alaya::store::on_access           [TRACE]       ├── per-result
    alaya::graph::co_retrieval        [TRACE]       └── per-pair
```

### Complete Span Tree

#### Write Path

```
alaya::store_episode                  [INFO]
  alaya::store::episodic_insert       [DEBUG]
  alaya::store::embedding_store       [DEBUG]  (if embedding provided)
  alaya::store::strength_init         [TRACE]
  alaya::graph::temporal_link         [TRACE]  (if preceding_episode set)
```

#### Query Path (Retrieval Pipeline)

```
alaya::query                          [INFO]
  alaya::retrieval::bm25              [DEBUG]
    alaya::retrieval::bm25_sanitize   [TRACE]
    alaya::retrieval::bm25_fts5      [TRACE]
    alaya::retrieval::bm25_normalize  [TRACE]
  alaya::retrieval::vector            [DEBUG]
    alaya::retrieval::vector_search   [TRACE]
  alaya::retrieval::graph_activation  [DEBUG]
    alaya::graph::spread              [TRACE]  (per hop)
  alaya::retrieval::rrf_fusion        [DEBUG]
  alaya::retrieval::enrich            [TRACE]
    alaya::store::get_episode         [TRACE]  (per candidate)
  alaya::retrieval::rerank            [DEBUG]
  alaya::retrieval::post_retrieval    [TRACE]
    alaya::store::on_access           [TRACE]  (per result)
    alaya::graph::co_retrieval        [TRACE]  (per pair)
```

#### Lifecycle Processes

**Consolidation (CLS replay):**

```
alaya::consolidate                    [INFO]
  alaya::lifecycle::fetch_episodes    [DEBUG]
  alaya::lifecycle::provider_extract  [DEBUG]
  alaya::lifecycle::store_node        [DEBUG]  (per node)
    alaya::store::semantic_insert     [TRACE]
    alaya::graph::causal_link         [TRACE]  (per source episode)
    alaya::store::strength_init       [TRACE]
```

**Perfuming (vasana):**

```
alaya::perfume                        [INFO]
  alaya::lifecycle::extract_impressions [DEBUG]
  alaya::lifecycle::store_impression  [TRACE]  (per impression)
  alaya::lifecycle::check_crystallization [DEBUG] (per domain)
    alaya::lifecycle::crystallize     [DEBUG]  (if threshold met)
    alaya::lifecycle::reinforce       [TRACE]  (if preference exists)
```

**Transformation (asraya-paravrtti):**

```
alaya::transform                      [INFO]
  alaya::lifecycle::dedup_semantic    [DEBUG]
  alaya::lifecycle::prune_links       [DEBUG]
  alaya::lifecycle::decay_preferences [DEBUG]
  alaya::lifecycle::prune_preferences [DEBUG]
  alaya::lifecycle::prune_impressions [DEBUG]
```

**Forgetting (Bjork dual-strength):**

```
alaya::forget                         [INFO]
  alaya::lifecycle::decay_retrieval   [DEBUG]
  alaya::lifecycle::find_archivable   [DEBUG]
  alaya::lifecycle::archive_node      [TRACE]  (per archived node)
```

#### Admin Operations

```
alaya::status                         [DEBUG]
alaya::purge                          [INFO]
  alaya::purge::delete_episodes       [DEBUG]
  alaya::purge::delete_semantic       [DEBUG]
  alaya::purge::delete_all            [DEBUG]
```

#### Database Initialization

```
alaya::open                           [INFO]
  alaya::schema::init                 [DEBUG]
  alaya::schema::pragmas              [TRACE]
  alaya::schema::tables               [TRACE]
  alaya::schema::indexes              [TRACE]
  alaya::schema::triggers             [TRACE]
```

### Span Level Guidelines

| Level | Usage | Example |
|---|---|---|
| `ERROR` | Operation failed, data may be inconsistent | SQLite BUSY, provider panicked |
| `WARN` | Degraded behavior, operation still succeeded | No embeddings provided, empty FTS5 result, archivable node deletion failed |
| `INFO` | Public method entry/exit with summary metrics | `query` completed, `consolidate` report, `store_episode` succeeded |
| `DEBUG` | Internal pipeline stage boundaries | BM25 stage, RRF fusion, dedup pass |
| `TRACE` | Per-item operations within a stage | Individual episode fetch, per-node strength update, per-link co-retrieval |

---

## 4. Structured Fields

### Field Naming Convention

All fields use `snake_case`. Fields that represent counts end with `_count`. Fields that represent durations end with `_ms` or `_us`. Fields that represent sizes end with `_len` or `_bytes`. Fields that reference IDs use the type name prefix (e.g., `episode_id`, `node_id`).

### Safe Fields by Operation

#### store_episode

```rust
fields(
    session_id,            // String -- session identifier
    role,                  // &str -- "user", "assistant", "system"
    content_len,           // usize -- byte length of content (NEVER content itself)
    has_embedding,         // bool -- whether an embedding vector was provided
    embedding_dim,         // Option<usize> -- dimension if embedding present
    has_preceding,         // bool -- whether preceding_episode was set
    topic_count,           // usize -- number of topics in context
    entity_count,          // usize -- number of mentioned_entities in context
    episode_id,            // i64 -- the assigned ID (recorded on exit)
)
```

#### query

```rust
fields(
    query_text_len,        // usize -- byte length of query text (NEVER text itself)
    has_embedding,         // bool -- whether a query embedding was provided
    embedding_dim,         // Option<usize> -- dimension if embedding present
    max_results,           // usize -- requested result limit
    topic_count,           // usize -- topics in query context
    entity_count,          // usize -- entities in query context
    result_count,          // usize -- actual results returned (recorded on exit)
    bm25_count,            // usize -- BM25 candidates
    vector_count,          // usize -- vector candidates
    graph_count,           // usize -- graph activation candidates
    fused_count,           // usize -- candidates after RRF
    retrieval_channels,    // u8 -- how many channels contributed (1-3)
)
```

#### consolidate

```rust
fields(
    episodes_available,    // usize -- unconsolidated episodes found
    episodes_processed,    // u32 -- episodes sent to provider
    nodes_created,         // u32 -- semantic nodes stored
    links_created,         // u32 -- causal links created
    below_threshold,       // bool -- true if < 3 episodes, skipped
)
```

#### perfume

```rust
fields(
    session_id,            // String
    role,                  // &str
    interaction_text_len,  // usize -- byte length (NEVER text itself)
    impressions_stored,    // u32
    domains_checked,       // usize
    preferences_crystallized, // u32
    preferences_reinforced,   // u32
)
```

#### transform

```rust
fields(
    duplicates_merged,     // u32
    links_pruned,          // u32
    preferences_decayed,   // u32
    impressions_pruned,    // u32
    semantic_embeddings_checked, // usize -- number of embeddings compared
)
```

#### forget

```rust
fields(
    nodes_decayed,         // u32 -- nodes with RS reduced
    nodes_archived,        // u32 -- nodes below both thresholds, deleted
    decay_factor,          // f32 -- the RS decay factor used (0.95)
)
```

#### status

```rust
fields(
    episode_count,         // u64
    semantic_node_count,   // u64
    preference_count,      // u64
    impression_count,      // u64
    link_count,            // u64
    embedding_count,       // u64
)
```

#### purge

```rust
fields(
    filter_type,           // &str -- "session", "older_than", "all"
    // session_id only if filter is Session (it is not PII -- it is an opaque identifier)
    session_id,            // Option<String>
    episodes_deleted,      // u32
    nodes_deleted,         // u32
    links_deleted,         // u32
    embeddings_deleted,    // u32
)
```

### Fields That Must NEVER Appear

These fields are explicitly prohibited in any span or event. See Section 6 for the full PII-safe logging policy.

| Prohibited Field | Reason | Safe Alternative |
|---|---|---|
| `content` | Episode text is high-sensitivity PII | `content_len` |
| `query_text` | Query text may contain PII | `query_text_len` |
| `observation` | Impression observation text | `observation_len` |
| `preference_text` | Crystallized preference text | `preference_domain` |
| `embedding` | Invertible to approximate content | `embedding_dim` |
| `mentioned_entities` values | May contain personal names | `entity_count` |
| `topics` values | May reveal conversation subject | `topic_count` |

---

## 5. Event Catalog

Every structured event emitted by Alaya is cataloged here with its level, target, fields, and the condition that triggers it.

### Write Path Events

| Event | Level | Target | Fields | When |
|---|---|---|---|---|
| `episode_stored` | INFO | `alaya::store_episode` | `episode_id`, `session_id`, `content_len`, `role` | After successful INSERT |
| `embedding_stored` | DEBUG | `alaya::store_episode` | `episode_id`, `embedding_dim`, `model` | After embedding INSERT |
| `strength_initialized` | TRACE | `alaya::store_episode` | `node_type`, `node_id` | After node_strengths INSERT |
| `temporal_link_created` | TRACE | `alaya::store_episode` | `from_episode_id`, `to_episode_id` | After temporal link INSERT |
| `no_preceding_episode` | TRACE | `alaya::store_episode` | `episode_id` | When preceding_episode is None |

### Retrieval Pipeline Events

| Event | Level | Target | Fields | When |
|---|---|---|---|---|
| `query_started` | INFO | `alaya::query` | `query_text_len`, `has_embedding`, `max_results` | On query entry |
| `bm25_completed` | DEBUG | `alaya::retrieval::bm25` | `result_count`, `query_sanitized_len` | After FTS5 search |
| `bm25_empty_query` | DEBUG | `alaya::retrieval::bm25` | -- | When sanitized query is empty |
| `bm25_no_matches` | DEBUG | `alaya::retrieval::bm25` | `query_sanitized_len` | When FTS5 returns 0 rows |
| `vector_completed` | DEBUG | `alaya::retrieval::vector` | `result_count`, `embedding_dim` | After vector search |
| `vector_skipped` | DEBUG | `alaya::retrieval::vector` | -- | When no query embedding provided |
| `graph_activation_completed` | DEBUG | `alaya::retrieval::graph` | `seed_count`, `activated_count`, `hops` | After spreading activation |
| `graph_activation_skipped` | DEBUG | `alaya::retrieval::graph` | -- | When no seeds available |
| `rrf_fusion_completed` | DEBUG | `alaya::retrieval::fusion` | `input_sets`, `fused_count`, `k` | After RRF merge |
| `rerank_completed` | DEBUG | `alaya::retrieval::rerank` | `candidate_count`, `output_count` | After reranking |
| `query_completed` | INFO | `alaya::query` | `result_count`, `retrieval_channels` | On query exit |
| `query_empty_result` | WARN | `alaya::query` | `query_text_len`, `has_embedding` | When query returns 0 results and DB is non-empty |
| `on_access_updated` | TRACE | `alaya::retrieval::post` | `node_type`, `node_id`, `new_ss`, `new_rs` | After strength update |
| `co_retrieval_ltp` | TRACE | `alaya::retrieval::post` | `source_type`, `source_id`, `target_type`, `target_id` | After Hebbian LTP |

### Lifecycle Events

| Event | Level | Target | Fields | When |
|---|---|---|---|---|
| `consolidation_started` | INFO | `alaya::consolidate` | `episodes_available` | On entry |
| `consolidation_skipped` | DEBUG | `alaya::consolidate` | `episodes_available`, `threshold` | When < 3 episodes |
| `provider_extract_completed` | DEBUG | `alaya::consolidate` | `nodes_returned` | After provider.extract_knowledge() |
| `semantic_node_stored` | DEBUG | `alaya::consolidate` | `node_id`, `node_type`, `confidence`, `source_count` | After semantic INSERT |
| `consolidation_completed` | INFO | `alaya::consolidate` | `episodes_processed`, `nodes_created`, `links_created` | On exit |
| `perfuming_started` | INFO | `alaya::perfume` | `session_id`, `interaction_text_len` | On entry |
| `impressions_extracted` | DEBUG | `alaya::perfume` | `impression_count` | After provider.extract_impressions() |
| `crystallization_check` | DEBUG | `alaya::perfume` | `domain`, `impression_count`, `threshold` | Per domain check |
| `preference_crystallized` | INFO | `alaya::perfume` | `domain`, `confidence` | New preference created |
| `preference_reinforced` | DEBUG | `alaya::perfume` | `preference_id`, `domain`, `new_evidence_count` | Existing preference reinforced |
| `perfuming_completed` | INFO | `alaya::perfume` | `impressions_stored`, `preferences_crystallized`, `preferences_reinforced` | On exit |
| `transformation_started` | INFO | `alaya::transform` | -- | On entry |
| `dedup_completed` | DEBUG | `alaya::transform` | `embeddings_compared`, `duplicates_found` | After dedup pass |
| `links_pruned` | DEBUG | `alaya::transform` | `links_pruned`, `threshold` | After link pruning |
| `preferences_decayed` | DEBUG | `alaya::transform` | `preferences_decayed`, `half_life_days` | After preference decay |
| `impressions_pruned` | DEBUG | `alaya::transform` | `impressions_pruned`, `max_age_days` | After impression pruning |
| `transformation_completed` | INFO | `alaya::transform` | `duplicates_merged`, `links_pruned`, `preferences_decayed`, `impressions_pruned` | On exit |
| `forgetting_started` | INFO | `alaya::forget` | -- | On entry |
| `retrieval_decay_applied` | DEBUG | `alaya::forget` | `nodes_decayed`, `decay_factor` | After RS decay sweep |
| `archivable_found` | DEBUG | `alaya::forget` | `archivable_count`, `ss_threshold`, `rs_threshold` | After archivable query |
| `node_archived` | DEBUG | `alaya::forget` | `node_type`, `node_id` | Per archived node deletion |
| `forgetting_completed` | INFO | `alaya::forget` | `nodes_decayed`, `nodes_archived` | On exit |

### Database Events

| Event | Level | Target | Fields | When |
|---|---|---|---|---|
| `db_opened` | INFO | `alaya::open` | `path` (file path, not content), `is_memory` | After successful open |
| `schema_initialized` | DEBUG | `alaya::schema` | `tables`, `triggers`, `indexes` | After schema creation |
| `purge_completed` | INFO | `alaya::purge` | `filter_type`, `episodes_deleted` | After purge |
| `sqlite_busy` | WARN | `alaya::db` | `operation`, `retry_count` | On SQLITE_BUSY error |
| `sqlite_error` | ERROR | `alaya::db` | `operation`, `error_code` | On unrecoverable SQLite error |

### Degradation Events

These events signal that Alaya is operating in a degraded mode. They are WARN level because the operation still succeeds, but the consumer should know that quality may be reduced.

| Event | Level | Target | Fields | When |
|---|---|---|---|---|
| `degraded_no_embeddings` | WARN | `alaya::query` | `query_text_len` | Query has no embedding, vector channel skipped |
| `degraded_no_links` | WARN | `alaya::query` | `seed_count` | Graph activation skipped, no links in DB |
| `degraded_bm25_only` | WARN | `alaya::query` | `query_text_len` | Only BM25 channel available |
| `degraded_empty_db` | DEBUG | `alaya::query` | -- | Database has zero episodes |
| `provider_returned_empty` | DEBUG | `alaya::consolidate` | `episodes_sent` | Provider returned no nodes |
| `no_op_provider_detected` | DEBUG | `alaya::consolidate` | -- | Provider implements no-op pattern |

---

## 6. PII-Safe Logging Policy

### Classification

Alaya's data classification (from the Security Architecture) divides data into three sensitivity tiers. The logging policy maps directly:

| Sensitivity | Tables | Logging Rule |
|---|---|---|
| **High** | `episodes`, `semantic_nodes`, `episodes_fts` | NEVER log content. Log only metadata: ID, length, type, timestamp, count. |
| **Medium** | `impressions`, `preferences`, `embeddings` | NEVER log observation/preference text or embedding vectors. Log domain, valence, confidence, dimension. |
| **Low** | `links`, `node_strengths` | Safe to log all fields. These contain only structural metadata. |

### What Is NEVER Logged

The following data categories must never appear in any span field, event field, or debug format output:

1. **Episode content** -- The `content` field of `NewEpisode`, `Episode`, or `ScoredMemory`. This is raw user conversation text. Always use `content_len` (byte length) instead.

2. **Query text** -- The `text` field of `Query`. This often contains user questions verbatim. Always use `query_text_len` instead.

3. **Interaction text** -- The `text` field of `Interaction`. Same as episode content. Always use `interaction_text_len` instead.

4. **Semantic node content** -- The `content` field of `SemanticNode` or `NewSemanticNode`. These contain extracted facts about the user (e.g., "User lives in San Francisco"). Always use `content_len` and `node_type` instead.

5. **Impression observations** -- The `observation` field of `Impression` or `NewImpression`. These contain behavioral observations (e.g., "prefers concise answers"). Always use `observation_len` and `domain` instead.

6. **Preference text** -- The `preference` field of `Preference`. This is a crystallized behavioral pattern. Always use `domain` and `confidence` instead.

7. **Embedding vectors** -- The `embedding` field on any type, and raw BLOB data from the `embeddings` table. Embeddings are invertible to approximate the original content. Always use `embedding_dim` (dimension count) instead.

8. **Entity names** -- The `mentioned_entities` values in `EpisodeContext` or `QueryContext`. These may contain personal names. Always use `entity_count` instead.

9. **Topic values** -- The `topics` values in `EpisodeContext` or `QueryContext`. These may reveal sensitive conversation subjects. Always use `topic_count` instead.

### What IS Safe to Log

| Data | Reason |
|---|---|
| Episode IDs, Node IDs, Link IDs | Surrogate keys with no PII content |
| Session IDs | Opaque identifiers, not user-chosen |
| Roles (`user`, `assistant`, `system`) | Enum values, not content |
| Timestamps (Unix seconds) | Temporal metadata |
| Counts (episode_count, result_count, etc.) | Aggregate numbers |
| Byte lengths (content_len, query_text_len) | Size metadata, not content |
| Confidence scores, weights, strengths | Numeric metadata |
| Node types, link types, semantic types | Enum categorizations |
| Domain names (for preferences/impressions) | Category labels chosen by provider, not by user |
| Embedding dimensions | Integer, not the vector |
| Boolean flags (has_embedding, has_preceding) | Binary metadata |

### Content Hashing for Correlation

When consumers need to correlate events across spans without exposing content, Alaya provides a content hash field. This is a truncated BLAKE3 hash (first 8 bytes, hex-encoded as 16 characters) that allows matching "the same episode was stored and later retrieved" without revealing what the episode contains:

```rust
/// Compute a PII-safe correlation token from content.
/// Returns the first 16 hex characters of a BLAKE3 hash.
/// This is NOT cryptographically binding (truncated), but sufficient
/// for log correlation within a session.
#[cfg(feature = "tracing")]
fn content_token(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
```

The `content_token` field appears in `store_episode` and `query` result spans, allowing consumers to correlate "episode stored with token X was retrieved by query Y" without seeing the text. This uses `DefaultHasher` (SipHash) rather than a cryptographic hash to avoid adding a dependency; it is not intended for security, only for log correlation.

### Enforcement Mechanism

PII safety is enforced through code structure, not runtime checks:

1. **`skip` in `#[instrument]`** -- Every `#[instrument]` attribute on a public method explicitly skips `self` and all content-bearing arguments. Only safe fields are listed.

2. **No `Debug` on content types** -- The `NewEpisode`, `Interaction`, and `Query` structs derive `Debug`, but the `#[instrument]` macro is configured with `skip_all` plus explicit `fields(...)` to prevent automatic `Debug` formatting.

3. **Code review rule** -- Any PR that adds a `tracing::event!` or `tracing::span!` with a string field that is not in the safe list requires explicit justification.

4. **Integration test** -- A test subscriber captures all events during a `store_episode -> query -> consolidate` cycle and asserts that no event field contains the original episode content string. This test runs in CI.

```rust
#[cfg(all(test, feature = "tracing"))]
mod pii_tests {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::Layer;
    use std::sync::{Arc, Mutex};

    struct ContentCapture {
        events: Arc<Mutex<Vec<String>>>,
        forbidden: Vec<String>,
    }

    // ... subscriber implementation that captures all event field values
    // and asserts none match the forbidden content strings ...

    #[test]
    fn test_no_content_in_spans() {
        let secret_content = "My social security number is 123-45-6789";
        // ... store episode with secret_content, query it, consolidate ...
        // ... assert secret_content does not appear in any captured event ...
    }
}
```

---

## 7. Performance Counters

### Counters Derivable from Spans

When a consumer installs a tracing subscriber with timing support (e.g., `tracing-timing`, `tracing-opentelemetry`, or a custom `Layer`), the span hierarchy automatically provides:

| Metric | Derivation | Unit |
|---|---|---|
| `alaya.query.duration` | Duration of `alaya::query` span | microseconds |
| `alaya.query.bm25.duration` | Duration of `alaya::retrieval::bm25` span | microseconds |
| `alaya.query.vector.duration` | Duration of `alaya::retrieval::vector` span | microseconds |
| `alaya.query.graph.duration` | Duration of `alaya::retrieval::graph` span | microseconds |
| `alaya.query.fusion.duration` | Duration of `alaya::retrieval::fusion` span | microseconds |
| `alaya.query.rerank.duration` | Duration of `alaya::retrieval::rerank` span | microseconds |
| `alaya.query.post_retrieval.duration` | Duration of post-retrieval updates | microseconds |
| `alaya.store_episode.duration` | Duration of `alaya::store_episode` span | microseconds |
| `alaya.consolidate.duration` | Duration of `alaya::consolidate` span | milliseconds |
| `alaya.perfume.duration` | Duration of `alaya::perfume` span | milliseconds |
| `alaya.transform.duration` | Duration of `alaya::transform` span | milliseconds |
| `alaya.forget.duration` | Duration of `alaya::forget` span | milliseconds |

### Counters Derivable from Event Fields

| Metric | Source Event | Field |
|---|---|---|
| `alaya.query.result_count` | `query_completed` | `result_count` |
| `alaya.query.retrieval_channels` | `query_completed` | `retrieval_channels` |
| `alaya.query.bm25_hits` | `bm25_completed` | `result_count` |
| `alaya.query.vector_hits` | `vector_completed` | `result_count` |
| `alaya.query.graph_activations` | `graph_activation_completed` | `activated_count` |
| `alaya.consolidation.episodes_processed` | `consolidation_completed` | `episodes_processed` |
| `alaya.consolidation.nodes_created` | `consolidation_completed` | `nodes_created` |
| `alaya.perfuming.impressions_stored` | `perfuming_completed` | `impressions_stored` |
| `alaya.perfuming.preferences_crystallized` | `perfuming_completed` | `preferences_crystallized` |
| `alaya.transformation.duplicates_merged` | `transformation_completed` | `duplicates_merged` |
| `alaya.transformation.links_pruned` | `transformation_completed` | `links_pruned` |
| `alaya.forgetting.nodes_archived` | `forgetting_completed` | `nodes_archived` |
| `alaya.degradation.count` | `degraded_*` events | count per type |

### Latency Budgets

Based on the architecture (single-threaded SQLite with WAL mode, brute-force vector search), expected latency budgets for a database with 10,000 episodes:

| Operation | Expected P50 | Expected P99 | Bottleneck |
|---|---|---|---|
| `store_episode` (no embedding) | 50 us | 200 us | SQLite INSERT + FTS5 trigger |
| `store_episode` (with embedding) | 100 us | 500 us | + embedding BLOB INSERT |
| `query` (BM25 only) | 200 us | 2 ms | FTS5 MATCH + normalize |
| `query` (BM25 + vector) | 2 ms | 20 ms | + brute-force cosine scan |
| `query` (full hybrid) | 3 ms | 30 ms | + graph activation + RRF |
| `consolidate` (10 episodes) | 500 us | 5 ms | Provider call dominates if not NoOp |
| `transform` (dedup pass) | 10 ms | 100 ms | O(n^2) embedding comparison |
| `forget` (decay sweep) | 100 us | 1 ms | Single UPDATE statement |

These budgets are targets for v0.2 benchmarking (divan). The tracing spans allow consumers to measure actual latency against these targets in their environment.

### Degradation Detection

Alaya emits specific events when operating in a degraded mode. Consumers can alert on these:

| Degradation | Event | Impact | Consumer Action |
|---|---|---|---|
| No embeddings in query | `degraded_no_embeddings` | Vector channel disabled, recall reduced | Ensure embedding provider is wired up |
| No links in database | `degraded_no_links` | Graph activation disabled, serendipity reduced | Run consolidation to create links |
| BM25-only retrieval | `degraded_bm25_only` | Single-channel retrieval, lowest quality | Add embeddings and run lifecycle |
| Empty database | `degraded_empty_db` | No results possible | Store episodes first |
| Provider returns empty | `provider_returned_empty` | Consolidation/perfuming no-ops | Check provider implementation |

---

## 8. Consumer Integration Guide

### Minimal Setup (Development)

```rust
use alaya::AlayaStore;

fn main() {
    // Install a simple stdout subscriber
    tracing_subscriber::fmt()
        .with_env_filter("alaya=debug")
        .init();

    let store = AlayaStore::open("./my_agent.db").unwrap();
    // All Alaya operations now emit spans and events to stdout
}
```

Output:

```
2026-02-26T10:30:00.000Z  INFO alaya::open: db_opened path="./my_agent.db" is_memory=false
2026-02-26T10:30:00.001Z DEBUG alaya::schema: schema_initialized tables=7 triggers=3 indexes=9
2026-02-26T10:30:00.050Z  INFO alaya::store_episode: episode_stored episode_id=1 session_id="s1" content_len=42 role="user"
2026-02-26T10:30:00.100Z  INFO alaya::query: query_started query_text_len=18 has_embedding=false max_results=5
2026-02-26T10:30:00.101Z DEBUG alaya::retrieval::bm25: bm25_completed result_count=1 query_sanitized_len=18
2026-02-26T10:30:00.101Z DEBUG alaya::retrieval::vector: vector_skipped
2026-02-26T10:30:00.102Z DEBUG alaya::retrieval::fusion: rrf_fusion_completed input_sets=1 fused_count=1 k=60
2026-02-26T10:30:00.102Z DEBUG alaya::retrieval::rerank: rerank_completed candidate_count=1 output_count=1
2026-02-26T10:30:00.103Z  INFO alaya::query: query_completed result_count=1 retrieval_channels=1
```

### Structured JSON Logging (Production)

```rust
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::fmt::format::FmtSpan;

fn setup_logging() {
    let subscriber = fmt::Subscriber::builder()
        .json()
        .with_env_filter(EnvFilter::new("alaya=info"))
        .with_span_events(FmtSpan::CLOSE) // emit duration on span close
        .with_target(true)
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
```

Output:

```json
{
  "timestamp": "2026-02-26T10:30:00.103Z",
  "level": "INFO",
  "target": "alaya::query",
  "message": "query_completed",
  "span": {
    "name": "query",
    "query_text_len": 18,
    "has_embedding": false,
    "max_results": 5
  },
  "fields": {
    "result_count": 1,
    "retrieval_channels": 1
  },
  "spans": [
    { "name": "query", "query_text_len": 18 }
  ]
}
```

### OpenTelemetry / Jaeger (Distributed Tracing)

```rust
use opentelemetry::global;
use opentelemetry_sdk::trace::TracerProvider;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

fn setup_otel() {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .unwrap();
    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter)
        .build();
    global::set_tracer_provider(provider.clone());

    let tracer = provider.tracer("alaya");
    let telemetry = OpenTelemetryLayer::new(tracer);

    let subscriber = Registry::default()
        .with(telemetry)
        .with(tracing_subscriber::EnvFilter::new("alaya=debug"));
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
```

This gives the consumer a full Jaeger trace view of the retrieval pipeline, with each stage as a child span of the root `query` span, including duration and all structured fields.

### tokio-console (Async Runtime Debugging)

For consumers using the `async` feature flag (planned v0.2):

```rust
use console_subscriber::ConsoleLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

fn setup_console() {
    let console_layer = ConsoleLayer::builder()
        .retention(std::time::Duration::from_secs(60))
        .server_addr(([127, 0, 0, 1], 6669))
        .init();

    let subscriber = Registry::default()
        .with(console_layer)
        .with(tracing_subscriber::EnvFilter::new("alaya=trace"));
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
```

### Custom Layer (Metrics Extraction)

Consumers who want to extract specific metrics from Alaya's spans without a full tracing pipeline can write a custom `Layer`:

```rust
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct AlayaMetricsLayer {
    pub query_count: Arc<AtomicU64>,
    pub total_results: Arc<AtomicU64>,
    pub degraded_count: Arc<AtomicU64>,
}

impl<S: tracing::Subscriber> Layer<S> for AlayaMetricsLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: Context<'_, S>,
    ) {
        let meta = event.metadata();
        if meta.target().starts_with("alaya::") {
            match meta.name() {
                "query_completed" => {
                    self.query_count.fetch_add(1, Ordering::Relaxed);
                    // Extract result_count from event fields...
                }
                name if name.starts_with("degraded_") => {
                    self.degraded_count.fetch_add(1, Ordering::Relaxed);
                }
                _ => {}
            }
        }
    }
}
```

### No Subscriber (Production Silence)

When no subscriber is installed, `tracing` macros are effectively free. The consumer pays only for the atomic load to check if a subscriber exists. For consumers who want absolute zero overhead, they simply do not enable the `tracing` feature flag:

```toml
[dependencies]
alaya = "0.1"  # No tracing, all macros are no-ops
```

---

## 9. Feature-Flagged Metrics

### The metrics Feature Flag (Planned v0.2)

Beyond `tracing` spans, some consumers want a higher-level metrics interface -- counters and histograms they can feed into Prometheus, StatsD, Datadog, or their own monitoring system. Alaya provides this through an optional `AlayaMetrics` trait:

```toml
# Cargo.toml
[features]
metrics = []  # No additional dependencies -- trait-only
```

```rust
/// Trait for consumers to receive periodic metric snapshots.
/// Alaya calls these methods at the end of each operation.
/// The consumer implements the trait to route metrics to their
/// monitoring system (Prometheus, StatsD, custom).
#[cfg(feature = "metrics")]
pub trait AlayaMetrics: Send + Sync {
    /// Called after each query() completes.
    fn on_query(
        &self,
        duration_us: u64,
        result_count: usize,
        retrieval_channels: u8,
        bm25_hits: usize,
        vector_hits: usize,
        graph_hits: usize,
    );

    /// Called after each store_episode() completes.
    fn on_store(&self, duration_us: u64, has_embedding: bool);

    /// Called after each lifecycle operation completes.
    fn on_lifecycle(
        &self,
        operation: &str, // "consolidate", "perfume", "transform", "forget"
        duration_us: u64,
        items_processed: u32,
        items_created: u32,
    );

    /// Called when a degradation event occurs.
    fn on_degradation(&self, degradation_type: &str);

    /// Called when a SQLite error occurs.
    fn on_error(&self, operation: &str, error_code: i32);
}
```

The `AlayaStore` optionally holds a `Box<dyn AlayaMetrics>`:

```rust
pub struct AlayaStore {
    conn: Connection,
    #[cfg(feature = "metrics")]
    metrics: Option<Box<dyn AlayaMetrics>>,
}

impl AlayaStore {
    /// Attach a metrics receiver. If not called, metrics are silently dropped.
    #[cfg(feature = "metrics")]
    pub fn with_metrics(mut self, metrics: impl AlayaMetrics + 'static) -> Self {
        self.metrics = Some(Box::new(metrics));
        self
    }
}
```

### Example Prometheus Implementation

```rust
use prometheus::{Histogram, IntCounter, IntGauge, register_histogram, register_int_counter};

struct PrometheusMetrics {
    query_duration: Histogram,
    query_count: IntCounter,
    store_count: IntCounter,
    degradation_count: IntCounter,
}

impl AlayaMetrics for PrometheusMetrics {
    fn on_query(
        &self,
        duration_us: u64,
        result_count: usize,
        retrieval_channels: u8,
        _bm25_hits: usize,
        _vector_hits: usize,
        _graph_hits: usize,
    ) {
        self.query_duration.observe(duration_us as f64 / 1_000_000.0);
        self.query_count.inc();
    }

    fn on_store(&self, _duration_us: u64, _has_embedding: bool) {
        self.store_count.inc();
    }

    fn on_lifecycle(&self, _op: &str, _dur: u64, _proc: u32, _created: u32) {}

    fn on_degradation(&self, _degradation_type: &str) {
        self.degradation_count.inc();
    }

    fn on_error(&self, _operation: &str, _error_code: i32) {}
}
```

### Relationship Between tracing and metrics

The two feature flags are independent and composable:

| tracing | metrics | Behavior |
|---|---|---|
| off | off | Reports only (return values). Zero overhead. |
| on | off | Spans and events emitted. Consumer-chosen subscriber. |
| off | on | AlayaMetrics trait called. No spans. |
| on | on | Both active. Consumer gets spans AND metrics callbacks. |

The `metrics` trait is intentionally simpler than `tracing`. It provides pre-aggregated, typed method calls instead of raw span/event streams. Consumers who want fine-grained pipeline stage timing should use `tracing`; consumers who want high-level dashboards should use `metrics`.

---

## 10. Debug Diagnostics

### The status() Method

`AlayaStore::status()` already returns a `MemoryStatus` struct with counts across all stores:

```rust
pub struct MemoryStatus {
    pub episode_count: u64,
    pub semantic_node_count: u64,
    pub preference_count: u64,
    pub impression_count: u64,
    pub link_count: u64,
    pub embedding_count: u64,
}
```

This is the primary diagnostic mechanism. It requires no feature flags, no subscriber, no configuration. Consumers call `store.status()` and get a complete snapshot.

### Extended Diagnostics (Planned v0.2)

A more detailed `diagnostics()` method will provide deeper introspection:

```rust
pub struct AlayaDiagnostics {
    // Store counts (same as MemoryStatus)
    pub status: MemoryStatus,

    // Schema
    pub schema_version: u32,
    pub sqlite_version: String,
    pub fts5_available: bool,

    // WAL health
    pub wal_pages: u64,
    pub wal_checkpointed: u64,
    pub journal_mode: String,

    // Strength distribution
    pub avg_storage_strength: f32,
    pub avg_retrieval_strength: f32,
    pub nodes_below_rs_threshold: u64,
    pub nodes_below_both_thresholds: u64,

    // Graph topology
    pub avg_links_per_node: f32,
    pub max_links_per_node: u32,
    pub orphan_node_count: u64,
    pub link_type_distribution: Vec<(String, u64)>,

    // Embedding health
    pub embedding_dimensions: Option<usize>,
    pub embedding_coverage_episodes: f32,    // % of episodes with embeddings
    pub embedding_coverage_semantic: f32,    // % of semantic nodes with embeddings

    // Retrieval quality indicators
    pub avg_bm25_result_count: f32,          // rolling average (last N queries)
    pub degradation_rate: f32,               // fraction of queries hitting degraded mode
}
```

This struct is designed for programmatic health checks. A consumer's monitoring system can call `diagnostics()` periodically and alert on:

- `embedding_coverage_episodes < 0.5` -- Most episodes lack embeddings, vector channel is mostly unused
- `avg_retrieval_strength < 0.1` -- Most nodes have decayed RS, queries will return stale results
- `nodes_below_both_thresholds > 100` -- Many archivable nodes, `forget()` should be called
- `orphan_node_count > 50` -- Nodes with no links, graph activation is disconnected
- `wal_pages > 10000` -- WAL file is growing, checkpoint needed

### SQLite PRAGMA Diagnostics

The `diagnostics()` method internally queries SQLite PRAGMAs to surface storage health:

```sql
PRAGMA journal_mode;          -- Should be "wal"
PRAGMA wal_checkpoint(PASSIVE);  -- Returns (busy, log, checkpointed)
PRAGMA page_count;
PRAGMA page_size;
PRAGMA freelist_count;
PRAGMA integrity_check;       -- Only in explicit health_check() method, expensive
```

### Health Check Levels

| Level | Method | Cost | Frequency |
|---|---|---|---|
| Quick | `status()` | 6 COUNT queries | Every request (cheap) |
| Standard | `diagnostics()` | ~20 queries + aggregation | Every minute / on-demand |
| Deep | `health_check()` | PRAGMA integrity_check | Weekly / after crash |

---

## 11. Implementation Roadmap

### v0.1 (Current Release)

Observability through typed reports only. No `tracing` dependency.

**Already implemented:**
- `ConsolidationReport` with `episodes_processed`, `nodes_created`, `links_created`
- `PerfumingReport` with `impressions_stored`, `preferences_crystallized`, `preferences_reinforced`
- `TransformationReport` with `duplicates_merged`, `links_pruned`, `preferences_decayed`, `impressions_pruned`
- `ForgettingReport` with `nodes_decayed`, `nodes_archived`
- `PurgeReport` with `episodes_deleted`, `nodes_deleted`, `links_deleted`, `embeddings_deleted`
- `MemoryStatus` with counts across all 6 tables
- `AlayaError` with typed variants (`Db`, `NotFound`, `InvalidInput`, `Serialization`, `Provider`)

### v0.1.x (Patch Series)

Add `tracing` as optional dependency behind feature flag.

**Tasks:**
1. Add `tracing = { version = "0.1", optional = true }` to `Cargo.toml`
2. Create `src/instrument.rs` with `alaya_span!` and `alaya_event!` macros
3. Add `#[cfg_attr(feature = "tracing", instrument(...))]` to all 9 public methods on `AlayaStore`
4. Add span entry/exit events to `execute_query()` pipeline stages
5. Add span entry/exit events to all 4 lifecycle functions
6. Add degradation events to query pipeline
7. Add PII safety integration test
8. Document `tracing` feature flag in README and crate-level docs

**Priority order:** query path first (most consumer-visible), then store_episode, then lifecycle, then admin.

### v0.2

Add `metrics` feature flag and `AlayaDiagnostics`.

**Tasks:**
1. Define `AlayaMetrics` trait
2. Add `metrics: Option<Box<dyn AlayaMetrics>>` to `AlayaStore`
3. Implement `with_metrics()` builder method
4. Call `metrics.on_query()` etc. from instrumented methods
5. Implement `diagnostics()` method with extended health data
6. Add divan benchmarks for all operations (baseline for latency budgets)
7. Document metrics integration patterns in examples/

### v0.3

Add `health_check()` with integrity verification, rolling query statistics, and degradation rate tracking.

---

## Appendix: Full Span and Event Reference

### Span Reference Table

| Span Name | Target | Level | Parent | Fields |
|---|---|---|---|---|
| `open` | `alaya::open` | INFO | none | `path`, `is_memory` |
| `store_episode` | `alaya::store_episode` | INFO | none | `session_id`, `role`, `content_len`, `has_embedding` |
| `episodic_insert` | `alaya::store::episodic` | DEBUG | `store_episode` | `episode_id` |
| `embedding_store` | `alaya::store::embedding` | DEBUG | `store_episode` | `episode_id`, `embedding_dim` |
| `strength_init` | `alaya::store::strengths` | TRACE | `store_episode` | `node_type`, `node_id` |
| `temporal_link` | `alaya::graph::links` | TRACE | `store_episode` | `from_id`, `to_id` |
| `query` | `alaya::query` | INFO | none | `query_text_len`, `has_embedding`, `max_results` |
| `bm25` | `alaya::retrieval::bm25` | DEBUG | `query` | `query_sanitized_len` |
| `vector` | `alaya::retrieval::vector` | DEBUG | `query` | `embedding_dim` |
| `graph_activation` | `alaya::retrieval::graph` | DEBUG | `query` | `seed_count`, `hops` |
| `rrf_fusion` | `alaya::retrieval::fusion` | DEBUG | `query` | `input_sets`, `k` |
| `enrich` | `alaya::retrieval::enrich` | TRACE | `query` | `candidate_count` |
| `rerank` | `alaya::retrieval::rerank` | DEBUG | `query` | `candidate_count` |
| `post_retrieval` | `alaya::retrieval::post` | TRACE | `query` | `result_count` |
| `consolidate` | `alaya::consolidate` | INFO | none | `episodes_available` |
| `fetch_episodes` | `alaya::lifecycle::consolidation` | DEBUG | `consolidate` | `batch_size` |
| `provider_extract` | `alaya::lifecycle::consolidation` | DEBUG | `consolidate` | `episodes_sent` |
| `store_node` | `alaya::lifecycle::consolidation` | DEBUG | `consolidate` | `node_type`, `confidence` |
| `perfume` | `alaya::perfume` | INFO | none | `session_id`, `interaction_text_len` |
| `extract_impressions` | `alaya::lifecycle::perfuming` | DEBUG | `perfume` | -- |
| `crystallize` | `alaya::lifecycle::perfuming` | DEBUG | `perfume` | `domain`, `impression_count` |
| `reinforce` | `alaya::lifecycle::perfuming` | TRACE | `perfume` | `preference_id`, `domain` |
| `transform` | `alaya::transform` | INFO | none | -- |
| `dedup_semantic` | `alaya::lifecycle::transformation` | DEBUG | `transform` | `embeddings_checked` |
| `prune_links` | `alaya::lifecycle::transformation` | DEBUG | `transform` | `threshold` |
| `decay_preferences` | `alaya::lifecycle::transformation` | DEBUG | `transform` | `half_life_days` |
| `prune_impressions` | `alaya::lifecycle::transformation` | DEBUG | `transform` | `max_age_days` |
| `forget` | `alaya::forget` | INFO | none | -- |
| `decay_retrieval` | `alaya::lifecycle::forgetting` | DEBUG | `forget` | `decay_factor` |
| `find_archivable` | `alaya::lifecycle::forgetting` | DEBUG | `forget` | `ss_threshold`, `rs_threshold` |
| `archive_node` | `alaya::lifecycle::forgetting` | TRACE | `forget` | `node_type`, `node_id` |
| `status` | `alaya::status` | DEBUG | none | -- |
| `purge` | `alaya::purge` | INFO | none | `filter_type` |

### Event Reference Table

| Event Name | Level | Target | Key Fields |
|---|---|---|---|
| `db_opened` | INFO | `alaya::open` | `path`, `is_memory` |
| `schema_initialized` | DEBUG | `alaya::schema` | `tables`, `triggers`, `indexes` |
| `episode_stored` | INFO | `alaya::store_episode` | `episode_id`, `session_id`, `content_len`, `role` |
| `embedding_stored` | DEBUG | `alaya::store_episode` | `episode_id`, `embedding_dim` |
| `strength_initialized` | TRACE | `alaya::store_episode` | `node_type`, `node_id` |
| `temporal_link_created` | TRACE | `alaya::store_episode` | `from_episode_id`, `to_episode_id` |
| `query_started` | INFO | `alaya::query` | `query_text_len`, `has_embedding`, `max_results` |
| `bm25_completed` | DEBUG | `alaya::retrieval::bm25` | `result_count`, `query_sanitized_len` |
| `bm25_empty_query` | DEBUG | `alaya::retrieval::bm25` | -- |
| `bm25_no_matches` | DEBUG | `alaya::retrieval::bm25` | `query_sanitized_len` |
| `vector_completed` | DEBUG | `alaya::retrieval::vector` | `result_count`, `embedding_dim` |
| `vector_skipped` | DEBUG | `alaya::retrieval::vector` | -- |
| `graph_activation_completed` | DEBUG | `alaya::retrieval::graph` | `seed_count`, `activated_count`, `hops` |
| `graph_activation_skipped` | DEBUG | `alaya::retrieval::graph` | -- |
| `rrf_fusion_completed` | DEBUG | `alaya::retrieval::fusion` | `input_sets`, `fused_count`, `k` |
| `rerank_completed` | DEBUG | `alaya::retrieval::rerank` | `candidate_count`, `output_count` |
| `query_completed` | INFO | `alaya::query` | `result_count`, `retrieval_channels` |
| `query_empty_result` | WARN | `alaya::query` | `query_text_len`, `has_embedding` |
| `on_access_updated` | TRACE | `alaya::retrieval::post` | `node_type`, `node_id` |
| `co_retrieval_ltp` | TRACE | `alaya::retrieval::post` | `source_type`, `source_id`, `target_type`, `target_id` |
| `consolidation_started` | INFO | `alaya::consolidate` | `episodes_available` |
| `consolidation_skipped` | DEBUG | `alaya::consolidate` | `episodes_available`, `threshold` |
| `provider_extract_completed` | DEBUG | `alaya::consolidate` | `nodes_returned` |
| `semantic_node_stored` | DEBUG | `alaya::consolidate` | `node_id`, `node_type`, `confidence`, `source_count` |
| `consolidation_completed` | INFO | `alaya::consolidate` | `episodes_processed`, `nodes_created`, `links_created` |
| `perfuming_started` | INFO | `alaya::perfume` | `session_id`, `interaction_text_len` |
| `impressions_extracted` | DEBUG | `alaya::perfume` | `impression_count` |
| `crystallization_check` | DEBUG | `alaya::perfume` | `domain`, `impression_count`, `threshold` |
| `preference_crystallized` | INFO | `alaya::perfume` | `domain`, `confidence` |
| `preference_reinforced` | DEBUG | `alaya::perfume` | `preference_id`, `domain` |
| `perfuming_completed` | INFO | `alaya::perfume` | `impressions_stored`, `preferences_crystallized`, `preferences_reinforced` |
| `transformation_started` | INFO | `alaya::transform` | -- |
| `dedup_completed` | DEBUG | `alaya::transform` | `embeddings_compared`, `duplicates_found` |
| `links_pruned` | DEBUG | `alaya::transform` | `links_pruned`, `threshold` |
| `preferences_decayed` | DEBUG | `alaya::transform` | `preferences_decayed`, `half_life_days` |
| `impressions_pruned` | DEBUG | `alaya::transform` | `impressions_pruned`, `max_age_days` |
| `transformation_completed` | INFO | `alaya::transform` | `duplicates_merged`, `links_pruned`, `preferences_decayed`, `impressions_pruned` |
| `forgetting_started` | INFO | `alaya::forget` | -- |
| `retrieval_decay_applied` | DEBUG | `alaya::forget` | `nodes_decayed`, `decay_factor` |
| `archivable_found` | DEBUG | `alaya::forget` | `archivable_count`, `ss_threshold`, `rs_threshold` |
| `node_archived` | DEBUG | `alaya::forget` | `node_type`, `node_id` |
| `forgetting_completed` | INFO | `alaya::forget` | `nodes_decayed`, `nodes_archived` |
| `purge_completed` | INFO | `alaya::purge` | `filter_type`, `episodes_deleted` |
| `degraded_no_embeddings` | WARN | `alaya::query` | `query_text_len` |
| `degraded_no_links` | WARN | `alaya::query` | `seed_count` |
| `degraded_bm25_only` | WARN | `alaya::query` | `query_text_len` |
| `degraded_empty_db` | DEBUG | `alaya::query` | -- |
| `provider_returned_empty` | DEBUG | `alaya::consolidate` | `episodes_sent` |
| `sqlite_busy` | WARN | `alaya::db` | `operation`, `retry_count` |
| `sqlite_error` | ERROR | `alaya::db` | `operation`, `error_code` |

---

## Design Decisions

### Why tracing and Not log?

The `log` crate provides unstructured string messages. The `tracing` crate provides structured spans with typed fields, hierarchical context propagation, and zero-overhead when unused. For a library with a multi-stage pipeline where consumers need to see per-stage latency breakdowns, `tracing` is the correct choice. The `tracing` crate also interoperates with `log` through the `tracing-log` bridge, so consumers using `log`-based infrastructure can still receive events.

### Why Feature-Flagged and Not Always-On?

Alaya's core dependency list is minimal: `rusqlite`, `serde`, `serde_json`, `thiserror`. Adding `tracing` increases compile time and binary size. For embedded use cases (IoT, WASM, resource-constrained environments), every dependency matters. Feature-gating `tracing` respects the principle that consumers who do not need instrumentation should not pay for it.

### Why a Separate metrics Trait Instead of Just tracing?

The `tracing` ecosystem is powerful but complex. Some consumers want a simple callback interface: "tell me the query latency and result count after each query." The `AlayaMetrics` trait provides this without requiring the consumer to understand subscribers, layers, or span hierarchies. It is a higher-level abstraction for consumers who want dashboards, not traces.

### Why No Automatic Metrics Aggregation?

Alaya does not maintain rolling averages, histograms, or percentile calculations internally. That is the consumer's responsibility. A library should emit raw observations; the monitoring system should aggregate them. This avoids bringing in histogram implementations, lock-free data structures, or timing dependencies that bloat the library for consumers who do not need them.

### Why content_len Instead of Content Hash by Default?

Content length is cheaper to compute, reveals less information, and is sufficient for most debugging scenarios ("was this a short message or a long document?"). Content hashing is provided as an opt-in (`content_token` function) for consumers who need correlation, but it is not emitted by default because even a truncated hash can serve as a fingerprint for content matching.

---

## Cross-References

- **Architecture Blueprint** (`architecture.yml`) -- Components, retrieval pipeline stages, lifecycle processes, tech stack
- **Security Architecture** (`security.yml`) -- Data classification (high/medium/low sensitivity), PII persistence threats, guardrails
- **Agent Prompts** (`agent-prompts.yml`) -- Consumer integration patterns, system prompt guidelines
- **ADR Index** (`adr.yml`) -- ADR-009 (zero network calls), ADR-008 (sync-first API)
