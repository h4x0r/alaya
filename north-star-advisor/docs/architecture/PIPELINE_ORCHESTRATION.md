# Pipeline Orchestration: Alaya Retrieval Pipeline

> **Reframing note.** This document adapts the PIPELINE_ORCHESTRATION template
> (designed for multi-agent LLM state machines) to Alaya's domain: a
> synchronous, in-process Rust retrieval pipeline that runs inside a single
> function call (`execute_query`). "Agents" become pipeline stages. "Handoffs"
> become stage transition logic. "State schema" becomes query and result types.
> Every claim below is grounded in the actual source code committed to the
> repository.

---

## Table of Contents

1. [Query and Result Type Definitions](#1-query-and-result-type-definitions)
2. [Retrieval Pipeline Architecture](#2-retrieval-pipeline-architecture)
3. [BM25 Stage Implementation](#3-bm25-stage-implementation)
4. [Vector Search Stage](#4-vector-search-stage)
5. [Graph Spreading Activation Stage](#5-graph-spreading-activation-stage)
6. [RRF Fusion Stage](#6-rrf-fusion-stage)
7. [Reranking Stage](#7-reranking-stage)
8. [Post-Retrieval Side Effects](#8-post-retrieval-side-effects)
9. [Graceful Degradation Logic](#9-graceful-degradation-logic)
10. [Configuration and Tuning](#10-configuration-and-tuning)
11. [Stage Failure Handling](#11-stage-failure-handling)
12. [Performance Characteristics](#12-performance-characteristics)
13. [Known Gaps and Planned Improvements](#13-known-gaps-and-planned-improvements)

---

## 1. Query and Result Type Definitions

The retrieval pipeline operates on a small set of strongly-typed Rust structs
defined in `src/types.rs`. These types form the contract between the consumer
(agent) and the pipeline internals. Every value that crosses a stage boundary
is one of these types.

### 1.1 Query (Input)

```rust
pub struct Query {
    pub text: String,                  // Natural-language query text
    pub embedding: Option<Vec<f32>>,   // Pre-computed embedding vector (optional)
    pub context: QueryContext,         // Contextual signals for reranking
    pub max_results: usize,            // Final result cap
}
```

`Query` has a convenience constructor for the common case:

```rust
impl Query {
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

The `simple` constructor is the day-1 API. A consumer that has no embedding
model, no context signals, and no opinions about result count can call
`query(&Query::simple("Rust async"))` and get useful results from BM25 alone.
The full `Query` struct exposes every knob for consumers who want vector search,
context-aware reranking, or larger result sets.

### 1.2 QueryContext (Reranking Signals)

```rust
pub struct QueryContext {
    pub topics: Vec<String>,               // Topic tags for the current query
    pub sentiment: f32,                     // Sentiment polarity [-1.0, 1.0]
    pub mentioned_entities: Vec<String>,    // Entity names in the query
    pub current_timestamp: Option<i64>,     // Override wall-clock time (Unix seconds)
}
```

`QueryContext` provides the signals that the reranking stage uses to boost
contextually relevant results. Every field defaults to zero/empty via
`#[derive(Default)]`, so omitting the context entirely is safe and common. The
`current_timestamp` override exists primarily for deterministic testing; in
production it falls back to `SystemTime::now()`.

### 1.3 NodeRef (Polymorphic Identity)

```rust
pub enum NodeRef {
    Episode(EpisodeId),
    Semantic(NodeId),
    Preference(PreferenceId),
}
```

`NodeRef` is the universal identifier that flows through every pipeline stage.
It is `Copy`, `Hash`, and `Eq`, so it can serve as both a HashMap key in RRF
fusion and a direct reference in strength tracking. The polymorphic design means
the pipeline can fuse results from episodes (BM25), semantic nodes (vector
search), and preferences into a single ranked list. Currently, only episodes
produce results; semantic and preference node retrieval is planned.

Each variant wraps a newtype ID:

```rust
pub struct EpisodeId(pub i64);   // AUTO INCREMENT from episodes table
pub struct NodeId(pub i64);      // AUTO INCREMENT from semantic_nodes table
pub struct PreferenceId(pub i64); // AUTO INCREMENT from preferences table
```

The `NodeRef::from_parts(type_str, id)` constructor bridges the database
representation (two columns: `node_type TEXT, node_id INTEGER`) to the enum,
and `type_str()` / `id()` go the other direction.

### 1.4 ScoredMemory (Output)

```rust
pub struct ScoredMemory {
    pub node: NodeRef,         // Which memory item
    pub content: String,       // Human-readable text
    pub score: f64,            // Final reranked score
    pub role: Option<Role>,    // Speaker role (for episodes)
    pub timestamp: i64,        // Creation time (Unix seconds)
}
```

`ScoredMemory` is what the consumer receives. The `score` field reflects the
full pipeline: BM25 rank, vector similarity, graph activation, RRF fusion,
context similarity, and recency decay all contribute to this single `f64`. The
consumer sorts by score (the pipeline returns results pre-sorted, highest
first) and injects the top results into their LLM system prompt.

### 1.5 Intermediate Types

Several types exist only between stages and never cross the public API boundary:

| Type | Lives Between | Shape |
|------|---------------|-------|
| `Vec<(EpisodeId, f64)>` | BM25 output | Episode IDs with normalized FTS5 scores |
| `Vec<(NodeRef, f64)>` | Vector output, RRF output | Polymorphic IDs with similarity or RRF scores |
| `HashMap<NodeRef, f32>` | Graph activation output | Activated nodes with activation levels |
| `Vec<(NodeRef, f64, String, Option<Role>, i64, EpisodeContext)>` | Pre-rerank enrichment | Full candidate tuples ready for scoring |

These intermediate types are not exported. They are local variables inside
`execute_query` in `src/retrieval/pipeline.rs`.

### 1.6 EpisodeContext (Stored Metadata)

```rust
pub struct EpisodeContext {
    pub topics: Vec<String>,
    pub sentiment: f32,
    pub conversation_turn: u32,
    pub mentioned_entities: Vec<String>,
    pub preceding_episode: Option<EpisodeId>,
}
```

`EpisodeContext` is serialized to JSON and stored alongside each episode in
the `context_json` column. During reranking, it is deserialized and compared
against the `QueryContext` to compute context similarity. The
`preceding_episode` field also triggers temporal link creation on the write
path (not the retrieval path) via `graph::links::create_link`.

---

## 2. Retrieval Pipeline Architecture

### 2.1 Entry Point

The entire pipeline lives in a single synchronous function:

```rust
// src/retrieval/pipeline.rs
pub fn execute_query(conn: &Connection, query: &Query) -> Result<Vec<ScoredMemory>>
```

It takes a borrowed SQLite connection and a borrowed `Query`, and returns an
owned vector of scored results. The function is called by `AlayaStore::query`,
which simply delegates:

```rust
// src/lib.rs
pub fn query(&self, q: &Query) -> Result<Vec<ScoredMemory>> {
    retrieval::pipeline::execute_query(&self.conn, q)
}
```

There is no background processing, no async, no thread pool. The consumer's
calling thread runs every stage sequentially. This is deliberate: Alaya is a
library, and the consumer controls the execution context.

### 2.2 Stage Diagram

```
                         Query
                           |
                           v
                  +------------------+
                  | Timestamp Setup  |  now = query.context.current_timestamp
                  +------------------+      .unwrap_or(SystemTime::now())
                           |
              +------------+------------+
              |            |            |
              v            v            v
         +--------+  +----------+  +---------+
         | BM25   |  | Vector   |  | Graph   |  <-- "parallel" retrieval
         | (FTS5) |  | (cosine) |  | (spread)|      (sequential in code,
         +--------+  +----------+  +---------+       conceptually parallel)
              |            |            |
              v            v            v
         Vec<(NodeRef, f64)>  each stage produces
              |            |            |
              +------+-----+-----+-----+
                     |
                     v
              +-------------+
              | RRF Fusion  |  k = 60
              | (merge)     |
              +-------------+
                     |
                     v
              +-------------+
              | Enrichment  |  Fetch episode content, role, context_json
              +-------------+
                     |
                     v
              +-------------+
              | Reranking   |  base * (1 + 0.3*ctx) * (1 + 0.2*recency)
              +-------------+
                     |
                     v
              +-------------------+
              | Post-Retrieval    |  strengths::on_access()
              | Side Effects      |  links::on_co_retrieval()
              +-------------------+
                     |
                     v
               Vec<ScoredMemory>
```

### 2.3 Data Flow Summary

1. **Timestamp setup** -- Resolve `current_timestamp` or fall back to wall clock. Compute `fetch_limit = max_results * 3` as the over-fetch factor for the initial retrieval stages.

2. **BM25 retrieval** -- Produces `Vec<(NodeRef::Episode, f64)>`. Score range: `[0.0, 1.0]` (min-max normalized FTS5 rank). Empty input or no FTS5 matches yields `vec![]`.

3. **Vector retrieval** -- Produces `Vec<(NodeRef, f64)>`. Score: cosine similarity cast to f64. Skipped entirely (returns `vec![]`) when `query.embedding` is `None`.

4. **Graph spreading activation** -- Seeds from top 3 BM25 + top 3 vector results. Produces `HashMap<NodeRef, f32>` converted to `Vec<(NodeRef, f64)>` after excluding seed nodes. Skipped when no seeds exist.

5. **RRF fusion** -- Merges 1-3 result sets using Reciprocal Rank Fusion with `k=60`. Produces a single sorted `Vec<(NodeRef, f64)>`.

6. **Enrichment** -- For each fused candidate, fetches the full `Episode` from SQLite to populate content, role, timestamp, and context. Non-episode nodes are currently filtered out (TODO).

7. **Reranking** -- Applies `score = base * (1 + 0.3 * context_sim) * (1 + 0.2 * recency)`. Sorts descending, truncates to `max_results`.

8. **Post-retrieval effects** -- For every returned result: `strengths::on_access(node)` resets RS to 1.0 and increments SS. For every pair of returned results: `links::on_co_retrieval(a, b)` strengthens or creates co-retrieval links.

### 2.4 Ownership and Borrowing

The pipeline borrows `&Connection` and `&Query` but owns all intermediate data.
No allocations escape the function except the returned `Vec<ScoredMemory>`.
The post-retrieval writes (`on_access`, `on_co_retrieval`) use the same
borrowed connection; errors from these writes are silently discarded via
`let _ = ...` to avoid failing the read path due to a write-path side effect.

---

## 3. BM25 Stage Implementation

**Source:** `src/retrieval/bm25.rs`

### 3.1 Function Signature

```rust
pub fn search_bm25(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<(EpisodeId, f64)>>
```

### 3.2 Input Sanitization

FTS5 interprets certain characters as operators (`AND`, `OR`, `NOT`, `NEAR`,
`*`, `"`, `(`, `)`, `+`, `-`, `:`, `^`). Passing unsanitized user input
directly into a `MATCH` clause causes SQLite errors. The BM25 stage strips
all non-alphanumeric, non-whitespace characters:

```rust
let sanitized: String = query
    .chars()
    .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
    .collect();
```

This is a conservative approach: it sacrifices phrase search and boolean
operators in exchange for zero FTS5 injection risk. The sanitized string is
passed to the query via a parameterized bind (`?1`), providing a second layer
of protection against SQL injection.

### 3.3 Empty Input Guards

Two guards prevent wasted work:

1. `query.trim().is_empty()` -- The raw input is empty or whitespace-only.
2. `sanitized.trim().is_empty()` -- After sanitization, nothing remains (e.g., input was all punctuation).

Both return `Ok(vec![])` immediately.

### 3.4 FTS5 Query Execution

```sql
SELECT e.id, rank
FROM episodes_fts fts
JOIN episodes e ON e.id = fts.rowid
WHERE episodes_fts MATCH ?1
ORDER BY rank
LIMIT ?2
```

The `rank` column is SQLite FTS5's built-in BM25 relevance score. FTS5 ranks
are negative floats where lower (more negative) means more relevant. The
`ORDER BY rank` sorts most relevant first. The `LIMIT` is `fetch_limit * 3`
(9x the consumer's `max_results`) to give downstream stages ample candidates.

The join from `episodes_fts` back to `episodes` via `rowid` is necessary
because the FTS5 table is configured as external content
(`content=episodes, content_rowid=id`), kept in sync by three triggers
defined in `src/schema.rs`: `episodes_ai` (after insert), `episodes_ad`
(after delete), and `episodes_au` (after update).

### 3.5 Score Normalization

FTS5 ranks are heterogeneous negative values that cannot be meaningfully
compared with cosine similarities or graph activation levels. The BM25 stage
normalizes them to `[0.0, 1.0]` using min-max scaling:

```rust
let normalized = if range.abs() < 1e-10 {
    1.0  // Single result gets perfect score
} else {
    1.0 - ((rank - min_rank) / range)
};
```

The inversion (`1.0 - ...`) is necessary because FTS5 uses "lower is better"
ranking. After normalization, the best match has score 1.0 and the worst match
in the result set has score 0.0. When only one result exists, it receives 1.0.

### 3.6 Output

A vector of `(EpisodeId, f64)` tuples, truncated to the requested `limit`.
These are converted to `(NodeRef::Episode, f64)` in `execute_query` before
being passed to RRF fusion.

---

## 4. Vector Search Stage

**Source:** `src/retrieval/vector.rs` (thin wrapper) and `src/store/embeddings.rs` (implementation)

### 4.1 Function Signature

```rust
// retrieval/vector.rs
pub fn search_vector(
    conn: &Connection,
    query_embedding: &[f32],
    limit: usize,
) -> Result<Vec<(NodeRef, f64)>>

// store/embeddings.rs (actual search)
pub fn search_by_vector(
    conn: &Connection,
    query_vec: &[f32],
    node_type_filter: Option<&str>,
    limit: usize,
) -> Result<Vec<(NodeRef, f32)>>
```

### 4.2 Skip Condition

In `execute_query`, the vector stage is gated on `query.embedding`:

```rust
let vector_results: Vec<(NodeRef, f64)> = match &query.embedding {
    Some(emb) => vector::search_vector(conn, emb, fetch_limit)?,
    None => vec![],
};
```

When the consumer provides no embedding (the common case with `Query::simple`),
the entire vector stage is skipped with zero cost. This is the primary
graceful degradation mechanism: consumers without an embedding model still get
full BM25 + graph retrieval.

### 4.3 Embedding Storage Format

Embeddings are stored as BLOBs in the `embeddings` table:

```sql
CREATE TABLE IF NOT EXISTS embeddings (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    node_type TEXT    NOT NULL,     -- "episode", "semantic", "preference"
    node_id   INTEGER NOT NULL,
    embedding BLOB    NOT NULL,     -- f32 values as little-endian bytes
    model     TEXT    NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL
);
```

Serialization packs `Vec<f32>` to `Vec<u8>` via `f32::to_le_bytes()`:

```rust
pub fn serialize_embedding(vec: &[f32]) -> Vec<u8> {
    vec.iter().flat_map(|f| f.to_le_bytes()).collect()
}
```

A 384-dimensional embedding (common for MiniLM) occupies 1,536 bytes. A
1,536-dimensional embedding (OpenAI ada-002) occupies 6,144 bytes. The BLOB
storage is dimension-agnostic; any dimensionality works as long as the query
embedding matches the stored embeddings.

### 4.4 Brute-Force Cosine Search

Vector search loads all embeddings from SQLite into memory and computes cosine
similarity against each one:

```rust
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    // Accumulates dot product and norms in f64 for precision
    let dot = a.iter().zip(b.iter()).map(|(x,y)| (*x as f64) * (*y as f64)).sum();
    let norm_a = a.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    let norm_b = b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    (dot / (norm_a * norm_b)) as f32
}
```

Key implementation details:
- Accumulation in `f64` avoids catastrophic cancellation with large vectors.
- Dimension mismatch (`a.len() != b.len()`) returns 0.0 silently.
- Zero-norm vectors return 0.0 (division-by-zero guard).
- Results with similarity <= 0.0 are filtered out.

### 4.5 Performance Characteristics

Brute-force search is O(N * D) where N is the number of stored embeddings and
D is the dimensionality. For Alaya's target workload (personal agent memory,
< 10K episodes), this is sub-millisecond on modern hardware. The planned
`vec-sqlite` feature flag (v0.2) will add sqlite-vec SIMD-accelerated search
for larger datasets.

### 4.6 Polymorphic Results

Unlike BM25 (which only searches episodes), vector search returns `NodeRef`
values that can be episodes, semantic nodes, or preferences -- any entity with
a stored embedding. The `node_type_filter` parameter on `search_by_vector`
allows restricting to a single type, but the retrieval pipeline currently
passes `None` (search all types).

---

## 5. Graph Spreading Activation Stage

**Source:** `src/graph/activation.rs`

### 5.1 Theoretical Foundation

The graph stage implements Collins and Loftus (1975) spreading activation
theory. The core principle: when a memory node is activated (found by BM25 or
vector search), activation energy spreads through weighted edges to
neighboring nodes. Strongly connected neighbors receive more activation.
Activation decays with graph distance.

In Alaya's context, this means: if the user asks about "Rust programming" and
BM25 finds episode #42, the graph stage can discover episode #38 (which
discussed async in Rust) via a topical or co-retrieval link, even if episode
#38 does not contain the literal words "Rust programming."

### 5.2 Function Signature

```rust
pub fn spread_activation(
    conn: &Connection,
    seeds: &[NodeRef],       // Starting nodes (from BM25 + vector)
    max_depth: u32,          // Number of hops (1 in retrieval pipeline)
    threshold: f32,          // Minimum activation to keep (0.1 in pipeline)
    decay_per_hop: f32,      // Multiplicative decay per hop (0.6 in pipeline)
) -> Result<HashMap<NodeRef, f32>>
```

### 5.3 Seed Selection

In `execute_query`, seeds are the top 3 results from BM25 and the top 3 from
vector search:

```rust
let seed_nodes: Vec<NodeRef> = bm25_results.iter().take(3)
    .chain(vector_results.iter().take(3))
    .map(|(nr, _)| *nr)
    .collect();
```

This produces at most 6 seed nodes. When vector search is skipped, at most 3
BM25 seeds remain. When BM25 returns nothing, up to 3 vector seeds remain.
When both return nothing, `seed_nodes` is empty and the graph stage is skipped
entirely.

### 5.4 Activation Algorithm

The algorithm proceeds in discrete rounds (one round per hop, up to
`max_depth`):

1. **Initialize:** Every seed node receives activation 1.0.

2. **For each round:**
   a. For each activated node above `threshold`:
      - Fetch all outgoing links via `links::get_links_from(conn, node)`.
      - For each outgoing link, compute spread: `activation * forward_weight * decay_per_hop`.
      - If spread exceeds `threshold * 0.1`, add it to the delta accumulator for the target node.
   b. Merge deltas into the activation map, capping each node at 2.0 to prevent runaway activation in cyclic graphs.

3. **Filter:** Remove all nodes with activation below `threshold`.

**Important design choice:** The implementation uses absolute edge weights for
spreading, not proportional weights. The comment in the source explains why:

> "Use absolute weight (not proportion) so weak links carry weak signal
> regardless of how many other links exist. This matches neuroscience:
> synaptic strength is absolute, not relative to other synapses."

This means a node with one strong link (weight 0.9) and one weak link
(weight 0.1) will spread 90% of its activation through the strong link and
10% through the weak link. If it had ten additional weak links, those
additional links would each receive 10% too -- the strong link does not get
diluted.

### 5.5 Link Types and Their Role in Spreading

The graph contains several link types:

| Link Type | Created By | Typical Weight | Meaning |
|-----------|-----------|----------------|---------|
| `Temporal` | `store_episode` (when `preceding_episode` set) | 0.5 | Sequential conversation turns |
| `Topical` | Consolidation process | Varies | Shared topic between nodes |
| `Entity` | Consolidation process | Varies | Shared entity reference |
| `Causal` | Consolidation process | Varies | Causal relationship |
| `CoRetrieval` | Post-retrieval side effects | 0.3 initial, grows via LTP | Nodes retrieved together |

Spreading activation treats all link types equally -- it uses `forward_weight`
regardless of type. This is intentional: the weight encodes how strongly two
nodes are associated, and the type is metadata for other purposes (display,
filtering).

### 5.6 Skip Condition and Output

When no seed nodes exist, the graph stage produces an empty HashMap. In
`execute_query`, seed nodes are explicitly excluded from the graph results to
avoid double-counting:

```rust
let graph_results: Vec<(NodeRef, f64)> = graph_activation
    .into_iter()
    .filter(|(nr, _)| !seed_nodes.contains(nr))
    .map(|(nr, act)| (nr, act as f64))
    .collect();
```

The activation values (`f32`) are cast to `f64` to match the common type used
by RRF fusion.

### 5.7 Pipeline Parameters

In the retrieval pipeline, spreading activation is called with:

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| `max_depth` | 1 | Single hop keeps latency low; most useful associations are direct |
| `threshold` | 0.1 | Filters noise while keeping moderately connected nodes |
| `decay_per_hop` | 0.6 | Aggressive decay ensures only strongly linked nodes surface |

These values are hardcoded in `execute_query`. They are not exposed to the
consumer in v0.1. The planned `RetrievalConfig` struct (v0.2) will make them
configurable.

---

## 6. RRF Fusion Stage

**Source:** `src/retrieval/fusion.rs`

### 6.1 Theoretical Foundation

Reciprocal Rank Fusion (Cormack, Clarke & Buettcher, 2009) merges multiple
ranked lists without requiring score calibration. This is critical for Alaya
because the three retrieval stages produce fundamentally different score types:

- **BM25:** Min-max normalized FTS5 rank, `[0.0, 1.0]`
- **Vector:** Cosine similarity, `[0.0, 1.0]` in practice (negative values filtered)
- **Graph:** Spreading activation level, `[threshold, 2.0]`

These scores are not comparable. A BM25 score of 0.8 means something entirely
different from a cosine similarity of 0.8 or an activation level of 0.8. RRF
sidesteps this by using only rank position, not score magnitude.

### 6.2 Algorithm

For each document `d` appearing at rank `r_i` (0-based) in result set `i`:

```
score(d) = sum over all sets i of: 1 / (k + r_i + 1)
```

Where `k` is a smoothing constant. The `+1` converts from 0-based to 1-based
ranking.

### 6.3 Implementation

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
    let mut merged: Vec<(NodeRef, f64)> = scores.into_iter().collect();
    merged.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    merged
}
```

Key observations:

1. **Original scores are discarded.** Only rank position matters. The
   `_original_score` parameter is received but unused.

2. **Overlap is rewarded.** A document appearing in two result sets receives
   contributions from both. If it is rank 0 in both BM25 and vector, its RRF
   score is `2 / (k + 1)`. If it appears at rank 0 in only one set, its score
   is `1 / (k + 1)`.

3. **The `k` parameter (60) controls rank sensitivity.** With `k=60`, the
   difference between rank 0 and rank 1 is small:
   - Rank 0: `1/61 = 0.01639`
   - Rank 1: `1/62 = 0.01613`
   - Rank 10: `1/71 = 0.01408`

   This high `k` value means appearing in multiple lists is much more important
   than exact rank within a single list. This is appropriate for Alaya because
   multi-signal agreement (found by BM25 AND vector AND graph) is a stronger
   relevance signal than being rank 1 vs. rank 3 in any single retrieval
   method.

### 6.4 Conditional Set Construction

The pipeline conditionally includes result sets in RRF based on what each
stage produced:

```rust
let mut sets: Vec<Vec<(NodeRef, f64)>> = vec![bm25_results]; // Always included
if !vector_results.is_empty() {
    sets.push(vector_results);
}
if !graph_results.is_empty() {
    sets.push(graph_results);
}
let fused = fusion::rrf_merge(&sets, 60);
```

BM25 results are always included (even if empty, because an empty set
contributes no scores). Vector and graph results are only included when
non-empty. This means:

- With all three stages producing results: 3-way RRF (strongest signal)
- With BM25 + vector only (no graph links): 2-way RRF
- With BM25 only (no embedding, no links): 1-way RRF (degenerates to BM25 ranking)

---

## 7. Reranking Stage

**Source:** `src/retrieval/rerank.rs`

### 7.1 Enrichment (Pre-Rerank)

Before reranking, fused candidates are enriched with their full content from
the database:

```rust
let candidates: Vec<(NodeRef, f64, String, Option<Role>, i64, EpisodeContext)> = fused
    .into_iter()
    .take(fetch_limit)
    .filter_map(|(node_ref, score)| {
        match node_ref {
            NodeRef::Episode(eid) => {
                episodic::get_episode(conn, eid).ok().map(|ep| {
                    (node_ref, score, ep.content, Some(ep.role), ep.timestamp, ep.context)
                })
            }
            _ => None // TODO: enrich semantic and preference nodes
        }
    })
    .collect();
```

Each enrichment requires one SQLite query (`get_episode`). The
`take(fetch_limit)` bounds this to `max_results * 3` lookups. If a lookup fails
(episode deleted between search and enrichment), that candidate is silently
dropped via `filter_map` and `.ok()`.

Non-episode `NodeRef` variants (semantic, preference) currently produce `None`
and are filtered out. This is a known gap: vector search can find semantic
nodes, but they will not survive enrichment. The fix is straightforward (fetch
from `semantic_nodes` or `preferences` table) and planned for v0.1.

### 7.2 Rerank Formula

```rust
pub fn rerank(
    candidates: Vec<(NodeRef, f64, String, Option<Role>, i64, EpisodeContext)>,
    query_context: &QueryContext,
    now: i64,
    max_results: usize,
) -> Vec<ScoredMemory> {
    // For each candidate:
    let recency = recency_decay(timestamp, now);
    let context_sim = context_similarity(&ctx, query_context);
    let final_score = base_score * (1.0 + 0.3 * context_sim) * (1.0 + 0.2 * recency);
}
```

The formula is multiplicative, not additive. This means:

- **Base score matters most.** A candidate with a high RRF score retains its
  advantage. The context and recency factors are boosters, not overrides.
- **Context similarity** provides up to 30% boost (`0.3 * 1.0 = 0.3`).
- **Recency** provides up to 20% boost (`0.2 * 1.0 = 0.2`).
- **Maximum combined boost** is 56% (`1.3 * 1.2 = 1.56`).
- **When context is empty** (no topics, no entities, neutral sentiment) and
  the memory is very old, the boost factors approach `1.0 * 1.0 = 1.0` and
  the final score equals the base score.

### 7.3 Recency Decay Function

```rust
fn recency_decay(timestamp: i64, now: i64) -> f64 {
    let age_secs = (now - timestamp).max(0) as f64;
    let age_days = age_secs / 86400.0;
    (-age_days / 30.0).exp()
}
```

This produces an exponential decay curve:

| Age | Recency Value | Boost Factor (0.2 * recency) |
|-----|--------------|------------------------------|
| 0 (now) | 1.000 | +20.0% |
| 1 day | 0.967 | +19.3% |
| 7 days | 0.792 | +15.8% |
| 30 days | 0.368 | +7.4% |
| 60 days | 0.135 | +2.7% |
| 90 days | 0.050 | +1.0% |
| 180 days | 0.002 | +0.05% |

The half-life of the decay is approximately 20.8 days (`ln(2) * 30`). Memories
older than 90 days receive negligible recency boost, making them essentially
equal to base score. This is appropriate: very old memories should surface only
when they are the best textual match, not because of recency.

### 7.4 Context Similarity Function

```rust
fn context_similarity(candidate: &EpisodeContext, query: &QueryContext) -> f64 {
    let topic_sim = jaccard(&candidate.topics, &query.topics);
    let entity_sim = jaccard(&candidate.mentioned_entities, &query.mentioned_entities);
    let sentiment_sim = 1.0 - ((candidate.sentiment - query.sentiment).abs() as f64 / 2.0);
    topic_sim * 0.5 + entity_sim * 0.25 + sentiment_sim * 0.25
}
```

Three signals contribute to context similarity:

| Signal | Weight | Metric | Range |
|--------|--------|--------|-------|
| Topic overlap | 50% | Jaccard index of topic tag sets | [0.0, 1.0] |
| Entity overlap | 25% | Jaccard index of entity name sets | [0.0, 1.0] |
| Sentiment proximity | 25% | `1 - abs(delta) / 2` | [0.0, 1.0] |

The Jaccard index is `|A intersect B| / |A union B|`. When both sets are empty,
it returns 0.0 (not 1.0), which means context similarity contributes nothing
when neither the candidate nor the query has topic or entity metadata. This
is the correct default: absence of context should not penalize or boost.

Sentiment proximity measures how similar the emotional tone is. Sentiment
values range from -1.0 to 1.0, so the maximum absolute difference is 2.0.
Dividing by 2.0 normalizes to [0.0, 1.0] where 1.0 means identical sentiment.

### 7.5 Output

Reranking produces a sorted, truncated `Vec<ScoredMemory>` -- the final shape
the consumer receives (after post-retrieval side effects, which do not modify
the results).

---

## 8. Post-Retrieval Side Effects

**Source:** `src/store/strengths.rs` and `src/graph/links.rs`

The fundamental design principle behind post-retrieval effects is that
**memory is a process, not a database.** Every retrieval changes what will be
remembered and what will be forgotten in the future. These side effects
implement two neuroscience-grounded mechanisms.

### 8.1 Bjork Dual-Strength Update (on_access)

For every result in the returned `Vec<ScoredMemory>`:

```rust
for scored in &results {
    let _ = strengths::on_access(conn, scored.node);
}
```

The `on_access` function implements Bjork and Bjork's (1992) desirable
difficulty theory via a dual-strength model:

```sql
INSERT INTO node_strengths (node_type, node_id, storage_strength, retrieval_strength, access_count, last_accessed)
VALUES (?1, ?2, 0.6, 1.0, 1, ?3)
ON CONFLICT(node_type, node_id) DO UPDATE SET
    storage_strength = MIN(1.0, storage_strength + 0.05 * (1.0 - storage_strength)),
    retrieval_strength = 1.0,
    access_count = access_count + 1,
    last_accessed = ?3
```

Two strengths are tracked independently:

**Retrieval Strength (RS):** How easily the memory can be found right now.
Reset to 1.0 on every access. Decays over time when `forget()` is called
(`RS *= 0.95` per cycle). This models the "tip of the tongue" phenomenon:
recently accessed memories are easy to find; unused memories become hard to
retrieve but are not lost.

**Storage Strength (SS):** How deeply the memory is encoded. Increments
asymptotically: `SS += 0.05 * (1 - SS)`. Never decreases. This models the
spacing effect: each retrieval deepens the memory trace, but the gain
diminishes as SS approaches 1.0. A memory accessed 20 times has much higher
SS than one accessed twice, but the 20th access adds less SS than the 2nd.

The UPSERT pattern (`INSERT ... ON CONFLICT DO UPDATE`) handles the case where
a node has no strength record yet (first access creates one with SS=0.6, RS=1.0).

### 8.2 Hebbian Co-Retrieval Strengthening (on_co_retrieval)

For every pair of results:

```rust
let retrieved_nodes: Vec<NodeRef> = results.iter().map(|r| r.node).collect();
for i in 0..retrieved_nodes.len() {
    for j in (i + 1)..retrieved_nodes.len() {
        let _ = crate::graph::links::on_co_retrieval(conn, retrieved_nodes[i], retrieved_nodes[j]);
    }
}
```

This is an O(n^2) loop over result pairs. With `max_results = 5`, this is
10 pairs. With `max_results = 10`, this is 45 pairs. The quadratic growth is
bounded by `max_results`, which the consumer controls.

For each pair, `on_co_retrieval` implements Hebbian Long-Term Potentiation
(LTP):

```sql
UPDATE links SET
    forward_weight = forward_weight + 0.1 * (1.0 - forward_weight),
    last_activated = ?5,
    activation_count = activation_count + 1
WHERE source_type = ?1 AND source_id = ?2
  AND target_type = ?3 AND target_id = ?4
```

The formula `w += 0.1 * (1 - w)` is asymptotic: weights approach 1.0 but never
reach it. The learning rate is 0.1 (hardcoded). If no link exists between the
pair, a new `CoRetrieval` link is created with initial weight 0.3:

```rust
if updated == 0 {
    create_link(conn, source, target, LinkType::CoRetrieval, 0.3)?;
}
```

**The emergent behavior:** Memories that are frequently retrieved together
become strongly linked. When one is found in the future, spreading activation
will surface the other even if it does not match the query text. The graph
reshapes through use.

### 8.3 Error Handling for Side Effects

Both side effects use `let _ = ...` to discard errors:

```rust
let _ = strengths::on_access(conn, scored.node);
let _ = crate::graph::links::on_co_retrieval(conn, retrieved_nodes[i], retrieved_nodes[j]);
```

This is deliberate: the retrieval pipeline must never fail because a write-side
effect encounters an error (e.g., SQLite busy, disk full). The read path (query
results) takes priority. If side effects fail silently, the memory system
degrades gradually (less accurate strength tracking, fewer co-retrieval links)
but remains functional.

### 8.4 Missing Side Effect: LTD

Long-Term Depression (LTD) -- the weakening of links between nodes that are
NOT co-retrieved -- is not implemented in the retrieval pipeline. The
`decay_links` function exists in `src/graph/links.rs` but is called only from
the `transform()` lifecycle process, not from `execute_query`. This is a
known gap documented in `architecture.yml`.

---

## 9. Graceful Degradation Logic

Alaya's retrieval pipeline is designed so that every capability has a fallback.
The consumer can start with the simplest possible setup (no embeddings, no
lifecycle, no context) and add capabilities incrementally. The pipeline adapts
automatically.

### 9.1 Degradation Chain

```
Level 0: Full Pipeline
  BM25 + Vector + Graph -> 3-way RRF -> Rerank
  Requirements: FTS5 matches, embedding provided, links exist

Level 1: No Embeddings
  BM25 + Graph -> 2-way RRF -> Rerank
  Trigger: query.embedding is None

Level 2: No Graph Links
  BM25 + Vector -> 2-way RRF -> Rerank
  Trigger: no links in database (or no seed nodes)

Level 3: No Embeddings AND No Links
  BM25 -> 1-way RRF (passthrough) -> Rerank
  Trigger: both conditions above

Level 4: No FTS5 Matches
  Vector + Graph -> 2-way RRF -> Rerank
  Trigger: query terms not found in episodes_fts

Level 5: Minimal
  BM25-only -> Rerank (recency only, no context)
  Trigger: default QueryContext (empty topics/entities)

Level 6: Empty Database
  [] (empty Vec, no error)
  Trigger: no episodes stored
```

### 9.2 Decision Points in Code

Each degradation decision is a simple conditional, not a configuration flag:

**Vector skip:** `match &query.embedding { Some(emb) => ..., None => vec![] }`

**Graph skip:** `if !seed_nodes.is_empty() { spread_activation(...) } else { HashMap::new() }`

**RRF input reduction:** Empty result sets are not added to the `sets` vector.

**Context degradation:** When `QueryContext` uses defaults, `context_similarity`
returns 0.0 for topic/entity overlap (both sets empty) and 1.0 for sentiment
proximity (both are 0.0, delta is 0.0). The net context boost is
`0.0 * 0.5 + 0.0 * 0.25 + 1.0 * 0.25 = 0.25`, which provides a 7.5% boost
(`0.3 * 0.25 = 0.075`) -- nearly neutral.

**Empty database:** BM25 returns `vec![]`, vector returns `vec![]`, graph has
no seeds. RRF merges zero-length vectors. Enrichment finds nothing. Reranking
returns `vec![]`. No errors, no panics, just an empty result.

### 9.3 Decision Tree Diagram

```
execute_query(conn, query)
  |
  +-- Is query.text empty?
  |     Yes -> return Ok(vec![])  (BM25 short-circuits)
  |     No  -> run BM25
  |
  +-- Is query.embedding Some?
  |     Yes -> run vector search
  |     No  -> vector_results = vec![]
  |
  +-- Are there any seed nodes (from BM25 + vector)?
  |     Yes -> run spreading activation
  |     No  -> graph_results = vec![]
  |
  +-- Are vector_results non-empty?
  |     Yes -> include in RRF sets
  |     No  -> omit (1 fewer set)
  |
  +-- Are graph_results non-empty?
  |     Yes -> include in RRF sets
  |     No  -> omit (1 fewer set)
  |
  +-- RRF merge (1-3 sets)
  |
  +-- Enrichment (fetch episodes)
  |     Missing episodes -> silently dropped
  |
  +-- Rerank (context + recency)
  |
  +-- Post-retrieval (on_access, on_co_retrieval)
  |     Errors -> silently ignored
  |
  +-- return Ok(results)
```

### 9.4 Incremental Capability Adoption

From the consumer's perspective, the degradation chain maps to an adoption
path:

| Day | Consumer Action | Pipeline Level |
|-----|----------------|----------------|
| 1 | `query(&Query::simple("..."))` | BM25-only (Level 3) |
| 1+ | Store episodes with `preceding_episode` | BM25 + Graph (Level 1) |
| 2 | Provide embeddings via `NewEpisode.embedding` and `Query.embedding` | Full (Level 0) |
| 3 | Populate `QueryContext` with topics/entities | Full + context reranking |
| Week 1 | Run `consolidate()` to create semantic nodes and links | Richer graph |
| Ongoing | Run `forget()` to decay retrieval strengths | Better signal-to-noise |

No configuration changes are required. The pipeline detects what is available
and uses it.

---

## 10. Configuration and Tuning

### 10.1 Consumer-Controlled Parameters (v0.1)

| Parameter | Location | Default | Effect |
|-----------|----------|---------|--------|
| `Query.text` | Consumer | (required) | BM25 search terms |
| `Query.embedding` | Consumer | `None` | Enables/disables vector stage |
| `Query.max_results` | Consumer | 5 | Final result cap, also affects fetch_limit (3x) |
| `Query.context.topics` | Consumer | `vec![]` | Reranking topic boost |
| `Query.context.mentioned_entities` | Consumer | `vec![]` | Reranking entity boost |
| `Query.context.sentiment` | Consumer | 0.0 | Reranking sentiment proximity |
| `Query.context.current_timestamp` | Consumer | `None` (wall clock) | Recency baseline |

### 10.2 Hardcoded Parameters (v0.1)

These values are embedded in the source code and not exposed to consumers:

| Parameter | Value | File | Line | Rationale |
|-----------|-------|------|------|-----------|
| `fetch_limit` multiplier | 3 | `pipeline.rs` | `query.max_results * 3` | Over-fetch for reranking headroom |
| RRF `k` | 60 | `pipeline.rs` | `fusion::rrf_merge(&sets, 60)` | Standard value from literature |
| BM25 over-fetch | 3 | `bm25.rs` | `(limit * 3) as u32` | Additional headroom for normalization |
| Graph `max_depth` | 1 | `pipeline.rs` | `spread_activation(conn, &seeds, 1, ...)` | Single hop for latency |
| Graph `threshold` | 0.1 | `pipeline.rs` | `..., 0.1, 0.6)` | Noise floor |
| Graph `decay_per_hop` | 0.6 | `pipeline.rs` | `..., 0.1, 0.6)` | Aggressive single-hop decay |
| Seed count (BM25) | 3 | `pipeline.rs` | `.take(3)` | Top 3 BM25 results as seeds |
| Seed count (vector) | 3 | `pipeline.rs` | `.take(3)` | Top 3 vector results as seeds |
| Context weight (topics) | 0.5 | `rerank.rs` | `topic_sim * 0.5` | Topics most important for context |
| Context weight (entities) | 0.25 | `rerank.rs` | `entity_sim * 0.25` | Entities secondary |
| Context weight (sentiment) | 0.25 | `rerank.rs` | `sentiment_sim * 0.25` | Sentiment tertiary |
| Context boost factor | 0.3 | `rerank.rs` | `1.0 + 0.3 * context_sim` | Up to 30% context boost |
| Recency boost factor | 0.2 | `rerank.rs` | `1.0 + 0.2 * recency` | Up to 20% recency boost |
| Recency half-life | 30 days | `rerank.rs` | `(-age_days / 30.0).exp()` | Exponential decay constant |
| SS increment rate | 0.05 | `strengths.rs` | `0.05 * (1.0 - storage_strength)` | Asymptotic SS growth |
| Co-retrieval LTP rate | 0.1 | `links.rs` | `0.1 * (1.0 - forward_weight)` | Hebbian learning rate |
| New co-retrieval weight | 0.3 | `links.rs` | `create_link(..., 0.3)` | Initial link strength |
| Activation cap | 2.0 | `activation.rs` | `.min(2.0)` | Prevents runaway in cycles |
| Cosine similarity floor | 0.0 | `embeddings.rs` | `if sim > 0.0` | Exclude anti-correlated |

### 10.3 Planned Configuration (v0.2)

A `RetrievalConfig` struct is planned for v0.2 to expose the most impactful
tuning knobs:

```rust
// Planned API (not yet implemented)
pub struct RetrievalConfig {
    pub rrf_k: u32,                    // Default: 60
    pub graph_max_depth: u32,          // Default: 1
    pub graph_decay: f32,              // Default: 0.6
    pub graph_threshold: f32,          // Default: 0.1
    pub seed_count: usize,             // Default: 3
    pub recency_half_life_days: f64,   // Default: 30.0
    pub context_boost: f64,            // Default: 0.3
    pub recency_boost: f64,            // Default: 0.2
    pub fetch_multiplier: usize,       // Default: 3
}
```

### 10.4 Tuning Guidance

**Want more diverse results?** Lower `rrf_k`. With `k=1`, rank position
dominates and results from different stages are more evenly weighted.

**Want recency to matter more?** Increase the recency boost factor (currently
0.2). Setting it to 0.5 gives recent memories up to 50% boost.

**Want deeper graph exploration?** Increase `max_depth` to 2 or 3. Be aware
that each hop adds a round of SQLite queries proportional to the number of
activated nodes.

**Want context similarity to matter more?** Increase the context boost factor
(currently 0.3). Requires that both episodes and queries have populated
`topics` and `mentioned_entities` fields.

**Want less aggressive graph decay?** Increase `decay_per_hop` toward 1.0
(no decay). This makes distant neighbors contribute as strongly as nearby
ones, which may introduce noise in dense graphs.

---

## 11. Stage Failure Handling

### 11.1 Error Propagation Strategy

The retrieval pipeline uses Rust's `?` operator for propagation, meaning any
SQLite error in BM25, vector search, or graph activation will abort the
entire query and return `AlayaError::Db`. The only exceptions are
post-retrieval side effects, which silently discard errors.

| Stage | Error Source | Behavior |
|-------|-------------|----------|
| BM25 | FTS5 MATCH query | Propagates via `?` |
| BM25 | Empty/sanitized-to-empty input | Returns `Ok(vec![])` (not an error) |
| Vector | `search_by_vector` SQL query | Propagates via `?` |
| Vector | No embedding provided | Returns `vec![]` (not an error) |
| Graph | `get_links_from` SQL query | Propagates via `?` |
| Graph | No seed nodes | Returns empty HashMap (not an error) |
| RRF | (pure function, no I/O) | Cannot fail |
| Enrichment | `get_episode` SQL query | `.ok()` -- silently drops missing episodes |
| Rerank | (pure function, no I/O) | Cannot fail |
| on_access | SQL UPDATE/INSERT | `let _ =` -- silently discarded |
| on_co_retrieval | SQL UPDATE/INSERT | `let _ =` -- silently discarded |

### 11.2 Malformed Query Handling

| Input | Behavior | Rationale |
|-------|----------|-----------|
| Empty string | `Ok(vec![])` | Nothing to search for |
| All punctuation | `Ok(vec![])` | Sanitizes to empty |
| Unicode text | Works (FTS5 handles Unicode) | `is_alphanumeric()` preserves Unicode letters |
| Very long text | Works (FTS5 handles long input) | No length limit enforced |
| SQL injection attempt | Sanitized + parameterized | Double protection |
| `None` embedding with vector-dependent query | BM25-only results | Graceful degradation |
| Mismatched embedding dimensions | Cosine similarity returns 0.0 | `a.len() != b.len()` guard |

### 11.3 Data Integrity During Retrieval

The pipeline does not use transactions. Each stage executes one or more
independent SQL queries. If an episode is deleted between the BM25 search
(which finds its ID) and the enrichment step (which fetches its content),
the enrichment silently drops it via `.ok()`. This is a race condition in
theory but benign in practice because Alaya is `Send` but not `Sync` -- the
consumer holds exclusive access to the connection.

### 11.4 AlayaError Variants

```rust
pub enum AlayaError {
    Db(rusqlite::Error),           // SQLite errors (most common in pipeline)
    NotFound(String),              // Used by get_episode, not surfaced in pipeline
    InvalidInput(String),          // Not currently used in pipeline
    Serialization(serde_json::Error), // context_json deserialization in get_episode
    Provider(String),              // Not used in retrieval (lifecycle only)
}
```

The pipeline can produce `AlayaError::Db` (from any SQL query) or
`AlayaError::Serialization` (from deserializing `context_json` in enrichment,
though this would require corrupt data in the database). `InvalidInput` is not
currently validated at the pipeline boundary -- another known gap.

---

## 12. Performance Characteristics

### 12.1 Complexity Analysis

| Stage | Time Complexity | SQLite Queries | Notes |
|-------|----------------|----------------|-------|
| BM25 | O(M log M) | 1 | M = FTS5 match count, dominated by sort |
| Vector | O(N * D) | 1 | N = total embeddings, D = dimensions |
| Graph | O(S * L * depth) | S * depth | S = seeds, L = avg links per node |
| RRF | O(R log R) | 0 | R = total unique results across sets |
| Enrichment | O(E) | E | E = candidates after RRF (up to fetch_limit) |
| Rerank | O(E log E) | 0 | Sort + truncate |
| Post-retrieval | O(K^2) | K + K*(K-1)/2 | K = final result count |

### 12.2 Expected Latency

For the target workload (personal agent, < 10K episodes, max_results = 5):

| Stage | Expected Latency | Dominant Factor |
|-------|-----------------|-----------------|
| BM25 | < 1 ms | FTS5 index lookup |
| Vector (1K embeddings, 384-dim) | < 1 ms | Brute-force cosine |
| Vector (10K embeddings, 384-dim) | ~5 ms | Brute-force cosine |
| Graph (6 seeds, 1 hop) | < 1 ms | 6 SQLite queries for link fetching |
| RRF | < 0.1 ms | In-memory HashMap |
| Enrichment (15 candidates) | < 1 ms | 15 SQLite primary-key lookups |
| Rerank | < 0.1 ms | In-memory sort |
| Post-retrieval (5 results) | < 1 ms | 5 UPSERTs + 10 UPDATEs |
| **Total** | **< 5 ms** | **SQLite I/O** |

### 12.3 Bottleneck Analysis

1. **Vector search at scale** -- The brute-force scan is the first bottleneck
   as the database grows beyond 10K embeddings. The planned `vec-sqlite`
   feature flag addresses this with SIMD-accelerated approximate nearest
   neighbor search.

2. **Post-retrieval writes** -- The O(K^2) co-retrieval updates produce
   `K*(K-1)/2` SQL UPDATE/INSERT operations. With `max_results = 10`, this is
   45 writes per query. Each write is a separate SQLite statement execution.
   Batching these into a single transaction would reduce overhead.

3. **Enrichment** -- Each candidate requires a separate `get_episode` query.
   A single `WHERE id IN (...)` query would be more efficient but requires
   dynamic SQL construction.

---

## 13. Known Gaps and Planned Improvements

### 13.1 Gaps in Current Implementation

| Gap | Impact | Severity | Location |
|-----|--------|----------|----------|
| Semantic/preference node enrichment missing | Vector results for non-episode nodes are silently dropped | Medium | `pipeline.rs:69-71` |
| LTD not called from retrieval pipeline | Links that should weaken (non-co-retrieved) never decay during query | Low | `pipeline.rs` (missing) |
| No transaction wrapping for post-retrieval writes | Individual writes could partially fail | Low | `pipeline.rs:80-90` |
| No input validation at pipeline boundary | Empty `max_results` (0) causes division in `fetch_limit` | Low | `pipeline.rs:17` |
| Hardcoded retrieval parameters | Consumers cannot tune RRF k, graph depth, boost factors | Medium | Multiple files |
| No caching of embeddings | Every vector search reloads all embeddings from SQLite | Low | `embeddings.rs:84-106` |
| Session scoping not available in query | Cannot restrict retrieval to a session | Low | `types.rs:Query` |

### 13.2 Planned Improvements (v0.2)

1. **RetrievalConfig struct** -- Expose tuning knobs (RRF k, graph parameters,
   boost factors, fetch multiplier) via a configuration struct passed to
   `execute_query` or stored on `AlayaStore`.

2. **Semantic and preference enrichment** -- Extend the enrichment step to
   fetch content from `semantic_nodes` and `preferences` tables for non-episode
   `NodeRef` variants.

3. **sqlite-vec integration** -- Feature flag `vec-sqlite` for SIMD-accelerated
   vector search, replacing the brute-force scan for large embedding sets.

4. **Batch post-retrieval writes** -- Wrap `on_access` and `on_co_retrieval`
   calls in a single transaction to reduce SQLite overhead.

5. **Query scoping** -- Add `session_id`, `time_range`, and `node_type` filters
   to `Query` to allow consumers to restrict the search space.

6. **Async API** -- Feature flag `async` wrapping `execute_query` in
   `spawn_blocking` for consumers using async runtimes.

### 13.3 Planned Improvements (v0.3+)

1. **Learned RRF weights** -- Adjust the relative contribution of BM25 vs.
   vector vs. graph based on historical query success patterns.

2. **Multi-hop graph exploration** -- Dynamic depth selection based on result
   quality (if direct retrieval is insufficient, explore deeper).

3. **Embedding provider trait** -- `EmbeddingProvider` trait allowing Alaya to
   compute embeddings on-demand during query when `query.embedding` is `None`
   but stored embeddings exist.

4. **Retrieval-induced forgetting (RIF)** -- Suppress retrieval strength of
   related-but-not-retrieved nodes, implementing the spacing effect more
   completely.

---

## Appendix A: Module Map

```
src/
  lib.rs                      # AlayaStore::query() -> delegates to pipeline
  types.rs                    # Query, QueryContext, NodeRef, ScoredMemory, ...
  error.rs                    # AlayaError, Result<T>
  schema.rs                   # DDL: episodes, episodes_fts, embeddings, links, ...
  retrieval/
    mod.rs                    # Module declarations
    pipeline.rs               # execute_query() -- full pipeline orchestration
    bm25.rs                   # search_bm25() -- FTS5 search + sanitization
    vector.rs                 # search_vector() -- thin wrapper over embeddings
    fusion.rs                 # rrf_merge() -- Reciprocal Rank Fusion
    rerank.rs                 # rerank() -- context + recency scoring
  graph/
    mod.rs                    # Module declarations
    activation.rs             # spread_activation() -- Collins & Loftus
    links.rs                  # create_link(), on_co_retrieval(), decay_links()
  store/
    mod.rs                    # Module declarations
    episodic.rs               # CRUD for episodes table
    semantic.rs               # CRUD for semantic_nodes table
    implicit.rs               # CRUD for impressions + preferences
    embeddings.rs             # Embedding storage, cosine search, serialization
    strengths.rs              # Bjork dual-strength: on_access(), decay_all_retrieval()
  provider.rs                 # ConsolidationProvider trait, NoOpProvider
  lifecycle/                  # Not part of retrieval pipeline (called separately)
    consolidation.rs          # CLS replay
    perfuming.rs              # Vasana preference emergence
    transformation.rs         # Dedup, prune, decay
    forgetting.rs             # RS decay, archival
```

## Appendix B: SQL Tables Involved in Retrieval

| Table | Stage | Access Pattern |
|-------|-------|---------------|
| `episodes_fts` | BM25 | `MATCH ?1 ORDER BY rank LIMIT ?2` |
| `episodes` | BM25 (join), Enrichment | Primary key lookup by `id` |
| `embeddings` | Vector | Full table scan, cosine computation |
| `links` | Graph | Index scan by `(source_type, source_id)` |
| `node_strengths` | Post-retrieval | UPSERT by `(node_type, node_id)` |

## Appendix C: Research References

| Concept | Reference | Where Used |
|---------|-----------|------------|
| BM25 | Robertson et al., "The Probabilistic Relevance Framework" (2009) | FTS5 built-in ranking |
| Reciprocal Rank Fusion | Cormack, Clarke & Buettcher, "Reciprocal Rank Fusion Outperforms Condorcet and Individual Rank Learning Methods" (2009) | `fusion.rs` |
| Spreading Activation | Collins & Loftus, "A Spreading-Activation Theory of Semantic Processing" (1975) | `activation.rs` |
| Dual-Strength Model | Bjork & Bjork, "A New Theory of Disuse and an Old Theory of Stimulus Fluctuation" (1992) | `strengths.rs` |
| Hebbian Learning | Hebb, "The Organization of Behavior" (1949); Bliss & Lomo, "Long-lasting potentiation of synaptic transmission" (1973) | `links.rs on_co_retrieval` |
| Desirable Difficulty | Bjork, "Memory and Metamemory Considerations in the Training of Human Beings" (1994) | Retrieval-induced SS increment |

---

*Document generated for Alaya v0.1.0. All code references verified against
`src/retrieval/`, `src/graph/`, `src/store/`, and `src/types.rs` in the
repository.*
