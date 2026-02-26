# Alaya Architecture Blueprint

> Library Architecture Blueprint for Alaya v0.1 -- Embeddable Rust Memory Engine

**Document type:** Library Architecture Blueprint
**Version:** 0.1.0
**Last updated:** 2026-02-26
**Status:** Living document, tracks implementation

---

## 1. Executive Summary

Alaya is a Rust library crate, not a service. There are no agents to orchestrate, no API routes, no running processes. The architecture is the internal component topology of a memory engine that compiles into the consumer's binary via `cargo add alaya`.

The design serves a single organizing belief: **Memory is a process, not a database.** Every retrieval changes what is remembered. The graph reshapes through use. Preferences emerge from accumulated observations without declaration. Forgetting improves retrieval quality through principled decay.

### Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Rust (stable) | Zero GC, memory safety, FFI-embeddable, single binary |
| Storage | SQLite via rusqlite 0.32 (bundled) | Zero-config, single-file invariant, WAL for concurrency, FTS5 built-in |
| Retrieval | BM25 + vector + graph spreading activation, fused via RRF (k=60) | Three independent signals; any subset degrades gracefully to the rest |
| Lifecycle | Explicit calls (consolidate/perfume/transform/forget) | The agent controls when lifecycle runs; no background threads, no timers |
| Extension | Trait-based providers (ConsolidationProvider, EmbeddingProvider) | Agent owns the LLM connection; Alaya never makes network calls |
| Error model | `Result<T, AlayaError>` everywhere, `#[non_exhaustive]` on error enum | Forward-compatible, no panics in library code, caller decides recovery |
| Entry point | `AlayaStore::open(path)` | Single struct owns the connection; all interaction through one handle |

### Axiom Hierarchy (Conflict Resolution Order)

```
Safety > Privacy > Correctness > Simplicity > Performance > Features
```

When any design decision creates tension between these values, the leftward value wins. This is not aspirational -- it is a decision procedure applied to every PR.

---

## 2. Component Topology

The following diagram shows the internal layering of the Alaya crate. Data flows downward for writes and upward for reads. Lifecycle processes cut across all layers.

```
+-----------------------------------------------------------+
|                       AlayaStore                          |  <-- Public API
|  open() | store_episode() | query() | consolidate()      |
|  perfume() | transform() | forget() | status() | purge() |
+-----------------------------------------------------------+
      |               |               |
      v               v               v
+-------------+ +-------------+ +-------------+
|  Episodic   | |  Semantic   | |  Implicit   |  <-- Three Stores
|   Store     | |   Store     | |   Store     |
| (episodes,  | | (semantic_  | | (impressions|
|  episodes_  | |  nodes)     | |  preferences|
|  fts)       | |             | |  )          |
+------+------+ +------+------+ +------+------+
       |               |               |
       +-------+-------+-------+-------+
               |               |
               v               v
+-------------------------------------------+
|       Graph Overlay (Hebbian)             |  <-- Shared Layer
|  links table, adjacency by (type, id)    |
|  LTP: on_co_retrieval() strengthens      |
|  LTD: decay_links() weakens on disuse    |
|  Spreading Activation: iterative BFS     |
|    with decay_per_hop and threshold       |
+-------------------------------------------+
               |               |
               v               v
+-------------------------------------------+
|       Retrieval Pipeline                  |  <-- Query Path
|  Stage 1: BM25 (FTS5 MATCH + rank)       |
|  Stage 2: Vector (cosine similarity)      |
|  Stage 3: Graph (spreading activation)    |
|  Stage 4: RRF fusion (k=60)              |
|  Stage 5: Rerank (recency + context)      |
|  Stage 6: Post-retrieval updates          |
|    - Strength tracking (on_access)        |
|    - Co-retrieval Hebbian LTP             |
+-------------------------------------------+
               |               |
               v               v
+-------------------------------------------+
|       Lifecycle Processes                 |  <-- Cognitive Layer
|  consolidate(): episodes -> semantic      |
|    via ConsolidationProvider trait         |
|  perfume(): interactions -> impressions   |
|    -> preferences (vasana crystallization)|
|  transform(): dedup, prune, decay         |
|    (asraya-paravrtti toward clarity)      |
|  forget(): Bjork dual-strength decay      |
|    RS decays, low SS+RS nodes archived    |
+-------------------------------------------+
               |
               v
+-------------------------------------------+
|       Provider Traits (Extension Points)  |  <-- Agent Boundary
|  ConsolidationProvider:                   |
|    extract_knowledge(episodes) -> nodes   |
|    extract_impressions(interaction) ->    |
|      impressions                          |
|    detect_contradiction(a, b) -> bool     |
|  NoOpProvider: default, returns empty     |
+-------------------------------------------+
               |
               v
+-------------------------------------------+
|       SQLite Storage Layer                |  <-- Persistence
|  WAL mode | journal_size_limit            |
|  synchronous = NORMAL                     |
|  foreign_keys = ON                        |
|  7 tables + 1 FTS5 virtual table          |
|  3 sync triggers (insert/delete/update)   |
|  9 indexes (including unique constraints) |
|  Single file: all state in one .db file   |
+-------------------------------------------+
```

### Module Map

The Rust module structure directly mirrors the component topology:

```
src/
  lib.rs              AlayaStore struct, public API surface
  types.rs            All public types, newtypes, enums, reports
  error.rs            AlayaError enum, Result type alias
  schema.rs           SQLite DDL, open_db(), init_db(), PRAGMA config
  provider.rs         ConsolidationProvider trait, NoOpProvider, MockProvider
  store/
    mod.rs            Module declarations
    episodic.rs       Episodes CRUD, session queries, unconsolidated fetch
    semantic.rs       Semantic nodes CRUD, corroboration tracking
    implicit.rs       Impressions and preferences CRUD, decay, pruning
    embeddings.rs     Embedding storage, brute-force cosine search, serialize/deserialize
    strengths.rs      Bjork dual-strength tracking, decay, archival detection
  graph/
    mod.rs            Module declarations
    links.rs          Link CRUD, co-retrieval LTP, decay, pruning
    activation.rs     Spreading activation (Collins & Loftus 1975)
  retrieval/
    mod.rs            Module declarations
    bm25.rs           FTS5 search with input sanitization, score normalization
    vector.rs         Vector similarity search delegation
    fusion.rs         Reciprocal Rank Fusion (Cormack et al. 2009)
    rerank.rs         Context similarity + recency decay reranking
    pipeline.rs       Full query orchestration: BM25 -> vector -> graph -> RRF -> rerank -> post-retrieval
  lifecycle/
    mod.rs            Module declarations
    consolidation.rs  CLS replay: episodes -> semantic nodes via provider
    perfuming.rs      Vasana: impressions -> preference crystallization
    transformation.rs Asraya-paravrtti: dedup, prune, decay
    forgetting.rs     Bjork dual-strength: RS decay, archive low SS+RS nodes
```

### Ownership and Lifetimes

```
AlayaStore
  owns -> rusqlite::Connection (single owner, &self borrows for all operations)

All public methods take &self (shared reference).
SQLite handles its own internal locking via WAL mode.
No Arc, no Mutex, no interior mutability in Alaya's public API.
Thread safety: AlayaStore is Send but not Sync.
Multi-thread access: caller wraps in Mutex or uses connection pooling.
```

---

## 3. Data Model

### SQLite Schema

All persistent state lives in a single SQLite file. The schema is created idempotently on `AlayaStore::open()` via `CREATE TABLE IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS`.

#### 3.1 Episodic Store (Hippocampus)

```sql
CREATE TABLE episodes (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    content      TEXT    NOT NULL,
    role         TEXT    NOT NULL,           -- 'user' | 'assistant' | 'system'
    session_id   TEXT    NOT NULL,
    timestamp    INTEGER NOT NULL,           -- Unix seconds
    context_json TEXT    NOT NULL DEFAULT '{}'
);

-- Indexes
CREATE INDEX idx_episodes_session   ON episodes(session_id);
CREATE INDEX idx_episodes_timestamp ON episodes(timestamp);

-- FTS5 external-content index
CREATE VIRTUAL TABLE episodes_fts
    USING fts5(content, content=episodes, content_rowid=id);

-- Sync triggers: INSERT, DELETE, UPDATE OF content
-- (3 triggers keep FTS5 and episodes table in sync)
```

The `context_json` column stores a serialized `EpisodeContext` struct containing topics, sentiment, conversation turn, mentioned entities, and optional preceding episode reference.

#### 3.2 Semantic Store (Neocortex)

```sql
CREATE TABLE semantic_nodes (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    content              TEXT    NOT NULL,
    node_type            TEXT    NOT NULL,    -- 'fact' | 'relationship' | 'event' | 'concept'
    confidence           REAL    NOT NULL DEFAULT 0.5,
    source_episodes_json TEXT    NOT NULL DEFAULT '[]',
    created_at           INTEGER NOT NULL,
    last_corroborated    INTEGER NOT NULL,
    corroboration_count  INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX idx_semantic_type ON semantic_nodes(node_type);
```

Semantic nodes are created by the consolidation process, not by direct user insertion. Each node tracks which episodes it was extracted from and how many times it has been corroborated by independent evidence.

#### 3.3 Implicit Store (Vasana)

```sql
-- Raw behavioral traces
CREATE TABLE impressions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    domain      TEXT    NOT NULL,
    observation TEXT    NOT NULL,
    valence     REAL    NOT NULL DEFAULT 0.0,   -- [-1.0, 1.0]
    timestamp   INTEGER NOT NULL
);

CREATE INDEX idx_impressions_domain    ON impressions(domain);
CREATE INDEX idx_impressions_timestamp ON impressions(timestamp);

-- Crystallized preferences (emergent from impressions)
CREATE TABLE preferences (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    domain          TEXT    NOT NULL,
    preference      TEXT    NOT NULL,
    confidence      REAL    NOT NULL DEFAULT 0.5,
    evidence_count  INTEGER NOT NULL DEFAULT 1,
    first_observed  INTEGER NOT NULL,
    last_reinforced INTEGER NOT NULL
);

CREATE INDEX idx_preferences_domain ON preferences(domain);
```

Impressions accumulate as raw traces. When a domain reaches the crystallization threshold (5 impressions), a preference emerges. Preferences decay if not reinforced.

#### 3.4 Shared Infrastructure

```sql
-- Embeddings (polymorphic across all stores)
CREATE TABLE embeddings (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    node_type  TEXT    NOT NULL,      -- 'episode' | 'semantic' | 'preference'
    node_id    INTEGER NOT NULL,
    embedding  BLOB    NOT NULL,      -- f32 array, little-endian
    model      TEXT    NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL
);

CREATE UNIQUE INDEX idx_embeddings_node ON embeddings(node_type, node_id);

-- Hebbian graph overlay
CREATE TABLE links (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    source_type      TEXT    NOT NULL,
    source_id        INTEGER NOT NULL,
    target_type      TEXT    NOT NULL,
    target_id        INTEGER NOT NULL,
    forward_weight   REAL    NOT NULL DEFAULT 0.5,
    backward_weight  REAL    NOT NULL DEFAULT 0.5,
    link_type        TEXT    NOT NULL,   -- 'temporal' | 'topical' | 'entity' | 'causal' | 'co_retrieval'
    created_at       INTEGER NOT NULL,
    last_activated   INTEGER NOT NULL,
    activation_count INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX  idx_links_source ON links(source_type, source_id);
CREATE INDEX  idx_links_target ON links(target_type, target_id);
CREATE UNIQUE INDEX idx_links_pair
    ON links(source_type, source_id, target_type, target_id, link_type);

-- Bjork dual-strength model
CREATE TABLE node_strengths (
    node_type          TEXT    NOT NULL,
    node_id            INTEGER NOT NULL,
    storage_strength   REAL    NOT NULL DEFAULT 0.5,   -- Monotonically increases with access
    retrieval_strength REAL    NOT NULL DEFAULT 1.0,    -- Decays over time, reset on access
    access_count       INTEGER NOT NULL DEFAULT 1,
    last_accessed      INTEGER NOT NULL,
    PRIMARY KEY (node_type, node_id)
);
```

### Entity Relationship Diagram

```
                   +------------------+
                   |   episodes       |
                   +------------------+
                   | id (PK)          |
                   | content          |       +------------------+
                   | role             |       |  episodes_fts    |
                   | session_id       |<----->|  (FTS5 virtual)  |
                   | timestamp        |       |  synced by       |
                   | context_json     |       |  triggers        |
                   +--------+---------+       +------------------+
                            |
            +---------------+----------------+
            |               |                |
            v               |                v
    +-------+------+        |        +-------+------+
    | embeddings   |        |        | links        |
    +--------------+        |        +--------------+
    | node_type    |  NodeRef        | source_type  |
    | node_id      |  (polymorphic   | source_id    |
    | embedding    |   FK via        | target_type  |
    | model        |   type+id)      | target_id    |
    +--------------+        |        | link_type    |
            ^               |        | weights      |
            |               |        +------+-------+
    +-------+------+        |               ^
    | semantic_    |        |               |
    | nodes        |--------+        +------+-------+
    +--------------+        |        | node_        |
    | id (PK)      |        |        | strengths    |
    | content      |        |        +--------------+
    | node_type    |        |        | node_type PK |
    | confidence   |        |        | node_id   PK |
    | corroboration|        |        | storage_str  |
    +--------------+        |        | retrieval_str|
                            |        +--------------+
    +-------+------+        |               ^
    | impressions  |        |               |
    +--------------+--------+               |
    | id (PK)      |                        |
    | domain       |                        |
    | observation  |                        |
    | valence      |                        |
    +--------------+                        |
                                            |
    +-------+------+                        |
    | preferences  |------------------------+
    +--------------+
    | id (PK)      |
    | domain       |
    | preference   |
    | confidence   |
    | evidence_cnt |
    +--------------+
```

**Polymorphic references:** The `embeddings`, `links`, and `node_strengths` tables use `(node_type, node_id)` pairs to reference rows across episodes, semantic_nodes, and preferences. This avoids separate join tables per entity while keeping the graph overlay unified. The Rust type `NodeRef` enum enforces valid combinations at compile time.

### Schema Migration Strategy

For v0.1, the schema is created fresh and versioned implicitly by the crate version. Future versions will add a `schema_version` table and apply migrations sequentially via `init_db()`. The `CREATE TABLE IF NOT EXISTS` pattern already provides idempotent re-initialization.

Migration plan for post-v0.1:
1. Add `CREATE TABLE IF NOT EXISTS _alaya_meta (key TEXT PRIMARY KEY, value TEXT)`
2. Store `schema_version` as an integer
3. Apply migration functions in order: `migrate_1_to_2()`, `migrate_2_to_3()`, etc.
4. Each migration runs inside a transaction with `BEGIN IMMEDIATE`

---

## 4. Retrieval Pipeline

The retrieval pipeline is the hot path. A call to `AlayaStore::query()` executes the following stages in sequence:

```
query(&Query)
  |
  v
+-- Stage 1: Parallel Retrieval Signals ---------------------+
|                                                            |
|  BM25 Search              Vector Search    Graph Activation|
|  (FTS5 MATCH)             (cosine sim)     (spread from   |
|  -> ranked episodes       -> ranked nodes   top BM25+vec  |
|  -> normalized [0,1]      -> [0,1] scores   seeds, 1 hop) |
|                                                            |
+-- Stage 2: RRF Fusion (k=60) -----------------------------+
|                                                            |
|  For each node d across all result sets:                   |
|    score(d) = SUM( 1 / (60 + rank_i + 1) )               |
|  Sort by descending fused score                            |
|                                                            |
+-- Stage 3: Enrich + Rerank --------------------------------+
|                                                            |
|  Load full episode content for top candidates              |
|  Rerank by: base_score * (1 + 0.3*context_sim)            |
|                        * (1 + 0.2*recency_decay)           |
|  context_sim = 0.5*topic_jaccard + 0.25*entity_jaccard     |
|                + 0.25*sentiment_similarity                  |
|  recency_decay = exp(-age_days / 30)                       |
|                                                            |
+-- Stage 4: Post-Retrieval Updates -------------------------+
|                                                            |
|  For each returned result:                                 |
|    strengths::on_access() -> RS=1.0, SS += 0.05*(1-SS)    |
|  For each pair of returned results:                        |
|    links::on_co_retrieval() -> Hebbian LTP                 |
|      (weight += 0.1 * (1 - weight), asymptotic to 1.0)    |
|                                                            |
+------------------------------------------------------------+
  |
  v
Vec<ScoredMemory>
```

### 4.1 BM25 Stage (FTS5)

**Input:** Query text string.
**Process:**
1. Sanitize input: strip all non-alphanumeric, non-whitespace characters to prevent FTS5 syntax injection.
2. Execute `SELECT ... FROM episodes_fts WHERE episodes_fts MATCH ?1 ORDER BY rank LIMIT ?2`.
3. FTS5 `rank` values are negative (lower = more relevant). Normalize to [0.0, 1.0] via min-max scaling.
4. Return `Vec<(EpisodeId, f64)>`.

**Graceful degradation:** If the query is empty or sanitization produces an empty string, return an empty vec. The pipeline continues with other signals.

### 4.2 Vector Stage (Brute-Force Cosine)

**Input:** Optional query embedding (`Vec<f32>`).
**Process:**
1. If no embedding provided, return empty vec (graceful degradation -- the pipeline works without embeddings).
2. Load all embeddings from the `embeddings` table.
3. Compute cosine similarity between query embedding and each stored embedding.
4. Filter out zero or negative similarities.
5. Sort descending, truncate to limit.

**Embedding format:** `Vec<f32>` serialized as little-endian bytes in a BLOB column. The brute-force scan is O(n) in embedding count, viable for up to ~10K embeddings.

**Tiered scaling plan:**
| Embedding Count | Backend | Latency Profile |
|----------------|---------|-----------------|
| < 10,000 | Brute-force (in-crate) | < 5ms |
| < 50,000 | sqlite-vec (feature flag `vec-sqlite`) | < 10ms (SIMD) |
| > 50,000 | External HNSW via EmbeddingProvider trait | Implementation-dependent |

### 4.3 Graph Stage (Spreading Activation)

**Input:** Seed nodes (top 3 from BM25 + top 3 from vector).
**Process:** Implements Collins & Loftus (1975) spreading activation:
1. Initialize seed nodes with activation = 1.0.
2. For each iteration (max_depth = 1 hop for query path):
   - For each active node above threshold (0.1):
     - Retrieve outgoing links via `get_links_from()`.
     - Spread activation: `spread = activation * forward_weight * decay_per_hop`.
     - Accumulate activation at target nodes, capped at 2.0 to prevent runaway.
3. Filter results below threshold.
4. Exclude seed nodes from graph results (they are already in BM25/vector results).

**Parameters:**
- `decay_per_hop`: 0.6 (40% loss per hop)
- `threshold`: 0.1 (minimum activation to propagate)
- `max_depth`: 1 for query path, 2-3 for `neighbors()` exploration

### 4.4 RRF Fusion

Reciprocal Rank Fusion merges the three ranked result sets into a single ordering:

```
score(d) = SUM over all result_sets of: 1 / (k + rank_i + 1)
```

Where `k = 60` (standard RRF constant from Cormack, Clarke & Buettcher 2009). Documents appearing in multiple result sets receive contributions from each, naturally boosting documents with broad relevance.

**Key property:** RRF is robust to score miscalibration between the three signals. It only uses rank ordering, not raw scores. This is critical because BM25, cosine similarity, and activation levels have fundamentally different score distributions.

### 4.5 Reranking

Post-fusion reranking adjusts scores using contextual signals:

```
final_score = base_score * (1 + 0.3 * context_similarity) * (1 + 0.2 * recency_decay)
```

- **Context similarity:** Jaccard overlap of topics (50%), entities (25%), and sentiment distance (25%) between the candidate episode's encoding context and the query context.
- **Recency decay:** Exponential `exp(-age_days / 30)`. Episodes from the last hour score ~1.0; from 30 days ago ~0.37; from 90 days ~0.05.

### 4.6 Post-Retrieval Effects

Every retrieval mutates the memory store. This is the core implementation of "memory is a process":

1. **Strength tracking:** Each returned result's `retrieval_strength` is reset to 1.0 and `storage_strength` accumulates via `SS += 0.05 * (1 - SS)` (asymptotic approach to 1.0). This models the Bjork principle that retrieval practice is the most powerful memory strengthener.

2. **Hebbian co-retrieval LTP:** For every pair of results returned together, their link weight increases: `w += 0.1 * (1 - w)`. New co-retrieval links are created at weight 0.3 if they do not exist. This means memories that are frequently retrieved together become more strongly connected over time.

### 4.7 Graceful Degradation Chain

```
Full pipeline:      BM25 + vector + graph -> RRF -> rerank
No embeddings:      BM25 + graph -> RRF -> rerank
No graph links:     BM25 + vector -> RRF -> rerank (or BM25-only)
No FTS matches:     vector + graph -> RRF -> rerank
Empty database:     [] (empty result, no error)
```

Each signal independently produces results or returns an empty vec. The pipeline never fails due to a missing signal -- it degrades to whatever signals are available, down to an empty result on an empty database.

---

## 5. Cognitive Lifecycle Pipeline

Lifecycle processes are explicit method calls on `AlayaStore`. The agent decides when to run them. There are no background threads, no timers, no automatic triggers. This respects the axiom "the agent owns identity" -- the agent decides when memory should be processed.

```
store_episode() -----> Episodes accumulate
                           |
                           v (agent calls consolidate())
                   +-------------------+
                   |   Consolidation   |  CLS replay
                   | episodes -> nodes |  via ConsolidationProvider
                   +--------+----------+
                            |
                            v
                   Semantic nodes + Causal links created
                            |
perfume(interaction) -----> |
                            |
                   +--------v----------+
                   |    Perfuming      |  Vasana crystallization
                   | interaction ->    |  impressions accumulate
                   | impressions ->    |  threshold -> preference
                   | preferences       |
                   +--------+----------+
                            |
                            v (agent calls transform())
                   +--------+----------+
                   |  Transformation   |  Asraya-paravrtti
                   | dedup semantic    |  cosine > 0.95 -> merge
                   | prune weak links  |  weight < 0.02 -> delete
                   | decay preferences |  half-life 30 days
                   | prune impressions |  max age 90 days
                   +--------+----------+
                            |
                            v (agent calls forget())
                   +--------+----------+
                   |    Forgetting     |  Bjork dual-strength
                   | decay RS * 0.95   |  per sweep
                   | archive low SS+RS |  SS < 0.1 AND RS < 0.05
                   +-------------------+
```

### 5.1 Consolidation (CLS Replay)

**Reference:** Complementary Learning Systems theory (McClelland et al., 1995).

The episodic store (hippocampus) accumulates raw conversation episodes. Consolidation transfers knowledge to the semantic store (neocortex) through the ConsolidationProvider trait.

**Process:**
1. Fetch unconsolidated episodes (those not linked to any semantic node). Batch size: 10 max, minimum 3 required.
2. Call `provider.extract_knowledge(episodes)` -- the agent's LLM extracts semantic nodes.
3. For each extracted node:
   - Store in `semantic_nodes` table.
   - Create `Causal` links from the new semantic node to its source episodes (weight 0.7).
   - Initialize Bjork strength tracking.
4. Return `ConsolidationReport { episodes_processed, nodes_created, links_created }`.

**With NoOpProvider:** `extract_knowledge()` returns an empty vec. Episodes accumulate indefinitely. Consolidation is a no-op but does not error. The episodic store remains fully queryable via BM25.

### 5.2 Perfuming (Vasana Crystallization)

**Reference:** Yogacara Buddhist concept of vasana -- subtle imprints left by experience.

Each interaction leaves behavioral traces (impressions). When enough traces accumulate in a domain, a preference crystallizes.

**Process:**
1. Call `provider.extract_impressions(interaction)` -- the agent extracts behavioral observations.
2. Store each impression in the `impressions` table.
3. For each affected domain:
   - Count total impressions in that domain.
   - If count >= crystallization threshold (5):
     - If no preference exists: crystallize a new preference from recent impressions. Confidence scales with evidence count: `min(0.9, count / 20)`.
     - If preference exists: reinforce it (evidence_count += 1, confidence += 0.1).
4. Return `PerfumingReport { impressions_stored, preferences_crystallized, preferences_reinforced }`.

**Crystallization threshold:** 5 impressions per domain. This prevents premature preference formation from a single observation.

### 5.3 Transformation (Asraya-paravrtti)

**Reference:** Yogacara concept of "basis transformation" -- the store purifies through cycles.

Periodic refinement that moves the memory store toward greater clarity and accuracy.

**Steps (in order):**
1. **Semantic deduplication:** Load all semantic node embeddings. For each pair with cosine similarity >= 0.95, merge by keeping the older node, transferring links, incrementing corroboration count, and deleting the duplicate.
2. **Link pruning:** Delete all links with `forward_weight < 0.02 AND backward_weight < 0.02`. These connections have decayed below usefulness.
3. **Preference decay:** For preferences not reinforced within `PREFERENCE_HALF_LIFE_SECS` (30 days), multiply confidence by 0.95.
4. **Preference pruning:** Delete preferences with `confidence < 0.05`.
5. **Impression pruning:** Delete impressions older than `MAX_IMPRESSION_AGE_SECS` (90 days). These have already contributed to preference crystallization or are stale.

Returns `TransformationReport { duplicates_merged, links_pruned, preferences_decayed, impressions_pruned }`.

### 5.4 Forgetting (Bjork Dual-Strength)

**Reference:** Bjork & Bjork (1992), "A New Theory of Disuse."

The Bjork model distinguishes two independent strength dimensions:

- **Storage Strength (SS):** How deeply encoded a memory is. Monotonically increases with each access. Approaches 1.0 asymptotically: `SS += 0.05 * (1 - SS)`.
- **Retrieval Strength (RS):** How accessible a memory is right now. Decays over time, reset to 1.0 on access. Per-sweep decay: `RS *= 0.95`.

**Key insight:** A memory can have high SS (deeply learned) but low RS (hard to retrieve without strong cue). This models the common experience of "I know I know this, but I can't recall it right now."

**Archival rule:** Nodes with `SS < 0.1 AND RS < 0.05` are deleted. These are memories that were never deeply encoded and have also become inaccessible -- they contribute noise, not signal.

**What forgetting does not do:** Forgetting does not delete preferences (those are managed by transformation's confidence decay). Forgetting does not delete embeddings directly (those are cascade-deleted with their parent entities).

---

## 6. Extension Model

Alaya's extension model uses Rust traits to define boundaries between the memory engine and the agent's capabilities (typically an LLM). The core crate never makes network calls, never instantiates an LLM client, and never requires any specific provider.

### 6.1 ConsolidationProvider Trait

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

The agent implements this trait, typically by formatting episodes into an LLM prompt, parsing the response into typed structs, and returning them. Alaya does not know or care how the extraction happens.

**Contract:**
- `extract_knowledge()`: Called during `consolidate()`. Receives a batch of 3-10 episodes. Must return zero or more `NewSemanticNode` values. Each node specifies its source episodes for provenance linking.
- `extract_impressions()`: Called during `perfume()`. Receives a single interaction. Must return zero or more `NewImpression` values with domain, observation text, and valence.
- `detect_contradiction()`: Called during `transform()` (planned). Receives two semantic nodes. Returns true if they contradict each other. Used for contradiction resolution.

### 6.2 NoOpProvider

```rust
pub struct NoOpProvider;

impl ConsolidationProvider for NoOpProvider {
    fn extract_knowledge(&self, _: &[Episode]) -> Result<Vec<NewSemanticNode>> { Ok(vec![]) }
    fn extract_impressions(&self, _: &Interaction) -> Result<Vec<NewImpression>> { Ok(vec![]) }
    fn detect_contradiction(&self, _: &SemanticNode, _: &SemanticNode) -> Result<bool> { Ok(false) }
}
```

NoOpProvider is the default. When used:
- `consolidate()` processes episodes but creates no semantic nodes (BM25 retrieval still works).
- `perfume()` stores no impressions (no preferences emerge).
- All lifecycle methods succeed without error.

This implements the graceful degradation axiom: everything works without an LLM. The episodic store alone provides a fully functional conversational memory via BM25 search.

### 6.3 EmbeddingProvider Trait (Planned for v0.2)

```rust
pub trait EmbeddingProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>>;
    fn dimensions(&self) -> usize;
    fn model_id(&self) -> &str;
}
```

Planned for v0.2, this trait will allow agents to plug in any embedding model. Feature flags `embed-ort` and `embed-fastembed` will provide optional in-process implementations.

### 6.4 How Providers Plug In

```rust
// Agent code:
let store = AlayaStore::open("memory.db")?;

// Without provider (everything works, just no consolidation/perfuming):
let noop = NoOpProvider;
store.consolidate(&noop)?;  // No-op, returns empty report

// With a custom provider:
struct MyProvider { client: OpenAIClient }
impl ConsolidationProvider for MyProvider {
    fn extract_knowledge(&self, episodes: &[Episode]) -> Result<Vec<NewSemanticNode>> {
        // Format episodes into prompt, call LLM, parse response
        // ...
    }
    // ...
}

let provider = MyProvider { client };
store.consolidate(&provider)?;  // Creates semantic nodes from episodes
```

---

## 7. Technology Stack

| Layer | Technology | Version | Rationale |
|-------|-----------|---------|-----------|
| Language | Rust | stable (edition 2021) | Zero GC, memory safety, FFI-embeddable, cross-platform, single binary compilation |
| Storage | rusqlite | 0.32 | Bundled SQLite, WAL mode, `modern_sqlite` feature for latest SQLite features and FTS5 |
| Full-text search | SQLite FTS5 | built-in | External content tables, BM25 ranking, porter stemmer support, synced via triggers |
| Serialization | serde + serde_json | 1.x | De-facto Rust serialization standard, derive macros, zero-copy deserialization |
| Error handling | thiserror | 2.x | Derive macros for Display/Error, `#[from]` for automatic conversion, zero runtime cost |
| Vector search (default) | Brute-force cosine | in-crate | Zero additional dependencies, O(n) scan, viable to ~10K embeddings |
| Vector search (opt) | sqlite-vec | feature flag | SIMD-accelerated, SQL-native, viable to ~50K (planned for v0.2) |
| Embedding (opt) | ort / fastembed-rs | feature flags | In-process ONNX inference, no network calls (planned for v0.2) |
| Benchmarking | divan | latest | Attribute-based benches, allocation profiling (planned for v0.2) |
| Semver checking | cargo-semver-checks | CI only | Validates API compatibility before publish |
| FFI (Tier 2) | cbindgen | latest | C header generation from Rust types (planned for v0.2) |
| Python bindings (Tier 3) | PyO3 | latest | Python bindings with zero-copy where possible (planned for v0.3) |

### Dependency Philosophy

The runtime dependency set is minimal by design:

```toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled", "modern_sqlite"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
```

Four crates. No tokio, no reqwest, no HTTP, no TLS, no async runtime. This is the complete transitive dependency set for the default feature set (excluding rusqlite's bundled SQLite C compilation). The agent adds whatever provider dependencies it needs in its own crate.

---

## 8. SQLite Configuration

### 8.1 PRAGMA Configuration (Applied on Every `open()`)

```sql
PRAGMA journal_mode = WAL;       -- Write-Ahead Logging for concurrent reads + single writer
PRAGMA foreign_keys = ON;        -- Enforce referential integrity
PRAGMA synchronous = NORMAL;     -- Balanced durability: sync at WAL checkpoints, not every commit
```

### 8.2 WAL Mode Details

WAL (Write-Ahead Logging) is the right choice for an embedded library:
- **Readers never block writers.** Multiple `query()` calls can proceed while a `store_episode()` is committing.
- **Writers never block readers.** An in-progress write does not prevent concurrent reads.
- **Single writer at a time.** This is fine for a library embedded in one process.

**WAL growth management (planned for v0.2):**
```sql
PRAGMA journal_size_limit = 16777216;   -- 16 MB WAL size limit
PRAGMA wal_autocheckpoint = 1000;       -- Checkpoint every 1000 pages
```

Periodic `PRAGMA wal_checkpoint(PASSIVE)` should be called by the agent during idle periods to reclaim WAL space without blocking.

### 8.3 FTS5 Configuration

The current FTS5 configuration uses defaults (Unicode61 tokenizer). Planned enhancements for v0.2:

```sql
-- Future: custom tokenizer configuration
CREATE VIRTUAL TABLE episodes_fts USING fts5(
    content,
    content=episodes,
    content_rowid=id,
    tokenize='porter unicode61'  -- Porter stemmer for morphological matching
);
```

**Input sanitization (currently implemented):**
```rust
let sanitized: String = query.chars()
    .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
    .collect();
```

All FTS5 MATCH input is sanitized by replacing non-alphanumeric characters with spaces. This prevents FTS5 syntax injection (e.g., `OR`, `NOT`, `NEAR`, column filters) from user-controlled query text.

### 8.4 Transaction Discipline

**Current state:** Individual operations execute as implicit transactions (SQLite's autocommit mode).

**Planned for v0.1 hardening:** All write operations must use `BEGIN IMMEDIATE`:

```rust
conn.execute_batch("BEGIN IMMEDIATE")?;
// ... write operations ...
conn.execute_batch("COMMIT")?;
```

`BEGIN IMMEDIATE` acquires a reserved lock immediately, preventing SQLITE_BUSY errors from deferred locks that upgrade mid-transaction. This is a correctness requirement documented in the constraint axioms.

---

## 9. Feature Flags

Current v0.1 ships with no feature flags -- all behavior is included by default. The following flags are planned for v0.2+:

| Flag | What It Enables | Dependencies Added | Target Version |
|------|----------------|-------------------|----------------|
| `vec-sqlite` | sqlite-vec SIMD vector search | sqlite-vec crate | v0.2 |
| `embed-ort` | ONNX Runtime embedding backend | ort crate | v0.2 |
| `embed-fastembed` | Turnkey embeddings (fastembed-rs) | fastembed-rs crate | v0.2 |
| `async` | Async API via `spawn_blocking` wrappers | tokio (dep) | v0.2 |

### Flag Design Principles

1. **Default off, additive only.** Each flag adds capabilities without removing any. `cargo add alaya` gives you the full synchronous API with brute-force vector search.
2. **Maximum 4-6 flags.** Feature flag explosion is a maintenance burden. Each flag must justify its existence.
3. **No flag combinations required.** Each flag is independent. `embed-ort` does not require `vec-sqlite`. `async` does not require any embedding flag.
4. **Test matrix stays bounded.** CI tests `default`, `all-features`, and each flag individually. This is 6-8 configurations, not combinatorial explosion.

---

## 10. Deployment Considerations (Library Publishing)

Alaya is published as a Rust crate on crates.io. There is no server to deploy, no container to build, no infrastructure to provision.

### 10.1 `cargo publish` Checklist

Before each release:

- [ ] All tests pass: `cargo test --all-features`
- [ ] No clippy warnings: `cargo clippy --all-features -- -D warnings`
- [ ] Documentation builds: `cargo doc --all-features --no-deps`
- [ ] Semver compliance: `cargo semver-checks check-release`
- [ ] MSRV verification: test on declared minimum Rust version
- [ ] Changelog updated
- [ ] Version bumped in `Cargo.toml`
- [ ] `cargo publish --dry-run` succeeds

### 10.2 Semver Compliance

Alaya follows Rust's semver conventions strictly:
- **Patch (0.1.x):** Bug fixes, documentation, performance improvements. No public API changes.
- **Minor (0.x.0):** New public methods, new types, new feature flags. All existing code continues to compile.
- **Major (x.0.0):** Breaking changes to public API. Delayed as long as possible via `#[non_exhaustive]`.

`cargo-semver-checks` runs in CI to catch accidental breaking changes before they are published.

### 10.3 MSRV Policy

Minimum Supported Rust Version: **1.75** (or latest stable minus 2 releases at time of publish). Declared in `Cargo.toml` via `rust-version` field. Tested in CI.

### 10.4 Cross-Platform CI Matrix

| Platform | Tier | Notes |
|----------|------|-------|
| Linux x86_64 | 1 | Primary development and test target |
| macOS aarch64 | 1 | Apple Silicon, primary dev machine |
| macOS x86_64 | 1 | Intel Mac compatibility |
| Windows x86_64 | 1 | rusqlite bundled SQLite handles compilation |
| Linux aarch64 | 2 | ARM servers, Raspberry Pi |
| wasm32-unknown-unknown | 3 | Experimental, depends on rusqlite WASM support |

### 10.5 Documentation Build

All public items have doc comments. All public methods have compilable doctests (gap identified in Phase 5d -- to be filled before v0.1 publish). Documentation is built and hosted via `docs.rs` automatically on publish.

---

## 11. Phase 2 Extensions

These are separate crates that depend on `alaya` but ship independently.

### 11.1 MCP Server (`alaya-mcp`)

A Model Context Protocol server that wraps `AlayaStore` and exposes it as MCP tools:
- `store_memory` -- stores an episode
- `query_memory` -- runs the hybrid retrieval pipeline
- `get_preferences` -- returns crystallized preferences
- `memory_status` -- returns store statistics

The MCP server lives in a separate crate (`alaya-mcp`) and adds its own dependencies (MCP SDK, async runtime). The core `alaya` crate does not know about MCP.

### 11.2 C FFI Layer (`alaya-ffi`)

A C-compatible API generated via `cbindgen`:
- `alaya_open(path) -> *mut AlayaStore`
- `alaya_store_episode(store, json) -> i64`
- `alaya_query(store, json) -> *mut char`
- `alaya_free(store)`

This enables embedding Alaya in any language with C FFI: Swift, Kotlin, Go, Ruby, Elixir, Zig, etc.

### 11.3 Python Bindings (`alaya-py`)

PyO3-based Python bindings:
```python
from alaya import AlayaStore, Query
store = AlayaStore.open("memory.db")
store.store_episode(content="Hello", role="user", session_id="s1")
results = store.query("Hello")
```

### 11.4 Benchmarking Harness

Using `divan` for attribute-based benchmarks:
- `bench_store_episode` -- write throughput
- `bench_query_bm25` -- BM25-only retrieval latency
- `bench_query_hybrid` -- full pipeline latency
- `bench_consolidation` -- lifecycle processing cost
- `bench_activation_spread` -- graph traversal scaling

Benchmarks are run against the LoCoMo dataset for standardized comparison with competitors.

---

## 12. Known Gaps and Risks

Identified from research (Phase 1) and gap analysis (Phase 5d). These are not aspirational TODOs -- they are concrete defects that must be addressed before v0.1 publish.

### 12.1 Missing Implementations

| Gap | Impact | Priority | Status |
|-----|--------|----------|--------|
| LTD (Long-Term Depression) on graph links | Links only strengthen, never weaken from disuse during retrieval | High | `decay_links()` exists but is not called from retrieval pipeline |
| `BEGIN IMMEDIATE` for write transactions | Potential SQLITE_BUSY under concurrent access | High | Not yet implemented in store modules |
| `#[non_exhaustive]` on public enums | Adding enum variants is a breaking change | Medium | Missing on `Role`, `SemanticType`, `LinkType`, `PurgeFilter`, `AlayaError` |
| Input validation at API boundary | Empty strings, negative timestamps, malformed data | Medium | No validation in `store_episode()`, `store_impression()`, etc. |
| Compilable doctests on public methods | Documentation examples may be wrong | Medium | Most public methods lack doctests |
| Memory resurrection after deletion | Deleted nodes can be re-referenced by stale links | Low | No tombstone mechanism yet |
| WAL checkpoint management | Unbounded WAL growth under sustained writes | Low | No `journal_size_limit` or periodic checkpoint |

### 12.2 Architectural Risks

| Risk | Mitigation |
|------|-----------|
| Brute-force vector search does not scale beyond ~10K embeddings | Planned sqlite-vec feature flag for v0.2; trait-based HNSW escape hatch |
| FTS5 external content tables require trigger maintenance | Already implemented; tested in `schema.rs` |
| No concurrent write safety (no `BEGIN IMMEDIATE`) | Must be implemented before v0.1 publish |
| Graph traversal is O(edges) per hop, iterative not SQL-CTE | Acceptable for <100K links; recursive CTE alternative ready if needed |
| Single `Connection` is not `Sync` | Documented: caller must wrap in `Mutex` for multi-thread. `async` feature flag planned for v0.2 |

---

## Appendix A: Research References

| Concept | Reference | Where Used |
|---------|-----------|------------|
| Spreading Activation | Collins & Loftus (1975) | `graph/activation.rs` |
| Complementary Learning Systems | McClelland, McNaughton & O'Reilly (1995) | `lifecycle/consolidation.rs` |
| Bjork Dual-Strength Model | Bjork & Bjork (1992) | `lifecycle/forgetting.rs`, `store/strengths.rs` |
| Reciprocal Rank Fusion | Cormack, Clarke & Buettcher (2009) | `retrieval/fusion.rs` |
| Hebbian Learning (LTP) | Hebb (1949) | `graph/links.rs` on_co_retrieval |
| Vasana (Perfuming) | Yogacara Buddhist psychology | `lifecycle/perfuming.rs` |
| Asraya-paravrtti (Basis Transformation) | Yogacara Buddhist psychology | `lifecycle/transformation.rs` |
| Retrieval-Induced Forgetting | Anderson, Bjork & Bjork (1994) | `retrieval/pipeline.rs` post-retrieval updates |

## Appendix B: Glossary

| Term | Definition |
|------|-----------|
| Episode | A single conversation turn with content, role, session, timestamp, and context |
| Semantic Node | A knowledge unit extracted from episodes via consolidation (fact, relationship, event, concept) |
| Impression | A raw behavioral trace from a single interaction, stored in a domain |
| Preference | A crystallized behavioral pattern that emerged from accumulated impressions in a domain |
| Link | A weighted directed edge in the graph overlay connecting any two nodes |
| NodeRef | A polymorphic reference to an episode, semantic node, or preference |
| NodeStrength | Bjork dual-strength tracking: storage strength (how learned) + retrieval strength (how accessible) |
| Consolidation | CLS-inspired transfer from episodic to semantic store via provider |
| Perfuming | Vasana-inspired accumulation of impressions and crystallization of preferences |
| Transformation | Periodic refinement: dedup, prune, decay toward clarity |
| Forgetting | Bjork-based decay of retrieval strength and archival of deeply forgotten nodes |
| RRF | Reciprocal Rank Fusion -- rank-based merging of multiple result sets |
| LTP | Long-Term Potentiation -- strengthening of graph edges through co-activation |
| LTD | Long-Term Depression -- weakening of graph edges through disuse |
