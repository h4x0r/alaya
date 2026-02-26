# Action Roadmap: Alaya Memory Engine

**Generated:** 2026-02-26 | **Phase:** 12 (Action Roadmap)
**Status:** Active | **Review Date:** 2026-04-01
**Path:** B -- Developer Experience First, with Benchmark Fast-Follow
**Confidence:** 8/10

---

## How to Read This Document

This is the execution plan for the Alaya project. It translates the strategic recommendation (Phase 11) into week-by-week tasks with concrete file paths, code changes, and measurable completion criteria. Every task traces back to a known gap, a security threat, a strategic decision, or a competitive requirement documented in preceding phases.

The audience is the solo maintainer making daily task-selection decisions. When you sit down to work on Alaya, open this document, find the current week, and pick the next unchecked task. When a review checkpoint arrives, evaluate the decision criteria listed for that checkpoint and adjust accordingly.

**Cross-reference index:**
- Strategic recommendation: `recommendation.yml` (path selection, sequence, avoid list)
- North Star: `northstar.yml` (MACC metric, phases, personas)
- Competition: `competitive.yml` (market timing, OpenClaw window)
- Scaffold: `scaffold.yml` (module tree, known gaps GAP-001 through GAP-013)
- Testing: `testing.yml` (coverage gaps, benchmark plans, CI pipeline)
- Post-publication: `post-deployment.yml` (release gates, quality metrics)
- Security: `security.yml` (threats T1-T9, hardening priorities)
- Architecture decisions: `adr.yml` (ADR-001 through ADR-010)

---

## Part 1: Strategic Context Summary

### 1.1 North Star Reminder

**Metric:** Monthly Active Crate Consumers (MACC) -- unique Rust projects calling `AlayaStore::open()` and executing `store_episode()` + `query()` in a 30-day period.

**Targets:**

| Version | MACC Target | Timeline |
|---------|-------------|----------|
| v0.1.0  | 5           | 6-8 weeks from now |
| v0.2.0  | 25          | 6-8 weeks after v0.1 |
| v0.3.0  | 100         | 3-6 months after v0.2 |

**Proxy measurement sources:** crates.io downloads (weekly), GitHub dependents (monthly), non-maintainer issues (continuous), reverse dependency search (monthly).

MACC cannot be measured directly for a library crate. The proxy signals are imperfect but collectively diagnostic. A library with steady weekly downloads and no issues is more likely experiencing bot traffic than genuine consumption. A library with low downloads but active issues has real users.

### 1.2 Strategic Moves in Play

Four offensive moves from the competitive analysis are active during this roadmap period:

1. **O1: Benchmark-first credibility** -- Publish LoCoMo numbers before any marketing language. The benchmark blog post is the marketing. This move is scheduled for Phase 4 (weeks 7-9) per the recommended path.

2. **O2: OpenClaw integration** -- Engage with a published, working crate and benchmark data. Not a pitch deck. Not a prototype. A crate they can `cargo add` and run. Outreach is scheduled for Phase 3 (week 6) with follow-up in Phase 4 (week 9).

3. **O3: Zero dependencies as brand identity** -- Every piece of documentation, every example, every blog post should demonstrate that `cargo add alaya` is the entire setup story. This is not a talking point; it is a structural fact verified by `cargo tree`.

4. **D2: Benchmark parity monitoring** -- After LoCoMo baseline is established, re-run benchmarks per release. Mem0 at 68.5% LoCoMo is the floor to contextualize against (not necessarily beat -- honest numbers are the brand).

### 1.3 Current Phase Assessment

**Where we are:** Pre-publication. The codebase is functional (4,064 lines, 25 files, 43 passing tests) with a sound architecture (10 accepted ADRs) and defensible positioning (unoccupied quadrant: high cognitive completeness + high operational simplicity). But it has 13 identified hardening gaps, zero documentation beyond doc comments, zero external users, and no CI pipeline.

**What must be true before publication:**
- All P0 gaps closed (GAP-001 through GAP-005)
- All P1 gaps closed (GAP-006 through GAP-009)
- CI pipeline operational (GAP-012)
- MSRV verified and pinned (GAP-013)
- README with working quickstart
- At least one complete example in `examples/`
- `cargo publish --dry-run` succeeds
- All release gates from `post-deployment.yml` pass

**What is explicitly deferred past publication:**
- WAL checkpoint management (GAP-010) -- can ship without, deferred to v0.1.x
- Content-hash integrity column (GAP-011) -- nice-to-have, not blocking
- Benchmarks -- published as fast-follow within 3 weeks
- Feature flags -- v0.2 scope
- MCP server -- v0.2 scope
- Python bindings -- v0.3 scope

### 1.4 Axiom Guardrails

Every task in this roadmap has been filtered through the axiom hierarchy. When in doubt about task priority, apply this resolution order:

1. **Privacy > Features** -- No task should introduce network calls, telemetry, or data exfiltration paths.
2. **Process > Storage** -- The cognitive lifecycle (consolidation, forgetting, perfuming, transformation) is the product. Tasks that strengthen lifecycle integrity take priority over new storage features.
3. **Correctness > Speed** -- BEGIN IMMEDIATE before performance benchmarks. Input validation before new API methods. Research grounding before optimization.
4. **Simplicity > Completeness** -- Ship with zero deps, not with every feature. Every task that adds complexity must justify itself against the alternative of shipping sooner.
5. **Honesty > Marketing** -- Publish benchmark numbers even when they are bad. Document known limitations in the README. Never use superlatives without evidence.

---

## Part 2: The Next 30 Days

### 2.1 Phase 1: P0 Hardening (Weeks 1-2)

**Goal:** Close all 5 P0 gaps that represent semver safety, transaction integrity, and API contract risks. After this phase, the public API surface is safe to stabilize for v0.1.

**Estimated effort:** 16-24 hours of focused development.

#### Week 1: Semver Safety and Transaction Integrity

##### Task 1.1: Add `#[non_exhaustive]` to all public enums (GAP-001)

**Files:** `src/types.rs`, `src/error.rs`
**Traces to:** GAP-001, ADR-008 (Sync-First API), security threat T3 (memory resurrection -- tombstone support requires new enum variant)
**Estimated time:** 30 minutes

**What to change:**

In `src/types.rs`, add `#[non_exhaustive]` to every public enum. The current enums are:

```rust
// src/types.rs -- NodeRef enum (line ~27)
#[non_exhaustive]  // ADD THIS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeRef {
    Episode(EpisodeId),
    Semantic(NodeId),
    Preference(PreferenceId),
}
```

Apply the same attribute to: `Role`, `SemanticType`, `LinkType`, `PurgeFilter`.

In `src/error.rs`, add `#[non_exhaustive]` to `AlayaError`:

```rust
// src/error.rs -- AlayaError enum (line 3)
#[non_exhaustive]  // ADD THIS
#[derive(Debug, Error)]
pub enum AlayaError {
    // ...
}
```

**Completion criteria:**
- All 6 public enums annotated with `#[non_exhaustive]`
- All 43 existing tests still pass
- `cargo doc` builds without warnings

**Why this is P0:** Without `#[non_exhaustive]`, adding any variant to any public enum is a semver-breaking change. Since we know we will add variants (tombstone node types, new error variants for validation, new semantic types), this must be in place before v0.1 publication locks the API surface.

##### Task 1.2: Implement BEGIN IMMEDIATE for write transactions (GAP-002)

**Files:** `src/schema.rs`, `src/lib.rs`
**Traces to:** GAP-002, ADR-001 (SQLite as Sole Storage Engine), security threat T7 (transaction deadlock)
**Estimated time:** 2-3 hours

**What to change:**

In `src/schema.rs`, add a helper function:

```rust
/// Begin a write transaction with IMMEDIATE locking.
///
/// SQLite WAL mode with BEGIN DEFERRED can deadlock when a read
/// transaction is promoted to write. BEGIN IMMEDIATE acquires the
/// write lock immediately, failing fast with SQLITE_BUSY instead
/// of deadlocking.
pub(crate) fn immediate_transaction(conn: &Connection) -> Result<Transaction<'_>> {
    conn.execute_batch("BEGIN IMMEDIATE")?;
    // Safety: we started the transaction manually; Transaction::new_unchecked
    // is not available, so use the behavior flag approach
    Ok(conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?)
}
```

Note: The exact implementation depends on rusqlite's API. The alternative is to use `conn.transaction_with_behavior(TransactionBehavior::Immediate)` directly at each call site. Either approach works. The goal is to ensure every method that writes to the database uses IMMEDIATE, not DEFERRED.

In `src/lib.rs`, update every write method to use the immediate transaction helper. The write methods are:
- `store_episode()` (line ~55)
- `consolidate()` (line ~95)
- `perfume()` (line ~110)
- `transform()` (line ~125)
- `forget()` (line ~135)
- `purge()` (line ~145)

**Completion criteria:**
- All write methods use BEGIN IMMEDIATE
- New test: concurrent write from two threads does not deadlock (requires `Arc<Mutex<AlayaStore>>` pattern)
- All 43 existing tests still pass
- `cargo clippy -- -W clippy::pedantic` clean

**Why this is P0:** The deadlock scenario is reproducible: wrap AlayaStore in `Arc<Mutex<_>>`, spawn two threads calling `store_episode()`, and observe that under DEFERRED the read-to-write promotion can fail silently or deadlock. This is not a theoretical risk -- it is the standard multi-threaded usage pattern for an embedded database library.

##### Task 1.3: Add input validation at API boundary (GAP-003)

**Files:** `src/lib.rs`
**Traces to:** GAP-003, security threats T1 (memory poisoning), T9 (provider output injection)
**Estimated time:** 3-4 hours

**What to change:**

Add a validation module or inline validation functions in `src/lib.rs` that check inputs before they reach the storage layer. Validation rules:

For `store_episode(&self, episode: &NewEpisode)`:
- `episode.content` must not be empty (return `AlayaError::InvalidInput`)
- `episode.content.len()` must not exceed 1,000,000 bytes (configurable in future; hard limit for now)
- `episode.session_id` must not be empty
- `episode.timestamp` must be positive (Unix seconds)

For `query(&self, q: &Query)`:
- `q.text` must not be empty if no embedding is provided
- `q.max_results` must be in range 1..=100
- If `q.embedding` is provided, validate no NaN/Inf values, validate non-zero norm

For `consolidate()`, `perfume()`:
- Validate provider output: non-empty content strings, confidence in [0.0, 1.0], source episode IDs actually exist

For embedding-related inputs:
- Dimension consistency (all embeddings for a store must have same dimension)
- NaN/Infinity rejection
- Zero-norm rejection

**Completion criteria:**
- Each validation rule has a corresponding unit test that exercises the error path
- `AlayaError::InvalidInput` messages follow the pattern "what happened: why: what to do"
- At least 8 new tests covering validation paths
- All 43 existing tests still pass (existing valid inputs should continue working)

**Why this is P0:** Without input validation, any garbage input passes through to SQLite, potentially corrupting the FTS5 index, producing meaningless embeddings, or creating nodes with empty content that pollute retrieval results. The library's public API is the trust boundary.

##### Task 1.4: Change internal modules to `pub(crate)` (GAP-004)

**Files:** `src/lib.rs`
**Traces to:** GAP-004, semver safety
**Estimated time:** 30 minutes

**What to change:**

In `src/lib.rs`, change all internal module declarations from `pub mod` to `pub(crate) mod`:

```rust
// src/lib.rs -- module declarations (lines 10-17)
pub mod error;          // KEEP pub (re-exported)
pub mod types;          // KEEP pub (re-exported)
pub(crate) mod schema;  // CHANGE from pub
pub(crate) mod store;   // CHANGE from pub
pub(crate) mod graph;   // CHANGE from pub
pub(crate) mod retrieval; // CHANGE from pub
pub(crate) mod lifecycle; // CHANGE from pub
pub mod provider;       // KEEP pub (re-exported)
```

The modules that stay `pub` are those whose types or traits are part of the public API: `error` (AlayaError), `types` (all public types), and `provider` (ConsolidationProvider, NoOpProvider). Everything else is implementation detail.

**Completion criteria:**
- Internal modules not accessible from external code
- All 43 existing tests still pass (internal tests access via `crate::` paths)
- `cargo doc` only shows public API surface

**Why this is P0:** If internal modules are `pub`, consumers can depend on internal types and functions. Changing them later is a semver break. This must happen before publication locks the API surface.

#### Week 2: Documentation Foundation

##### Task 1.5: Add compilable doctests on all public methods (GAP-005)

**Files:** `src/lib.rs` (all 12 public methods)
**Traces to:** GAP-005, Priya persona (first contact with API is docs.rs), post-publication release gate
**Estimated time:** 4-6 hours

**What to change:**

Every public method on `AlayaStore` needs a compilable doctest. The pattern:

```rust
/// Store a new episode in episodic memory.
///
/// # Example
///
/// ```
/// # use alaya::{AlayaStore, NewEpisode, Role};
/// let store = AlayaStore::open_in_memory()?;
/// let episode = NewEpisode {
///     content: "The user prefers dark mode.".into(),
///     role: Role::User,
///     session_id: "session-001".into(),
///     timestamp: 1700000000,
///     context: None,
/// };
/// let id = store.store_episode(&episode)?;
/// # Ok::<(), alaya::AlayaError>(())
/// ```
pub fn store_episode(&self, episode: &NewEpisode) -> Result<EpisodeId> {
```

Methods requiring doctests (12 total):
1. `open` -- Show file-based creation with tempdir
2. `open_in_memory` -- Show ephemeral creation
3. `store_episode` -- Show basic episode storage
4. `query` -- Show text query with results
5. `preferences` -- Show preference listing
6. `knowledge` -- Show semantic node listing
7. `neighbors` -- Show graph neighbor traversal
8. `consolidate` -- Show lifecycle with NoOpProvider
9. `perfume` -- Show impression pipeline
10. `transform` -- Show maintenance cycle
11. `forget` -- Show forgetting cycle
12. `status` -- Show memory status inspection
13. `purge` -- Show data deletion

Additionally, key types in `src/types.rs` need doc comments (not necessarily doctests, but clear `///` documentation describing each field).

**Completion criteria:**
- `cargo test --doc` passes with 0 failures
- Every public method has at least one compilable example
- Examples use `open_in_memory()` (no filesystem side effects)
- No `unwrap()` in doctests -- use `?` with `Ok::<(), alaya::AlayaError>(())`
- docs.rs preview (`cargo doc --no-deps --open`) shows examples for all methods

**Why this is P0:** docs.rs is the first thing a Rust developer sees. A method with no example communicates "this library is not ready for use." Priya (privacy-first agent developer) evaluates libraries by copying the quickstart example. If there is no example to copy, she moves on to the next crate in her search results.

##### Task 1.6: Add `tempfile` dev-dependency

**Files:** `Cargo.toml`
**Traces to:** scaffold.yml planned dependencies, persistence tests
**Estimated time:** 5 minutes

**What to change:**

```toml
[dev-dependencies]
tempfile = "3"
```

This enables file-backed database tests without polluting the filesystem. Used in the `open()` doctest and integration tests (Phase 2).

##### Task 1.7: Schema versioning with PRAGMA user_version

**Files:** `src/schema.rs`
**Traces to:** post-deployment.yml pre-publication blockers (schema versioning), breaking change response (schema migration)
**Estimated time:** 1-2 hours

**What to change:**

In the `open_db()` function in `src/schema.rs`, after running the schema initialization DDL:

```rust
// Set schema version for forward compatibility
conn.pragma_update(None, "user_version", 1)?;
```

And in the init path, check the existing version:

```rust
let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
match version {
    0 => { /* fresh database, run full init */ },
    1 => { /* current version, no migration needed */ },
    v => return Err(AlayaError::InvalidInput(
        format!("database schema version {v} is newer than this library supports (max: 1)")
    )),
}
```

**Completion criteria:**
- New databases get `user_version = 1`
- Existing v0 databases are migrated (currently version 0, which means fresh)
- Future schema changes have a migration path
- Test: open, close, reopen verifies version persists
- Test: version > current returns error

**Why this is needed before publication:** Without schema versioning, there is no way to detect or handle schema changes between Alaya versions. A user who upgrades from v0.1.0 to v0.1.1 with a schema change would get silent corruption or cryptic SQLite errors.

#### Phase 1 Completion Review (End of Week 2)

**Review checklist:**
- [ ] All 6 public enums have `#[non_exhaustive]`
- [ ] All write methods use BEGIN IMMEDIATE
- [ ] Input validation covers all public methods
- [ ] Internal modules are `pub(crate)`
- [ ] All 12 public methods have compilable doctests
- [ ] Schema versioning via PRAGMA user_version
- [ ] `tempfile` added to dev-dependencies
- [ ] Test count increased from 43 to approximately 65-75
- [ ] `cargo test` passes
- [ ] `cargo test --doc` passes
- [ ] `cargo clippy -- -W clippy::pedantic` clean
- [ ] No regressions in existing functionality

**Decision point:** If Phase 1 takes longer than 2 calendar weeks, evaluate whether to compress Phase 2 or extend the timeline. The critical constraint is: P0 gaps must be closed before moving to P1 work. Skipping hardening to reach publication faster violates `Correctness > Speed`.

---

### 2.2 Phase 2: P1 Quality and Documentation (Weeks 3-4)

**Goal:** Close the 4 P1 gaps that affect lifecycle correctness and retrieval completeness. Write the documentation that makes the library discoverable and usable. Expand test coverage toward the v0.1 target of 150 tests.

**Estimated effort:** 24-32 hours of focused development.

#### Week 3: Lifecycle Correctness and Retrieval Completeness

##### Task 2.1: Wire LTD in transform() (GAP-006)

**Files:** `src/lifecycle/transformation.rs`, `src/graph/links.rs`
**Traces to:** GAP-006, ADR-003 (Hebbian Graph Overlay -- LTD mechanism), lifecycle invariant I10 (idempotent on clean data)
**Estimated time:** 1-2 hours

**What to change:**

In `src/lifecycle/transformation.rs`, the `transform()` function currently calls dedup, link prune, preference decay, and impression prune. It does not call `decay_links()` from `src/graph/links.rs`, which implements Long-Term Depression (LTD) -- the Hebbian counterpart to LTP that weakens links that are not co-retrieved.

Add a call to `decay_links()` in the transform pipeline, positioned after dedup but before prune:

```rust
// In transform() function body:
// 1. Dedup semantic nodes
let dedup_count = dedup_semantic_nodes(conn)?;
// 2. LTD: decay link weights (NEW)
let decayed_count = graph::links::decay_links(conn, decay_factor)?;
// 3. Prune weak links
let pruned_count = graph::links::prune_links(conn, prune_threshold)?;
// ... rest of pipeline
```

The `decay_factor` should match the Hebbian parameters from ADR-003: link weights are multiplied by 0.95 per transform cycle, and links below the prune threshold of 0.02 are removed.

**Completion criteria:**
- `transform()` calls `decay_links()` before `prune_links()`
- `TransformationReport` includes decayed link count
- Test: create links, run transform, verify weights decreased
- Test: weak link (below 0.02) is pruned after decay
- Existing transform tests still pass

##### Task 2.2: Semantic and preference node enrichment in retrieval pipeline (GAP-007)

**Files:** `src/retrieval/pipeline.rs`
**Traces to:** GAP-007, architecture pipeline stages (Enrichment stage), three-store architecture (ADR-002)
**Estimated time:** 3-4 hours

**What to change:**

Currently, the retrieval pipeline only returns episode-type nodes. When RRF fusion produces semantic or preference NodeRefs, they are dropped during enrichment because the enrichment stage only knows how to fetch episodes. This means the three-store architecture's value is not fully realized in retrieval.

In `src/retrieval/pipeline.rs`, update the enrichment stage to handle all three node types:

```rust
// For each NodeRef in fused results:
match node_ref {
    NodeRef::Episode(id) => {
        // existing: fetch episode content, metadata
    },
    NodeRef::Semantic(id) => {
        // NEW: fetch semantic node content, type, confidence
        // Create ScoredMemory with semantic node content
    },
    NodeRef::Preference(id) => {
        // NEW: fetch preference statement, confidence, evidence_count
        // Create ScoredMemory with preference content
    },
}
```

This requires either extending `ScoredMemory` to carry a `NodeRef` (to indicate source type) or adding a `source_type` field. The consumer can then distinguish between episode memories, semantic knowledge, and emergent preferences in their query results.

**Completion criteria:**
- Semantic nodes appear in query results when relevant
- Preference nodes appear in query results when relevant
- ScoredMemory includes source type information
- Test: store episodes, consolidate (producing semantic nodes), query, verify semantic nodes in results
- Test: accumulate impressions, crystallize preference, query, verify preference in results
- Existing retrieval tests still pass
- Graceful degradation: if no semantic/preference nodes exist, behavior unchanged

##### Task 2.3: Tombstone mechanism for deleted nodes (GAP-008)

**Files:** `src/schema.rs`, `src/store/episodic.rs`, `src/lifecycle/consolidation.rs`
**Traces to:** GAP-008, security threat T3 (memory resurrection), GDPR right to erasure
**Estimated time:** 3-4 hours

**What to change:**

In `src/schema.rs`, add a tombstones table to the schema initialization:

```sql
CREATE TABLE IF NOT EXISTS tombstones (
    id          INTEGER PRIMARY KEY,
    node_type   TEXT NOT NULL,          -- 'episode', 'semantic', 'preference'
    node_id     INTEGER NOT NULL,
    content_hash TEXT,                   -- SHA-256 of original content (optional)
    deleted_at  INTEGER NOT NULL,        -- Unix timestamp
    UNIQUE(node_type, node_id)
);
```

When a node is deleted (via `purge()` or any deletion path):
1. Insert a tombstone record before deleting the node
2. Cascade-delete related data (embeddings, links, strengths, FTS5 entries)

When `consolidate()` produces new semantic nodes:
1. Check content against tombstones (by content hash if available, by source episode IDs otherwise)
2. If a tombstone match is found, skip the node (prevent resurrection)

**Completion criteria:**
- Tombstones table exists in schema
- Deletion creates tombstone before removing node
- Consolidation checks tombstones before creating nodes
- Test: store episode, consolidate (creates semantic node), purge episode, re-consolidate, verify no resurrection
- Test: purge all, verify tombstones survive (consumer can clear tombstones explicitly)
- Schema version incremented from 1 to 2 (with migration for existing databases)

##### Task 2.4: RIF suppression in retrieval pipeline (GAP-009)

**Files:** `src/retrieval/pipeline.rs`, `src/store/strengths.rs`
**Traces to:** GAP-009, ADR-005 (Bjork Dual-Strength Forgetting -- RIF), competitive differentiator
**Estimated time:** 2-3 hours

**What to change:**

Retrieval-Induced Forgetting (RIF) is a key differentiator of the Bjork dual-strength model: when a memory is retrieved, its competitors (memories similar in content but not retrieved) have their retrieval strength suppressed. The `suppress()` function exists in `src/store/strengths.rs` but is not called from the retrieval pipeline.

In `src/retrieval/pipeline.rs`, after the post-retrieval strength update (which boosts retrieved nodes):

```rust
// Post-retrieval: boost retrieved nodes (existing)
for result in &results {
    strengths::on_access(conn, result.node_ref)?;
}

// RIF: suppress competitors of retrieved nodes (NEW)
// Competitors are nodes that matched the query but were NOT in the top results
let retrieved_set: HashSet<NodeRef> = results.iter().map(|r| r.node_ref).collect();
for candidate in &all_candidates {
    if !retrieved_set.contains(&candidate.node_ref) {
        strengths::suppress(conn, candidate.node_ref, rif_factor)?;
    }
}
```

The `rif_factor` should reduce retrieval strength by a small amount (0.05-0.1). This is tunable and should be documented as a constant.

**Completion criteria:**
- Non-retrieved candidates have RS reduced after query
- Retrieved nodes have RS boosted (existing behavior preserved)
- Test: query returns A over B, verify B's RS decreased while A's RS increased
- Test: repeated queries amplify the gap between A and B
- RIF factor is a named constant, not a magic number
- Existing retrieval tests still pass

#### Week 4: Documentation and Test Expansion

##### Task 2.5: README rewrite with quickstart

**Files:** `README.md` (root)
**Traces to:** Priya persona (first contact), Marcus persona (benchmark evaluation), competitive.yml (zero-dep brand identity)
**Estimated time:** 3-4 hours

**Structure:**

```markdown
# Alaya

Embeddable Rust memory engine with cognitive lifecycle processes for AI agents.

## What It Does
[2-3 sentences: three stores, lifecycle, zero deps, single file]

## Quickstart
[cargo add, 10-line example, expected output]

## Why Alaya
[Table: Alaya vs Mem0 vs Engram vs build-from-scratch]

## Architecture
[One paragraph + link to architecture doc]

## Status
[Current version, what works, known limitations, roadmap link]

## License
MIT
```

The quickstart must be a complete, compilable example that demonstrates:
1. Create a store
2. Store an episode
3. Query for it
4. See the result

Nothing else. No lifecycle methods in the quickstart. No provider traits. No embeddings. The goal is `<2 minutes from cargo add to working code`.

**Completion criteria:**
- README compiles as a standalone Rust program when pasted into `main.rs`
- Time from `cargo add alaya` to working output is under 2 minutes (measured manually)
- No marketing language (no "revolutionary", "cutting-edge", "AI-powered")
- Known limitations section is honest
- Word count: 300-600 words (short; every word earns its place)

##### Task 2.6: Examples directory

**Files:** `examples/basic_agent.rs`, `examples/lifecycle_demo.rs`, `examples/custom_provider.rs`
**Traces to:** design.yml journeys (First-Time Developer, Deepening Integration), accessibility (skill levels)
**Estimated time:** 3-4 hours

Three examples, progressing in complexity:

1. **`examples/basic_agent.rs`** -- Store episodes, query, print results. Demonstrates the 3-method core (open, store, query). ~30 lines. For Priya on day 1.

2. **`examples/lifecycle_demo.rs`** -- Store multiple sessions, run consolidate/perfume/transform/forget, show how memory evolves. Demonstrates the lifecycle value proposition. ~80 lines. For Priya on day 2.

3. **`examples/custom_provider.rs`** -- Implement a mock ConsolidationProvider that extracts keywords as semantic nodes. Demonstrates the trait extension model. ~100 lines. For Marcus evaluating architecture.

Each example must:
- Compile with `cargo run --example <name>`
- Use `open_in_memory()` (no filesystem side effects)
- Print clear output showing what happened
- Include comments explaining each step
- Not use any external dependencies beyond alaya

**Completion criteria:**
- All 3 examples compile and run
- Output is self-explanatory
- No `unwrap()` -- proper error handling with `?`
- Each example is self-contained (copy-pasteable)

##### Task 2.7: Cargo.toml metadata completion

**Files:** `Cargo.toml`
**Traces to:** GAP-013 (partial), post-deployment.yml release gates, crates.io discoverability
**Estimated time:** 30 minutes

The current `Cargo.toml` already has basic metadata. Additions needed:

```toml
[package]
name = "alaya"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"                    # ADD: MSRV
authors = ["Albert Hui <albert@securityronin.com>"]
description = "Embeddable memory engine with cognitive lifecycle for AI agents"
license = "MIT"
repository = "https://github.com/h4x0r/alaya"
homepage = "https://github.com/h4x0r/alaya"  # ADD
documentation = "https://docs.rs/alaya"       # ADD
readme = "README.md"                           # ADD
keywords = ["memory", "ai", "agent", "sqlite", "embedding"]  # UPDATE: more specific
categories = ["database", "science"]

[package.metadata.docs.rs]                # ADD: docs.rs config
all-features = true
rustdoc-args = ["--cfg", "docsrs"]

[lints.clippy]                            # ADD: lint configuration
pedantic = { level = "warn", priority = -1 }
```

##### Task 2.8: CHANGELOG.md

**Files:** `CHANGELOG.md`
**Traces to:** post-deployment.yml release gates (manual), semver policy
**Estimated time:** 20 minutes

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - YYYY-MM-DD

### Added
- Three-store memory architecture (episodic, semantic, implicit)
- Hebbian graph overlay with LTP/LTD dynamics
- Hybrid retrieval pipeline (BM25 + vector + graph -> RRF fusion -> reranking)
- Bjork dual-strength forgetting with retrieval-induced forgetting (RIF)
- Vasana preference emergence (impressions -> preferences)
- CLS-inspired consolidation (episodes -> semantic nodes)
- Transformation pipeline (dedup, prune, decay)
- ConsolidationProvider trait with NoOpProvider fallback
- Graceful degradation (full -> BM25+graph -> BM25-only -> empty)
- GDPR-compatible purge() with session, time, and full-reset filters
- Input validation at API boundary
- Schema versioning (PRAGMA user_version)
- Compilable doctests on all public methods
```

##### Task 2.9: Expand test coverage

**Files:** Various test modules across the codebase
**Traces to:** testing.yml unit_coverage gaps, post-deployment.yml quality metrics (80% target)
**Estimated time:** 6-8 hours (spread across week 4)

Priority test additions (from testing.yml coverage gaps), ordered by risk:

**High priority (security and correctness):**
1. `src/store/embeddings.rs` -- NaN handling, dimension mismatch, zero vector (3 tests)
2. `src/store/strengths.rs` -- upper bounds, RS reset on access, archive safety (3 tests)
3. `src/retrieval/bm25.rs` -- FTS5 injection prevention with adversarial strings (2 tests)
4. `src/retrieval/pipeline.rs` -- BM25-only degradation, empty database, hybrid query (3 tests)

**Medium priority (lifecycle invariants):**
5. `src/lifecycle/consolidation.rs` -- NoOp produces nothing, provider error propagation (2 tests)
6. `src/lifecycle/forgetting.rs` -- accessed nodes resist forgetting, skips preferences (2 tests)
7. `src/lifecycle/transformation.rs` -- idempotent on clean data, dedup preserves stronger (2 tests)
8. `src/lifecycle/perfuming.rs` -- below threshold no crystallization, multi-domain isolation (2 tests)

**Lower priority (type safety and error paths):**
9. `src/types.rs` -- NodeRef roundtrip, Role roundtrip, Query::simple defaults (3 tests)
10. `src/error.rs` -- display messages, From impls (3 tests)
11. `src/provider.rs` -- NoOp all methods, MockProvider returns data (2 tests)

**Target:** 43 (current) + ~40 new = ~83 unit tests by end of Phase 2. Integration tests (20 target) deferred to Phase 2.5 / ongoing.

**Completion criteria:**
- Test count >= 80
- All tests pass
- No modules with 0 tests (types.rs, error.rs, provider.rs currently at 0)
- Security-critical paths (FTS5, embeddings, strengths) have adversarial input tests

##### Task 2.10: Add proptest for mathematical invariants

**Files:** `Cargo.toml` (add proptest dev-dep), new test files
**Traces to:** testing.yml property_tests (15 target for v0.1)
**Estimated time:** 3-4 hours

Add `proptest = "1.4"` to `[dev-dependencies]` and create property tests for:

1. **Bjork dual-strength** (5 properties):
   - Storage strength monotonic: SS_new >= SS for all SS in [0, 1]
   - Retrieval strength decay monotonic: RS * 0.95 <= RS
   - Storage strength convergence: approaches 1.0 after many accesses
   - Retrieval strength approaches zero after many cycles
   - Archival safety: nodes above thresholds are never archivable

2. **Hebbian weights** (4 properties):
   - LTP bounded: w + 0.1 * (1 - w) in [0, 1]
   - LTP monotonic: preserves weight ordering
   - LTP convergence: approaches 1.0 after many co-retrievals
   - Decay non-negative: w * factor >= 0

3. **RRF fusion** (2 properties):
   - Scores positive for non-empty inputs
   - Presence in more sets >= presence in fewer

4. **Cosine similarity** (4 properties):
   - Range: cos(a, b) in [-1, 1] for non-zero vectors
   - Symmetric: cos(a, b) == cos(b, a)
   - Self-identity: cos(a, a) ~= 1.0 for non-zero a
   - Embedding roundtrip: serialize/deserialize preserves bits

**Completion criteria:**
- 15 property tests implemented
- All pass with default proptest config (256 cases)
- No flaky tests

#### Phase 2 Completion Review (End of Week 4)

**Review checklist:**
- [ ] LTD wired in transform() (GAP-006 closed)
- [ ] Semantic/preference enrichment in pipeline (GAP-007 closed)
- [ ] Tombstone mechanism operational (GAP-008 closed)
- [ ] RIF wired in retrieval pipeline (GAP-009 closed)
- [ ] README with working quickstart
- [ ] 3 examples in examples/ directory
- [ ] Cargo.toml metadata complete
- [ ] CHANGELOG.md written
- [ ] Test count >= 80 (unit) + 15 (proptest)
- [ ] All P0 and P1 gaps closed
- [ ] `cargo test --all-targets` passes
- [ ] `cargo clippy -- -W clippy::pedantic` clean

**Decision point:** All 9 gaps (P0 + P1) must be closed before proceeding to Phase 3. If behind schedule, compress documentation tasks (README can be shorter, 2 examples instead of 3) but never skip hardening tasks.

---

### 2.3 The Avoid List (First 30 Days)

These are explicitly scoped out for the first 30 days. Each has a rationale traced to a strategic decision:

| Avoid | Rationale | Source |
|-------|-----------|--------|
| Optimizing retrieval quality | Ship, then measure, then optimize based on data | recommendation.yml avoid_list |
| Building MCP server | v0.2 scope; core library must exist first | recommendation.yml avoid_list |
| Adding feature flags | v0.2 scope; no complexity before v0.1 | recommendation.yml avoid_list |
| Chasing GitHub stars | Vanity metric; MACC is the north star | recommendation.yml avoid_list |
| Python bindings, C FFI | v0.2/v0.3 scope | recommendation.yml avoid_list |
| Website, logo, branding assets | No users yet; premature investment | recommendation.yml avoid_list |
| Skipping CI pipeline | Phase 3 blocker; CI is a release gate | recommendation.yml avoid_list |
| Making v0.1 perfect | Good enough to publish beats perfect and unpublished | axiom: Simplicity > Completeness |
| WAL checkpoint management | GAP-010; nice-to-have, deferred to v0.1.x | recommendation.yml sequence |
| Content-hash integrity column | GAP-011; deferred to v0.1.x | recommendation.yml sequence |
| Async API | v0.2 scope (ADR-008) | adr.yml ADR-008 |
| Quarantine API | v0.2 scope | security.yml planned_v0_2 |

---

## Part 3: The 90-Day Horizon

### 3.1 Phase 3: CI Pipeline and Publication (Weeks 5-6)

**Goal:** Build the CI pipeline that validates every commit. Run all release gates. Publish v0.1.0 to crates.io. Execute launch communications.

**Estimated effort:** 12-16 hours.

#### Week 5: CI Pipeline (GAP-012, GAP-013)

##### Task 3.1: Create GitHub Actions CI workflow

**Files:** `.github/workflows/ci.yml`
**Traces to:** GAP-012, testing.yml ci_pipeline, post-deployment.yml release_gates
**Estimated time:** 3-4 hours

The CI pipeline must enforce every automated release gate:

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-22.04, macos-14, windows-2022]
        rust: [1.75.0, stable]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
      - run: cargo build --all-targets
      - run: cargo test --all-targets
      - run: cargo test --doc

  lint:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - run: cargo clippy --all-targets -- -D warnings -W clippy::pedantic
      - run: cargo fmt --check

  audit:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v2

  doc:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: RUSTDOCFLAGS='-D warnings' cargo doc --no-deps

  no-network:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Verify no networking dependencies
        run: |
          if cargo tree --no-default-features 2>/dev/null | grep -E '(reqwest|hyper|h2|tokio.*net)'; then
            echo "ERROR: Networking dependency detected (ADR-009 violation)"
            exit 1
          fi
```

**Completion criteria:**
- CI passes on all 3 platforms (ubuntu, macos, windows)
- CI passes on MSRV (1.75) and stable
- All 6 jobs green on the first commit to `main` with CI
- No networking dependencies detected (ADR-009 enforcement)

##### Task 3.2: MSRV pinning (GAP-013)

**Files:** `rust-toolchain.toml`, `Cargo.toml`
**Traces to:** GAP-013, post-deployment.yml msrv gate
**Estimated time:** 30 minutes

Create `rust-toolchain.toml`:

```toml
[toolchain]
channel = "stable"
```

Verify that `Cargo.toml` has `rust-version = "1.75"` (added in Task 2.7).

Manually verify: `cargo +1.75.0 check --all-features` compiles. If it does not, adjust MSRV upward to the minimum version that compiles and document the change.

##### Task 3.3: Pre-publication dry run

**Files:** None (verification step)
**Traces to:** post-deployment.yml release_gates (automated)
**Estimated time:** 1-2 hours

Run every automated release gate in sequence:

```bash
# Test suite
cargo test --all-targets
cargo test --doc

# Lint
cargo clippy --all-targets -- -D warnings -W clippy::pedantic
cargo fmt --check

# Security audit
cargo audit
cargo deny check  # if deny.toml is configured

# Documentation
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps

# MSRV
cargo +1.75.0 check

# Publish dry run
cargo publish --dry-run

# Network dependency ban
cargo tree --no-default-features | grep -E '(reqwest|hyper|h2|tokio.*net)' && exit 1 || echo "OK: no network deps"
```

All must pass. Any failure is a blocker.

##### Task 3.4: Cargo.toml final review

Verify all fields required by crates.io:
- `name`, `version`, `edition` (exist)
- `license` (MIT, exists)
- `description` (exists, update if needed)
- `repository` (exists)
- `homepage` (added in Task 2.7)
- `documentation` (added in Task 2.7)
- `readme` (added in Task 2.7)
- `keywords` (max 5, relevant)
- `categories` (max 5, from crates.io categories list)
- `rust-version` (added in Task 2.7)

Verify `[dev-dependencies]` includes `tempfile` and `proptest`. Verify no unnecessary dependencies.

#### Week 6: Publication and Launch

##### Task 3.5: Publish v0.1.0 to crates.io

**Estimated time:** 30 minutes

```bash
# Final verification
cargo publish --dry-run

# Publish
cargo publish

# Tag
git tag v0.1.0
git push origin v0.1.0
```

Verify:
- `https://crates.io/crates/alaya` shows v0.1.0
- `https://docs.rs/alaya` builds successfully with examples visible
- `cargo add alaya` works from a fresh project

##### Task 3.6: Launch communications

**Traces to:** northstar.yml launch_channels, brand.yml voice rules
**Estimated time:** 3-4 hours (writing) + 1 hour (posting)

Channels and content:

1. **r/rust** -- "Show & Tell" post. 300-word technical description. Focus on architecture, not marketing. Include quickstart code block. Link to docs.rs and GitHub. Be explicit about what it is (memory engine) and what it is not (not a service, not cloud). Mention zero dependencies.

2. **HackerNews** -- "Show HN: Alaya -- Embeddable memory engine for AI agents (Rust, single SQLite file, zero deps)". Link to GitHub README. Let the README speak. Do not oversell.

3. **Blog post** -- 800-1200 word technical introduction. Cover: the problem (memory systems require cloud/LLM), the approach (cognitive lifecycle in SQLite), the architecture (three stores, Hebbian graph, Bjork forgetting), the trade-offs (scale ceiling, no async yet), what is next (benchmarks in 3 weeks). Include code examples. Use the brand voice: technical, research-grounded, honest about limitations.

4. **OpenClaw outreach** -- Direct message or issue on the relevant repository. "Here is a published Rust memory crate with MIT license, zero dependencies, and a cognitive lifecycle. Benchmark results coming in 3 weeks. Here is how it could integrate." Include a minimal example showing the API.

**Voice rules for all communications:**
- Active voice, present tense for features
- No superlatives without evidence ("fast" -> "sub-millisecond BM25 query at 1K episodes")
- No "revolutionary", "cutting-edge", "AI-powered"
- Specific trade-offs mentioned alongside features
- Benchmarks promised with timeline, not claimed without data

**Completion criteria:**
- All 4 channels posted within 48 hours of publication
- No marketing language (review against brand.yml avoid list)
- OpenClaw outreach includes working code, not just a pitch
- HackerNews and Reddit posts are factual, not promotional

#### Phase 3 Completion Review (End of Week 6)

This is the **pre-publication gate** -- the most important checkpoint in the entire roadmap.

**Ship decision criteria:**
- [ ] All P0 gaps closed (GAP-001 through GAP-005)
- [ ] All P1 gaps closed (GAP-006 through GAP-009)
- [ ] CI pipeline green on all platforms
- [ ] `cargo publish --dry-run` succeeds
- [ ] README with working quickstart
- [ ] docs.rs preview shows compilable examples
- [ ] CHANGELOG.md complete
- [ ] Test count >= 80
- [ ] Zero clippy warnings with pedantic
- [ ] No networking dependencies in `cargo tree`
- [ ] Schema versioning operational

**If any P0 gap is open:** Do not publish. Fix the gap first.
**If a P1 gap is open:** Evaluate. If the gap is GAP-007 (enrichment) or GAP-009 (RIF), these can ship with a documented limitation. If the gap is GAP-006 (LTD) or GAP-008 (tombstone), these are correctness issues that should block.

---

### 3.2 Phase 4: Benchmark Fast-Follow (Weeks 7-9)

**Goal:** Implement the LoCoMo benchmark harness, run baselines, publish results honestly regardless of numbers. This is the credibility move.

**Estimated effort:** 20-30 hours.

##### Task 4.1: LoCoMo benchmark harness

**Files:** `benches/locomo.rs` or `tests/locomo/`
**Traces to:** testing.yml golden_datasets, competitive.yml (Mem0 68.5% baseline), northstar.yml retrieval quality targets
**Estimated time:** 8-12 hours

The LoCoMo benchmark (arXiv:2402.14088) tests multi-session recall, temporal reasoning, and preference tracking. Implementation:

1. Parse the LoCoMo dataset into Alaya episodes (multi-session conversations)
2. Store all episodes via `store_episode()` with proper session IDs and timestamps
3. Run lifecycle (consolidate, perfume, transform) after each session
4. Execute the benchmark's query set
5. Measure: precision@5, recall@10, nDCG@5, MRR

Two configurations:
- **BM25-only:** No embeddings, no provider (NoOpProvider). Target: P@5 > 0.55 (based on testing.yml targets)
- **Hybrid:** With embeddings (requires an embedding provider for the benchmark). Target: P@5 > 0.70

**Context:** Mem0 scores 68.5% on LoCoMo. Letta scores 74%. Hindsight claims 89.61%. Alaya's BM25-only baseline will likely be lower than hybrid systems, but the lifecycle differentiators (forgetting, preference emergence, graph dynamics) may close the gap over multi-session scenarios.

**Completion criteria:**
- LoCoMo benchmark harness produces reproducible results
- Both BM25-only and hybrid configurations measured
- Results documented with methodology (reproducibility is non-negotiable)
- Numbers reported honestly regardless of where they fall

##### Task 4.2: Run and document baselines

**Estimated time:** 4-6 hours

Run the benchmarks, document:
- BM25-only P@5, R@10, nDCG@5
- Hybrid P@5, R@10, nDCG@5 (if embedding provider available)
- Latency measurements: median, p95, p99 for query at 1K and 10K episodes
- Lifecycle process timings: consolidate, transform, forget

Store results in `benchmarks/results/` as machine-readable YAML for future comparison.

##### Task 4.3: LongMemEval baseline (stretch goal)

**Files:** `tests/longmemeval/` or `benches/longmemeval.rs`
**Traces to:** testing.yml golden_datasets (LongMemEval)
**Estimated time:** 6-8 hours

If time permits after LoCoMo, implement LongMemEval (arXiv:2410.10813) which tests factual recall, preference consistency, and temporal ordering. This provides a second data point and validates the lifecycle differentiators more thoroughly.

**Completion criteria:**
- At least LoCoMo completed; LongMemEval is a bonus
- Results are honest and reproducible

##### Task 4.4: Benchmark blog post

**Traces to:** axiom: Honesty > Marketing, offensive move O1 (benchmark-first credibility)
**Estimated time:** 4-6 hours writing

Blog post structure:
- What we measured (LoCoMo methodology, dataset characteristics)
- How we measured (Alaya configuration, no tuning for the benchmark)
- What we found (raw numbers, comparison table with published results from competitors)
- What the numbers mean (lifecycle processes as differentiator, not raw retrieval quality)
- What is next (optimization targets for v0.2, LongMemEval if not done)
- Reproducibility (exact commands to reproduce, dataset source, configuration)

If numbers are bad (below 50% P@5), frame honestly: "BM25-only retrieval without embeddings scores X%. Here is why: [explanation]. Here is how lifecycle processes add value beyond raw retrieval: [evidence]. Hybrid numbers with embeddings are the meaningful comparison, and here is the path to improving them."

##### Task 4.5: OpenClaw follow-up with benchmark data

**Estimated time:** 1 hour

Follow up on the initial outreach (Task 3.6) with:
- Published benchmark results
- Any API feedback incorporated since v0.1.0
- Specific integration proposal if conversations have progressed

#### Phase 4 Completion Review (3 Weeks Post-Publish)

**Review checklist:**
- [ ] LoCoMo results measured and published
- [ ] Blog post published with honest numbers
- [ ] Benchmark results stored in machine-readable format
- [ ] OpenClaw follow-up completed
- [ ] Any v0.1.x patches published for issues discovered since launch

**Decision point:** Based on benchmark results:
- If P@5 >= 0.70 (BM25-only): Strong position. Focus v0.2 on ecosystem expansion.
- If P@5 between 0.50-0.70: Acceptable. Document the gap, plan retrieval optimization for v0.2.
- If P@5 < 0.40: Reframe. Focus blog on lifecycle differentiators. Prioritize retrieval quality in v0.2.

---

### 3.3 Phase 5: v0.1.x Iteration and v0.2 Planning (Weeks 10-12)

**Goal:** Respond to user feedback. Fix issues discovered post-publication. Plan v0.2 scope based on real data (MACC, benchmark results, feedback quality).

##### Task 5.1: Post-launch monitoring (ongoing from week 6)

**Frequency:** Daily for first 2 weeks, then weekly
**Channels:** GitHub issues, crates.io downloads, r/rust thread, HackerNews thread, email

For each piece of feedback, classify:
- **Bug:** Fix immediately if SEV-1/SEV-2, next release if SEV-3/SEV-4
- **API friction:** Document for v0.2 planning
- **Feature request:** Check against kill list and axiom hierarchy before considering
- **Positive signal:** Record as evidence for positioning validation

##### Task 5.2: v0.1.x patch releases (as needed)

**Traces to:** post-deployment.yml hotfix_process, semver_policy
**Trigger:** Any SEV-1 through SEV-3 issue

Patch release process:
1. Branch from v0.1.0 tag
2. Write failing test
3. Fix
4. Run full release gates
5. Publish v0.1.1 (or v0.1.2, etc.)
6. Yank broken version if SEV-1 (data corruption) or SEV-2 (semver violation)

##### Task 5.3: P2 polish tasks (weeks 10-12, lower priority)

These are deferred P2 items that improve quality but were not blocking for v0.1.0:

1. **WAL checkpoint management (GAP-010)** -- Add `PRAGMA wal_autocheckpoint = 1000` and `PRAGMA journal_size_limit = 67108864` to `src/schema.rs` init. Add a `compact()` method that runs VACUUM and checkpoints the WAL.

2. **Content-hash integrity column (GAP-011)** -- Add SHA-256 content hash to episodes table for dedup detection and tombstone matching. Requires schema migration (user_version 2 -> 3).

3. **Integration tests** -- Create `tests/` directory with external crate integration tests:
   - `tests/store_lifecycle.rs` -- Full CRUD + lifecycle flow
   - `tests/degradation_chain.rs` -- All 5 degradation levels
   - `tests/persistence.rs` -- File-backed DB roundtrip
   - `tests/concurrent_access.rs` -- Arc<Mutex> pattern
   - `tests/purge_compliance.rs` -- GDPR purge completeness

4. **cargo-semver-checks** -- Add to CI to catch accidental breaking changes in future releases.

##### Task 5.4: v0.2 scope planning

**Trigger:** MACC >= 3 or 6 weeks post-publication (whichever comes first)
**Traces to:** northstar.yml phases (v0.2 Ecosystem)

v0.2 scope candidates (prioritized by user feedback when available):

| Feature | Dependency | Estimated Effort | MACC Impact |
|---------|------------|-----------------|-------------|
| `vec-sqlite` feature flag | sqlite-vec 0.1 | 1-2 weeks | Medium (performance users) |
| `embed-ort` feature flag | ort 2 | 1 week | High (removes embedding barrier) |
| `async` feature flag | tokio 1 | 1 week | Medium (async ecosystem) |
| MCP server | Separate crate | 2-3 weeks | High (non-Rust users) |
| AlayaConfig builder | None | 3 days | Medium (ergonomics) |
| Quarantine API | None | 1 week | Low (security) |
| Benchmark regression CI | divan 0.1 | 1 week | Low (quality) |

The v0.2 scope decision is made at the "v0.2 Planning" checkpoint (when MACC >= 5), informed by:
- Which persona is more active (Priya -> privacy features, Marcus -> performance features)
- Whether OpenClaw conversations have progressed (MCP server priority)
- User-reported friction points (ergonomics vs features)

---

### 3.4 Dependencies and Risk Map

#### External Dependencies

| Dependency | Risk | Mitigation |
|-----------|------|------------|
| crates.io availability | Low | Can delay publication by days, not weeks |
| OpenClaw component selection timeline | Medium | Outreach with published crate; cannot control their timeline |
| LoCoMo dataset access | Low | Publicly available (arXiv:2402.14088) |
| Competitor ships zero-dep cognitive library | Medium | Emergency publish: close P0 in 1 week, publish with minimal docs |
| rusqlite 0.32 security advisory | Low | Monitor cargo audit; update immediately |

#### Internal Dependencies (Task Ordering)

```
Phase 1 (P0 Hardening) ─── must complete before ──→ Phase 2 (P1 Quality)
Phase 2 (P1 Quality)   ─── must complete before ──→ Phase 3 (CI + Publish)
Phase 3 (Publication)   ─── must complete before ──→ Phase 4 (Benchmarks)
Phase 4 (Benchmarks)    ─── can run alongside ────→ Phase 5 (v0.1.x patches)
```

Within phases, tasks are largely independent (can be done in any order) except:
- Task 1.2 (BEGIN IMMEDIATE) before Task 1.3 (validation) -- validation may use transactions
- Task 2.3 (tombstones) before Task 2.4 (RIF) -- both modify pipeline behavior
- Task 3.1 (CI) before Task 3.5 (publish) -- CI must be green
- Task 4.1 (LoCoMo harness) before Task 4.2 (run baselines) -- tool before measurement

#### Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| P0 hardening takes >2 weeks | 20% | Medium -- delays everything | Keep scope tight; each task is <4 hours |
| CI pipeline has platform-specific failures | 40% | Low -- delays Phase 3 by days | Use dtolnay/rust-toolchain; test locally on macOS |
| LoCoMo numbers are embarrassingly low (<30% P@5) | 15% | Medium -- requires messaging pivot | Frame as honest baseline; lifecycle is differentiator |
| Zero MACC after 4 weeks | 30% | High -- positioning or DX failure | 5 developer interviews; likely README or discoverability issue |
| OpenClaw selects alternative memory system | 20% | Medium -- loses highest-value adoption | Engage early with published crate; MCP server in v0.2 |
| Solo developer capacity constraint (life happens) | 25% | Medium -- Phase 2 delayed | Drop Phase 4 (benchmarks); minimum viable is Phases 1-3 |
| Competitor ships Rust memory library with lifecycle | 5% | High -- quadrant contested | Emergency publish; differentiate on research grounding |
| Research invalidates Bjork/CLS/Hebbian mechanism | <5% | Low -- modular replacement | ADR architecture supports mechanism substitution |

---

## Part 4: Tracking and Review

### 4.1 Success Metrics

#### 90-Day Success Criteria

| Criterion | Target | Measurement |
|-----------|--------|-------------|
| v0.1.0 published on crates.io | Binary: yes/no | `https://crates.io/crates/alaya` exists |
| MACC | >= 3 | GitHub dependents + reverse dependency search |
| LoCoMo baseline measured | Binary: yes/no | Blog post published with reproducible numbers |
| External API review | >= 1 | Non-maintainer opens issue or PR about API design |
| OpenClaw outreach | Completed | Direct engagement with benchmark data |
| SEV-1/SEV-2 incidents | 0 | GitHub issues labeled bug/data-corruption or bug/security |
| Test count | >= 100 | `cargo test -- --list 2>/dev/null | grep 'test$' | wc -l` |
| CI pipeline | Green on all platforms | GitHub Actions status |

#### 90-Day Failure Criteria

| Criterion | Threshold | Response |
|-----------|-----------|----------|
| v0.1.0 not published | End of week 8 | Execution failure -- evaluate blockers, consider reducing scope |
| Zero external engagement | 4 weeks post-publish | Positioning failure -- 5 developer interviews, likely DX/README issue |
| Competitor occupies quadrant | Any time | Timing failure -- emergency publish if not yet published; differentiate if already published |

### 4.2 Review Cadence

#### Scheduled Reviews

| Review | Timing | Focus | Decision |
|--------|--------|-------|----------|
| Phase 1 completion | End of Week 2 | P0 gaps closed? Blockers? | Proceed to Phase 2 or extend Phase 1 |
| Phase 2 completion | End of Week 4 | P1 gaps closed? Docs adequate? | Proceed to CI+publish or extend Phase 2 |
| Pre-publication gate | End of Week 6 | All gates pass? Ship decision. | Publish or fix remaining blockers |
| Post-launch check | 2 weeks post-publish | Any MACC? Feedback quality? | Adjust v0.1.x priorities |
| Benchmark review | 3 weeks post-publish | LoCoMo numbers available | Strategy adjustment if needed |
| 90-day review | 90 days post-publish | MACC vs target | Full strategy reassessment |
| v0.2 scope decision | When MACC >= 5 | User-driven prioritization | Scope v0.2 based on feedback |

#### Weekly Check-in (Self-Review)

Every Sunday, 15 minutes:
1. What did I ship this week?
2. What is blocking next week?
3. Am I on track for the next scheduled review?
4. Did any review trigger fire? (Check the trigger list below)

### 4.3 Pivot Triggers

These are conditions that require deviating from the plan. Each has a pre-defined response:

| Trigger | Detection | Response |
|---------|-----------|----------|
| OpenClaw announces component selection within 4 weeks | OpenClaw Discord/GitHub/mailing list | Evaluate accelerating Phase 1-2; consider v0.1.0-alpha pre-release |
| Competitor ships zero-dep embedded memory library with cognitive lifecycle | Monthly competitor scan | Emergency publication: close P0 in 1 week, publish with minimal docs |
| LoCoMo baseline below 40% P@5 (BM25-only) | Phase 4 benchmark results | Reframe benchmark post as analysis; focus on lifecycle differentiators in messaging |
| Zero MACC 90 days post-publication | Monthly MACC check | Conduct 5 developer interviews; likely positioning or DX issue, not architecture |
| MACC exceeds 5 within 4 weeks of publication | Monthly MACC check | Accelerate v0.2 planning; publish roadmap RFC for community input |
| Solo-developer capacity constraint (Phase 2 incomplete after 6 calendar weeks) | Self-assessment | Drop Phase 4 (benchmarks); minimum viable is Phases 1-3 |
| Research invalidates core mechanism (Bjork, CLS, Hebbian) | Bi-weekly arxiv scan | Update affected mechanism; modular architecture supports replacement (ADR-003, ADR-005) |
| SEV-1 or SEV-2 incident post-publication | GitHub issues, crates.io feedback | Immediate response per post-deployment.yml: 4h ack, 24h fix, yank if needed |
| docs.rs build failure | Post-publish verification | Fix within 1 week; blocks discoverability |

### 4.4 Progress Tracking

Use this table to track task completion. Update after each work session:

#### Phase 1: P0 Hardening (Weeks 1-2)

| # | Task | Gap | Status | Date |
|---|------|-----|--------|------|
| 1.1 | `#[non_exhaustive]` on public enums | GAP-001 | Not started | |
| 1.2 | BEGIN IMMEDIATE for write transactions | GAP-002 | Not started | |
| 1.3 | Input validation at API boundary | GAP-003 | Not started | |
| 1.4 | Internal modules to `pub(crate)` | GAP-004 | Not started | |
| 1.5 | Compilable doctests on all pub methods | GAP-005 | Not started | |
| 1.6 | Add `tempfile` dev-dependency | -- | Not started | |
| 1.7 | Schema versioning (PRAGMA user_version) | -- | Not started | |

#### Phase 2: P1 Quality and Documentation (Weeks 3-4)

| # | Task | Gap | Status | Date |
|---|------|-----|--------|------|
| 2.1 | Wire LTD in transform() | GAP-006 | Not started | |
| 2.2 | Semantic/preference enrichment in pipeline | GAP-007 | Not started | |
| 2.3 | Tombstone mechanism for deleted nodes | GAP-008 | Not started | |
| 2.4 | RIF suppression in retrieval pipeline | GAP-009 | Not started | |
| 2.5 | README rewrite with quickstart | -- | Not started | |
| 2.6 | Examples directory (3 examples) | -- | Not started | |
| 2.7 | Cargo.toml metadata completion | GAP-013 | Not started | |
| 2.8 | CHANGELOG.md | -- | Not started | |
| 2.9 | Expand test coverage (+40 tests) | -- | Not started | |
| 2.10 | Proptest for mathematical invariants (15 props) | -- | Not started | |

#### Phase 3: CI Pipeline and Publication (Weeks 5-6)

| # | Task | Gap | Status | Date |
|---|------|-----|--------|------|
| 3.1 | GitHub Actions CI workflow | GAP-012 | Not started | |
| 3.2 | MSRV pinning | GAP-013 | Not started | |
| 3.3 | Pre-publication dry run (all gates) | -- | Not started | |
| 3.4 | Cargo.toml final review | -- | Not started | |
| 3.5 | Publish v0.1.0 to crates.io | -- | Not started | |
| 3.6 | Launch communications (4 channels) | -- | Not started | |

#### Phase 4: Benchmark Fast-Follow (Weeks 7-9)

| # | Task | Gap | Status | Date |
|---|------|-----|--------|------|
| 4.1 | LoCoMo benchmark harness | -- | Not started | |
| 4.2 | Run and document baselines | -- | Not started | |
| 4.3 | LongMemEval baseline (stretch) | -- | Not started | |
| 4.4 | Benchmark blog post | -- | Not started | |
| 4.5 | OpenClaw follow-up with data | -- | Not started | |

#### Phase 5: v0.1.x Iteration (Weeks 10-12)

| # | Task | Gap | Status | Date |
|---|------|-----|--------|------|
| 5.1 | Post-launch monitoring | -- | Not started | |
| 5.2 | v0.1.x patches (as needed) | -- | Not started | |
| 5.3 | P2 polish (WAL, content-hash, integration tests) | GAP-010, GAP-011 | Not started | |
| 5.4 | v0.2 scope planning | -- | Not started | |

---

## Appendix A: File-Level Change Map

Every file that will be modified or created during this roadmap, organized by phase:

### Phase 1 Changes

| File | Action | Task |
|------|--------|------|
| `src/types.rs` | Modify: add `#[non_exhaustive]` to 4 enums | 1.1 |
| `src/error.rs` | Modify: add `#[non_exhaustive]` to AlayaError | 1.1 |
| `src/schema.rs` | Modify: add `immediate_transaction()` helper, add PRAGMA user_version | 1.2, 1.7 |
| `src/lib.rs` | Modify: update write methods to use BEGIN IMMEDIATE, add validation, change `pub mod` to `pub(crate) mod`, add doctests | 1.2, 1.3, 1.4, 1.5 |
| `Cargo.toml` | Modify: add `tempfile` dev-dependency | 1.6 |

### Phase 2 Changes

| File | Action | Task |
|------|--------|------|
| `src/lifecycle/transformation.rs` | Modify: call `decay_links()` | 2.1 |
| `src/retrieval/pipeline.rs` | Modify: enrich semantic/preference nodes, wire RIF suppression | 2.2, 2.4 |
| `src/schema.rs` | Modify: add tombstones table, increment schema version | 2.3 |
| `src/store/episodic.rs` | Modify: create tombstone on deletion | 2.3 |
| `src/lifecycle/consolidation.rs` | Modify: check tombstones before creating nodes | 2.3 |
| `README.md` | Create/rewrite | 2.5 |
| `examples/basic_agent.rs` | Create | 2.6 |
| `examples/lifecycle_demo.rs` | Create | 2.6 |
| `examples/custom_provider.rs` | Create | 2.6 |
| `Cargo.toml` | Modify: metadata, MSRV, docs.rs config, proptest dev-dep | 2.7, 2.10 |
| `CHANGELOG.md` | Create | 2.8 |
| Various test modules | Modify: add ~40 tests | 2.9 |

### Phase 3 Changes

| File | Action | Task |
|------|--------|------|
| `.github/workflows/ci.yml` | Create | 3.1 |
| `rust-toolchain.toml` | Create | 3.2 |

### Phase 4 Changes

| File | Action | Task |
|------|--------|------|
| `benches/locomo.rs` or `tests/locomo/` | Create | 4.1 |
| `benchmarks/results/*.yml` | Create | 4.2 |
| `tests/longmemeval/` | Create (stretch) | 4.3 |

---

## Appendix B: Gap Closure Tracker

| Gap ID | Title | Phase | Priority | Status |
|--------|-------|-------|----------|--------|
| GAP-001 | `#[non_exhaustive]` on public enums | 1 | P0 | Open |
| GAP-002 | BEGIN IMMEDIATE for write transactions | 1 | P0 | Open |
| GAP-003 | Input validation at API boundary | 1 | P0 | Open |
| GAP-004 | Internal modules to `pub(crate)` | 1 | P0 | Open |
| GAP-005 | Compilable doctests on all pub methods | 1 | P0 | Open |
| GAP-006 | LTD not called from transform() | 2 | P1 | Open |
| GAP-007 | Semantic/preference enrichment in pipeline | 2 | P1 | Open |
| GAP-008 | Tombstone mechanism for deleted nodes | 2 | P1 | Open |
| GAP-009 | RIF suppression not wired in pipeline | 2 | P1 | Open |
| GAP-010 | WAL checkpoint management | 5 | P2 | Deferred to v0.1.x |
| GAP-011 | Content-hash integrity column | 5 | P2 | Deferred to v0.1.x |
| GAP-012 | CI pipeline | 3 | P0 | Open |
| GAP-013 | MSRV pinning + Cargo.toml metadata | 2-3 | P2 | Open |

---

## Appendix C: Test Count Trajectory

| Milestone | Unit | Prop | Doc | Integration | Total |
|-----------|------|------|-----|-------------|-------|
| Current | 43 | 0 | 0 | 0 | 43 |
| End Phase 1 | 65-75 | 0 | 12+ | 0 | 77-87 |
| End Phase 2 | 80-90 | 15 | 12+ | 0 | 107-117 |
| End Phase 3 | 80-90 | 15 | 12+ | 5+ | 112-122 |
| v0.1.x target | 100+ | 15+ | 15+ | 20+ | 150+ |
| v0.2 target | 150+ | 30+ | 30+ | 40+ | 250+ |

---

## Appendix D: Competitive Timing Reference

| Competitor | Current Status | Threat to Alaya's Quadrant |
|-----------|---------------|----------------------------|
| Mem0 | 68.5% LoCoMo, cloud-only | Low (different quadrant) |
| Zep/Graphiti | Neo4j-dependent | Low (different quadrant) |
| Letta (MemGPT) | 74% LoCoMo, LLM-dependent | Low (different quadrant) |
| Supermemory | 16.6K stars, cloud-optimized | Low (different quadrant) |
| Hindsight | 89.6% LoCoMo claimed | Low (requires infrastructure) |
| Memvid | Rust, single file | Medium (same quadrant, no lifecycle) |
| Engram | Zero-dep, SQLite | Medium (same quadrant, no lifecycle) |
| SYNAPSE | Research code, spreading activation | Medium (closest architecture, not shipping) |
| MAGMA | Research code, multi-graph | Low (not shipping) |
| RL-trained (Memory-R1 etc.) | Research phase | Low now, High in 12-18 months |

The key insight: no competitor currently occupies the "high cognitive completeness + high operational simplicity" quadrant. The competitors closest to this quadrant (Memvid, Engram) lack lifecycle depth. The competitors with lifecycle depth (Mem0, Letta, Hindsight) require external infrastructure. This window is open now but will not remain open indefinitely.

---

*This roadmap is a living document. Update the progress tracking tables after each work session. Revisit the full plan at each scheduled review checkpoint. When a pivot trigger fires, adjust the plan and document the change.*

*Strategic guidance: When overwhelmed by the task list, return to the axiom hierarchy. The single most important output of this 90-day period is a published crate on crates.io. Everything else -- benchmarks, community, ecosystem -- flows from that. Ship.*
