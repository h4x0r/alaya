# Architecture Research: Alaya Memory Engine

## Generated
2026-02-26T12:50:00+08:00

## 1. Three-Tier Store Architecture

### Episodic Store (Hippocampus)
- Fast encoding of raw conversation events
- SQLite with FTS5 for full-text indexing
- Session-scoped with timestamp indexing
- Append-mostly pattern (immutable after creation)

### Semantic Store (Neocortex)
- Distilled knowledge from consolidation
- Typed nodes (Fact, Belief, Entity, etc.)
- Confidence scoring with corroboration tracking
- Links to source episodes for provenance

### Implicit Store (Alaya-vijnana)
- Two-phase: raw impressions -> crystallized preferences
- Domain-scoped observations with valence
- Evidence-count-based confidence
- Novel contribution: no other system models this as a first-class store

## 2. Hebbian Graph Overlay

### LTP (Long-Term Potentiation)
- Strengthen links on co-retrieval and co-activation
- Formula: `w_new = w_old + lr * (1 - w_old)` (asymptotic approach to 1.0)
- Learning rate should be configurable (default ~0.1)

### LTD (Long-Term Depression)
- Weaken links through disuse -- CRITICAL: currently missing from implementation
- Apply multiplicative decay: `w_new = w_old * (1 - decay_rate)`
- Decay on each lifecycle cycle, not on every query
- Recommendation: add LTD to Hebbian updates

### Small-World Topology
- Emerges naturally from use-dependent LTP/LTD
- High clustering coefficient + short path lengths
- Benefits spreading activation by creating efficient retrieval paths

## 3. Spreading Activation

### Collins & Loftus (1975) Model
- Activation spreads from seed nodes through weighted edges
- Decay per hop (typically 0.5-0.7 factor)
- Threshold cutoff prevents combinatorial explosion
- Maximum depth 2-3 hops is sufficient for most queries

### Implementation via Recursive CTEs
- Efficient in SQLite for graphs up to ~100K edges
- Cycle prevention essential
- Aggregate by MAX(activation) when multiple paths reach same node

## 4. Retrieval Pipeline (RRF)

### Reciprocal Rank Fusion
- `score(d) = sum(1 / (k + rank_i(d)))` where k=60 is standard
- Merge BM25, vector similarity, and graph activation rankings
- Robust to scale differences between scoring methods
- No hyperparameter tuning needed beyond k

### Context-Weighted Reranking
- Boost results matching current session context
- Temporal recency as a secondary signal
- Working memory limits: return 3-5 results (Cowan's 4 +/- 1)

## 5. Bjork Dual-Strength Forgetting

### Storage Strength (SS)
- Increases monotonically with each encoding/reinforcement
- Never decreases (represents the "depth" of memory trace)
- Formula: `SS_new = SS_old + (1 - SS_old) * increment`

### Retrieval Strength (RS)
- Decays with time since last access
- Rate of decay inversely proportional to SS (well-encoded memories decay slower)
- Formula: `RS(t) = RS_0 * exp(-decay * t / SS)`
- FSRS (Free Spaced Repetition Scheduler) provides validated reference implementation

### Retrieval-Induced Forgetting (RIF)
- Retrieving memories suppresses competitors
- Apply small RS penalty to non-retrieved results that were candidates
- Improves retrieval quality over time by increasing differentiation

## 6. CLS Consolidation

### Triggers
- Episode count in uncondensed batch exceeds threshold (e.g., 20)
- Time since last consolidation exceeds threshold
- Consumer explicitly calls `consolidate()`

### Process
1. Select batch of unconsolidated episodes
2. Pass to `ConsolidationProvider::extract_knowledge()`
3. Store resulting semantic nodes
4. Create provenance links (episode -> semantic node)
5. Mark episodes as consolidated (but don't delete)
6. Strengthen graph links between co-occurring entities

## 7. Preference Emergence (Vasana/Perfuming)

### Two-Phase Model
1. **Impression accumulation:** Raw behavioral observations stored with domain, valence, timestamp
2. **Crystallization:** When sufficient observations accumulate, extract preference patterns

### PreferenceTension (New Design)
- Captures observed conflicts between values
- Tracks resolution patterns with context, win ratio, confidence, trend
- Enables `resolve_tradeoff()` and `preference_ranking()` queries
- Genuinely novel: no other system models preference tradeoffs

## 8. Trait-Based Plugin Architecture

### ConsolidationProvider
- `extract_knowledge()`: episodes -> semantic nodes
- `extract_impressions()`: interaction -> behavioral observations
- `detect_contradiction()`: identify conflicting knowledge

### EmbeddingProvider (Recommended Addition)
- `embed()`: text -> vector
- `embed_batch()`: batch encoding
- `dimensions()`: vector dimensionality
- `model_id()`: tracking which model generated embeddings

## 9. Scalability Considerations

### Vector Search
- Brute-force viable up to ~10K vectors (perfect recall)
- 10K-50K: consider sqlite-vec for SIMD acceleration
- 50K+: HNSW or similar approximate index needed
- SIMD optimization via SimSIMD for hot path

### Graph Scale
- Adjacency list with recursive CTEs viable to ~100K edges
- Depth-limited traversal (2-3 hops) prevents explosion
- Edge weight pruning prevents unbounded growth

### SQLite Limits
- WAL mode handles concurrent reads well
- Single writer is sufficient for embedded use
- `PRAGMA journal_size_limit` to bound WAL growth
- Periodic checkpoints via `PRAGMA wal_checkpoint(PASSIVE)`

## Sources

- Collins & Loftus (1975) - Spreading Activation Theory
- McClelland et al. (1995) - Complementary Learning Systems
- Bjork & Bjork (1992) - Dual-Strength Model
- Anderson et al. (1994) - Retrieval-Induced Forgetting
- Cowan (2001) - Working Memory Limits
- Cormack et al. (2009) - Reciprocal Rank Fusion
- FSRS (Free Spaced Repetition Scheduler)
- SQLite FTS5 Documentation
- sqlite-vec GitHub
- SimSIMD GitHub
