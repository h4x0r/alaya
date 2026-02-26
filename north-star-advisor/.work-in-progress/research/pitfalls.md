# Common Pitfalls in AI Agent Memory Libraries

## Generated
2026-02-26T12:50:00+08:00

## 1. Memory System Design Pitfalls

### Context Flooding ("Dumb RAG")
Dumping all data into vector storage hoping the LLM sorts it out. Prevention: relevance-gated retrieval, optimize for context precision not volume, configurable retrieval limits.

### Monolithic Memory Store
Treating all memory types as undifferentiated blob. Prevention: clear separation between stores, atomic transactions across types via SQLite.

### Over-Engineering for Multi-User/Enterprise
Building multi-tenant infrastructure before cognitive model is proven. Already an anti-goal for Alaya.

### Premature Abstraction
Wrong abstractions become load-bearing. Prevention: start concrete with SQLite, prefer concrete types over trait objects until patterns stabilize.

## 2. SQLite Pitfalls

### WAL Checkpoint Stalls
WAL grows without bound if checkpoints not performed. Prevention: periodic `PRAGMA wal_checkpoint(PASSIVE)`, `TRUNCATE` checkpoint on init, expose `checkpoint()` method, set `PRAGMA journal_size_limit`.

### Deferred Transaction Upgrade Trap (CRITICAL)
`BEGIN` acquires read lock; write upgrade can fail with `SQLITE_BUSY` WITHOUT invoking busy handler. Prevention: ALWAYS use `BEGIN IMMEDIATE` for write transactions.

### Busy Timeout Misconfiguration
Default timeout can change; too low causes spurious failures. Prevention: explicitly set `busy_timeout` on every connection, make configurable, log events.

### Thread Safety
`rusqlite::Connection` is `Send` but NOT `Sync`. Prevention: single-writer/multiple-reader pattern, `SQLITE_OPEN_READONLY` for readers, `spawn_blocking` for async integration.

### In-Memory Pool Trap
Each pool connection gets separate empty database. Prevention: use `file::memory:?cache=shared` for shared in-memory.

### Startup Race Condition
Multiple simultaneous connection opens can trigger BUSY during WAL recovery. Prevention: open writer first, perform migration, then open readers.

## 3. Security Vulnerabilities

### Memory Poisoning (OWASP ASI06)
MINJA attack achieves 95%+ injection success. Memory poisoning persists across sessions. Prevention: content validation hooks, content-hash integrity checks, quarantine API, document attack surface.

### FTS5 Query Injection (CRITICAL)
FTS5 has its own query language; even parameterized SQL doesn't protect MATCH operator from malformed expressions. Prevention: sanitize FTS5 strings (wrap in double quotes), expose safe `search()` API, never let raw input reach MATCH.

### Data Leakage Between Sessions
Shared database can leak memories across contexts. Prevention: mandatory `context_id` on all tables, enforce context isolation at query layer, optional per-context SQLite files.

### Embedding Poisoning
AgentPoison: poisoning <0.1% of RAG index achieves 80%+ attack success. Prevention: metadata tracking, embedding re-generation capability, content-hash verification.

## 4. Privacy Pitfalls

### PII in Stored Episodes
Episodic memories naturally capture PII. Prevention: PII scrubbing hook, document all persistence locations, support field-level encryption.

### GDPR Right-to-Deletion
GDPR requires deletion within 1 month; EU AI Act requires 10-year audit trails. Prevention: crypto-shredding, surrogate keys, `forget(entity_id)` API, Forgettable Payload pattern, `VACUUM` after deletion.

### Memory Forensics Exposure
SQLite delete doesn't zero data on disk. Prevention: `VACUUM` after GDPR deletion, `PRAGMA secure_delete = ON`, recommend full-disk encryption.

## 5. Performance Pitfalls

### Brute-Force Vector Search Degradation
O(N) linear scan degrades at scale. Prevention: start brute-force, design swappable index API, document degradation curve (~50K threshold).

### HNSW Recall Degradation
99% recall at 10K may drop to 85% at 10M. Prevention: expose tuning parameters, periodic index rebuilding, monitor recall metrics.

### FTS5 Index Bloat
Fragmented segments degrade over time. Prevention: configure `automerge`, incremental merges during idle, `optimize` after bulk ops, mark non-searchable columns `UNINDEXED`.

### Graph Traversal Explosion
3-hop query on dense graph can touch millions of edges. Prevention: maximum traversal depth, visited-node tracking, edge weight thresholds, incremental results with limits.

## 6. API Design Pitfalls

### Semver Violations
1 in 6 top Rust crates has violated semver. Prevention: `cargo-semver-checks` in CI, `#[non_exhaustive]`, prefer opaque types, reach 1.0 when API stabilizes.

### Feature Flag Explosion
Combinatorial testing burden. Prevention: limit to genuinely optional heavy deps, thoughtful defaults, test feature matrix in CI.

### Compile Time Bloat
Heavy deps increase compile times. Prevention: optional feature flags for heavy deps, minimize monomorphization, audit with `cargo tree`, profile with `cargo build --timings`.

## 7. Forgetting System Pitfalls

### Over-Aggressive Decay
Destroys important infrequent memories. Prevention: differential decay rates (importance-based), multiple signal importance scoring, cold storage tier before deletion.

### Catastrophic Forgetting of Consolidated Knowledge
If consolidation was wrong, ground truth is lost. Prevention: retain source episode references, consolidation confidence score, support "unconsolidation".

### Memory Resurrection After Deletion
Deleted memories can be re-derived by consolidation. Prevention: tombstone table, hard deletes not soft deletes, cascade deletion across all tables, verify no references remain.

## 8. Graph Overlay Pitfalls

### Runaway Edge Accumulation
Without pruning, edge count grows super-linearly. Prevention: max edge count per node, periodic weight-threshold pruning, KGTrimmer-style importance scoring.

### Weight Overflow
Popular edges accumulate unbounded weight creating hubs. Prevention: periodic normalization, multiplicative decay, logarithmic reinforcement.

### Disconnected Subgraphs
Pruning can fragment graph into islands. Prevention: periodic connected-component checks, bridge edges with higher pruning resistance.

## 9. Testing Pitfalls

### Non-Deterministic Time-Dependent Tests
Memory decay is time-dependent. Prevention: inject `Clock` trait, mock clock in tests, never call `SystemTime::now()` directly.

### Decay Function Testing
Floating-point equality is flaky. Prevention: property-based testing, approximate equality with epsilon, test at fixed time deltas.

### SQLite State Leakage
Tests sharing database interfere. Prevention: unique tempfile per test, no shared databases in parallel tests.

## Top 10 Most Critical Pitfalls

| Rank | Pitfall | Severity | Likelihood |
|------|---------|----------|------------|
| 1 | Deferred transaction upgrade SQLITE_BUSY | Critical | High |
| 2 | FTS5 query injection via MATCH | Critical | High |
| 3 | Memory resurrection after deletion | Critical | Medium |
| 4 | Over-aggressive decay | High | High |
| 5 | Hallucinated semantic knowledge from LLM consolidation | High | High |
| 6 | Non-deterministic tests from system clock | High | High |
| 7 | WAL unbounded growth | High | Medium |
| 8 | Deduplication false positives | High | Medium |
| 9 | Runaway graph edge accumulation | Medium | High |
| 10 | Semver violations | Medium | Medium |

## Sources

- OWASP ASI06 (2026 Agentic Applications Top 10)
- MINJA attack paper, AgentPoison (Chen et al. 2024)
- FadeMem: Biologically-Inspired Forgetting (arXiv)
- SQLite FTS5 Documentation
- Rusqlite GitHub (issues, discussions)
- Martin Fowler: Eradicating Non-Determinism in Tests
- EU AI Act (August 2026)
- Knowledge Graph Pruning (IJCAI 2018, KGTrimmer)
- SemVer in Rust (FOSDEM 2024, cargo-semver-checks)
- Rust API Guidelines
