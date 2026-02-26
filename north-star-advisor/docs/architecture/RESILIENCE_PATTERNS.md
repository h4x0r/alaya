# Resilience Patterns for Alaya

> An embedded Rust memory library does not have network failures, service outages, or distributed consensus problems. It has a different class of resilience concerns: capability availability at runtime, SQLite transaction contention under WAL mode, safe re-execution of lifecycle operations, resource budget enforcement to prevent unbounded computation, and data integrity preservation across cascading deletions. This document defines Alaya's resilience architecture in those terms.

**Status**: Living document, grounded in codebase as of 2026-02-26
**Applies to**: Alaya v0.1.x
**Cross-references**: [ARCHITECTURE_BLUEPRINT](../ARCHITECTURE_BLUEPRINT.md), [SECURITY_ARCHITECTURE](../SECURITY_ARCHITECTURE.md), [ADR](../ADR.md)

---

## Table of Contents

1. [Resilience Philosophy for Embedded Libraries](#1-resilience-philosophy-for-embedded-libraries)
2. [Capability Detection and Graceful Degradation](#2-capability-detection-and-graceful-degradation)
3. [Retrieval Degradation Chain](#3-retrieval-degradation-chain)
4. [SQLite Transaction Resilience](#4-sqlite-transaction-resilience)
5. [Lifecycle Operation Idempotency](#5-lifecycle-operation-idempotency)
6. [Resource Budgets](#6-resource-budgets)
7. [Data Integrity Patterns](#7-data-integrity-patterns)
8. [Error Propagation Strategy](#8-error-propagation-strategy)
9. [Recovery Patterns](#9-recovery-patterns)
10. [Testing Resilience](#10-testing-resilience)

---

## 1. Resilience Philosophy for Embedded Libraries

### 1.1 What Resilience Means Without a Network

Traditional resilience engineering focuses on distributed systems: circuit breakers prevent cascading failures across services, retries with exponential backoff absorb transient network errors, bulkheads isolate resource pools. None of this applies to Alaya. The library lives inside the consumer's process. It makes zero network calls (ADR-009). Its sole external dependency is a single SQLite file on the local filesystem.

Resilience for Alaya means something fundamentally different:

| Distributed System Concept | Alaya Equivalent | Rationale |
|---|---|---|
| Circuit breaker | Capability detection | Detect what data is available (embeddings? links? FTS matches?) and route around missing capabilities |
| Retry with backoff | Transaction retry | Handle SQLite BUSY errors from WAL contention when a concurrent reader or writer holds a lock |
| Fallback chain | Retrieval degradation chain | The documented six-level fallback from full hybrid retrieval down to empty results |
| Timeout handling | Resource budgets | Cap embedding scan counts, link traversal depth, and result set sizes to prevent unbounded computation |
| Idempotency tokens | Operation idempotency | Ensure consolidate(), transform(), and forget() produce consistent results when re-executed |
| Health checks | Status probing | The `status()` method exposes counts across all stores, enabling the consumer to reason about memory health |

### 1.2 The Fundamental Invariant

Alaya's core resilience invariant is: **every public method returns `Result<T, AlayaError>` and never panics**. The library must never crash the consumer's process. This is enforced by the typed error model defined in `src/error.rs`:

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
```

Every variant represents a class of failure that the consumer can reason about and recover from. There are no `unwrap()` calls in library code paths (test code uses `unwrap()` deliberately). The `AlayaError::Db` variant wraps all SQLite errors, including `SQLITE_BUSY`, `SQLITE_CORRUPT`, and `SQLITE_FULL`, preserving the underlying error for consumer-side classification.

### 1.3 Design Axioms That Shape Resilience

Three of Alaya's five axioms directly inform its resilience posture:

- **Privacy > Features** (ADR-009): No network calls means no network failures. The entire class of transient connectivity errors is structurally impossible. This is the single most impactful resilience decision.
- **Simplicity > Completeness** (ADR-001): A single SQLite file means no coordination between storage engines, no cache invalidation across stores, no eventual consistency. The database is always consistent because SQLite guarantees ACID within a single file.
- **Correctness > Speed** (ADR-005): The Bjork dual-strength model and corroboration tracking mean that data quality degrades gracefully over time rather than catastrophically. Incorrect data loses retrieval strength; correct data gains storage strength.

### 1.4 The Consumer Contract

Alaya is a library, not a service. The consumer is responsible for:

1. **Thread safety**: Wrapping `AlayaStore` in `Arc<Mutex<AlayaStore>>` for multi-threaded access. Alaya is `Send` but not `Sync`.
2. **Lifecycle scheduling**: Calling `consolidate()`, `transform()`, and `forget()` at appropriate intervals. Alaya does not run background threads.
3. **Error handling**: Matching on `AlayaError` variants and implementing retry logic for `AlayaError::Db` when the underlying error is `SQLITE_BUSY`.
4. **Provider reliability**: Implementing `ConsolidationProvider` such that it does not panic or return unbounded data.
5. **Backup**: Using the SQLite backup API or filesystem-level copies to protect against file corruption.

---

## 2. Capability Detection and Graceful Degradation

### 2.1 Runtime Capability Probing

Unlike a microservice that can health-check its dependencies at startup, Alaya's capabilities depend on what data the consumer has stored. An Alaya database with no embeddings cannot perform vector search. A database with no graph links cannot spread activation. A database with no episodes has nothing for FTS5 to match. These are not failures; they are states.

The retrieval pipeline in `src/retrieval/pipeline.rs` probes capabilities implicitly at query time:

```rust
// Stage 1: Parallel retrieval (BM25 + vector + graph)
let bm25_results = bm25::search_bm25(conn, &query.text, fetch_limit)?;

let vector_results = match &query.embedding {
    Some(emb) => vector::search_vector(conn, emb, fetch_limit)?,
    None => vec![],  // No embedding provided -> capability absent
};

let seed_nodes: Vec<NodeRef> = bm25_results.iter().take(3)
    .chain(vector_results.iter().take(3))
    .map(|(nr, _)| *nr)
    .collect();

let graph_activation = if !seed_nodes.is_empty() {
    activation::spread_activation(conn, &seed_nodes, 1, 0.1, 0.6)?
} else {
    HashMap::new()  // No seeds -> no graph traversal
};
```

Each retrieval signal is probed independently. If any signal returns an empty result, the pipeline continues with whatever signals did produce results. This is not error handling; it is architectural design. The `rrf_merge` function in `src/retrieval/fusion.rs` accepts any number of result sets, including a single set or even zero sets.

### 2.2 Capability States

At any point in time, an Alaya database exists in one of these capability states:

| State | Episodes | Embeddings | Graph Links | FTS Data | Available Signals |
|---|---|---|---|---|---|
| Empty | 0 | 0 | 0 | 0 | None (returns `Ok(vec![])`) |
| Episodes only | >0 | 0 | 0 | >0 | BM25 |
| With embeddings | >0 | >0 | 0 | >0 | BM25 + Vector |
| With graph | >0 | 0 | >0 | >0 | BM25 + Graph |
| Full capability | >0 | >0 | >0 | >0 | BM25 + Vector + Graph |

The transition between states is driven by consumer actions:

- **Episodes only -> With embeddings**: Consumer provides `embedding` field in `NewEpisode`, or calls an `EmbeddingProvider` to backfill.
- **Episodes only -> With graph**: Temporal links are created automatically by `store_episode()` when `preceding_episode` is set in `EpisodeContext`. Co-retrieval links form through repeated `query()` calls.
- **Any state -> Full capability**: Achieved over time as episodes accumulate with embeddings and retrieval creates co-retrieval links.

### 2.3 Provider Capability Detection

The `ConsolidationProvider` trait is the boundary between Alaya's deterministic core and the consumer's LLM (or absence thereof). The `NoOpProvider` is the structural fallback:

```rust
pub struct NoOpProvider;

impl ConsolidationProvider for NoOpProvider {
    fn extract_knowledge(&self, _episodes: &[Episode]) -> Result<Vec<NewSemanticNode>> {
        Ok(vec![])  // No LLM -> no semantic extraction -> no error
    }

    fn extract_impressions(&self, _interaction: &Interaction) -> Result<Vec<NewImpression>> {
        Ok(vec![])  // No LLM -> no impression extraction -> no error
    }

    fn detect_contradiction(&self, _a: &SemanticNode, _b: &SemanticNode) -> Result<bool> {
        Ok(false)  // No LLM -> assume no contradiction -> conservative
    }
}
```

When a consumer passes `NoOpProvider` to `consolidate()`:
1. `get_unconsolidated_episodes()` fetches episodes normally.
2. `provider.extract_knowledge()` returns an empty vec.
3. No semantic nodes are created, no links are formed.
4. The report shows `episodes_processed: N, nodes_created: 0, links_created: 0`.
5. This is a successful operation, not a failure.

The same pattern applies to `perfume()`: with `NoOpProvider`, no impressions are extracted, no preferences crystallize, and the report reflects zero activity. Episodes continue to accumulate and remain searchable via BM25, which requires no provider at all.

### 2.4 Capability Detection vs. Error Handling

A critical distinction: capability detection produces empty results and succeeds. Error handling produces `AlayaError` and fails. The consumer must understand this difference:

```rust
// Capability detection (not an error):
let results = store.query(&Query::simple("Rust"))?;  // Ok(vec![]) on empty DB

// Error handling (actual failure):
let results = store.query(&Query::simple("Rust"));
match results {
    Ok(memories) => { /* process results, possibly empty */ }
    Err(AlayaError::Db(e)) => { /* SQLite failure: corrupt DB, locked, full disk */ }
    Err(e) => { /* other errors */ }
}
```

An empty result is a valid answer. The consumer should never treat `Ok(vec![])` as a failure state. The retrieval pipeline communicates through the `MemoryStatus` struct what data is available, enabling the consumer to explain empty results to the end user if desired.

---

## 3. Retrieval Degradation Chain

### 3.1 The Six-Level Chain

The retrieval degradation chain is Alaya's most important resilience pattern. It defines how query quality degrades as capabilities become unavailable, while ensuring that the library always returns a valid result:

**Level 1: Full hybrid retrieval**
```
BM25 + Vector + Graph -> RRF (k=60) -> Rerank (context + recency)
```
All three retrieval signals contribute. RRF fusion merges ranked lists. Reranking applies context similarity (topic Jaccard, entity Jaccard, sentiment distance) and recency decay (exp(-age_days/30)). Post-retrieval effects fire: node strengths update via `on_access()`, co-retrieval Hebbian links strengthen via `on_co_retrieval()`. This is the steady-state behavior of a well-populated database with active embeddings and an evolved graph.

**Level 2: No embeddings**
```
BM25 + Graph -> RRF (k=60) -> Rerank
```
When the consumer does not provide `query.embedding` (or no embeddings exist in the database), vector search returns an empty result set. RRF fusion receives two result sets instead of three. The graph signal compensates partially for the absence of semantic similarity, because co-retrieval links encode historical similarity relationships. Quality degrades measurably but the system remains functional.

Trigger: `query.embedding` is `None`, or the `embeddings` table is empty.

**Level 3: No graph links**
```
BM25 + Vector -> RRF (k=60) -> Rerank
```
When the graph has no links (no temporal relationships, no co-retrieval history), spreading activation returns an empty `HashMap`. RRF receives two result sets: BM25 and vector. This is the typical state of a new database that has embeddings but has not yet accumulated enough retrieval history to form co-retrieval links. The Hebbian graph is self-healing: each successful query creates new co-retrieval links, so this state is transient in normal operation.

Trigger: `seed_nodes` is empty (no BM25 or vector results to seed from), or `links` table has no rows matching seed nodes.

**Level 4: No FTS matches**
```
Vector + Graph -> RRF (k=60) -> Rerank
```
When the BM25 query matches no documents (rare for well-populated databases, common for highly specific or misspelled queries), FTS5 returns an empty result. Vector search and graph activation provide the remaining signals. Since BM25 is the primary text-matching signal, this level indicates either a data sparsity problem or a query formulation issue.

Trigger: `search_bm25()` returns an empty vec after sanitization, either because the sanitized query is empty or because FTS5 MATCH finds no documents.

**Level 5: BM25 only (minimal)**
```
BM25 -> Rerank (recency only, no RRF needed)
```
When only BM25 produces results (no embeddings, no graph), the single result set passes through RRF trivially (each document gets score `1/(k+rank+1)`) and then through reranking. This is the "day-one" experience: a consumer that stores plain text episodes without embeddings or explicit context. Quality is limited to keyword matching with BM25 scoring, enhanced only by recency decay.

Trigger: Both `vector_results` and `graph_results` are empty, but `bm25_results` has entries.

**Level 6: Empty result**
```
[] (empty vec, no error)
```
When all retrieval signals produce empty results, the pipeline returns `Ok(vec![])`. This happens when the database is empty, when the query matches nothing across all signals, or when the query text sanitizes to an empty string and no embedding is provided. This is a valid, expected state, not an error condition.

Trigger: Empty database, or completely unrelated query, or empty/whitespace-only query text without embedding.

### 3.2 How RRF Enables Graceful Degradation

Reciprocal Rank Fusion (ADR-006) is the architectural enabler of the degradation chain. Because RRF operates on ranks rather than scores, it does not require score normalization across heterogeneous signals. More importantly, it naturally handles a variable number of input signals:

```rust
pub fn rrf_merge(
    result_sets: &[Vec<(NodeRef, f64)>],
    k: u32,
) -> Vec<(NodeRef, f64)> {
    let mut scores: HashMap<NodeRef, f64> = HashMap::new();
    for result_set in result_sets {
        for (rank, (node_ref, _original_score)) in result_set.iter().enumerate() {
            *scores.entry(*node_ref).or_default() += 1.0 / (k as f64 + rank as f64 + 1.0);
        }
    }
    // ...sort and return
}
```

With three result sets (full capability), a document appearing in all three gets the maximum RRF boost. With two result sets (one signal absent), the same document gets a lower but still meaningful score. With one result set, RRF degenerates to a simple rank-based scoring. With zero result sets, the output is an empty vec. No special-casing is needed at any level.

The pipeline code in `src/retrieval/pipeline.rs` constructs the result sets dynamically:

```rust
let mut sets: Vec<Vec<(NodeRef, f64)>> = vec![bm25_results];
if !vector_results.is_empty() {
    sets.push(vector_results);
}
if !graph_results.is_empty() {
    sets.push(graph_results);
}
let fused = fusion::rrf_merge(&sets, 60);
```

Empty result sets are excluded from fusion, meaning they do not dilute scores of documents that do appear in other signals. This is a deliberate design choice: an absent signal should not penalize a present signal.

### 3.3 Post-Retrieval Effects at Each Level

Regardless of which degradation level is active, post-retrieval effects fire for all returned results:

```rust
// Stage 4: Post-retrieval updates
for scored in &results {
    let _ = strengths::on_access(conn, scored.node);
}

let retrieved_nodes: Vec<NodeRef> = results.iter().map(|r| r.node).collect();
for i in 0..retrieved_nodes.len() {
    for j in (i + 1)..retrieved_nodes.len() {
        let _ = crate::graph::links::on_co_retrieval(conn, retrieved_nodes[i], retrieved_nodes[j]);
    }
}
```

Two critical observations:

1. **Error suppression**: The `let _ =` pattern intentionally ignores errors from strength tracking and co-retrieval link formation. These are side effects that enhance future queries but are not essential to the current query's correctness. If they fail (e.g., due to a concurrent writer holding a lock), the query still returns valid results.

2. **Self-healing**: Co-retrieval links form even at degradation Level 5 (BM25 only). Over time, these links populate the graph, promoting the database from Level 5 toward Level 1. The Hebbian graph is an emergent property of retrieval behavior, not a dependency that must be pre-populated.

### 3.4 FTS5 Input Sanitization as Degradation

The BM25 module in `src/retrieval/bm25.rs` implements input sanitization that acts as a degradation mechanism:

```rust
let sanitized: String = query
    .chars()
    .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
    .collect();

if sanitized.trim().is_empty() {
    return Ok(vec![]);
}
```

Queries containing only special characters (e.g., `"@#$%"`) sanitize to an empty string and return an empty result without executing SQL. This is a security measure (FTS5 injection prevention) that doubles as a degradation step: invalid queries fail silently rather than erroneously. The consumer receives `Ok(vec![])` and can fall through to other signals or report "no results" to the end user.

---

## 4. SQLite Transaction Resilience

### 4.1 WAL Mode and Concurrency

Alaya configures SQLite in WAL (Write-Ahead Logging) mode via `PRAGMA journal_mode = WAL` in `src/schema.rs`. WAL mode enables concurrent readers alongside a single writer: multiple threads can read the database while one thread writes, without blocking each other. However, WAL mode introduces specific contention scenarios that Alaya must handle.

The current pragma configuration:

```rust
fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch("PRAGMA synchronous = NORMAL;")?;
    // ... table creation
}
```

`PRAGMA synchronous = NORMAL` reduces fsync frequency compared to `FULL`, accepting a small risk of WAL corruption on power loss in exchange for significantly better write throughput. This is the standard tradeoff for application-level databases (as opposed to banking systems).

### 4.2 The BEGIN IMMEDIATE Gap

**Known gap**: Alaya currently uses implicit transactions (SQLite's autocommit mode) for most operations. None of the current code explicitly calls `BEGIN IMMEDIATE` or `BEGIN EXCLUSIVE`. This creates a vulnerability documented in both the architecture and security outputs: BEGIN DEFERRED transactions (SQLite's default) can deadlock when two connections simultaneously attempt to promote a read transaction to a write transaction.

The failure scenario:
1. Connection A begins a read (implicitly DEFERRED).
2. Connection B begins a read (implicitly DEFERRED).
3. Connection A attempts to write. SQLite promotes to a write lock.
4. Connection B attempts to write. SQLite returns `SQLITE_BUSY` because A holds the write lock.
5. If A is waiting on B for something (unlikely in Alaya's single-store model, but possible with external coordination), deadlock occurs.

In practice, Alaya mitigates this risk through its single-owner design: `AlayaStore` owns the sole `Connection`, and the consumer wraps it in `Mutex` for multi-threaded access. This means only one thread can execute any operation at a time, eliminating the multi-connection deadlock scenario entirely.

However, there is still a risk when the consumer opens multiple `AlayaStore` instances pointing at the same file (e.g., for read-heavy workloads). The planned mitigation for v0.1 is to use `BEGIN IMMEDIATE` for all write transactions, which acquires the write lock at transaction start rather than on first write statement, making contention fail fast and deterministically.

### 4.3 SQLITE_BUSY Handling

When SQLite returns `SQLITE_BUSY` (error code 5), the operation could not acquire the necessary lock. This surfaces as `AlayaError::Db(rusqlite::Error::SqliteFailure(...))` with the underlying error code. The consumer should handle this:

```rust
use alaya::{AlayaStore, AlayaError};

fn store_with_retry(store: &AlayaStore, episode: &NewEpisode) -> alaya::Result<EpisodeId> {
    let mut attempts = 0;
    loop {
        match store.store_episode(episode) {
            Ok(id) => return Ok(id),
            Err(AlayaError::Db(ref e)) if is_busy(e) && attempts < 3 => {
                attempts += 1;
                std::thread::sleep(std::time::Duration::from_millis(10 * attempts));
            }
            Err(e) => return Err(e),
        }
    }
}

fn is_busy(e: &rusqlite::Error) -> bool {
    matches!(e, rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error { code: rusqlite::ffi::ErrorCode::DatabaseBusy, .. }, _
    ))
}
```

Alaya does not implement retry logic internally because the appropriate retry strategy depends on the consumer's concurrency model. A single-threaded consumer never needs retries. A multi-threaded consumer with `Arc<Mutex<AlayaStore>>` serializes all access and also never needs retries. Only consumers that open multiple `AlayaStore` instances against the same file need retry logic, and they are in the best position to define the retry budget.

### 4.4 Transaction Boundaries in Current Code

Most Alaya operations execute as individual SQL statements in SQLite's autocommit mode. Some operations execute multiple statements sequentially without explicit transaction boundaries. Let us examine each:

**store_episode()** (in `src/lib.rs`):
```rust
pub fn store_episode(&self, episode: &NewEpisode) -> Result<EpisodeId> {
    let id = store::episodic::store_episode(&self.conn, episode)?;
    if let Some(ref emb) = episode.embedding {
        store::embeddings::store_embedding(&self.conn, "episode", id.0, emb, "")?;
    }
    store::strengths::init_strength(&self.conn, NodeRef::Episode(id))?;
    if let Some(prev) = episode.context.preceding_episode {
        graph::links::create_link(&self.conn, /*...*/)?;
    }
    Ok(id)
}
```

This method executes 1-4 SQL statements without an explicit transaction. If the episode is inserted but the embedding storage fails, the database contains an episode without its embedding. This is not catastrophic (the embedding can be added later), but it violates the principle of atomic operations. The planned fix is to wrap `store_episode()` in a `BEGIN IMMEDIATE ... COMMIT` block.

**consolidate()** (in `src/lifecycle/consolidation.rs`):
```rust
pub fn consolidate(conn: &Connection, provider: &dyn ConsolidationProvider) -> Result<ConsolidationReport> {
    let episodes = episodic::get_unconsolidated_episodes(conn, CONSOLIDATION_BATCH_SIZE)?;
    // ... provider.extract_knowledge(&episodes) ...
    for node_data in new_nodes {
        let node_id = semantic::store_semantic_node(conn, &node_data)?;
        for ep_id in &node_data.source_episodes {
            links::create_link(conn, /*...*/)?;
        }
        strengths::init_strength(conn, NodeRef::Semantic(node_id))?;
    }
    Ok(report)
}
```

Consolidation executes many SQL statements (node creation, link creation, strength initialization) without transaction boundaries. A failure midway through creates partial consolidation: some nodes created, others not, some links missing. Re-running `consolidate()` partially compensates for this (see Section 5.1), but the ideal fix is transactional grouping.

**purge(PurgeFilter::All)** (in `src/lib.rs`):
```rust
PurgeFilter::All => {
    self.conn.execute_batch(
        "DELETE FROM episodes;
         DELETE FROM semantic_nodes;
         DELETE FROM impressions;
         DELETE FROM preferences;
         DELETE FROM embeddings;
         DELETE FROM links;
         DELETE FROM node_strengths;",
    )?;
}
```

`execute_batch` executes all statements in a single implicit transaction, which is the correct behavior for a purge operation. This is the one case in the current codebase where transactional atomicity is handled correctly by accident.

### 4.5 WAL Checkpoint Management

**Known gap**: Alaya does not configure `wal_autocheckpoint` or `journal_size_limit`. In the default SQLite configuration, WAL checkpoints occur automatically when the WAL file reaches 1000 pages (approximately 4MB with the default page size of 4096 bytes). For most Alaya workloads, this is sufficient.

The risk arises under sustained write pressure (e.g., bulk-loading thousands of episodes): the WAL file grows continuously, and automatic checkpoints may not keep pace if a reader holds a snapshot (preventing the WAL from being truncated). The planned mitigations for v0.1:

```sql
PRAGMA wal_autocheckpoint = 1000;      -- explicit (same as default, but stated)
PRAGMA journal_size_limit = 67108864;  -- 64MB cap on WAL file
```

A future `compact()` method will expose `VACUUM` and explicit `PRAGMA wal_checkpoint(TRUNCATE)` for consumer-controlled maintenance.

### 4.6 PRAGMA synchronous = NORMAL: The Tradeoff

The `NORMAL` synchronous mode syncs the WAL file at critical moments but not after every transaction commit. This means:

- **Durability**: A committed transaction may be lost if the operating system crashes or power fails between the commit and the next WAL sync. The SQLite documentation estimates this window at "a few tens of milliseconds."
- **Consistency**: The database will never become corrupt. After a crash, SQLite replays the WAL and the database is either at the state before the last transaction or after it. No intermediate states are visible.

For Alaya's use case (conversational memory, not financial transactions), the `NORMAL` tradeoff is appropriate. Losing the last few episodes after a power failure is acceptable; losing the entire database is not. The consumer can override this by setting `PRAGMA synchronous = FULL` on the connection if they require stronger durability guarantees.

---

## 5. Lifecycle Operation Idempotency

### 5.1 consolidate() Idempotency

The consolidation process (CLS replay) extracts semantic knowledge from unconsolidated episodes. An episode is considered "unconsolidated" if no semantic node links to it via the graph:

```rust
let episodes = episodic::get_unconsolidated_episodes(conn, CONSOLIDATION_BATCH_SIZE)?;
if episodes.len() < 3 {
    return Ok(report);  // Not enough episodes, no-op
}
```

Re-running `consolidate()` after a successful run is safe because:

1. Episodes that were already consolidated (linked to semantic nodes) are excluded from `get_unconsolidated_episodes()`.
2. The `NOT EXISTS` subqueries check for links between episodes and semantic nodes, so previously processed episodes will not appear again.
3. If consolidation partially completed (some nodes created, some not), the next run picks up the remaining unconsolidated episodes.

However, there is a subtle non-idempotency: if the `ConsolidationProvider` is non-deterministic (e.g., an LLM that generates different extractions each time), re-running consolidation on the same episodes (if the previous run created nodes but the links were not yet formed) could create duplicate semantic nodes. This is mitigated by the deduplication step in `transform()` (Section 5.3), but it is not prevented at the consolidation level.

**Idempotency guarantee**: Convergent. Multiple runs converge toward a fully consolidated state, but the path may differ. No data is lost or corrupted by re-execution.

### 5.2 forget() Idempotency

The forgetting process decays retrieval strength across all nodes and archives those below dual thresholds:

```rust
pub fn forget(conn: &Connection) -> Result<ForgettingReport> {
    report.nodes_decayed = strengths::decay_all_retrieval(conn, DEFAULT_DECAY_FACTOR)?;
    let archivable = strengths::find_archivable(conn, ARCHIVE_STORAGE_THRESHOLD, ARCHIVE_RETRIEVAL_THRESHOLD)?;
    for node in &archivable {
        // delete node, delete strength record
    }
    Ok(report)
}
```

Re-running `forget()` immediately after a previous run:

1. **Decay**: Applies `RS *= 0.95` again, further reducing retrieval strength. This is intentional but potentially aggressive. Two consecutive `forget()` calls apply `RS *= 0.95 * 0.95 = 0.9025`, equivalent to 1.76 half-lives instead of 1. Consumers must schedule `forget()` at appropriate intervals.
2. **Archival**: Nodes already deleted in the first run do not appear in `find_archivable()` because their strength records were removed. No double-deletion occurs.

**Idempotency guarantee**: Partially idempotent. The archival step is idempotent (deleting an already-deleted node is a no-op). The decay step is not idempotent (it applies multiplicatively each time). The consumer is responsible for calling `forget()` at appropriate intervals rather than repeatedly.

### 5.3 transform() Idempotency

The transformation process performs five sub-operations:

```rust
pub fn transform(conn: &Connection) -> Result<TransformationReport> {
    report.duplicates_merged = dedup_semantic_nodes(conn)?;
    report.links_pruned = links::prune_weak_links(conn, LINK_PRUNE_THRESHOLD)?;
    report.preferences_decayed = implicit::decay_preferences(conn, now, PREFERENCE_HALF_LIFE_SECS)?;
    report.preferences_decayed += implicit::prune_weak_preferences(conn, MIN_PREFERENCE_CONFIDENCE)?;
    report.impressions_pruned = implicit::prune_old_impressions(conn, MAX_IMPRESSION_AGE_SECS)?;
    Ok(report)
}
```

Idempotency analysis per sub-operation:

1. **Deduplication**: Idempotent. If two semantic nodes have been merged, the surviving node has no near-duplicate remaining, and the next run finds nothing to merge.
2. **Link pruning**: Idempotent. Links below threshold 0.02 are deleted on first run. No links below threshold exist for the second run.
3. **Preference decay**: Not idempotent. Applies `confidence *= 0.95` to preferences older than the half-life. Two consecutive runs double the decay.
4. **Preference pruning**: Idempotent. Preferences below confidence 0.05 are deleted on first run. No weak preferences exist for the second run.
5. **Impression pruning**: Idempotent. Impressions older than 90 days are deleted on first run. No old impressions exist for the second run.

**Idempotency guarantee**: Mostly idempotent. Only preference decay is non-idempotent, and it follows the same pattern as `forget()`: the consumer must schedule calls at appropriate intervals. The pruning operations are fully idempotent.

### 5.4 perfume() Idempotency

Perfuming is inherently non-idempotent because it accumulates impressions:

```rust
for imp in &impressions {
    implicit::store_impression(conn, imp)?;
    report.impressions_stored += 1;
}
```

Calling `perfume()` twice with the same interaction stores duplicate impressions. This is by design: the system does not track whether a specific interaction has been processed. The rationale is that perfuming is tied to real interactions, and each interaction genuinely represents a new observation (even if textually similar to a previous one). The crystallization threshold (5 impressions) provides natural dampening against rapid repeated calls, and the deduplication/pruning in `transform()` provides eventual cleanup.

**Idempotency guarantee**: Not idempotent. Each call accumulates new impressions. The consumer must ensure each interaction is perfumed exactly once.

### 5.5 purge() Idempotency

Purging is fully idempotent for all filter types:

- `PurgeFilter::Session(id)`: Deleting episodes for a session that has already been purged deletes zero rows.
- `PurgeFilter::OlderThan(ts)`: Deleting episodes older than a timestamp that has already been purged deletes zero rows.
- `PurgeFilter::All`: Executing DELETE on empty tables succeeds with zero rows affected.

**Idempotency guarantee**: Fully idempotent. Re-execution is always safe.

### 5.6 Idempotency Summary

| Operation | Idempotent | Re-execution Behavior | Consumer Guidance |
|---|---|---|---|
| `consolidate()` | Convergent | Picks up remaining work | Safe to re-run; call periodically |
| `forget()` | Partial (decay is not) | Extra decay applied | Schedule at intervals, do not loop |
| `transform()` | Mostly (decay is not) | Pruning is safe, decay doubles | Schedule at intervals, do not loop |
| `perfume()` | No | Duplicate impressions stored | Call exactly once per interaction |
| `purge()` | Yes | No-op on empty data | Safe to re-run |
| `store_episode()` | No | Duplicate episodes stored | Consumer deduplicates externally |
| `query()` | Side-effects only | Strengths update, links form | Safe to re-run |

---

## 6. Resource Budgets

### 6.1 Embedding Scan Budget

Vector search in `src/store/embeddings.rs` performs a brute-force scan of all embeddings:

```rust
pub fn search_by_vector(conn: &Connection, query_vec: &[f32], node_type_filter: Option<&str>, limit: usize)
    -> Result<Vec<(NodeRef, f32)>>
{
    let candidates: Vec<(String, i64, Vec<u8>)> = /* SELECT * FROM embeddings */;
    let mut results: Vec<(NodeRef, f32)> = candidates.into_iter()
        .filter_map(|..| { /* cosine_similarity */ })
        .collect();
    results.sort_by(..);
    results.truncate(limit);
    Ok(results)
}
```

This loads all embeddings into memory, computes cosine similarity for each, then sorts and truncates. The resource implications:

- **Memory**: Each embedding of dimension D occupies 4*D bytes. For 384-dimensional embeddings (e.g., all-MiniLM-L6-v2), 50,000 embeddings consume approximately 73 MB of memory.
- **CPU**: Brute-force cosine similarity is O(N*D) where N is embedding count and D is dimension. For 50,000 embeddings at dimension 384, this is approximately 19 million floating-point multiplications per query.
- **Scale ceiling**: ADR-001 documents the ceiling at approximately 50,000 embeddings for acceptable query latency. Beyond this, the planned migration path is `sqlite-vec` (feature flag `vec-sqlite`, v0.2) which provides approximate nearest neighbor search.

**Current budget enforcement**: The `limit` parameter truncates results, but does not prevent the full scan. The `node_type_filter` parameter reduces the scan set to a single node type. There is no mechanism to abort the scan early if it exceeds a time or count budget.

**Planned budget enforcement** (v0.2): The `sqlite-vec` extension provides indexed vector search with configurable probe counts, effectively capping scan cost at a fixed budget regardless of total embedding count.

### 6.2 Graph Traversal Budget

Spreading activation in `src/graph/activation.rs` enforces multiple budget constraints:

```rust
pub fn spread_activation(
    conn: &Connection,
    seeds: &[NodeRef],
    max_depth: u32,           // Budget: maximum hops
    threshold: f32,           // Budget: minimum activation to continue
    decay_per_hop: f32,       // Budget: signal decay limits effective range
) -> Result<HashMap<NodeRef, f32>>
```

Budget enforcement mechanisms:

1. **max_depth**: Hard cap on traversal hops. The retrieval pipeline uses `max_depth = 1` for query-time activation and `max_depth = 3` is the documented maximum for `neighbors()`. Each hop requires a SQL query per active node, so this caps the number of database round-trips.

2. **threshold**: Minimum activation level. Nodes below threshold are excluded from further propagation. With `threshold = 0.1` and `decay_per_hop = 0.6`, activation drops below threshold within 3-4 hops even along maximum-weight links (weight 1.0).

3. **Activation cap**: `*entry = (*entry + extra).min(2.0)` prevents runaway activation from dense graph regions. Even if a node receives activation from many paths, its total activation is capped at 2.0.

4. **Proportional splitting**: Activation spreads proportionally to edge weight, meaning weak links carry weak signals regardless of graph density.

The combined effect is a natural budget: traversal cannot explore more than `max_depth` hops, activation cannot exceed 2.0, and weak signals die out quickly. The worst case is a fully connected graph where every node links to every other node, but even then, `max_depth = 1` limits the scan to at most `N` outgoing links from each seed node, and the threshold quickly eliminates low-activation results.

### 6.3 Query Result Budget

The retrieval pipeline enforces result limits at multiple levels:

```rust
let fetch_limit = query.max_results * 3;  // Fetch 3x to allow for filtering

let bm25_results = bm25::search_bm25(conn, &query.text, fetch_limit)?;
// BM25 internally fetches 3x: let fetch_limit = (limit * 3) as u32;

let vector_results = vector::search_vector(conn, emb, fetch_limit)?;
// Vector search truncates: results.truncate(limit);
```

The budget chain:
1. Consumer sets `query.max_results` (default 5 via `Query::simple()`).
2. Pipeline sets `fetch_limit = max_results * 3` (15 by default).
3. BM25 sets its own fetch limit at `fetch_limit * 3` (45 by default) to ensure enough candidates after FTS5 ranking.
4. Vector search truncates at `fetch_limit` (15 by default).
5. RRF fusion produces at most `bm25_count + vector_count + graph_count` unique results.
6. Enrichment phase takes only `fetch_limit` (15) candidates.
7. Reranking truncates to `max_results` (5 by default).

The consumer controls the budget through `max_results`. The pipeline multiplies this by constant factors at each stage, ensuring that the total work is bounded by a constant multiple of the consumer's requested result count.

### 6.4 Consolidation Batch Budget

```rust
const CONSOLIDATION_BATCH_SIZE: u32 = 10;

let episodes = episodic::get_unconsolidated_episodes(conn, CONSOLIDATION_BATCH_SIZE)?;
if episodes.len() < 3 {
    return Ok(report);
}
```

Consolidation processes at most 10 episodes per call. This caps:
- The amount of text sent to the `ConsolidationProvider` (which may be an expensive LLM call).
- The number of semantic nodes and links created per cycle.
- The time spent in a single consolidation operation.

The consumer can call `consolidate()` in a loop to process all unconsolidated episodes, but each iteration is budget-bounded.

### 6.5 Transformation Scan Budget

Deduplication in `src/lifecycle/transformation.rs` performs a pairwise comparison of all semantic node embeddings:

```rust
fn dedup_semantic_nodes(conn: &Connection) -> Result<u32> {
    let nodes: Vec<(i64, Vec<f32>)> = /* SELECT * FROM embeddings WHERE node_type = 'semantic' */;
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let sim = cosine_similarity(&nodes[i].1, &nodes[j].1);
            if sim >= DEDUP_SIMILARITY_THRESHOLD { /* merge */ }
        }
    }
}
```

This is O(N^2) in the number of semantic node embeddings, which is acceptable for small-to-medium databases (hundreds of semantic nodes) but becomes expensive for large ones. A database with 1,000 semantic nodes requires 499,500 cosine similarity computations. The planned mitigation is to use sqlite-vec for approximate deduplication in v0.2.

### 6.6 Resource Budget Summary

| Resource | Current Budget | Mechanism | Planned Improvement |
|---|---|---|---|
| Embedding scan | Unbounded (full scan) | `limit` truncates output only | sqlite-vec ANN index (v0.2) |
| Graph traversal | max_depth, threshold, activation cap | Structural decay + hard cap | Configurable via `AlayaConfig` (v0.2) |
| Query results | `max_results * 3` at each stage | Multi-level truncation | Expose budget in `Query` struct |
| Consolidation batch | 10 episodes per call | `CONSOLIDATION_BATCH_SIZE` constant | Configurable via `AlayaConfig` (v0.2) |
| Dedup scan | O(N^2) on semantic embeddings | None currently | sqlite-vec dedup index (v0.2) |
| FTS5 results | `limit * 3` per BM25 query | SQL LIMIT clause | Sufficient for current scale |
| Impression accumulation | Unbounded | Pruned by `transform()` | Age-based cap at API boundary (v0.1) |

---

## 7. Data Integrity Patterns

### 7.1 The Cascade Deletion Problem

Alaya's three-store architecture with a shared graph overlay creates complex deletion dependencies. Deleting an episode must clean up:

1. The episode row itself (triggers FTS5 deletion via `episodes_ad` trigger).
2. The episode's embedding in the `embeddings` table.
3. All links referencing the episode in the `links` table (both as source and target).
4. The episode's node strength record in `node_strengths`.
5. Derived semantic nodes that reference the episode (partially -- they may have other sources).
6. Impressions and preferences that were derived from the episode's session (indirect relationship, no direct foreign key).

The current implementation handles this partially. The `semantic::delete_node()` function demonstrates the cascade pattern:

```rust
pub fn delete_node(conn: &Connection, id: NodeId) -> Result<()> {
    conn.execute("DELETE FROM semantic_nodes WHERE id = ?1", [id.0])?;
    conn.execute("DELETE FROM embeddings WHERE node_type = 'semantic' AND node_id = ?1", [id.0])?;
    conn.execute("DELETE FROM links WHERE (source_type = 'semantic' AND source_id = ?1)
                  OR (target_type = 'semantic' AND target_id = ?1)", [id.0])?;
    conn.execute("DELETE FROM node_strengths WHERE node_type = 'semantic' AND node_id = ?1", [id.0])?;
    Ok(())
}
```

This is a manual cascade: four separate DELETE statements. If any statement fails (e.g., SQLITE_BUSY), partial cleanup occurs. The absence of explicit transaction boundaries here is a data integrity risk.

For episode deletion, `episodic::delete_episodes()` only deletes the episodes themselves. The FTS5 trigger handles the FTS index, but embeddings, links, and strengths for the deleted episodes are not cleaned up. This creates orphaned records.

### 7.2 Orphan Detection

Orphan records are rows in `embeddings`, `links`, or `node_strengths` that reference nodes that no longer exist. They waste storage and can cause stale results (e.g., vector search returning similarity scores for embeddings whose parent episodes were deleted).

Current orphan sources:
- Deleting episodes via `delete_episodes()` does not cascade to embeddings, links, or strengths.
- Deleting episodes via `purge(PurgeFilter::Session(...))` does not cascade to embeddings, links, or strengths.
- The `forget()` function cleans up strength records for archived nodes but does not clean up embeddings or all links.

**Planned mitigation** (v0.1): An orphan cleanup step in `transform()` that:
1. Finds embeddings referencing non-existent nodes.
2. Finds links where source or target no longer exists.
3. Finds strength records for non-existent nodes.
4. Deletes all orphaned records.

### 7.3 Tombstone Mechanism

**Known gap**: Alaya currently has no tombstone mechanism. When a node is deleted, there is no record that it ever existed. This creates the memory resurrection vulnerability documented in the security architecture: a deleted episode's content may persist in semantic nodes that were extracted from it. If `consolidate()` runs again and the provider happens to extract similar knowledge, the deleted information re-enters the database.

The planned tombstone mechanism (v0.1):

```sql
CREATE TABLE IF NOT EXISTS tombstones (
    node_type   TEXT    NOT NULL,
    node_id     INTEGER NOT NULL,
    content_hash TEXT   NOT NULL,
    deleted_at  INTEGER NOT NULL,
    PRIMARY KEY (node_type, node_id)
);
```

During consolidation, each candidate semantic node's content hash is checked against the tombstone table. If a match is found, the node is rejected, preventing resurrection. Tombstones persist for a configurable retention period (default 90 days, matching the impression max age) before being cleaned up by `transform()`.

### 7.4 FTS5 Consistency

The FTS5 external content table `episodes_fts` is kept in sync with the `episodes` table via three triggers:

```sql
CREATE TRIGGER episodes_ai AFTER INSERT ON episodes
BEGIN
    INSERT INTO episodes_fts(rowid, content) VALUES (new.id, new.content);
END;

CREATE TRIGGER episodes_ad AFTER DELETE ON episodes
BEGIN
    INSERT INTO episodes_fts(episodes_fts, rowid, content)
        VALUES ('delete', old.id, old.content);
END;

CREATE TRIGGER episodes_au AFTER UPDATE OF content ON episodes
BEGIN
    INSERT INTO episodes_fts(episodes_fts, rowid, content)
        VALUES ('delete', old.id, old.content);
    INSERT INTO episodes_fts(rowid, content) VALUES (new.id, new.content);
END;
```

This is the correct pattern for FTS5 external content tables. The triggers fire within the same transaction as the DML statement, ensuring atomic consistency. If the INSERT into episodes fails, the trigger never fires. If the trigger fails, the entire transaction rolls back.

However, if the FTS5 index becomes corrupted (e.g., due to a WAL corruption event), the `episodes` table remains intact. Recovery involves rebuilding the FTS5 index:

```sql
INSERT INTO episodes_fts(episodes_fts) VALUES('rebuild');
```

This is a full rebuild that reads all content from the `episodes` table and reconstructs the index. It is expensive (O(N) in total content size) but safe and produces a consistent index.

### 7.5 Corroboration as Integrity Signal

The `corroboration_count` column on `semantic_nodes` serves as a data integrity signal. A semantic node with high corroboration has been independently confirmed by multiple consolidation cycles. A node with `corroboration_count = 1` is unconfirmed and should be treated as lower-confidence.

The deduplication step in `transform()` increments corroboration when merging duplicates:

```rust
conn.execute(
    "UPDATE semantic_nodes SET corroboration_count = corroboration_count + 1 WHERE id = ?1",
    [nodes[i].0],
)?;
```

This means corroboration increases through two mechanisms:
1. Explicit `update_corroboration()` called when the provider confirms an existing fact.
2. Implicit deduplication in `transform()` when near-identical nodes are merged.

Consumers can use `corroboration_count` as a quality filter: nodes with count >= 2 have been independently confirmed and are more likely to be accurate.

### 7.6 Embedding Dimension Consistency

**Known gap**: Alaya does not validate embedding dimensions. The consumer can store a 384-dimensional embedding for one episode and a 768-dimensional embedding for another. Cosine similarity between vectors of different dimensions returns 0.0 (because the `cosine_similarity` function checks `a.len() != b.len()` and returns 0.0), which is safe but produces meaningless results.

The planned mitigation (v0.1): validate that all embeddings for a given node type have the same dimension, either by checking against the first stored embedding or by requiring the consumer to declare the dimension at `AlayaStore::open()` time.

---

## 8. Error Propagation Strategy

### 8.1 The AlayaError Hierarchy

Alaya's error model is intentionally flat. There are five error variants, each representing a distinct failure class:

| Variant | Source | Consumer Action |
|---|---|---|
| `Db(rusqlite::Error)` | SQLite failures: BUSY, CORRUPT, FULL, schema mismatch | Check underlying code; retry for BUSY, abort for CORRUPT |
| `NotFound(String)` | Query for non-existent row (episode, semantic node) | Expected in some contexts; handle gracefully |
| `InvalidInput(String)` | Input validation failure (planned v0.1) | Fix input and retry |
| `Serialization(serde_json::Error)` | JSON parse failure for stored context | Data corruption; log and skip affected row |
| `Provider(String)` | ConsolidationProvider returned an error | Provider-specific handling; may indicate LLM failure |

### 8.2 Error Classification for Consumers

Consumers should classify errors into three categories:

**Retryable errors**: `AlayaError::Db` when the underlying `rusqlite::Error` has code `SQLITE_BUSY` or `SQLITE_LOCKED`. These indicate transient contention and may succeed on retry.

**Recoverable errors**: `AlayaError::NotFound` when querying a specific ID that may have been deleted by concurrent `forget()` or `purge()` operations. The consumer should handle the absence gracefully.

**Fatal errors**: `AlayaError::Db` with `SQLITE_CORRUPT`, `SQLITE_NOTADB`, or `SQLITE_CANTOPEN`. These indicate the database file is damaged or inaccessible. The consumer should fall back to a backup or create a new database.

```rust
fn classify_error(e: &AlayaError) -> ErrorClass {
    match e {
        AlayaError::Db(rusqlite::Error::SqliteFailure(err, _)) => {
            match err.code {
                ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked => ErrorClass::Retryable,
                ErrorCode::DatabaseCorrupt | ErrorCode::NotADatabase => ErrorClass::Fatal,
                ErrorCode::DiskFull => ErrorClass::Fatal,
                _ => ErrorClass::Permanent,
            }
        }
        AlayaError::NotFound(_) => ErrorClass::Recoverable,
        AlayaError::InvalidInput(_) => ErrorClass::Permanent,
        AlayaError::Serialization(_) => ErrorClass::Recoverable,
        AlayaError::Provider(_) => ErrorClass::Retryable, // LLM may recover
    }
}
```

### 8.3 Error Suppression in Side Effects

As noted in Section 3.3, the retrieval pipeline uses `let _ =` to suppress errors from post-retrieval side effects:

```rust
let _ = strengths::on_access(conn, scored.node);
let _ = crate::graph::links::on_co_retrieval(conn, retrieved_nodes[i], retrieved_nodes[j]);
```

This is a deliberate design choice: the primary query result has already been computed and should be returned to the consumer regardless of whether side effects succeed. The side effects (strength tracking, link formation) are optimistic and improve future queries, but their failure does not invalidate the current result.

The tradeoff: if side effects consistently fail (e.g., because the database is read-only or disk-full), the Hebbian graph does not evolve and retrieval quality does not improve over time. The consumer is not notified of these failures. A future improvement could surface suppressed errors in the query result or through a diagnostic callback.

### 8.4 Provider Error Boundary

The `ConsolidationProvider` trait methods return `Result<T>`, allowing the provider to report errors:

```rust
pub trait ConsolidationProvider {
    fn extract_knowledge(&self, episodes: &[Episode]) -> Result<Vec<NewSemanticNode>>;
    fn extract_impressions(&self, interaction: &Interaction) -> Result<Vec<NewImpression>>;
    fn detect_contradiction(&self, a: &SemanticNode, b: &SemanticNode) -> Result<bool>;
}
```

Provider errors propagate as `AlayaError::Provider(String)` through the `?` operator. The consumer's provider implementation is responsible for:
1. Wrapping LLM API errors into `AlayaError::Provider`.
2. Implementing its own retry logic for transient LLM failures.
3. Returning empty results rather than errors when degradation is preferred over failure.

The `NoOpProvider` never returns errors, establishing a baseline: if the consumer's provider fails, they can fall back to `NoOpProvider` behavior by catching `AlayaError::Provider` and returning empty results.

### 8.5 Panic Safety

Alaya library code must never panic. The following patterns are enforced:

- No `unwrap()` on `Result` or `Option` in library code (only in tests).
- No `expect()` in library code.
- No array indexing that could be out of bounds (use `.get()` with fallback).
- No `unreachable!()` macro in library code.

The one exception is `SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default()`, which uses `unwrap_or_default()` rather than `unwrap()`. A system clock before the Unix epoch (theoretically possible on misconfigured systems) produces a zero duration rather than a panic.

If a consumer's `ConsolidationProvider` implementation panics, the panic unwinds through Alaya's call stack. Rust's default panic behavior (unwinding) means the `Connection` and all prepared statements are properly dropped, and SQLite's transaction is rolled back. The consumer catches the panic at their `Mutex` boundary or at the thread join point. Alaya does not use `catch_unwind` to absorb provider panics, because the provider is consumer code and the consumer should see the full panic trace.

---

## 9. Recovery Patterns

### 9.1 Corrupt Database Detection

SQLite provides `PRAGMA integrity_check` to detect database corruption. Alaya does not currently run this automatically, but the consumer can execute it:

```rust
// Consumer-side integrity check
fn check_integrity(store: &AlayaStore) -> bool {
    // This requires exposing the connection, which Alaya does not currently do.
    // Planned: store.integrity_check() -> Result<bool>
    true
}
```

**Planned** (v0.2): An `AlayaStore::integrity_check()` method that runs `PRAGMA integrity_check` and returns a structured result indicating any corruption found.

Corruption indicators that surface through normal operation:
- `AlayaError::Db` with `SQLITE_CORRUPT`: The database file is structurally damaged.
- `AlayaError::Db` with `SQLITE_NOTADB`: The file is not a valid SQLite database (may indicate file truncation or format mismatch).
- `AlayaError::Serialization`: JSON stored in `context_json` or `source_episodes_json` columns is malformed, indicating partial write corruption.

### 9.2 Schema Migration

`schema::init_db()` uses `CREATE TABLE IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS` for all schema objects. This means:

1. Opening an existing database with the current schema is a no-op (all IF NOT EXISTS checks pass).
2. Opening an existing database with a subset of the schema creates the missing tables and indexes.
3. Opening an existing database with a superset of the schema (from a newer version) ignores the extra objects.

The `test_idempotent_init` test in `src/schema.rs` verifies this:

```rust
#[test]
fn test_idempotent_init() {
    let conn = open_memory_db().unwrap();
    init_db(&conn).unwrap();  // Second init should not fail
}
```

For future schema changes that modify existing tables (adding columns, changing types), Alaya will need a migration system. The planned approach is:

1. A `schema_version` table tracking the current schema version.
2. Ordered migration functions: `migrate_v1_to_v2()`, `migrate_v2_to_v3()`, etc.
3. Migrations run inside explicit transactions for atomicity.
4. The consumer can inspect `schema_version` to verify compatibility.

### 9.3 Backup Strategies

Since Alaya stores all state in a single SQLite file, backup is straightforward:

**Online backup** (preferred): Use SQLite's backup API (`sqlite3_backup_init/step/finish`) to create a consistent copy of the database while it is open and in use. This is the recommended approach for production systems because it handles WAL mode correctly. Alaya does not currently expose this API but plans to add a `backup(destination: &Path) -> Result<()>` method in v0.2.

**Filesystem copy** (simple but risky): Copy the `.db` file and its `.db-wal` and `.db-shm` companion files. All three files must be copied atomically (or the WAL must be checkpointed first). Copying only the `.db` file without the WAL file may lose recent transactions.

**VACUUM INTO** (clean copy): `VACUUM INTO 'backup.db'` creates a fresh, compacted copy of the database. This is safe during WAL mode operation and produces a single file with no WAL.

**Consumer guidance**:
```rust
// Before backup, optionally checkpoint the WAL:
// conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
// Then copy the .db file (the WAL is now empty).
```

### 9.4 Recovery from Partial Lifecycle Operations

If a lifecycle operation fails midway through (e.g., `consolidate()` creates some semantic nodes but fails before creating their links), the database is in a partially consistent state. Recovery strategies:

**consolidate()**: Re-run. Unconsolidated episodes are detected by the absence of links to semantic nodes. Episodes that were partially consolidated (node created, links not yet created) have orphaned semantic nodes. The next `transform()` call detects these orphans (they have no links) and may merge them with correctly linked nodes.

**forget()**: Re-run. The decay step is multiplicative and applies uniformly. The archival step only deletes nodes below threshold, so partially archived states are self-correcting: the next `forget()` call archives any remaining sub-threshold nodes.

**transform()**: Re-run. Each sub-operation (dedup, prune, decay, prune, prune) is independently restartable. Partial completion of deduplication may leave some duplicates unmerged, which the next run catches.

**purge(PurgeFilter::All)**: The `execute_batch` call is atomic (all deletions in one implicit transaction). If it fails, no deletions occurred. Retry the entire operation.

### 9.5 New Database as Last Resort

If the database file is irrecoverably corrupt and no backup exists, the consumer can create a fresh database:

```rust
// Move corrupted file aside
std::fs::rename("alaya.db", "alaya.db.corrupt")?;
// Create fresh database
let store = AlayaStore::open("alaya.db")?;
// Memory is lost, but the system is operational
```

This is the ultimate degradation: complete memory loss. The consumer's agent starts with a fresh memory, as if meeting the user for the first time. The Alaya library is designed to function correctly with an empty database (degradation Level 6), so this always produces a working system.

---

## 10. Testing Resilience

### 10.1 Current Test Coverage

The codebase contains unit tests for every module, verifiable by examining the `#[cfg(test)]` blocks. Key resilience-relevant tests:

| Test | Module | What It Verifies |
|---|---|---|
| `test_empty_query` | `retrieval/pipeline.rs` | Empty query returns `Ok(vec![])`, not an error |
| `test_bm25_search` / `test_empty_query` | `retrieval/bm25.rs` | FTS5 handles valid and empty queries |
| `test_vector_search_empty` | `retrieval/vector.rs` | Vector search on empty DB returns `Ok(vec![])` |
| `test_rrf_single_set` / `test_rrf_disjoint` | `retrieval/fusion.rs` | RRF handles single set and disjoint sets |
| `test_consolidation_below_threshold` | `lifecycle/consolidation.rs` | Below-threshold episode count returns no-op report |
| `test_forget_empty_db` | `lifecycle/forgetting.rs` | Forgetting on empty DB returns zero report |
| `test_transform_empty_db` | `lifecycle/transformation.rs` | Transformation on empty DB returns zero report |
| `test_purge_all` | `lib.rs` | Full purge leaves database in clean empty state |
| `test_idempotent_init` | `schema.rs` | Schema initialization is idempotent |
| `test_cosine_similarity_orthogonal` | `store/embeddings.rs` | Orthogonal vectors produce zero similarity |
| `test_threshold_cutoff` | `graph/activation.rs` | Weak links are filtered by activation threshold |
| `test_full_lifecycle` | `lib.rs` | Full store-query-consolidate-transform-forget cycle |

### 10.2 Degradation Chain Testing

Each degradation level should be explicitly tested. The current test suite covers Level 6 (empty database) and Level 5 (BM25 only, via tests that store episodes without embeddings). The remaining levels require tests with controlled embedding and link data:

```rust
// Level 1: Full hybrid (all signals populated)
#[test]
fn test_degradation_level_1_full() {
    let store = setup_with_episodes_embeddings_and_links();
    let results = store.query(&Query { embedding: Some(vec![...]), ..}).unwrap();
    assert!(!results.is_empty());
}

// Level 2: No embeddings
#[test]
fn test_degradation_level_2_no_embeddings() {
    let store = setup_with_episodes_and_links_only();
    let results = store.query(&Query::simple("Rust")).unwrap();
    // Should return BM25 + graph results
    assert!(!results.is_empty());
}

// Level 3: No links
#[test]
fn test_degradation_level_3_no_links() {
    let store = setup_with_episodes_and_embeddings_only();
    let results = store.query(&Query { embedding: Some(vec![...]), ..}).unwrap();
    // Should return BM25 + vector results
    assert!(!results.is_empty());
}
```

### 10.3 Idempotency Testing

Each lifecycle operation should be tested for idempotency:

```rust
#[test]
fn test_consolidate_idempotent() {
    let store = setup_with_episodes();
    let provider = make_provider();
    let r1 = store.consolidate(&provider).unwrap();
    let r2 = store.consolidate(&provider).unwrap();
    // Second run should find no unconsolidated episodes
    assert_eq!(r2.episodes_processed, 0);
}

#[test]
fn test_purge_idempotent() {
    let store = setup_with_episodes();
    store.purge(PurgeFilter::All).unwrap();
    let r2 = store.purge(PurgeFilter::All).unwrap();
    assert_eq!(r2.episodes_deleted, 0);
}
```

### 10.4 Fault Injection

Planned fault injection strategies for v0.2:

**SQLite BUSY simulation**: Open two `AlayaStore` instances against the same file. Have one hold a write lock (via a long-running transaction) while the other attempts to write. Verify that the second connection receives `AlayaError::Db` with `SQLITE_BUSY`.

**Provider failure injection**: Implement a `FailingProvider` that returns `Err(AlayaError::Provider("simulated failure"))` after processing N items. Verify that partial consolidation is detected and recovered by subsequent runs.

**Disk full simulation**: Use a `tmpfs` with a size limit to simulate disk-full conditions during write operations. Verify that `AlayaError::Db` with `SQLITE_FULL` is returned and the database remains consistent.

**Corruption injection**: Overwrite random bytes in the database file and verify that `AlayaStore::open()` or subsequent operations return appropriate errors.

### 10.5 Property-Based Testing

Property-based testing (via `proptest` or `quickcheck`) can verify invariants that hold across arbitrary inputs:

**Invariant 1: Query never panics**. For any `Query` value (including empty strings, very long strings, strings with special characters, NaN embeddings), `store.query()` returns `Ok` or `Err`, never panics.

**Invariant 2: Lifecycle preserves data integrity**. After any sequence of `consolidate()`, `transform()`, `forget()` calls, `status()` returns valid counts and all returned counts are non-negative.

**Invariant 3: RRF is monotone**. Adding a result set to RRF never decreases the score of a document that appears in an existing result set.

**Invariant 4: Strength bounds**. After any sequence of operations, all `storage_strength` values are in `[0.0, 1.0]` and all `retrieval_strength` values are in `[0.0, 1.0]`.

**Invariant 5: FTS5 consistency**. After any sequence of `store_episode()` and `delete_episodes()` calls, the count of FTS5 rows matches the count of episode rows.

### 10.6 The MockProvider and NoOpProvider as Test Infrastructure

The existing `MockProvider` (test-only) and `NoOpProvider` (production) enable deterministic testing of the entire lifecycle:

```rust
// Deterministic test: known inputs produce known outputs
let provider = MockProvider::with_knowledge(vec![
    NewSemanticNode {
        content: "User discusses Rust programming".to_string(),
        node_type: SemanticType::Fact,
        confidence: 0.8,
        source_episodes: ep_ids,
        embedding: None,
    },
]);
let report = store.consolidate(&provider).unwrap();
assert_eq!(report.nodes_created, 1);

// Degradation test: NoOpProvider produces zero output
let report = store.consolidate(&NoOpProvider).unwrap();
assert_eq!(report.nodes_created, 0);
```

The `MockProvider` with `empty()` is functionally identical to `NoOpProvider`, but exists separately for test readability and to allow future divergence (e.g., `MockProvider` may gain assertion capabilities).

---

## Appendix A: Resilience Checklist for Consumers

Before deploying an agent using Alaya, verify:

- [ ] `AlayaStore` is wrapped in `Arc<Mutex<AlayaStore>>` if accessed from multiple threads
- [ ] Error handling matches on `AlayaError` variants, especially `Db` for BUSY detection
- [ ] `consolidate()`, `transform()`, and `forget()` are scheduled at regular intervals (not called in tight loops)
- [ ] `perfume()` is called exactly once per interaction, not on retry
- [ ] `query()` empty results (`Ok(vec![])`) are handled as valid states, not errors
- [ ] `ConsolidationProvider` implementation catches LLM errors and returns `AlayaError::Provider`
- [ ] Backup strategy is implemented (SQLite backup API, VACUUM INTO, or filesystem copy)
- [ ] `max_results` is set to a reasonable value (5-20) to cap resource usage
- [ ] Database file permissions are set to 0600
- [ ] WAL companion files (`.db-wal`, `.db-shm`) are included in backup and deployment

## Appendix B: Known Gaps and Remediation Timeline

| Gap | Risk Level | Current Impact | Planned Fix | Version |
|---|---|---|---|---|
| No BEGIN IMMEDIATE for write transactions | Medium | Potential deadlock with multiple AlayaStore instances on same file | Wrap all write operations in BEGIN IMMEDIATE...COMMIT | v0.1 |
| No WAL checkpoint management | Low | WAL file may grow under sustained write pressure | PRAGMA wal_autocheckpoint, journal_size_limit, compact() | v0.1 |
| No tombstone mechanism | Medium | Memory resurrection after deletion | Tombstone table, content-hash checking in consolidate() | v0.1 |
| No input validation at API boundary | Medium | Invalid data stored silently | Validate content length, embedding dimensions, timestamp ranges | v0.1 |
| No orphan cleanup | Low | Wasted storage from orphaned embeddings/links/strengths | Orphan detection and cleanup in transform() | v0.1 |
| Unbounded embedding scan | Medium | Query latency degrades beyond ~50K embeddings | sqlite-vec feature flag for ANN search | v0.2 |
| O(N^2) deduplication | Low | Transformation time degrades beyond ~1000 semantic nodes | sqlite-vec approximate dedup | v0.2 |
| No compact() method | Low | Consumer cannot trigger VACUUM + WAL checkpoint | Expose compact() on AlayaStore | v0.2 |
| No integrity_check() method | Low | Consumer cannot verify database health | Expose PRAGMA integrity_check wrapper | v0.2 |
| No backup() method | Low | Consumer must implement backup externally | Expose SQLite backup API wrapper | v0.2 |

## Appendix C: Cross-Reference Index

| This Document Section | References |
|---|---|
| Section 1.2 (Error model) | `src/error.rs` -- AlayaError enum |
| Section 2.1 (Pipeline probing) | `src/retrieval/pipeline.rs` -- execute_query() |
| Section 2.3 (NoOpProvider) | `src/provider.rs` -- NoOpProvider impl |
| Section 3.1 (Degradation chain) | architecture.yml -- graceful_degradation_chain |
| Section 3.2 (RRF) | `src/retrieval/fusion.rs` -- rrf_merge(), ADR-006 |
| Section 3.4 (FTS5 sanitization) | `src/retrieval/bm25.rs` -- search_bm25() |
| Section 4.1 (WAL mode) | `src/schema.rs` -- init_db() PRAGMAs |
| Section 4.2 (BEGIN IMMEDIATE) | architecture.yml -- known_gaps, security.yml -- transaction-deadlock |
| Section 5.1 (Consolidation) | `src/lifecycle/consolidation.rs` -- consolidate() |
| Section 5.2 (Forgetting) | `src/lifecycle/forgetting.rs` -- forget() |
| Section 5.3 (Transformation) | `src/lifecycle/transformation.rs` -- transform() |
| Section 5.4 (Perfuming) | `src/lifecycle/perfuming.rs` -- perfume() |
| Section 6.1 (Embedding scan) | `src/store/embeddings.rs` -- search_by_vector() |
| Section 6.2 (Graph traversal) | `src/graph/activation.rs` -- spread_activation() |
| Section 7.1 (Cascade deletion) | `src/store/semantic.rs` -- delete_node() |
| Section 7.4 (FTS5 triggers) | `src/schema.rs` -- episodes_ai/ad/au triggers |
| Section 7.5 (Corroboration) | `src/store/semantic.rs` -- update_corroboration() |
| Section 8.4 (Provider errors) | `src/provider.rs` -- ConsolidationProvider trait |
| Section 9.2 (Schema migration) | `src/schema.rs` -- init_db(), ADR-001 |
