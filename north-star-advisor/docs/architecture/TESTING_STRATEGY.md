# Alaya Testing Strategy

> Testing philosophy, coverage plan, and quality infrastructure for an embeddable Rust memory library.

**Version:** 0.1.0
**Generated:** 2026-02-26
**Status:** Active

---

## Table of Contents

1. [Testing Philosophy for Rust Libraries](#1-testing-philosophy-for-rust-libraries)
2. [Test Categories and Pyramid](#2-test-categories-and-pyramid)
3. [Unit Test Coverage Map](#3-unit-test-coverage-map)
4. [Integration Test Suites](#4-integration-test-suites)
5. [Property-Based Testing](#5-property-based-testing)
6. [Retrieval Quality Benchmarks](#6-retrieval-quality-benchmarks)
7. [Lifecycle Invariant Tests](#7-lifecycle-invariant-tests)
8. [Test Fixtures and Utilities](#8-test-fixtures-and-utilities)
9. [Performance Benchmarks](#9-performance-benchmarks)
10. [CI/CD Test Pipeline](#10-cicd-test-pipeline)
11. [Fuzzing Strategy](#11-fuzzing-strategy)
12. [Doc Test Requirements](#12-doc-test-requirements)

---

## 1. Testing Philosophy for Rust Libraries

### 1.1 Core Principles

Alaya is a Rust library, not a web application. Its testing strategy must reflect the realities of embedded, single-process, zero-network operation. Every design decision in the test architecture flows from these principles:

**Correctness > Speed.** Alaya implements research-grounded algorithms (Bjork dual-strength forgetting, Hebbian LTP/LTD, CLS consolidation, vasana preference emergence, RRF fusion). Mathematical properties of these algorithms are non-negotiable. A Bjork decay function that produces negative strengths is a bug regardless of how fast it runs. Property-based testing enforces these invariants at scale.

**Determinism by Default.** Tests must be reproducible. The codebase currently uses `SystemTime::now()` for timestamps in several store and lifecycle modules (`store/episodic.rs`, `store/semantic.rs`, `store/implicit.rs`, `store/strengths.rs`, `lifecycle/transformation.rs`, `lifecycle/forgetting.rs`). All test code must pass explicit timestamps or use controlled time to avoid flaky results. The existing retrieval pipeline test at `retrieval/pipeline.rs:123` correctly injects `current_timestamp: Some(3000)` in the `QueryContext` -- this pattern should be universal.

**Isolation Without Overhead.** SQLite in-memory databases (`AlayaStore::open_in_memory()` / `schema::open_memory_db()`) provide perfect test isolation with zero filesystem side effects. Each `#[test]` function gets its own database. This is already the pattern in all 18 test modules and must remain the standard. No test should ever write to the filesystem unless explicitly testing persistent-file behavior.

**The Library is the API Boundary.** Alaya has no HTTP endpoints, no WebSocket handlers, no CLI to test. The public API is `AlayaStore` and its 13 public methods, the `ConsolidationProvider` trait, and the `NoOpProvider` default. Tests operate at two levels: internal module functions (unit) and `AlayaStore` public methods (integration). There is no E2E/browser/API layer.

### 1.2 Current State Assessment

The codebase currently contains **43 unit tests** across **18 source files**. Every module with logic has at least a `#[cfg(test)] mod tests` block. This is a healthy starting point but far from comprehensive:

| Module | Current Tests | Gaps |
|--------|--------------|------|
| `schema.rs` | 3 (table existence, FTS5 triggers, idempotent init) | WAL mode verification, pragma verification, concurrent init |
| `store/episodic.rs` | 3 (CRUD, session query, count/delete) | Unicode content, large content, context serialization, timestamp ordering |
| `store/semantic.rs` | 2 (CRUD, corroboration) | Type filtering, confidence ordering, deletion cascade, source episode refs |
| `store/implicit.rs` | 2 (impressions CRUD, preferences CRUD) | Domain filtering, decay, reinforcement bounds, crystallization edge cases |
| `store/embeddings.rs` | 4 (serialize roundtrip, cosine identity, orthogonal, search) | Dimension mismatch, zero vectors, high-dimensionality, NaN handling |
| `store/strengths.rs` | 2 (init/access, suppress/decay) | Strength bounds [0,1], archivable query, concurrent access patterns |
| `retrieval/bm25.rs` | 2 (search, empty query) | Special characters, FTS5 injection, porter stemmer, multi-word, ranking |
| `retrieval/fusion.rs` | 3 (single set, overlap, disjoint) | Empty sets, single-item sets, many sets, k-parameter sensitivity |
| `retrieval/rerank.rs` | 3 (recency recent, recency old, jaccard) | Full rerank pipeline, context similarity, edge timestamps |
| `retrieval/vector.rs` | 1 (empty search) | Non-empty search (delegated to embeddings), mixed types |
| `retrieval/pipeline.rs` | 2 (basic query, empty query) | Multi-signal fusion, vector+BM25, post-retrieval updates, graceful degradation |
| `graph/links.rs` | 3 (create/query, co-retrieval, prune) | Duplicate link handling, bidirectional queries, decay, link types |
| `graph/activation.rs` | 3 (single hop, multi-hop, threshold) | Activation cap, cyclic graphs, empty graph, many seeds |
| `lifecycle/consolidation.rs` | 2 (below threshold, creates nodes) | Multiple batches, provider errors, link creation verification |
| `lifecycle/perfuming.rs` | 2 (store impressions, crystallization) | Reinforcement path, multiple domains, empty provider |
| `lifecycle/transformation.rs` | 2 (empty DB, prune links) | Dedup with embeddings, preference decay, impression pruning |
| `lifecycle/forgetting.rs` | 2 (empty DB, decay) | Archival, multi-cycle decay, strength floor, interaction with retrieval |
| `lib.rs` | 2 (full lifecycle, purge all) | Purge by session, purge by timestamp, knowledge filtering, neighbors |
| `provider.rs` | 0 (MockProvider defined only) | NoOpProvider behavior verification, error-returning provider |

**Total test count:** 43 in-module tests, 0 integration tests in `tests/`, 0 benchmarks, 0 doc tests, 0 property tests.

**Dev-dependencies:** Currently empty (`[dev-dependencies]` in `Cargo.toml` has no entries). This means no test framework beyond the standard library, no property testing, no benchmarking crate.

### 1.3 Testing Strategy Axiom Alignment

Each testing strategy decision maps to a project axiom:

| Axiom | Testing Implication |
|-------|-------------------|
| Privacy > Features | Verify zero network calls: `cargo tree` must show no HTTP/DNS/socket dependencies. CI enforces. |
| Process > Storage | Lifecycle processes are the product -- they need the most thorough testing. Property tests for every mathematical model. |
| Correctness > Speed | Bjork decay, Hebbian LTP, RRF fusion, cosine similarity all have mathematical invariants. Prove them with property tests. |
| Simplicity > Completeness | Test infrastructure should be simple. In-memory SQLite, no test containers, no mock frameworks, no external services. |
| Honesty > Marketing | Publish retrieval quality benchmarks even when numbers are bad. Track regression over time. |

---

## 2. Test Categories and Pyramid

### 2.1 The Rust Test Pyramid

For a Rust library, the test pyramid has different layers than a web application:

```
                    /\
                   /  \      Doc Tests (compilable examples on every pub method)
                  /----\
                 / Fuzz \    Fuzzing (query parsing, FTS5 input, BLOB handling)
                /--------\
               / Property  \  Property Tests (mathematical invariants)
              /------------\
             / Integration   \  Full lifecycle flows (tests/ directory)
            /------------------\
           /    Module Unit     \  In-module #[cfg(test)] tests
          /______________________\
```

**Layer 1: Module Unit Tests (in-module, `#[cfg(test)]`)**

These are the existing 43 tests. Each module tests its own functions in isolation using `open_memory_db()`. They validate individual CRUD operations, single algorithm steps, and basic error paths. Target: 200+ tests covering every public and significant private function.

**Layer 2: Integration Tests (`tests/` directory)**

Top-level crate integration tests that exercise `AlayaStore` public methods through realistic multi-step scenarios. These test the full pipeline: store episodes, query, run lifecycle processes, query again. They verify cross-module interactions that in-module tests cannot reach. Target: 30+ integration tests organized by scenario.

**Layer 3: Property-Based Tests (proptest/quickcheck)**

Automatically generated inputs testing mathematical invariants of core algorithms. These catch edge cases that hand-written tests miss. Priority targets: Bjork decay monotonicity, Hebbian weight bounds, RRF ordering stability, cosine similarity properties, embedding serialization roundtrip. Target: 25+ property tests.

**Layer 4: Fuzzing (`cargo-fuzz`)**

Long-running, coverage-guided input generation for attack surfaces. Priority targets: FTS5 query sanitization, embedding BLOB deserialization, JSON context deserialization, SQL parameter construction. Target: 5+ fuzz targets.

**Layer 5: Doc Tests**

Compilable code examples in `///` doc comments on every public type and method. These serve triple duty: documentation, compilation verification, and basic behavior assertion. Currently zero doc tests exist. Target: 100% coverage of all 13 `AlayaStore` public methods plus all public types.

### 2.2 Target Test Counts by Phase

| Category | v0.1 Target | v0.2 Target | v0.3 Target |
|----------|-------------|-------------|-------------|
| Module Unit Tests | 150+ | 250+ | 350+ |
| Integration Tests | 20+ | 40+ | 60+ |
| Property Tests | 15+ | 30+ | 40+ |
| Fuzz Targets | 3+ | 6+ | 8+ |
| Doc Tests | 30+ | 50+ | 70+ |
| Benchmarks | 5+ | 15+ | 20+ |

---

## 3. Unit Test Coverage Map

### 3.1 `schema.rs` -- Database Initialization

Current tests: `test_open_memory_db`, `test_fts5_trigger_sync`, `test_idempotent_init`.

Additional required tests:

```rust
// Verify WAL journal mode is set correctly
#[test]
fn test_wal_mode_enabled() {
    let conn = open_memory_db().unwrap();
    let mode: String = conn.query_row(
        "PRAGMA journal_mode", [], |row| row.get(0)
    ).unwrap();
    // In-memory databases report "memory", file databases report "wal"
    // This test validates the pragma was issued without error
    assert!(mode == "memory" || mode == "wal");
}

// Verify foreign keys are enabled
#[test]
fn test_foreign_keys_enabled() {
    let conn = open_memory_db().unwrap();
    let fk: i64 = conn.query_row(
        "PRAGMA foreign_keys", [], |row| row.get(0)
    ).unwrap();
    assert_eq!(fk, 1);
}

// Verify all expected indexes exist
#[test]
fn test_indexes_created() { /* ... */ }

// Verify FTS5 update trigger fires on content change
#[test]
fn test_fts5_update_trigger() { /* ... */ }

// Verify table count matches expected (7 core + 1 FTS5)
#[test]
fn test_table_count() { /* ... */ }
```

### 3.2 `store/episodic.rs` -- Episodic Store

Current tests: `test_store_and_get`, `test_get_by_session`, `test_count_and_delete`.

Additional required tests:

```rust
// Unicode content preservation
#[test]
fn test_unicode_content() {
    let conn = open_memory_db().unwrap();
    let id = store_episode(&conn, &make_episode("Hello in Japanese", 1000));
    // Content with CJK, emoji, diacritics roundtrips correctly
}

// EpisodeContext serialization roundtrip
#[test]
fn test_context_serialization() {
    let conn = open_memory_db().unwrap();
    let ep = NewEpisode {
        context: EpisodeContext {
            topics: vec!["rust".into(), "async".into()],
            sentiment: 0.8,
            conversation_turn: 5,
            mentioned_entities: vec!["tokio".into()],
            preceding_episode: Some(EpisodeId(42)),
        },
        ..make_episode("test", 1000)
    };
    let id = store_episode(&conn, &ep).unwrap();
    let retrieved = get_episode(&conn, id).unwrap();
    assert_eq!(retrieved.context.topics, vec!["rust", "async"]);
    assert_eq!(retrieved.context.preceding_episode, Some(EpisodeId(42)));
}

// get_episode returns NotFound for nonexistent ID
#[test]
fn test_get_nonexistent_episode() {
    let conn = open_memory_db().unwrap();
    let err = get_episode(&conn, EpisodeId(999)).unwrap_err();
    assert!(matches!(err, AlayaError::NotFound(_)));
}

// Timestamp ordering in get_recent_episodes
#[test]
fn test_recent_episodes_ordering() { /* ... */ }

// get_unconsolidated_episodes excludes linked episodes
#[test]
fn test_unconsolidated_excludes_linked() { /* ... */ }

// delete_episodes with empty slice returns 0
#[test]
fn test_delete_empty_slice() { /* ... */ }

// Large batch insert (100+ episodes) succeeds
#[test]
fn test_bulk_insert() { /* ... */ }

// Different sessions are isolated
#[test]
fn test_session_isolation() { /* ... */ }
```

### 3.3 `store/semantic.rs` -- Semantic Store

Current tests: `test_store_and_get`, `test_corroboration`.

Additional required tests:

```rust
// find_by_type returns only matching type
#[test]
fn test_find_by_type_filtering() { /* ... */ }

// find_by_type orders by confidence descending
#[test]
fn test_find_by_type_confidence_ordering() { /* ... */ }

// delete_node cascades to embeddings, links, and strengths
#[test]
fn test_delete_node_cascade() { /* ... */ }

// source_episodes_json roundtrips correctly with multiple IDs
#[test]
fn test_source_episodes_roundtrip() { /* ... */ }

// update_corroboration on nonexistent node returns NotFound
#[test]
fn test_corroboration_nonexistent() { /* ... */ }

// Embedding stored alongside semantic node via store_semantic_node
#[test]
fn test_semantic_node_with_embedding() { /* ... */ }
```

### 3.4 `store/implicit.rs` -- Impressions and Preferences

Current tests: `test_impressions_crud`, `test_preferences_crud`.

Additional required tests:

```rust
// get_preferences with None returns all domains
#[test]
fn test_get_all_preferences() { /* ... */ }

// decay_preferences only affects stale preferences
#[test]
fn test_decay_targets_stale_only() { /* ... */ }

// reinforce_preference caps confidence at 1.0
#[test]
fn test_reinforce_caps_confidence() { /* ... */ }

// prune_weak_preferences removes below threshold
#[test]
fn test_prune_weak_preferences() { /* ... */ }

// prune_old_impressions respects age cutoff
#[test]
fn test_prune_old_impressions() { /* ... */ }

// Multiple domains coexist independently
#[test]
fn test_multi_domain_isolation() { /* ... */ }

// count_impressions_by_domain is accurate
#[test]
fn test_impression_count_accuracy() { /* ... */ }
```

### 3.5 `store/embeddings.rs` -- Embedding Storage and Search

Current tests: `test_serialize_roundtrip`, `test_cosine_similarity_identical`, `test_cosine_similarity_orthogonal`, `test_store_and_search`.

Additional required tests:

```rust
// Dimension mismatch returns 0.0 (not panic)
#[test]
fn test_cosine_dimension_mismatch() {
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![1.0, 0.0];
    assert_eq!(cosine_similarity(&a, &b), 0.0);
}

// Zero vector returns 0.0 (not NaN or panic)
#[test]
fn test_cosine_zero_vector() {
    let a = vec![0.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    assert_eq!(cosine_similarity(&a, &b), 0.0);
}

// Empty vectors return 0.0
#[test]
fn test_cosine_empty_vectors() {
    assert_eq!(cosine_similarity(&[], &[]), 0.0);
}

// Anti-parallel vectors return -1.0 (approximately)
#[test]
fn test_cosine_antiparallel() {
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![-1.0, 0.0, 0.0];
    assert!((cosine_similarity(&a, &b) - (-1.0)).abs() < 1e-6);
}

// High-dimensional embeddings (384, 768, 1536 dims) serialize correctly
#[test]
fn test_high_dimensional_roundtrip() { /* ... */ }

// search_by_vector with node_type_filter restricts correctly
#[test]
fn test_vector_search_with_filter() { /* ... */ }

// get_unembedded_episodes identifies correct set
#[test]
fn test_unembedded_episodes() { /* ... */ }

// INSERT OR REPLACE updates existing embedding
#[test]
fn test_embedding_upsert() { /* ... */ }

// NaN in embeddings does not poison search
#[test]
fn test_nan_in_embedding() { /* ... */ }

// Subnormal floats survive serialization
#[test]
fn test_subnormal_float_roundtrip() { /* ... */ }
```

### 3.6 `store/strengths.rs` -- Bjork Dual-Strength Model

Current tests: `test_init_and_access`, `test_suppress_and_decay`.

Additional required tests:

```rust
// Storage strength never exceeds 1.0 after many accesses
#[test]
fn test_storage_strength_upper_bound() {
    let conn = open_memory_db().unwrap();
    let node = NodeRef::Episode(EpisodeId(1));
    init_strength(&conn, node).unwrap();
    for _ in 0..1000 {
        on_access(&conn, node).unwrap();
    }
    let s = get_strength(&conn, node).unwrap();
    assert!(s.storage_strength <= 1.0);
    assert!(s.storage_strength > 0.99); // Should asymptotically approach 1.0
}

// Retrieval strength resets to 1.0 on access
#[test]
fn test_retrieval_strength_reset_on_access() { /* ... */ }

// find_archivable returns nodes below both thresholds
#[test]
fn test_find_archivable() { /* ... */ }

// find_archivable excludes nodes above either threshold
#[test]
fn test_find_archivable_excludes_strong() { /* ... */ }

// get_strength returns default for untracked node
#[test]
fn test_get_strength_untracked() { /* ... */ }

// init_strength is idempotent (INSERT OR IGNORE)
#[test]
fn test_init_strength_idempotent() { /* ... */ }

// boost_retrieval caps at 1.0
#[test]
fn test_boost_retrieval_cap() { /* ... */ }

// decay_all_retrieval skips nodes below 0.01
#[test]
fn test_decay_floor() { /* ... */ }
```

### 3.7 `retrieval/bm25.rs` -- FTS5 BM25 Search

Current tests: `test_bm25_search`, `test_empty_query`.

Additional required tests:

```rust
// Special characters are sanitized (no FTS5 injection)
#[test]
fn test_fts5_injection_prevention() {
    let conn = open_memory_db().unwrap();
    episodic::store_episode(&conn, &make_episode("normal content", 1000)).unwrap();
    // These should not crash or produce unexpected results
    let r1 = search_bm25(&conn, "OR NOT AND NEAR", 10).unwrap();
    let r2 = search_bm25(&conn, "content*", 10).unwrap();
    let r3 = search_bm25(&conn, "\"quoted phrase\"", 10).unwrap();
    let r4 = search_bm25(&conn, "col:value", 10).unwrap();
    let r5 = search_bm25(&conn, "{braces} [brackets]", 10).unwrap();
    // None should error; results may be empty or contain matches
}

// Porter stemmer matches morphological variants
#[test]
fn test_stemming() {
    // "programming" should match "program", "programs", etc. via porter stemmer
}

// Whitespace-only query returns empty
#[test]
fn test_whitespace_query() { /* ... */ }

// Multi-word queries match correctly
#[test]
fn test_multi_word_query() { /* ... */ }

// BM25 score normalization produces [0, 1] range
#[test]
fn test_score_normalization_range() { /* ... */ }

// Single result gets score 1.0 (normalization with range=0)
#[test]
fn test_single_result_score() { /* ... */ }
```

### 3.8 `retrieval/fusion.rs` -- Reciprocal Rank Fusion

Current tests: `test_rrf_single_set`, `test_rrf_two_sets_overlap`, `test_rrf_disjoint`.

Additional required tests:

```rust
// Empty input returns empty output
#[test]
fn test_rrf_empty_input() {
    let merged = rrf_merge(&[], 60);
    assert!(merged.is_empty());
}

// Single empty set returns empty
#[test]
fn test_rrf_single_empty_set() {
    let merged = rrf_merge(&[vec![]], 60);
    assert!(merged.is_empty());
}

// Document present in all sets ranks highest
#[test]
fn test_rrf_all_sets_presence() { /* ... */ }

// k parameter affects score magnitudes but not ordering
#[test]
fn test_rrf_k_parameter_ordering_invariance() { /* ... */ }

// Large number of sets (10+) works correctly
#[test]
fn test_rrf_many_sets() { /* ... */ }

// Output is deterministically sorted by score descending
#[test]
fn test_rrf_output_sorted() { /* ... */ }
```

### 3.9 `retrieval/rerank.rs` -- Context-Aware Reranking

Current tests: `test_recency_recent`, `test_recency_old`, `test_jaccard`.

Additional required tests:

```rust
// Full rerank pipeline produces correct ordering
#[test]
fn test_rerank_full_pipeline() { /* ... */ }

// max_results truncation works
#[test]
fn test_rerank_truncation() { /* ... */ }

// Future timestamps (now < timestamp) produce recency >= 1.0
#[test]
fn test_recency_future_timestamp() { /* ... */ }

// Empty context similarity returns neutral score
#[test]
fn test_empty_context_similarity() { /* ... */ }

// Jaccard with identical sets returns 1.0
#[test]
fn test_jaccard_identical() { /* ... */ }

// Jaccard with disjoint sets returns 0.0
#[test]
fn test_jaccard_disjoint() { /* ... */ }

// Jaccard with both empty returns 0.0 (current implementation)
#[test]
fn test_jaccard_both_empty() { /* ... */ }
```

### 3.10 `retrieval/pipeline.rs` -- Full Retrieval Pipeline

Current tests: `test_basic_query`, `test_empty_query`.

Additional required tests:

```rust
// Graceful degradation: BM25-only (no embeddings, no links)
#[test]
fn test_degradation_bm25_only() { /* ... */ }

// Graceful degradation: vector-only (query has embedding, FTS5 returns empty)
#[test]
fn test_degradation_vector_only() { /* ... */ }

// Post-retrieval strength updates fire
#[test]
fn test_post_retrieval_strength_update() {
    let conn = open_memory_db().unwrap();
    let id = episodic::store_episode(&conn, &make_episode("Rust memory", 1000)).unwrap();
    strengths::init_strength(&conn, NodeRef::Episode(id)).unwrap();
    let before = strengths::get_strength(&conn, NodeRef::Episode(id)).unwrap();

    execute_query(&conn, &Query::simple("Rust memory")).unwrap();

    let after = strengths::get_strength(&conn, NodeRef::Episode(id)).unwrap();
    assert!(after.access_count > before.access_count);
}

// Co-retrieval creates Hebbian links between results
#[test]
fn test_post_retrieval_co_retrieval_links() { /* ... */ }

// Empty database returns empty results (not error)
#[test]
fn test_empty_database() { /* ... */ }

// Query with both text and embedding uses both signals
#[test]
fn test_hybrid_query() { /* ... */ }
```

### 3.11 `graph/links.rs` -- Hebbian Link Management

Current tests: `test_create_and_query_links`, `test_co_retrieval_strengthening`, `test_prune_weak`.

Additional required tests:

```rust
// Duplicate link creation is idempotent (INSERT OR IGNORE)
#[test]
fn test_duplicate_link_idempotent() { /* ... */ }

// Bidirectional query: get_links_to finds reverse links
#[test]
fn test_bidirectional_query() { /* ... */ }

// on_co_retrieval creates link if none exists
#[test]
fn test_co_retrieval_creates_link() { /* ... */ }

// decay_links reduces weights
#[test]
fn test_decay_links() { /* ... */ }

// Different link types coexist between same node pair
#[test]
fn test_multiple_link_types() { /* ... */ }

// activation_count increments correctly
#[test]
fn test_activation_count() { /* ... */ }
```

### 3.12 `graph/activation.rs` -- Spreading Activation

Current tests: `test_single_hop_spread`, `test_multi_hop_decay`, `test_threshold_cutoff`.

Additional required tests:

```rust
// Cyclic graph does not cause infinite loop
#[test]
fn test_cyclic_graph() {
    let conn = open_memory_db().unwrap();
    let a = NodeRef::Episode(EpisodeId(1));
    let b = NodeRef::Episode(EpisodeId(2));
    create_link(&conn, a, b, LinkType::Topical, 0.8).unwrap();
    create_link(&conn, b, a, LinkType::Topical, 0.8).unwrap();
    // Should terminate and not panic
    let result = spread_activation(&conn, &[a], 3, 0.05, 0.6).unwrap();
    assert!(result.contains_key(&a));
    assert!(result.contains_key(&b));
}

// Activation is capped at 2.0 (prevent runaway)
#[test]
fn test_activation_cap() { /* ... */ }

// Empty graph returns only seed activations
#[test]
fn test_empty_graph() { /* ... */ }

// Multiple seeds combine activation
#[test]
fn test_multiple_seeds() { /* ... */ }

// max_depth=0 returns only seeds
#[test]
fn test_zero_depth() { /* ... */ }
```

### 3.13 `lifecycle/consolidation.rs` -- CLS Consolidation

Current tests: `test_consolidation_below_threshold`, `test_consolidation_creates_nodes`.

Additional required tests:

```rust
// NoOpProvider produces empty report (no nodes created)
#[test]
fn test_consolidation_with_noop() { /* ... */ }

// Provider error propagates correctly
#[test]
fn test_consolidation_provider_error() { /* ... */ }

// Created nodes have links back to source episodes
#[test]
fn test_consolidation_creates_links() { /* ... */ }

// Created nodes have initialized strengths
#[test]
fn test_consolidation_initializes_strengths() { /* ... */ }

// After consolidation, episodes are considered "consolidated" (not re-fetched)
#[test]
fn test_consolidation_marks_episodes() { /* ... */ }

// Exactly threshold (3) episodes triggers consolidation
#[test]
fn test_consolidation_exact_threshold() { /* ... */ }
```

### 3.14 `lifecycle/perfuming.rs` -- Vasana Preference Emergence

Current tests: `test_perfuming_stores_impressions`, `test_crystallization_after_threshold`.

Additional required tests:

```rust
// Reinforcement path: existing preference gets reinforced, not duplicated
#[test]
fn test_perfuming_reinforcement() { /* ... */ }

// Multiple domains crystallize independently
#[test]
fn test_multi_domain_crystallization() { /* ... */ }

// Empty provider returns zero impressions
#[test]
fn test_perfuming_empty_provider() { /* ... */ }

// Crystallized preference has initialized strength
#[test]
fn test_crystallization_initializes_strength() { /* ... */ }

// Exactly threshold (5) impressions triggers crystallization
#[test]
fn test_crystallization_exact_threshold() { /* ... */ }

// Below threshold does not crystallize
#[test]
fn test_below_threshold_no_crystallization() { /* ... */ }
```

### 3.15 `lifecycle/transformation.rs` -- Asraya-Paravrtti

Current tests: `test_transform_empty_db`, `test_transform_prunes_weak_links`.

Additional required tests:

```rust
// Dedup merges near-identical semantic nodes (by embedding similarity)
#[test]
fn test_dedup_merges_similar_nodes() { /* ... */ }

// Dedup preserves distinct nodes (below similarity threshold)
#[test]
fn test_dedup_preserves_distinct() { /* ... */ }

// Preference decay targets only stale preferences
#[test]
fn test_preference_decay_targets_stale() { /* ... */ }

// Impression pruning removes old impressions
#[test]
fn test_impression_pruning() { /* ... */ }

// transform() is idempotent on clean data
#[test]
fn test_transform_idempotent() { /* ... */ }
```

### 3.16 `lifecycle/forgetting.rs` -- Bjork Dual-Strength Forgetting

Current tests: `test_forget_empty_db`, `test_decay_reduces_retrieval_strength`.

Additional required tests:

```rust
// Multiple forget cycles eventually archive weak nodes
#[test]
fn test_multi_cycle_archival() {
    let conn = open_memory_db().unwrap();
    // Store an episode, never access it, call forget() repeatedly
    episodic::store_episode(&conn, &make_episode("forgettable", 1000)).unwrap();
    // Set initial strength very low
    conn.execute(
        "INSERT INTO node_strengths (node_type, node_id, storage_strength, retrieval_strength, access_count, last_accessed)
         VALUES ('episode', 1, 0.08, 0.5, 1, 1000)",
        []
    ).unwrap();

    // Decay until archival
    let mut total_archived = 0;
    for _ in 0..50 {
        let report = forget(&conn).unwrap();
        total_archived += report.nodes_archived;
        if total_archived > 0 { break; }
    }
    assert!(total_archived > 0, "node should eventually be archived");
    assert_eq!(episodic::count_episodes(&conn).unwrap(), 0);
}

// Accessed nodes resist forgetting (high RS after access)
#[test]
fn test_accessed_nodes_resist_forgetting() { /* ... */ }

// Preferences are not archived by forget() (only by transform())
#[test]
fn test_forget_skips_preferences() { /* ... */ }

// Semantic nodes can be archived
#[test]
fn test_forget_archives_semantic_nodes() { /* ... */ }

// Strength records are cleaned up after archival
#[test]
fn test_archival_cleans_strength_records() { /* ... */ }
```

### 3.17 `provider.rs` -- ConsolidationProvider and MockProvider

Current: MockProvider is defined with `#[cfg(test)]` but has no tests of its own.

Required tests:

```rust
// NoOpProvider.extract_knowledge returns empty vec
#[test]
fn test_noop_extract_knowledge() {
    let noop = NoOpProvider;
    let result = noop.extract_knowledge(&[]).unwrap();
    assert!(result.is_empty());
}

// NoOpProvider.extract_impressions returns empty vec
#[test]
fn test_noop_extract_impressions() { /* ... */ }

// NoOpProvider.detect_contradiction returns false
#[test]
fn test_noop_detect_contradiction() { /* ... */ }

// MockProvider returns configured data
#[test]
fn test_mock_provider_returns_data() { /* ... */ }
```

### 3.18 `types.rs` -- Type Definitions

Currently no tests. Required:

```rust
// NodeRef::from_parts roundtrips correctly
#[test]
fn test_noderef_from_parts_roundtrip() {
    let cases = vec![
        ("episode", 1, Some(NodeRef::Episode(EpisodeId(1)))),
        ("semantic", 42, Some(NodeRef::Semantic(NodeId(42)))),
        ("preference", 7, Some(NodeRef::Preference(PreferenceId(7)))),
        ("unknown", 1, None),
    ];
    for (type_str, id, expected) in cases {
        assert_eq!(NodeRef::from_parts(type_str, id), expected);
    }
}

// Role::from_str and as_str roundtrip
#[test]
fn test_role_roundtrip() { /* ... */ }

// SemanticType::from_str and as_str roundtrip
#[test]
fn test_semantic_type_roundtrip() { /* ... */ }

// LinkType::from_str and as_str roundtrip
#[test]
fn test_link_type_roundtrip() { /* ... */ }

// Query::simple sets correct defaults
#[test]
fn test_query_simple_defaults() {
    let q = Query::simple("hello");
    assert_eq!(q.text, "hello");
    assert_eq!(q.max_results, 5);
    assert!(q.embedding.is_none());
}

// EpisodeContext::default produces expected defaults
#[test]
fn test_episode_context_default() { /* ... */ }

// All report types have Default producing zeroes
#[test]
fn test_report_defaults() { /* ... */ }
```

### 3.19 `error.rs` -- Error Types

Currently no tests. Required:

```rust
// AlayaError::Db wraps rusqlite errors
#[test]
fn test_error_db_display() {
    let err = AlayaError::NotFound("episode 42".to_string());
    assert_eq!(err.to_string(), "not found: episode 42");
}

// AlayaError::InvalidInput display
#[test]
fn test_error_invalid_input_display() { /* ... */ }

// AlayaError::Provider display
#[test]
fn test_error_provider_display() { /* ... */ }

// From<rusqlite::Error> conversion
#[test]
fn test_error_from_rusqlite() { /* ... */ }

// From<serde_json::Error> conversion
#[test]
fn test_error_from_serde() { /* ... */ }
```

---

## 4. Integration Test Suites

Integration tests live in `tests/` at the crate root and test `AlayaStore` as a consumer would use it. They import `alaya` as an external crate, meaning they can only access the public API.

### 4.1 Directory Structure

```
tests/
  store_lifecycle.rs     -- Full CRUD + lifecycle flows
  retrieval_quality.rs   -- Query relevance assertions
  degradation_chain.rs   -- Graceful degradation scenarios
  concurrent_access.rs   -- Multi-threaded Arc<Mutex> patterns
  persistence.rs         -- File-backed database operations
  error_paths.rs         -- Every AlayaError variant exercised
  purge_compliance.rs    -- GDPR purge completeness
```

### 4.2 Full Lifecycle Integration Tests

These tests model realistic agent usage patterns: store conversation turns, run lifecycle processes, query for context, and verify the memory system behaves coherently across the entire pipeline.

```rust
// tests/store_lifecycle.rs

use alaya::*;

/// Store 20 episodes across 3 sessions, consolidate, query, forget, query again.
/// Verifies the complete memory lifecycle from an agent's perspective.
#[test]
fn test_full_agent_lifecycle() {
    let store = AlayaStore::open_in_memory().unwrap();

    // Phase 1: Store conversation episodes
    let sessions = ["session-a", "session-b", "session-c"];
    for (i, session) in sessions.iter().enumerate() {
        for turn in 0..7 {
            store.store_episode(&NewEpisode {
                content: format!("User discusses topic {} in turn {}", i, turn),
                role: if turn % 2 == 0 { Role::User } else { Role::Assistant },
                session_id: session.to_string(),
                timestamp: (i * 1000 + turn * 100) as i64,
                context: EpisodeContext::default(),
                embedding: None,
            }).unwrap();
        }
    }

    let status = store.status().unwrap();
    assert_eq!(status.episode_count, 21);

    // Phase 2: Query before consolidation (BM25-only)
    let results = store.query(&Query::simple("topic 0")).unwrap();
    assert!(!results.is_empty());

    // Phase 3: Run lifecycle with NoOp (no LLM)
    let noop = NoOpProvider;
    let cr = store.consolidate(&noop).unwrap();
    // NoOp produces no semantic nodes, but episodes_processed should be > 0
    assert!(cr.episodes_processed >= 3 || cr.episodes_processed == 0);

    // Phase 4: Transform and forget
    let _tr = store.transform().unwrap();
    let _fr = store.forget().unwrap();

    // Phase 5: Query still works after lifecycle
    let results = store.query(&Query::simple("topic")).unwrap();
    // Results may be fewer due to forgetting, but no errors

    // Phase 6: Status reflects changes
    let final_status = store.status().unwrap();
    assert!(final_status.episode_count <= status.episode_count);
}

/// Store -> perfume -> verify preference crystallization end-to-end.
#[test]
fn test_preference_emergence_lifecycle() {
    // Store 6+ interactions with consistent style preference
    // Verify preference crystallizes after threshold
    // Verify preferences() returns it
}

/// Store with embeddings -> vector search -> verify hybrid retrieval.
#[test]
fn test_hybrid_retrieval_with_embeddings() {
    let store = AlayaStore::open_in_memory().unwrap();

    // Store episodes with synthetic embeddings
    let emb_rust = vec![0.9, 0.1, 0.0];
    let emb_python = vec![0.1, 0.9, 0.0];

    store.store_episode(&NewEpisode {
        content: "I love Rust".to_string(),
        embedding: Some(emb_rust.clone()),
        // ...
    }).unwrap();

    store.store_episode(&NewEpisode {
        content: "Python is great".to_string(),
        embedding: Some(emb_python),
        // ...
    }).unwrap();

    // Query with embedding should find Rust episode first
    let results = store.query(&Query {
        text: "programming".to_string(),
        embedding: Some(emb_rust),
        context: QueryContext::default(),
        max_results: 5,
    }).unwrap();

    assert!(results[0].content.contains("Rust"));
}
```

### 4.3 Graceful Degradation Tests

These verify the degradation chain specified in the architecture:

```
Full: BM25 + vector + graph -> RRF -> rerank
No embeddings: BM25 + graph -> RRF -> rerank
No links: BM25 + vector -> RRF -> rerank
No FTS matches: vector + graph -> RRF -> rerank
Minimal: BM25-only -> rerank
Empty DB: [] (empty result, no error)
```

```rust
// tests/degradation_chain.rs

/// Empty database returns empty results, no errors.
#[test]
fn test_degradation_empty_db() {
    let store = AlayaStore::open_in_memory().unwrap();
    let results = store.query(&Query::simple("anything")).unwrap();
    assert!(results.is_empty());
}

/// BM25-only: episodes with text, no embeddings, no links.
#[test]
fn test_degradation_bm25_only() { /* ... */ }

/// Vector-only: query with embedding, FTS5 finds nothing (no lexical overlap).
#[test]
fn test_degradation_vector_only() { /* ... */ }

/// BM25 + vector: both signals present, no graph links.
#[test]
fn test_degradation_bm25_and_vector() { /* ... */ }

/// Full pipeline: BM25 + vector + graph all contribute.
#[test]
fn test_degradation_full_pipeline() { /* ... */ }
```

### 4.4 Persistence Tests

```rust
// tests/persistence.rs

/// Write to file, close, reopen, verify data survives.
#[test]
fn test_persistence_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    {
        let store = AlayaStore::open(&db_path).unwrap();
        store.store_episode(&NewEpisode {
            content: "persistent data".to_string(),
            // ...
        }).unwrap();
    } // store dropped, connection closed

    {
        let store = AlayaStore::open(&db_path).unwrap();
        let results = store.query(&Query::simple("persistent")).unwrap();
        assert_eq!(results.len(), 1);
    }
}
```

### 4.5 Error Path Tests

```rust
// tests/error_paths.rs

/// Every AlayaError variant is exercised and has actionable .to_string()
#[test]
fn test_all_error_variants_display() {
    // NotFound
    let store = AlayaStore::open_in_memory().unwrap();
    // Purge a nonexistent session should still work (0 deleted)
    let report = store.purge(PurgeFilter::Session("nonexistent".into())).unwrap();
    assert_eq!(report.episodes_deleted, 0);
}
```

### 4.6 Concurrent Access Tests

```rust
// tests/concurrent_access.rs

/// Multiple threads sharing AlayaStore via Arc<Mutex>.
#[test]
fn test_arc_mutex_concurrent_writes() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let store = Arc::new(Mutex::new(AlayaStore::open_in_memory().unwrap()));
    let mut handles = vec![];

    for i in 0..10 {
        let store = Arc::clone(&store);
        handles.push(thread::spawn(move || {
            let s = store.lock().unwrap();
            s.store_episode(&NewEpisode {
                content: format!("thread {} message", i),
                role: Role::User,
                session_id: format!("thread-{}", i),
                timestamp: 1000 + i,
                context: EpisodeContext::default(),
                embedding: None,
            }).unwrap();
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let s = store.lock().unwrap();
    assert_eq!(s.status().unwrap().episode_count, 10);
}
```

---

## 5. Property-Based Testing

Property-based tests use `proptest` to generate random inputs and verify that invariants hold universally, not just for hand-picked examples. These are critical for Alaya's mathematical models.

### 5.1 Dev-Dependency Addition

```toml
# Cargo.toml
[dev-dependencies]
proptest = "1.4"
tempfile = "3.10"
```

### 5.2 Bjork Dual-Strength Properties

The Bjork model (ADR-005) has strict mathematical invariants that must hold for all inputs:

```rust
// In store/strengths.rs tests or tests/property_bjork.rs

use proptest::prelude::*;

proptest! {
    /// Storage strength is monotonically non-decreasing with access.
    /// SS_new = SS + 0.05 * (1 - SS) >= SS for all SS in [0, 1].
    #[test]
    fn prop_storage_strength_monotonic(
        initial_ss in 0.0f32..=1.0f32,
    ) {
        // SS_new = SS + 0.05 * (1 - SS)
        let new_ss = initial_ss + 0.05 * (1.0 - initial_ss);
        prop_assert!(new_ss >= initial_ss, "SS must be non-decreasing");
        prop_assert!(new_ss <= 1.0, "SS must not exceed 1.0");
    }

    /// Retrieval strength decays monotonically: RS * 0.95 <= RS.
    #[test]
    fn prop_retrieval_strength_decay_monotonic(
        initial_rs in 0.0f32..=1.0f32,
    ) {
        let decayed = initial_rs * 0.95;
        prop_assert!(decayed <= initial_rs, "RS must decay");
        prop_assert!(decayed >= 0.0, "RS must not go negative");
    }

    /// After N accesses, storage strength approaches 1.0.
    /// Specifically: SS_N = 1 - (1 - SS_0) * 0.95^N
    #[test]
    fn prop_storage_strength_convergence(
        n in 1u32..200,
    ) {
        let initial = 0.5f32;
        let mut ss = initial;
        for _ in 0..n {
            ss = ss + 0.05 * (1.0 - ss);
        }
        prop_assert!(ss >= initial);
        prop_assert!(ss <= 1.0);
        if n >= 100 {
            prop_assert!(ss > 0.99, "SS should be near 1.0 after 100+ accesses");
        }
    }

    /// After N forget cycles without access, RS approaches 0.
    /// RS_N = RS_0 * 0.95^N
    #[test]
    fn prop_retrieval_strength_approaches_zero(
        n in 1u32..200,
    ) {
        let mut rs = 1.0f32;
        for _ in 0..n {
            rs *= 0.95;
        }
        prop_assert!(rs >= 0.0);
        if n >= 90 {
            prop_assert!(rs < 0.01, "RS should be near 0 after 90+ cycles");
        }
    }

    /// Archival condition: nodes with SS < 0.1 AND RS < 0.05 are archivable.
    /// Nodes above either threshold are safe.
    #[test]
    fn prop_archival_safety(
        ss in 0.0f32..=1.0f32,
        rs in 0.0f32..=1.0f32,
    ) {
        let archivable = ss < 0.1 && rs < 0.05;
        if ss >= 0.1 || rs >= 0.05 {
            prop_assert!(!archivable, "safe nodes must not be archivable");
        }
    }
}
```

### 5.3 Hebbian Weight Properties

```rust
proptest! {
    /// Hebbian LTP: w += 0.1 * (1 - w) keeps w in [0, 1].
    #[test]
    fn prop_hebbian_ltp_bounded(
        initial_w in 0.0f32..=1.0f32,
    ) {
        let new_w = initial_w + 0.1 * (1.0 - initial_w);
        prop_assert!(new_w >= initial_w, "LTP must be non-decreasing");
        prop_assert!(new_w <= 1.0, "weight must not exceed 1.0");
        prop_assert!(new_w >= 0.0, "weight must not go negative");
    }

    /// Hebbian LTP is monotonically non-decreasing.
    #[test]
    fn prop_hebbian_ltp_monotonic(
        w_a in 0.0f32..=1.0f32,
        w_b in 0.0f32..=1.0f32,
    ) {
        let new_a = w_a + 0.1 * (1.0 - w_a);
        let new_b = w_b + 0.1 * (1.0 - w_b);
        if w_a <= w_b {
            prop_assert!(new_a <= new_b, "LTP preserves ordering");
        }
    }

    /// After N co-retrievals, weight approaches 1.0.
    #[test]
    fn prop_hebbian_ltp_convergence(
        n in 1u32..100,
    ) {
        let mut w = 0.3f32; // Initial co-retrieval weight
        for _ in 0..n {
            w = w + 0.1 * (1.0 - w);
        }
        prop_assert!(w <= 1.0);
        if n >= 50 {
            prop_assert!(w > 0.99);
        }
    }

    /// Link decay with factor f keeps weights non-negative.
    #[test]
    fn prop_link_decay_non_negative(
        w in 0.0f32..=1.0f32,
        factor in 0.0f32..=1.0f32,
    ) {
        let decayed = w * factor;
        prop_assert!(decayed >= 0.0);
        prop_assert!(decayed <= w);
    }
}
```

### 5.4 RRF Fusion Properties

```rust
proptest! {
    /// RRF scores are always positive for non-empty inputs.
    #[test]
    fn prop_rrf_scores_positive(
        n_sets in 1usize..5,
        n_items in 1usize..20,
        k in 1u32..200,
    ) {
        // Generate random result sets
        let sets: Vec<Vec<(NodeRef, f64)>> = (0..n_sets)
            .map(|_| {
                (0..n_items)
                    .map(|i| (NodeRef::Episode(EpisodeId(i as i64)), 0.5))
                    .collect()
            })
            .collect();
        let merged = fusion::rrf_merge(&sets, k);
        for (_, score) in &merged {
            prop_assert!(*score > 0.0, "RRF scores must be positive");
        }
    }

    /// A document present in more sets always scores >= one in fewer sets.
    #[test]
    fn prop_rrf_more_sets_higher_score(k in 1u32..200) {
        let a = NodeRef::Episode(EpisodeId(1));
        let b = NodeRef::Episode(EpisodeId(2));

        // a appears in 2 sets, b appears in 1 set, both at rank 0
        let sets = vec![
            vec![(a, 0.9), (b, 0.5)],
            vec![(a, 0.8)],
        ];
        let merged = fusion::rrf_merge(&sets, k);
        let score_a = merged.iter().find(|(n, _)| *n == a).unwrap().1;
        let score_b = merged.iter().find(|(n, _)| *n == b).unwrap().1;
        prop_assert!(score_a >= score_b);
    }
}
```

### 5.5 Cosine Similarity Properties

```rust
proptest! {
    /// Cosine similarity is in [-1, 1] for all non-zero inputs.
    #[test]
    fn prop_cosine_range(
        a in prop::collection::vec(-10.0f32..10.0, 3..64),
        b in prop::collection::vec(-10.0f32..10.0, 3..64),
    ) {
        if a.len() == b.len() {
            let sim = cosine_similarity(&a, &b);
            if a.iter().any(|x| *x != 0.0) && b.iter().any(|x| *x != 0.0) {
                prop_assert!(sim >= -1.0 - 1e-6 && sim <= 1.0 + 1e-6,
                    "cosine must be in [-1, 1], got {}", sim);
            }
        }
    }

    /// Cosine similarity is symmetric: cos(a, b) == cos(b, a).
    #[test]
    fn prop_cosine_symmetric(
        a in prop::collection::vec(-10.0f32..10.0, 3..16),
    ) {
        let b_data: Vec<f32> = a.iter().map(|x| x + 0.1).collect();
        let sim_ab = cosine_similarity(&a, &b_data);
        let sim_ba = cosine_similarity(&b_data, &a);
        prop_assert!((sim_ab - sim_ba).abs() < 1e-5,
            "cosine must be symmetric: {} vs {}", sim_ab, sim_ba);
    }

    /// Cosine of a vector with itself is 1.0 (for non-zero vectors).
    #[test]
    fn prop_cosine_self_identity(
        a in prop::collection::vec(0.1f32..10.0, 3..64),
    ) {
        let sim = cosine_similarity(&a, &a);
        prop_assert!((sim - 1.0).abs() < 1e-5, "cos(a, a) should be 1.0, got {}", sim);
    }

    /// Embedding serialization roundtrips for arbitrary f32 values.
    #[test]
    fn prop_embedding_roundtrip(
        vec in prop::collection::vec(prop::num::f32::ANY, 1..512),
    ) {
        let blob = serialize_embedding(&vec);
        let restored = deserialize_embedding(&blob);
        prop_assert_eq!(vec.len(), restored.len());
        for (a, b) in vec.iter().zip(restored.iter()) {
            if a.is_nan() {
                prop_assert!(b.is_nan());
            } else {
                prop_assert_eq!(a.to_bits(), b.to_bits());
            }
        }
    }
}
```

### 5.6 Recency Decay Properties

```rust
proptest! {
    /// Recency decay is monotonically decreasing with age.
    #[test]
    fn prop_recency_monotonic(
        age_a_days in 0u32..365,
        age_b_days in 0u32..365,
    ) {
        let now = 1_000_000i64;
        let ts_a = now - (age_a_days as i64 * 86400);
        let ts_b = now - (age_b_days as i64 * 86400);
        let decay_a = recency_decay(ts_a, now);
        let decay_b = recency_decay(ts_b, now);
        if age_a_days <= age_b_days {
            prop_assert!(decay_a >= decay_b - 1e-10,
                "newer should have higher recency");
        }
    }

    /// Recency decay is always in [0, 1].
    #[test]
    fn prop_recency_range(age_secs in 0i64..31_536_000) {
        let now = 1_000_000_000i64;
        let ts = now - age_secs;
        let decay = recency_decay(ts, now);
        prop_assert!(decay >= 0.0 && decay <= 1.0 + 1e-10);
    }
}
```

---

## 6. Retrieval Quality Benchmarks

### 6.1 Purpose

Retrieval quality benchmarks answer the question: "Does Alaya's hybrid retrieval pipeline return the right memories?" They measure precision, recall, and ranking quality against datasets with known-correct answers.

This is the "Honesty > Marketing" axiom in practice. Publish results even when they are poor. Track regression. Improve methodically.

### 6.2 Golden Datasets

| Dataset | Description | What It Tests | Availability |
|---------|-------------|---------------|--------------|
| **LoCoMo** | Long-context conversational memory benchmark | Multi-session recall, temporal reasoning, preference tracking | Public (arXiv:2402.14088) |
| **LongMemEval** | Long-term memory evaluation for chatbots | Factual recall, preference consistency, temporal ordering | Public (arXiv:2410.10813) |
| **Alaya-Internal** | Hand-curated test cases for Alaya-specific features | Bjork forgetting impact, vasana crystallization, Hebbian graph traversal | Created as part of this strategy |

### 6.3 Metrics

**Precision@k:** Of the top k results, how many are relevant?

```
P@k = |relevant results in top k| / k
```

Target: P@5 >= 0.70 for LoCoMo factual queries with BM25-only, P@5 >= 0.80 with hybrid retrieval.

**Recall@k:** Of all relevant results, how many appear in the top k?

```
R@k = |relevant results in top k| / |total relevant|
```

Target: R@10 >= 0.60 for LoCoMo queries.

**NDCG (Normalized Discounted Cumulative Gain):** Measures ranking quality, rewarding relevant results at higher positions.

```
DCG@k = sum(rel_i / log2(i + 1) for i in 1..k)
NDCG@k = DCG@k / ideal_DCG@k
```

Target: NDCG@5 >= 0.65 for LoCoMo, improving to >= 0.75 by v0.2.

**MRR (Mean Reciprocal Rank):** Average of 1/rank for the first correct result.

Target: MRR >= 0.70 for factual recall queries.

### 6.4 Benchmark Harness

```rust
// tests/retrieval_quality.rs or benches/retrieval_quality.rs

/// Load a golden dataset, populate Alaya, run queries, compute metrics.
struct RetrievalBenchmark {
    store: AlayaStore,
    queries: Vec<GoldenQuery>,
}

struct GoldenQuery {
    text: String,
    embedding: Option<Vec<f32>>,
    expected_episode_ids: Vec<EpisodeId>,
    relevance_grades: Vec<u8>, // 0=irrelevant, 1=relevant, 2=highly relevant
}

impl RetrievalBenchmark {
    fn precision_at_k(&self, k: usize) -> f64 { /* ... */ }
    fn recall_at_k(&self, k: usize) -> f64 { /* ... */ }
    fn ndcg_at_k(&self, k: usize) -> f64 { /* ... */ }
    fn mrr(&self) -> f64 { /* ... */ }
}
```

### 6.5 Internal Golden Datasets

For Alaya-specific features that public datasets do not cover:

**Bjork Forgetting Impact:** Store 100 episodes, call `forget()` N times, verify that frequently-accessed episodes survive while rarely-accessed ones are archived. Measure recall degradation curve.

**Vasana Preference Accuracy:** Feed consistent behavioral signals, verify crystallized preferences match expected patterns. Measure precision of preference extraction.

**Hebbian Graph Traversal:** Create known graph structures, verify that spreading activation retrieves semantically related nodes that BM25 alone would miss. Measure the delta between BM25-only and hybrid retrieval.

**Graceful Degradation Quality:** Same queries, measured under each degradation level. Verify that adding signals (vector, graph) monotonically improves or maintains quality.

### 6.6 Benchmark Reporting

Benchmark results are stored in `benches/results/` as JSON for historical tracking:

```json
{
  "version": "0.1.0",
  "date": "2026-03-15",
  "dataset": "locomo",
  "config": { "retrieval": "bm25_only", "episodes": 500 },
  "metrics": {
    "precision_at_5": 0.68,
    "recall_at_10": 0.55,
    "ndcg_at_5": 0.62,
    "mrr": 0.71
  }
}
```

---

## 7. Lifecycle Invariant Tests

Lifecycle processes (consolidation, perfuming, transformation, forgetting) are the product differentiator. They must satisfy strict invariants that hold regardless of database state.

### 7.1 Consolidation Invariants

```rust
/// I1: consolidate() is safe to call on any database state (including empty).
#[test]
fn invariant_consolidation_safe_on_any_state() {
    let store = AlayaStore::open_in_memory().unwrap();
    // Empty DB
    store.consolidate(&NoOpProvider).unwrap();
    // 1 episode (below threshold)
    store.store_episode(&make_episode("one", 1000)).unwrap();
    store.consolidate(&NoOpProvider).unwrap();
    // 100 episodes
    for i in 0..100 {
        store.store_episode(&make_episode(&format!("ep {}", i), 2000 + i)).unwrap();
    }
    store.consolidate(&NoOpProvider).unwrap();
}

/// I2: consolidate() with NoOpProvider never creates semantic nodes.
#[test]
fn invariant_noop_consolidation_creates_nothing() {
    // ... populate database, consolidate with NoOp, verify no semantic nodes
}

/// I3: Semantic nodes created by consolidation are linked to source episodes.
#[test]
fn invariant_consolidation_links_sources() { /* ... */ }

/// I4: consolidate() does not delete or modify episodes.
#[test]
fn invariant_consolidation_preserves_episodes() { /* ... */ }
```

### 7.2 Forgetting Invariants

```rust
/// I5: forget() never increases retrieval strength.
#[test]
fn invariant_forget_never_increases_rs() {
    let store = AlayaStore::open_in_memory().unwrap();
    // Store episodes, init strengths, snapshot RS values
    // Call forget()
    // Verify every RS_after <= RS_before
}

/// I6: forget() never modifies storage strength.
#[test]
fn invariant_forget_preserves_ss() {
    // SS is monotonically non-decreasing; forget() only touches RS
}

/// I7: Archived nodes are fully removed (no orphaned embeddings, links, strengths).
#[test]
fn invariant_archival_complete_cleanup() {
    // Create episode with embedding and links, force archival conditions
    // Call forget(), verify no orphaned rows in embeddings, links, node_strengths
}

/// I8: forget() is safe to call repeatedly (idempotent decay).
#[test]
fn invariant_forget_idempotent() {
    // Multiple calls produce consistent decay without errors
}

/// I9: Recently accessed nodes survive forgetting.
#[test]
fn invariant_recent_access_survives() {
    // Access a node, immediately call forget(), verify node still exists
}
```

### 7.3 Transformation Invariants

```rust
/// I10: transform() is idempotent when run on clean data.
#[test]
fn invariant_transform_idempotent() {
    let store = AlayaStore::open_in_memory().unwrap();
    // First transform
    let r1 = store.transform().unwrap();
    // Second transform should find nothing to do
    let r2 = store.transform().unwrap();
    assert_eq!(r2.duplicates_merged, 0);
    assert_eq!(r2.links_pruned, 0);
}

/// I11: Deduplication preserves the node with more corroborations.
#[test]
fn invariant_dedup_preserves_stronger() { /* ... */ }

/// I12: Link pruning only removes links below threshold.
#[test]
fn invariant_prune_preserves_strong_links() { /* ... */ }

/// I13: transform() never deletes episodes.
#[test]
fn invariant_transform_preserves_episodes() { /* ... */ }
```

### 7.4 Perfuming Invariants

```rust
/// I14: Preference crystallization requires >= 5 impressions.
#[test]
fn invariant_crystallization_requires_threshold() { /* ... */ }

/// I15: Perfuming with NoOpProvider stores zero impressions.
#[test]
fn invariant_noop_perfuming_stores_nothing() { /* ... */ }

/// I16: Preferences are never duplicated within a domain.
#[test]
fn invariant_no_duplicate_preferences() {
    // Perfume 10 times in same domain, verify at most 1 preference exists
}

/// I17: Reinforcement increments evidence_count, never resets it.
#[test]
fn invariant_reinforcement_monotonic() { /* ... */ }
```

### 7.5 Cross-Lifecycle Invariants

```rust
/// I18: Full lifecycle sequence does not corrupt database.
/// store -> consolidate -> transform -> forget -> query -> repeat
#[test]
fn invariant_full_lifecycle_sequence() {
    let store = AlayaStore::open_in_memory().unwrap();
    let noop = NoOpProvider;

    for cycle in 0..10 {
        // Store some episodes
        for i in 0..5 {
            store.store_episode(&NewEpisode {
                content: format!("cycle {} message {}", cycle, i),
                role: Role::User,
                session_id: format!("cycle-{}", cycle),
                timestamp: (cycle * 1000 + i * 100) as i64,
                context: EpisodeContext::default(),
                embedding: None,
            }).unwrap();
        }

        // Run full lifecycle
        store.consolidate(&noop).unwrap();
        store.transform().unwrap();
        store.forget().unwrap();

        // Query must succeed (no corruption)
        let _results = store.query(&Query::simple("message")).unwrap();

        // Status must be consistent
        let status = store.status().unwrap();
        // All counts must be non-negative (no underflow)
        assert!(status.episode_count <= 1_000_000);
    }
}

/// I19: Purge(All) leaves database in same state as open_in_memory().
#[test]
fn invariant_purge_all_resets_to_empty() {
    let store = AlayaStore::open_in_memory().unwrap();
    // Populate extensively, then purge
    store.purge(PurgeFilter::All).unwrap();
    let status = store.status().unwrap();
    assert_eq!(status.episode_count, 0);
    assert_eq!(status.semantic_node_count, 0);
    assert_eq!(status.preference_count, 0);
    assert_eq!(status.impression_count, 0);
    assert_eq!(status.link_count, 0);
    assert_eq!(status.embedding_count, 0);
}
```

---

## 8. Test Fixtures and Utilities

### 8.1 In-Memory Database Helper

Already exists: `schema::open_memory_db()` and `AlayaStore::open_in_memory()`. These are the standard test fixtures. Every test gets an isolated, empty database.

### 8.2 Episode Factory

A reusable builder for test episodes:

```rust
// tests/common/mod.rs or a test utility module

pub fn make_episode(content: &str, ts: i64) -> NewEpisode {
    NewEpisode {
        content: content.to_string(),
        role: Role::User,
        session_id: "test-session".to_string(),
        timestamp: ts,
        context: EpisodeContext::default(),
        embedding: None,
    }
}

pub fn make_episode_with_context(
    content: &str,
    ts: i64,
    topics: Vec<&str>,
    entities: Vec<&str>,
) -> NewEpisode {
    NewEpisode {
        content: content.to_string(),
        role: Role::User,
        session_id: "test-session".to_string(),
        timestamp: ts,
        context: EpisodeContext {
            topics: topics.into_iter().map(String::from).collect(),
            mentioned_entities: entities.into_iter().map(String::from).collect(),
            ..Default::default()
        },
        embedding: None,
    }
}

pub fn make_episode_with_embedding(
    content: &str,
    ts: i64,
    embedding: Vec<f32>,
) -> NewEpisode {
    NewEpisode {
        content: content.to_string(),
        role: Role::User,
        session_id: "test-session".to_string(),
        timestamp: ts,
        context: EpisodeContext::default(),
        embedding: Some(embedding),
    }
}
```

### 8.3 Pre-Populated Database Fixtures

For integration tests that need a database with realistic data:

```rust
/// Create a database with 50 episodes across 5 sessions,
/// 10 semantic nodes, 20 links, and 5 impressions.
pub fn populated_fixture() -> AlayaStore {
    let store = AlayaStore::open_in_memory().unwrap();

    let topics_pool = vec![
        ("Rust programming", vec![0.9, 0.1, 0.0]),
        ("Python data science", vec![0.1, 0.9, 0.0]),
        ("Machine learning", vec![0.3, 0.7, 0.1]),
        ("Database design", vec![0.5, 0.1, 0.5]),
        ("Async programming", vec![0.8, 0.2, 0.1]),
    ];

    for (session_idx, (topic, emb)) in topics_pool.iter().enumerate() {
        for turn in 0..10 {
            store.store_episode(&NewEpisode {
                content: format!("{} discussion turn {}", topic, turn),
                role: if turn % 2 == 0 { Role::User } else { Role::Assistant },
                session_id: format!("session-{}", session_idx),
                timestamp: (session_idx * 10000 + turn * 100) as i64,
                context: EpisodeContext {
                    topics: vec![topic.to_string()],
                    ..Default::default()
                },
                embedding: Some(emb.clone()),
            }).unwrap();
        }
    }

    store
}
```

### 8.4 Mock Provider Variants

The existing `MockProvider` in `provider.rs` is basic. For integration tests, richer mock providers are needed:

```rust
/// A provider that returns errors (for testing error propagation).
pub struct ErrorProvider;

impl ConsolidationProvider for ErrorProvider {
    fn extract_knowledge(&self, _: &[Episode]) -> Result<Vec<NewSemanticNode>> {
        Err(AlayaError::Provider("LLM unavailable".to_string()))
    }
    fn extract_impressions(&self, _: &Interaction) -> Result<Vec<NewImpression>> {
        Err(AlayaError::Provider("LLM unavailable".to_string()))
    }
    fn detect_contradiction(&self, _: &SemanticNode, _: &SemanticNode) -> Result<bool> {
        Err(AlayaError::Provider("LLM unavailable".to_string()))
    }
}

/// A provider that extracts one semantic node per episode (deterministic).
pub struct DeterministicProvider;

impl ConsolidationProvider for DeterministicProvider {
    fn extract_knowledge(&self, episodes: &[Episode]) -> Result<Vec<NewSemanticNode>> {
        Ok(vec![NewSemanticNode {
            content: format!("Knowledge from {} episodes", episodes.len()),
            node_type: SemanticType::Fact,
            confidence: 0.7,
            source_episodes: episodes.iter().map(|e| e.id).collect(),
            embedding: None,
        }])
    }
    fn extract_impressions(&self, interaction: &Interaction) -> Result<Vec<NewImpression>> {
        Ok(vec![NewImpression {
            domain: "test".to_string(),
            observation: format!("Observation from: {}", &interaction.text[..20.min(interaction.text.len())]),
            valence: 0.5,
        }])
    }
    fn detect_contradiction(&self, _: &SemanticNode, _: &SemanticNode) -> Result<bool> {
        Ok(false)
    }
}
```

### 8.5 Synthetic Embedding Generator

For tests that need embeddings without a real embedding model:

```rust
/// Generate a synthetic embedding for a topic string.
/// Uses hash-based deterministic generation for reproducibility.
pub fn synthetic_embedding(topic: &str, dims: usize) -> Vec<f32> {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();
    topic.hash(&mut hasher);
    let seed = hasher.finish();

    let mut vec = Vec::with_capacity(dims);
    let mut state = seed;
    for _ in 0..dims {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let val = (state >> 33) as f32 / (u32::MAX as f32) * 2.0 - 1.0;
        vec.push(val);
    }

    // Normalize to unit length
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in &mut vec {
            *v /= norm;
        }
    }

    vec
}
```

---

## 9. Performance Benchmarks

### 9.1 Benchmark Framework: divan

Per the architecture plan, Alaya uses `divan` (v0.2) for benchmarks. It provides low-overhead, statistical benchmarking with built-in support for parameterized tests.

```toml
# Cargo.toml
[dev-dependencies]
divan = "0.1"

[[bench]]
name = "store_bench"
harness = false

[[bench]]
name = "retrieval_bench"
harness = false

[[bench]]
name = "lifecycle_bench"
harness = false
```

### 9.2 What to Measure

**Store Operations:**

```rust
// benches/store_bench.rs
use divan::Bencher;

#[divan::bench(args = [10, 100, 1000, 10000])]
fn bench_store_episode(bencher: Bencher, n: usize) {
    let store = AlayaStore::open_in_memory().unwrap();
    // Pre-populate n episodes
    for i in 0..n {
        store.store_episode(&make_episode(&format!("ep {}", i), i as i64)).unwrap();
    }
    // Benchmark the (n+1)-th insert
    bencher.bench(|| {
        store.store_episode(&make_episode("benchmark", n as i64 + 1)).unwrap();
    });
}

#[divan::bench]
fn bench_store_episode_with_embedding(bencher: Bencher) {
    let store = AlayaStore::open_in_memory().unwrap();
    let emb = vec![0.1f32; 384];
    bencher.bench(|| {
        store.store_episode(&NewEpisode {
            embedding: Some(emb.clone()),
            ..make_episode("bench", 1000)
        }).unwrap();
    });
}
```

**Retrieval Operations:**

```rust
// benches/retrieval_bench.rs

#[divan::bench(args = [100, 1000, 5000, 10000])]
fn bench_bm25_query(bencher: Bencher, n_episodes: usize) {
    let store = populated_store(n_episodes);
    bencher.bench(|| {
        store.query(&Query::simple("Rust programming")).unwrap();
    });
}

#[divan::bench(args = [100, 1000, 5000])]
fn bench_vector_query(bencher: Bencher, n_episodes: usize) {
    let store = populated_store_with_embeddings(n_episodes, 384);
    let query_emb = synthetic_embedding("Rust", 384);
    bencher.bench(|| {
        store.query(&Query {
            text: "Rust".to_string(),
            embedding: Some(query_emb.clone()),
            context: QueryContext::default(),
            max_results: 5,
        }).unwrap();
    });
}

#[divan::bench(args = [100, 1000, 5000])]
fn bench_hybrid_query(bencher: Bencher, n_episodes: usize) {
    // BM25 + vector + graph
}
```

**Lifecycle Operations:**

```rust
// benches/lifecycle_bench.rs

#[divan::bench(args = [100, 500, 1000])]
fn bench_consolidation(bencher: Bencher, n_episodes: usize) {
    // Measure consolidation time with DeterministicProvider
}

#[divan::bench(args = [100, 500, 1000])]
fn bench_forget_cycle(bencher: Bencher, n_strength_records: usize) {
    // Measure forgetting sweep time
}

#[divan::bench(args = [100, 500, 1000])]
fn bench_transform(bencher: Bencher, n_nodes: usize) {
    // Measure transformation time
}
```

**Low-Level Operations:**

```rust
#[divan::bench(args = [128, 384, 768, 1536])]
fn bench_cosine_similarity(bencher: Bencher, dims: usize) {
    let a: Vec<f32> = (0..dims).map(|i| (i as f32 * 0.01).sin()).collect();
    let b: Vec<f32> = (0..dims).map(|i| (i as f32 * 0.02).cos()).collect();
    bencher.bench(|| {
        cosine_similarity(&a, &b);
    });
}

#[divan::bench(args = [128, 384, 768, 1536])]
fn bench_embedding_serialize(bencher: Bencher, dims: usize) {
    let vec: Vec<f32> = (0..dims).map(|i| i as f32 * 0.01).collect();
    bencher.bench(|| {
        serialize_embedding(&vec);
    });
}

#[divan::bench]
fn bench_spreading_activation(bencher: Bencher) {
    // Pre-populate graph with ~1000 links
}
```

### 9.3 Performance Targets

| Operation | Target (v0.1) | Target (v0.2) |
|-----------|--------------|--------------|
| store_episode (no embedding) | < 100 us | < 50 us |
| store_episode (384-dim embedding) | < 200 us | < 100 us |
| BM25 query (1K episodes) | < 1 ms | < 500 us |
| BM25 query (10K episodes) | < 5 ms | < 2 ms |
| Vector query (1K embeddings, 384-dim) | < 5 ms | < 1 ms (sqlite-vec) |
| Vector query (10K embeddings, 384-dim) | < 50 ms | < 5 ms (sqlite-vec) |
| Hybrid query (1K episodes) | < 10 ms | < 5 ms |
| cosine_similarity (384-dim) | < 1 us | < 500 ns |
| forget() sweep (1K nodes) | < 5 ms | < 2 ms |
| transform() (1K nodes, 5K links) | < 50 ms | < 20 ms |

### 9.4 Regression Detection

Performance benchmarks run in CI on every push to `main`. Results are compared against a baseline stored in `benches/baseline.json`. A regression of > 20% on any benchmark triggers a warning. A regression of > 50% fails the build.

```yaml
# .github/workflows/bench.yml
- name: Run benchmarks
  run: cargo bench -- --format json > bench_results.json
- name: Compare against baseline
  run: python scripts/bench_compare.py baseline.json bench_results.json --threshold 0.20
```

---

## 10. CI/CD Test Pipeline

### 10.1 GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-22.04, macos-14, windows-2022]
        rust: [stable, beta]
        include:
          - os: ubuntu-22.04
            rust: "1.75.0"  # MSRV
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
      - name: Build
        run: cargo build --all-targets
      - name: Run tests
        run: cargo test --all-targets
      - name: Run doc tests
        run: cargo test --doc

  clippy:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - run: cargo clippy --all-targets -- -D warnings

  fmt:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --check

  no-network-deps:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Verify no networking dependencies
        run: |
          ! cargo tree 2>/dev/null | grep -E "reqwest|hyper|tokio-net|ureq|attohttpc|surf"
          echo "No networking dependencies found (ADR-009 enforced)"

  property-tests:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Run property tests with extended cases
        run: PROPTEST_CASES=1000 cargo test --test property_tests

  quickstart-regression:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Extract and compile README quickstart
        run: |
          python scripts/extract_quickstart.py README.md > /tmp/quickstart.rs
          cd /tmp
          cargo init --name quickstart-test
          echo 'alaya = { path = "${{ github.workspace }}" }' >> Cargo.toml
          cp quickstart.rs src/main.rs
          cargo run
```

### 10.2 Feature Flag Test Matrix

When feature flags are introduced (v0.2), CI must test all supported combinations:

```yaml
feature-matrix:
  strategy:
    matrix:
      features:
        - ""                    # default features only
        - "--features vec-sqlite"
        - "--features embed-ort"
        - "--features embed-fastembed"
        - "--features async"
        - "--features vec-sqlite,embed-ort"
        - "--features vec-sqlite,async"
        - "--all-features"
        - "--no-default-features"
  steps:
    - run: cargo test ${{ matrix.features }}
```

### 10.3 Test Isolation

All tests run with `cargo test` which runs tests in parallel by default. Since each test creates its own in-memory SQLite database, there are no shared state issues. For file-backed persistence tests, each test uses `tempfile::tempdir()` for isolation.

Configuration:

```toml
# .cargo/config.toml
[build]
# Use all available cores for test compilation
jobs = 0

# No special test profile needed -- default is fine
```

### 10.4 MSRV (Minimum Supported Rust Version) Testing

Alaya targets Rust stable edition 2021. The MSRV is tested in CI to ensure no accidental dependency on nightly or recent-stable features.

The MSRV is declared in `Cargo.toml`:

```toml
rust-version = "1.75.0"
```

CI tests against this version on Ubuntu to catch MSRV regressions.

---

## 11. Fuzzing Strategy

### 11.1 Why Fuzz Alaya

Alaya processes untrusted input at several boundaries:

1. **FTS5 MATCH queries** -- user text is sanitized and passed to SQLite FTS5. The sanitization function must be robust against all inputs.
2. **Embedding BLOBs** -- f32 arrays serialized as little-endian bytes. Malformed BLOBs should not cause panics.
3. **JSON context** -- `EpisodeContext` deserialized from stored JSON. Corrupt JSON must be handled gracefully.
4. **Provider output** -- `ConsolidationProvider` returns arbitrary content. Alaya must not crash on adversarial provider output.

### 11.2 Fuzz Targets

```rust
// fuzz/fuzz_targets/fts5_sanitization.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use alaya::retrieval::bm25;

fuzz_target!(|data: &str| {
    let conn = alaya::schema::open_memory_db().unwrap();
    // Insert a known episode so the table is non-empty
    conn.execute(
        "INSERT INTO episodes (content, role, session_id, timestamp)
         VALUES ('baseline content', 'user', 's1', 1000)",
        [],
    ).unwrap();
    // This must never panic or return Err for FTS5 syntax reasons
    let _ = bm25::search_bm25(&conn, data, 10);
});
```

```rust
// fuzz/fuzz_targets/embedding_deserialize.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use alaya::store::embeddings;

fuzz_target!(|data: &[u8]| {
    // Deserialization must not panic on any input
    let vec = embeddings::deserialize_embedding(data);
    // Re-serialization must roundtrip on well-formed input
    if data.len() % 4 == 0 {
        let blob = embeddings::serialize_embedding(&vec);
        assert_eq!(blob.len(), data.len());
    }
});
```

```rust
// fuzz/fuzz_targets/context_json.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use alaya::types::EpisodeContext;

fuzz_target!(|data: &str| {
    // JSON deserialization must not panic
    let _: Result<EpisodeContext, _> = serde_json::from_str(data);
});
```

```rust
// fuzz/fuzz_targets/store_episode.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use alaya::*;

fuzz_target!(|data: &str| {
    let store = AlayaStore::open_in_memory().unwrap();
    let _ = store.store_episode(&NewEpisode {
        content: data.to_string(),
        role: Role::User,
        session_id: "fuzz".to_string(),
        timestamp: 1000,
        context: EpisodeContext::default(),
        embedding: None,
    });
});
```

```rust
// fuzz/fuzz_targets/cosine_similarity.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct CosineInput {
    a: Vec<f32>,
    b: Vec<f32>,
}

fuzz_target!(|input: CosineInput| {
    let sim = alaya::store::embeddings::cosine_similarity(&input.a, &input.b);
    // Must not be NaN (we check this) and must not panic
    assert!(!sim.is_nan() || input.a.iter().any(|x| x.is_nan()) || input.b.iter().any(|x| x.is_nan()));
});
```

### 11.3 Fuzzing Infrastructure

```toml
# fuzz/Cargo.toml
[package]
name = "alaya-fuzz"
version = "0.0.0"
publish = false
edition = "2021"

[dependencies]
libfuzzer-sys = "0.4"
arbitrary = { version = "1", features = ["derive"] }
alaya = { path = ".." }
```

Fuzz targets run locally for development and on a scheduled CI job (nightly) with a 10-minute timeout per target:

```yaml
fuzz:
  runs-on: ubuntu-22.04
  schedule:
    - cron: '0 3 * * *'  # 3 AM UTC daily
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@nightly
    - run: cargo install cargo-fuzz
    - run: cargo +nightly fuzz run fts5_sanitization -- -max_total_time=600
    - run: cargo +nightly fuzz run embedding_deserialize -- -max_total_time=600
    - run: cargo +nightly fuzz run context_json -- -max_total_time=600
    - run: cargo +nightly fuzz run store_episode -- -max_total_time=600
    - run: cargo +nightly fuzz run cosine_similarity -- -max_total_time=600
```

---

## 12. Doc Test Requirements

### 12.1 Coverage Target

Every public item in the Alaya crate must have a compilable doc test. The accessibility document (Phase 5c) specifies "100% of pub methods" for doctest coverage. This means:

- All 13 `AlayaStore` methods
- `AlayaStore::open()` and `AlayaStore::open_in_memory()`
- `ConsolidationProvider` trait and its 3 methods
- `NoOpProvider` struct
- `Query::simple()`
- All public types that consumers construct (`NewEpisode`, `EpisodeContext`, `Query`, `QueryContext`, `KnowledgeFilter`, `PurgeFilter`, `Interaction`)
- All public enums (`Role`, `SemanticType`, `LinkType`, `NodeRef`)
- All report types (consumers read these)

### 12.2 Doc Test Standards

1. **Every doctest must compile and pass.** No `no_run` or `ignore` attributes unless there is a documented reason (e.g., requires filesystem).
2. **Doctests use `open_in_memory()` exclusively.** Never write to the filesystem in a doctest.
3. **Doctests are self-contained.** Each example includes all necessary setup (no hidden setup lines unless truly boilerplate).
4. **Doctests demonstrate the most common usage pattern first, then edge cases.**

### 12.3 Examples for Key Public Methods

```rust
/// Open (or create) a persistent database at `path`.
///
/// # Examples
///
/// ```
/// # use alaya::AlayaStore;
/// # let dir = tempfile::tempdir().unwrap();
/// # let path = dir.path().join("memory.db");
/// let store = AlayaStore::open(&path).unwrap();
/// let status = store.status().unwrap();
/// assert_eq!(status.episode_count, 0);
/// ```
pub fn open(path: impl AsRef<Path>) -> Result<Self> { /* ... */ }

/// Open an ephemeral in-memory database (useful for tests).
///
/// # Examples
///
/// ```
/// use alaya::AlayaStore;
///
/// let store = AlayaStore::open_in_memory().unwrap();
/// let status = store.status().unwrap();
/// assert_eq!(status.episode_count, 0);
/// ```
pub fn open_in_memory() -> Result<Self> { /* ... */ }

/// Store a conversation episode with full context.
///
/// # Examples
///
/// ```
/// use alaya::*;
///
/// let store = AlayaStore::open_in_memory().unwrap();
/// let id = store.store_episode(&NewEpisode {
///     content: "I prefer Rust for systems programming".to_string(),
///     role: Role::User,
///     session_id: "session-1".to_string(),
///     timestamp: 1709251200,
///     context: EpisodeContext::default(),
///     embedding: None,
/// }).unwrap();
///
/// assert_eq!(store.status().unwrap().episode_count, 1);
/// ```
pub fn store_episode(&self, episode: &NewEpisode) -> Result<EpisodeId> { /* ... */ }

/// Hybrid retrieval: BM25 + vector + graph activation -> RRF -> rerank.
///
/// # Examples
///
/// ```
/// use alaya::*;
///
/// let store = AlayaStore::open_in_memory().unwrap();
///
/// // Store some episodes
/// store.store_episode(&NewEpisode {
///     content: "Rust has zero-cost abstractions and memory safety".to_string(),
///     role: Role::User,
///     session_id: "s1".to_string(),
///     timestamp: 1000,
///     context: EpisodeContext::default(),
///     embedding: None,
/// }).unwrap();
///
/// // Query with lexical overlap
/// let results = store.query(&Query::simple("Rust memory safety")).unwrap();
/// assert!(!results.is_empty());
/// assert!(results[0].content.contains("Rust"));
/// ```
pub fn query(&self, q: &Query) -> Result<Vec<ScoredMemory>> { /* ... */ }
```

### 12.4 Doctest CI Enforcement

Doc tests are run by `cargo test --doc` as part of the standard CI pipeline. Compilation failures in doc tests fail the build. This enforces that documentation stays in sync with the API.

```yaml
# Part of the main CI job
- name: Run doc tests
  run: cargo test --doc
```

---

## Appendix A: Test Execution Quick Reference

| Command | What It Runs |
|---------|-------------|
| `cargo test` | All unit tests + integration tests + doc tests |
| `cargo test --lib` | Only in-module unit tests |
| `cargo test --test store_lifecycle` | Single integration test file |
| `cargo test --doc` | Only doc tests |
| `cargo bench` | All benchmarks (divan) |
| `cargo +nightly fuzz run fts5_sanitization` | Single fuzz target |
| `PROPTEST_CASES=10000 cargo test prop_` | Property tests with 10K cases |
| `cargo test -- --nocapture` | Tests with stdout visible |
| `cargo test -- --test-threads=1` | Sequential execution (for debugging) |
| `cargo test bm25` | Only tests matching "bm25" |

## Appendix B: Test Naming Conventions

| Pattern | Meaning |
|---------|---------|
| `test_*` | Standard unit/integration test |
| `prop_*` | Property-based test (proptest) |
| `bench_*` | Performance benchmark (divan) |
| `invariant_*` | Lifecycle invariant test |
| `fuzz_*` | Fuzz target entry point |

## Appendix C: Dependencies to Add

```toml
[dev-dependencies]
proptest = "1.4"
tempfile = "3.10"
divan = "0.1"

# For fuzz targets (separate crate)
# libfuzzer-sys = "0.4"
# arbitrary = { version = "1", features = ["derive"] }
```

## Appendix D: File Manifest

| File | Purpose |
|------|---------|
| `src/*/mod.rs` | In-module `#[cfg(test)] mod tests` blocks |
| `tests/store_lifecycle.rs` | Full lifecycle integration tests |
| `tests/retrieval_quality.rs` | Retrieval quality benchmarks (P@k, NDCG) |
| `tests/degradation_chain.rs` | Graceful degradation scenarios |
| `tests/concurrent_access.rs` | `Arc<Mutex>` multi-thread patterns |
| `tests/persistence.rs` | File-backed roundtrip tests |
| `tests/error_paths.rs` | Every `AlayaError` variant exercised |
| `tests/purge_compliance.rs` | GDPR purge completeness |
| `tests/property_tests.rs` | proptest property tests |
| `benches/store_bench.rs` | Store operation benchmarks |
| `benches/retrieval_bench.rs` | Retrieval benchmarks at scale |
| `benches/lifecycle_bench.rs` | Lifecycle process benchmarks |
| `fuzz/fuzz_targets/fts5_sanitization.rs` | FTS5 input fuzzing |
| `fuzz/fuzz_targets/embedding_deserialize.rs` | BLOB deserialization fuzzing |
| `fuzz/fuzz_targets/context_json.rs` | JSON context fuzzing |
| `fuzz/fuzz_targets/store_episode.rs` | Episode storage fuzzing |
| `fuzz/fuzz_targets/cosine_similarity.rs` | Cosine similarity fuzzing |

---

## Cross-References

- **Architecture Blueprint** (Phase 6): Component topology, data model, retrieval pipeline stages, graceful degradation chain
- **ADR-001** (SQLite): Single-file storage, WAL mode -- informs persistence tests
- **ADR-004** (Traits): ConsolidationProvider, NoOpProvider -- informs mock provider strategy
- **ADR-005** (Bjork): Dual-strength model parameters -- informs property test invariants
- **ADR-006** (RRF): k=60, score-agnostic fusion -- informs fusion property tests
- **ADR-007** (Vasana): Crystallization threshold, impression pipeline -- informs perfuming invariant tests
- **ADR-009** (Zero Network): No HTTP/DNS/socket -- CI enforcement via cargo tree
- **ADR-010** (FTS5): Input sanitization -- informs fuzz target priority
- **Accessibility** (Phase 5c): Doctest coverage, quickstart regression, error path testing, cross-platform CI
- **Security Architecture** (Phase 8): FTS5 injection, memory resurrection, provider injection -- informs fuzz targets and adversarial tests
