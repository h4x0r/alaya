# Alaya Technology Stack Research

## Generated
2026-02-26T12:50:00+08:00

## 1. Rust Library Crate Design

### Recommendation: Follow Official Rust API Guidelines + thiserror

Current state is well-designed: single entry point `AlayaStore`, `Result<T>` alias, `thiserror` enum, clean `ConsolidationProvider` trait.

Key practices:
- Accept `impl AsRef<Path>`, `&str`, `&[T]` for inputs
- Builder pattern for `Query` and `NewEpisode`
- Derive `Debug`, `Clone`, `PartialEq`, `Default` on all public types
- `#[non_exhaustive]` on public enums
- `pub(crate)` for internal modules
- `cargo-semver-checks` in CI

Error handling:
- Add `#[non_exhaustive]` to `AlayaError`
- Improve `Provider(String)` to `Provider(Box<dyn std::error::Error + Send + Sync>)`
- Consider boxing the entire error enum if it grows large

## 2. SQLite in Rust

### Recommendation: rusqlite with `bundled` feature, upgrade to 0.38

rusqlite wins over sqlx for embedded library: no async runtime, minimal deps, simple FFI.

Upgrade to 0.38 breaking changes:
- `Connection::execute` rejects multi-statement SQL -- use `execute_batch`
- Statement cache now optional -- add `cache` feature
- Minimum bundled SQLite is 3.51.1

Recommended config:
```toml
rusqlite = { version = "0.38", features = ["bundled", "cache", "vtab"] }
```

Connection management: WAL mode (already enabled), single writer + multiple readers via `Mutex<Connection>`.

## 3. FTS5 Best Practices

### Recommendation: External content FTS5 (already correct)

Enhancements:
1. Use `ORDER BY rank` for BM25-sorted results
2. Add porter stemming: `tokenize='porter unicode61'`
3. Sanitize user input before MATCH (wrap in double quotes)
4. Add FTS5 to `semantic_nodes` table
5. Consider `prefix='2,3'` for autocomplete

## 4. Vector Similarity Search

### Recommendation: Tiered approach

- **Tier 1 (default):** Pure Rust brute-force cosine, works up to ~10K vectors
- **Tier 2 (feature flag):** sqlite-vec for SIMD-accelerated KNN
- **Tier 3 (optional):** HNSW via instant-distance or rust-cv/hnsw for 50K+ vectors

Expose `EmbeddingIndex` trait for custom implementations.

## 5. FFI Best Practices

### Recommendation: cbindgen + UniFFI in separate crates

- **Tier 1:** C ABI via cbindgen (opaque handle pattern, error codes)
- **Tier 2:** UniFFI for Kotlin/Swift/Python/Ruby
- **Tier 3:** PyO3 + maturin for Python-specific bindings

Ship `alaya-ffi` as separate crate. Core library never depends on FFI tooling.

## 6. Graph Data Structures

### Recommendation: Adjacency list with recursive CTEs (already correct)

Enhancements:
1. Recursive CTE for spreading activation with decay and depth limits
2. Cycle prevention via path tracking
3. Covering index: `(source_type, source_id, target_type, target_id, forward_weight)`
4. Performance viable up to ~100K edges with 3-5 hop depth limits

## 7. Embedding Model Integration

### Recommendation: `EmbeddingProvider` trait with optional ONNX backend

Design an `EmbeddingProvider` trait with `embed()`, `embed_batch()`, `dimensions()`, `model_id()`.

Optional backends:
- `embedding-onnx` feature using `ort` crate
- `embedding-candle` feature using Hugging Face's pure Rust ML framework
- `fastembed-rs` for turnkey ONNX-based embeddings

## 8. Benchmarking

### Recommendation: Divan

Divan over Criterion: attribute-based API, allocation profiling, better CI friendliness.

Key benchmarks: episode insertion, FTS5 query latency, vector search at scale, spreading activation, full pipeline E2E, consolidation cycle.

## 9. Comparable Rust Crate Patterns

- **Tantivy:** Builder pattern for schema, immutable data model, Directory trait abstraction
- **redb:** Transaction-based API, configuration builder, pure Rust validation
- **sled:** Config builder, iterator-based API

## 10. Consolidated Dependencies

```toml
rusqlite = { version = "0.38", features = ["bundled", "cache", "vtab"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
```

Architecture: core lib + optional `alaya-ffi` + optional `alaya-py` crates.

## Sources

- Rust API Guidelines, Rust API Guidelines Checklist
- Elegant Library APIs in Rust (deterministic.space)
- Rusqlite GitHub, r2d2_sqlite docs
- SQLite FTS5 Extension Documentation
- sqlite-vec GitHub and Rust Documentation
- FFI - The Rustonomicon, cbindgen guide
- UniFFI User Guide, PyO3 User Guide
- Divan GitHub, Criterion.rs GitHub
- Tantivy GitHub, redb crates.io, sled GitHub
