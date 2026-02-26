# Alaya -- North Star Extract

> Design DNA. Decisions that are not re-litigated.

**Version:** 1.0
**Date:** 2026-02-26
**Phase:** 4 of 13
**Status:** Active
**Owner:** Albert Hui

---

## 1. Core Axioms

These axioms resolve conflicts when values compete. When two good ideas collide, the axiom determines which wins.

### Axiom 1: Privacy > Features

Alaya never introduces a feature that requires network access in the core crate. If a capability cannot be delivered within a single SQLite file on the developer's machine, it does not ship. This is not a preference -- it is an architectural constraint enforced by the absence of networking dependencies in `Cargo.toml`.

**When it bites:** An embedding model hosted as a remote API would improve retrieval quality. The answer is still no. Quality improvements come through local embedding providers (`ort`, `fastembed-rs`) behind feature flags, or the developer plugs in their own provider via the `EmbeddingProvider` trait. The core crate never dials out.

**Derived from:** Brand belief "Graceful degradation," kill list item "Not cloud-dependent," competitive positioning as the only zero-network memory system in the field.

### Axiom 2: Process > Storage

Memory transforms through use. Every retrieval reshapes retrieval strength (Bjork), every co-access strengthens Hebbian links (LTP), every consolidation cycle compresses episodes into semantic knowledge (CLS). Alaya is not a database that happens to have lifecycle methods bolted on -- the lifecycle is the product.

**When it bites:** A feature request asks for a raw key-value store mode that bypasses lifecycle processes. The answer is no. If someone needs a key-value store, they should use `sled` or `redb`. Alaya's value is the transformation pipeline. Skipping it misses the point.

**Derived from:** Brand belief "Memory is a process, not a database," the Yogacara etymology (seeds ripen through contact, not through storage), competitive differentiation against append-only systems like Memvid.

### Axiom 3: Correctness > Speed

Research grounding comes before shipping velocity. Every mechanism in Alaya maps to a named algorithm or cited research concept: Bjork dual-strength, Hebbian LTP/LTD, CLS consolidation, vasana perfuming. If a technique cannot be grounded in established cognitive science or neuroscience, it does not enter the codebase -- no matter how clever it seems.

**When it bites:** A heuristic-based "smart forgetting" approach would be faster to implement than the Bjork dual-strength model. The answer is: implement Bjork correctly, or do not implement forgetting yet. Half-grounded mechanisms become technical debt that undermines the project's credibility.

**Derived from:** Brand belief "Preferences emerge, they are not declared" (emergence requires correct models), kill list item "Not hype-driven," brand voice attribute "Research-grounded."

### Axiom 4: Simplicity > Completeness

A single SQLite file. Zero runtime dependencies beyond `rusqlite`, `serde`, `serde_json`, `thiserror`. Four to six feature flags maximum. `cargo add alaya` and you are done. If a feature requires the developer to understand a complex configuration matrix, manage additional infrastructure, or add more than one dependency, the feature needs to be rethought or deferred to a trait extension.

**When it bites:** Adding Neo4j as an optional graph backend would provide better graph traversal performance at scale. The answer is no. The Hebbian graph lives in SQLite via recursive CTEs. If SQLite's scale ceiling becomes a real problem for real users, the solution is a `GraphProvider` trait -- not pulling in a graph database dependency.

**Derived from:** Kill list items "Not enterprise," "Not a standalone service," competitive positioning against infrastructure-heavy alternatives (Zep requires Neo4j, Mem0 requires vector DB + relational DB).

### Axiom 5: Honesty > Marketing

Publish benchmark results even when the numbers are bad. Document scale ceilings honestly. If brute-force vector search degrades past 10K vectors, say so in the README. If consolidation quality depends on the LLM the developer plugs in, say that too. Alaya does not hide limitations behind superlatives.

**When it bites:** The LoCoMo baseline score is lower than Mem0's reported 68.5%. The answer is: publish the score anyway, with a clear explanation of what the BM25-only pipeline does and does not capture, and a roadmap for improvement. A bad number you own is worth more than a good number you faked.

**Derived from:** Brand voice attribute "Honest about tradeoffs," brand voice rule "No superlatives without evidence," competitive strategy "Benchmark-first credibility."

---

## 2. Explicit Non-Goals

### 2.1 Features We Will Never Build

These map directly to the kill list. Each item has been evaluated and rejected. The rejection is permanent unless the "When to Re-evaluate" triggers in Section 6 are activated.

| # | Non-Goal | What We Build Instead | Rationale |
|---|----------|-----------------------|-----------|
| 1 | Cloud deployment / managed service | Embeddable library (`cargo add alaya`) | Cloud contradicts core positioning; Mem0 and Supermemory own that space |
| 2 | Multi-tenant isolation, RBAC, horizontal scaling | Single-agent `user_id` + `agent_id` scoping | Serves 2024 enterprise buyer, not 2026 solo developer |
| 3 | LLM-required operations | `NoOpProvider` default, optional `ConsolidationProvider` trait | Graceful degradation: every feature works without LLM |
| 4 | AI marketing language ("AI-powered," "intelligent") | Named algorithms and research citations | Brand voice: research-grounded, never hype-driven |
| 5 | Standalone memory service | Library + thin MCP server wrapper | Library-first: `alaya-mcp` is an access layer, not the product |
| 6 | Procedural memory (executable skills) | Observation storage: what happened, not how to do it | Different problem domain; outside Alaya's cognitive model |
| 7 | Parametric memory (model fine-tuning) | Non-parametric structured data in SQLite | No training loops, no weight updates, no GPU dependency |

### 2.2 Strategic Moves We Rejected

These were evaluated during competitive landscape analysis (Phase 3) and explicitly declined.

| Rejected Move | Why Rejected | What We Do Instead |
|---------------|--------------|--------------------|
| Building a managed cloud service | Contradicts positioning; cannot compete with VC-funded Mem0/Supermemory on cloud | Invest in embeddability, MCP server, and FFI for universal access |
| Enterprise features | Wrong buyer (2024 enterprise vs. 2026 solo dev); feature bloat | Serve individual developers and small teams; simplicity is the moat |
| Building an agent framework | Market saturated (LangChain, CrewAI, AutoGen, etc.); framework-agnostic is a strength | Library that works with any framework via traits and MCP |
| Competing on GitHub stars / social hype | Leads to feature bloat; undermines "quiet confidence" brand | Compete on benchmarks, research grounding, and developer experience |
| Adding procedural memory | On kill list; different problem, different research base | Store observations; let the agent decide what actions to take |
| Adopting complex external backend | Destroys zero-dependency value proposition | SQLite + optional trait-based extensions |

### 2.3 Technical Approaches We Rejected

| Rejected Approach | What We Do Instead | Why |
|-------------------|--------------------|-----|
| External vector database (Pinecone, Qdrant, Chroma) | Tiered search: brute-force cosine on BLOBs -> `sqlite-vec` behind feature flag -> `EmbeddingProvider` trait for custom backends | Zero-dep constraint; SQLite is sufficient for single-agent scale; trait extension path exists for users who outgrow it |
| LLM-required entity/knowledge extraction | Rule-based extraction with optional `ConsolidationProvider` trait | Graceful degradation: episodes accumulate and are searchable even without LLM; LLM enhances, never gates |
| Network-dependent features (remote embedding APIs, cloud sync) | Everything local to a single SQLite file; remote providers only via developer-supplied trait implementations | Privacy by architecture; no networking dependency in `Cargo.toml` |
| Complex configuration / environment variables | `AlayaConfig::builder()` with sensible defaults, 4-6 feature flags maximum | Simplicity axiom; developer should not need to read a configuration guide to get started |
| Framework-specific integration (LangChain adapter, CrewAI plugin) | Framework-agnostic trait API + MCP server | Frameworks come and go; memory outlives frameworks |
| Custom binary storage format | SQLite with WAL mode | SQLite is battle-tested, inspectable, and well-understood; custom formats are a maintenance burden |

---

## 3. Structural Patterns

These patterns recur throughout Alaya's design. They are not accidental -- they encode the project's architectural philosophy.

### 3.1 Graceful Degradation Chain

Every capability in Alaya has a fallback path that degrades gracefully when optional components are unavailable. The chain always terminates at a useful baseline.

```
Full pipeline (embeddings + BM25 + graph + RRF fusion + RIF)
    |
    v  [no EmbeddingProvider]
BM25 + graph activation + fusion
    |
    v  [no graph links yet]
BM25 full-text search only
    |
    v  [no FTS5 index / empty corpus]
Recency scan (most recent episodes)
    |
    v  [no episodes stored]
Empty result set (not an error)
```

The same pattern applies to lifecycle processes:

```
Full consolidation (LLM-powered entity extraction + semantic compression)
    |
    v  [no ConsolidationProvider]
Episodes accumulate, searchable via BM25/recency
    |
    v  [no LLM for perfuming]
Impressions accumulate, preference crystallization deferred
```

**Rule:** No code path in Alaya panics or returns an error because an optional component is missing. The system always does the best it can with what it has.

### 3.2 Trait Extension Pattern

Alaya separates core behavior (in the library) from optional enhancement (via traits the agent provides). This pattern appears everywhere:

```
Core library (ships with Alaya)
    |
    defines trait
    |
    v
Optional enhancement (agent provides implementation)
    |
    v
Enhanced behavior (library + agent implementation)
```

Instances:

| Trait | Core Behavior Without It | Enhanced Behavior With It |
|-------|--------------------------|---------------------------|
| `ConsolidationProvider` | Episodes stored and searchable via BM25 | Episodes compressed into semantic knowledge nodes |
| `EmbeddingProvider` | BM25 + graph retrieval | Hybrid vector + BM25 + graph with RRF fusion |
| `TransformationProvider` | Graph reshapes through Hebbian LTP/LTD | Agent-driven semantic transformation (asraya-paravrtti) |

**Rule:** The trait boundary is the extension boundary. Alaya never reaches outside the trait to access external systems. The agent owns the connection.

### 3.3 Cognitive Lifecycle Pipeline

Memory in Alaya flows through a defined lifecycle. Each stage is independent (can run alone) and composable (stages chain).

```
Ingest                    Lifecycle Processes                     Retrieval
------                    --------------------                    ---------

store_episode()  --->  consolidate()  --->  perfume()  --->  query()
     |                      |                    |                |
     v                      v                    v                v
  Episodes            Semantic nodes        Impressions     ScoredMemory
  (raw experience)    (compressed facts)    + Preferences   (ranked results)
                            |                    |
                            v                    v
                      transform()          forget()
                            |                    |
                            v                    v
                      Graph reshaping      Strength decay
                      (LTP/LTD)            (Bjork dual-strength)
```

**Rule:** Each lifecycle process produces a typed report. No silent mutations. The calling agent always knows what changed.

### 3.4 Conflict Resolution Hierarchy

When values or requirements conflict, this hierarchy determines the winner. Higher entries override lower entries.

```
1. Safety       (no panics, no data corruption, no injection vulnerabilities)
2. Privacy      (no network calls, no data leakage, single-file containment)
3. Correctness  (research-grounded algorithms, accurate results, honest benchmarks)
4. Simplicity   (zero deps, single file, minimal config, small API surface)
5. Performance  (sub-ms retrieval, low memory footprint, fast compilation)
6. Features     (new capabilities, integrations, convenience APIs)
```

**Example resolution:** A performance optimization that requires `unsafe` code violating memory safety guarantees is rejected (Safety > Performance). A feature that improves retrieval quality but requires a network call is rejected (Privacy > Features). A simpler algorithm with slightly worse benchmark scores is preferred over a complex one with marginal gains (Simplicity > Performance), unless the benchmark gap is significant enough to affect Correctness.

### 3.5 Single-File Invariant

All persistent state lives in one SQLite file. This pattern constrains every storage decision:

- Episodes, semantic nodes, impressions, preferences: SQLite tables
- Embedding vectors: BLOB columns in SQLite
- Graph structure: SQLite tables with recursive CTE traversal
- FTS5 index: SQLite virtual table
- Node strengths (Bjork model): SQLite columns
- Configuration overrides: not persisted (runtime only via `AlayaConfig`)

**Rule:** If it cannot go in the SQLite file, it does not persist. Temporary state (in-memory caches, runtime config) is acceptable but must be reconstructable from the SQLite file alone.

---

## 4. What We Always Do

These are invariant behaviors. Every code change, every PR, every design decision must satisfy these constraints.

### 4.1 Transaction Safety

- **`BEGIN IMMEDIATE` for all write transactions.** SQLite WAL mode allows concurrent readers but only one writer. `BEGIN IMMEDIATE` acquires the write lock at transaction start, preventing deadlocks and ensuring deterministic failure on contention. Never use `BEGIN DEFERRED` for writes.

- **Cascading cleanup on delete.** When an episode is deleted, its FTS5 index entry, embedding BLOB, graph links, and node strength record are all deleted in the same transaction. No orphaned data.

### 4.2 Input Sanitization

- **Sanitize all FTS5 MATCH input.** FTS5 query syntax allows operators (`AND`, `OR`, `NOT`, `NEAR`, column filters) that can be weaponized. All user-provided search terms are sanitized before reaching a `MATCH` clause. Special characters are escaped or the query is wrapped in double quotes.

- **Validate content before storage.** Empty episodes, zero-length embeddings, and malformed metadata are rejected at the API boundary, not deep in the storage layer. Error messages are actionable and never blame the caller.

### 4.3 API Design

- **`#[non_exhaustive]` on all public enums.** Adding a variant to a public enum is a breaking change in Rust. `#[non_exhaustive]` reserves the right to add variants without a semver bump. Every public enum in Alaya carries this attribute.

- **Builder pattern with sensible defaults.** `AlayaConfig::builder()`, `NewEpisode::quick()`, `Query::new()` -- all entry points use builders that work with zero configuration. The developer opts into complexity; they are never forced into it.

- **Typed reports from lifecycle processes.** `consolidate()` returns a `ConsolidationReport`. `forget()` returns a `ForgettingReport`. The agent always knows what changed: how many episodes were consolidated, how many nodes decayed below threshold, which preferences crystallized. No void returns on mutation operations.

### 4.4 Testing

- **Compilable doctests on every public method.** If it is `pub`, it has a `///` doc comment with at least one code example that compiles and runs under `cargo test --doc`. No exceptions.

- **Integration tests for every CRUD operation.** Store, retrieve, update, delete -- each path has a test that exercises the full stack from `AlayaStore` API to SQLite and back.

### 4.5 Documentation

- **Research citations for non-obvious mechanisms.** Bjork dual-strength gets a citation. Hebbian LTP/LTD gets a citation. CLS consolidation gets a citation. Vasana perfuming gets a citation and an etymology note. The developer should be able to read the papers if they want to understand why the code works the way it does.

- **Honest performance characterization.** If brute-force cosine search degrades at N vectors, the doc comment says so with the approximate N. If consolidation quality depends on the LLM provider, the trait doc says so.

### 4.6 Dependency Discipline

- **Zero runtime dependencies beyond the core four** (`rusqlite`, `serde`, `serde_json`, `thiserror`). Every additional dependency goes behind a feature flag. The default `cargo add alaya` pulls in only these four.

- **Feature flags for optional capabilities.** Embedding providers (`embeddings-ort`, `embeddings-fastembed`), async support (`async`), extended diagnostics -- all gated. Maximum 4-6 feature flags total.

---

## 5. What We Never Do

These are prohibited behaviors. Violating any of these is a bug, not a design tradeoff.

### 5.1 Network Access

- **Never make network calls in the core crate.** No HTTP clients, no DNS resolution, no socket connections. The `Cargo.toml` for the core `alaya` crate contains no networking dependency. This is verified by dependency audit. If a future contributor adds `reqwest` or `hyper` to the core crate's dependencies, that is a reject-on-sight PR.

- **Never phone home.** No telemetry, no analytics, no crash reporting, no version checking. The library is invisible on the network.

### 5.2 LLM Coupling

- **Never require an LLM for basic operations.** `store_episode()`, `query()`, `get_episode()`, `delete_episode()`, `session_history()` -- all of these work with `NoOpProvider`. An LLM enhances consolidation, perfuming, and transformation, but its absence never prevents the developer from storing and retrieving memories.

- **Never embed LLM API keys or model references in the core crate.** The `ConsolidationProvider` and `EmbeddingProvider` traits are the boundary. Alaya does not know or care which model the agent uses.

### 5.3 Data Integrity

- **Never delete without tombstone consideration.** The `forget()` process decays retrieval strength below threshold; it does not hard-delete rows on first pass. `purge()` is the explicit, developer-initiated hard delete. The distinction between "forgotten" (low retrieval strength, not surfaced in queries) and "deleted" (row removed, space reclaimed via VACUUM) is intentional and must be preserved.

- **Never allow cross-user data leakage.** Every query is scoped by `user_id` and `agent_id`. There is no "global query" mode. A bug that returns another user's episodes is a security vulnerability, not a feature gap.

### 5.4 API Surface

- **Never expose SQLite internals through the public API.** No raw SQL queries, no connection handles, no pragma access. The `AlayaStore` API is the contract. SQLite is an implementation detail that could theoretically be replaced (it will not be, but the abstraction discipline matters).

- **Never break semver without a major version bump.** Adding an enum variant on a `#[non_exhaustive]` enum is fine. Removing a public method is not. Changing a return type is not. If a breaking change is genuinely necessary, it waits for a major version.

### 5.5 Marketing

- **Never claim benchmark results that have not been independently reproducible.** Benchmarks are published with the harness code, the dataset version, and instructions to reproduce. "We score X% on LoCoMo" means anyone can run the harness and get the same number (within statistical variance).

- **Never use competitor names in marketing copy.** The comparison table in the README presents facts (architecture, dependencies, features). It does not editorialize about competitors. "Requires Neo4j" is a fact. "Neo4j is bad" is not in scope.

---

## 6. When to Re-evaluate

The axioms, non-goals, and patterns in this document are durable but not eternal. The following triggers indicate that the strategic assumptions underlying this extract may need revision.

### 6.1 Metric Triggers

| Trigger | Threshold | Response Protocol |
|---------|-----------|-------------------|
| MACC stalls | Below phase target for 2 consecutive months | Conduct 5 developer interviews. Diagnose: API friction? Missing feature? Positioning failure? Adjust roadmap, not axioms, unless interviews reveal axiom conflict. |
| LoCoMo plateau | Score stuck below 65% after 3 optimization cycles | Evaluate whether BM25+graph pipeline has fundamental ceiling. Consider whether embedding-required mode should become the default (tension with Axiom 1 if it requires remote API). |
| Zero adoption after 90 days | MACC = 0 at 90 days post-crates.io publish | Full strategy review. The problem is likely positioning or developer experience, not architecture. But if 5 interviews all say "I need cloud sync," re-evaluate the privacy axiom's scope. |
| Build time regression | Clean build exceeds 45 seconds | Aggressive feature-gating, workspace splitting. Simplicity axiom applies to developer experience including compile time. |

### 6.2 External Triggers

| Trigger | Signal | Response Protocol |
|---------|--------|-------------------|
| Competitor ships zero-dep + lifecycle | Mem0, Zep, or new entrant ships embedded, zero-dep memory with consolidation + forgetting | Accelerate v0.2. Differentiate on cognitive depth (perfuming, vasana, Bjork dual-strength). The lifecycle completeness is the moat -- if someone matches it, re-evaluate what "complete" means. |
| Privacy regulation shift | GDPR/AI Act enforcement makes on-device memory architecturally mandated | Accelerate messaging. This is a tailwind, not a trigger for change. Double down on positioning. |
| MCP deprecation or replacement | MCP protocol abandoned or superseded by alternative standard | Adapt the access layer (alaya-mcp). Core library is unaffected. The MCP server is a thin wrapper, not a dependency. |
| OpenClaw ecosystem decision | OpenClaw adopts a different memory system | Analyze why. If API mismatch: adapt. If performance: optimize. If the decision was political or social: accept and focus on other adoption channels. |
| SQLite fundamentally limited | Real user (not theoretical) hits SQLite scale ceiling and trait extension path is insufficient | Evaluate alternative embedded databases (DuckDB, redb) as optional backends via trait. The single-file invariant may need to relax to "single embedded database" -- but only for a real user with a real workload, not a hypothetical. |

### 6.3 Strategic Triggers

| Trigger | Signal | Response Protocol |
|---------|--------|-------------------|
| RL-trained memory superiority | Published results showing RL-trained memory management significantly outperforms hand-crafted lifecycle processes on standard benchmarks | Evaluate `MemoryPolicyProvider` trait for RL-based strategy plugins. The trait extension pattern means Alaya can adopt RL without abandoning its architecture -- but the "Correctness > Speed" axiom may need nuance if RL demonstrates empirically better results with less theoretical grounding. |
| Research invalidation | New cognitive science research fundamentally challenges Bjork dual-strength model or CLS consolidation theory | Update the affected mechanism. Research grounding is a two-way commitment: if the science changes, the code changes. This is expensive but non-negotiable under Axiom 3. |
| Solo maintainer departure | Primary maintainer unable to continue | MIT license + clean architecture + comprehensive documentation + typed reports mean the project can be forked and continued. The "bus factor" risk is mitigated by architecture, not by organizational structure. |
| Rust ecosystem shift | Rust adoption declines significantly or a clear successor emerges | The core algorithms are language-agnostic. The investment in Rust is justified by FFI, performance, and safety guarantees. If Rust genuinely declines (not just hype-cycle noise), evaluate porting to the successor with the same architectural constraints. |

### 6.4 What Does NOT Trigger Re-evaluation

These are things that might feel like triggers but are not:

- **A competitor getting more GitHub stars.** Stars are vanity metrics. Alaya competes on benchmarks and developer experience, not social proof.
- **A blog post saying "Alaya is too simple."** Simplicity is a feature. If the criticism is "Alaya cannot do X" where X is on the kill list, the answer is "correct, by design."
- **A request to add cloud sync.** On the kill list. The answer is no unless the metric triggers in Section 6.1 indicate that privacy-first positioning is the reason for zero adoption (not just one user's preference).
- **A faster competitor in a different language.** Performance is fifth in the conflict resolution hierarchy. If someone builds a faster memory system in Go, Alaya's response is to verify its own performance is acceptable, not to chase benchmarks at the expense of correctness or simplicity.

---

## 7. Document Governance

### 7.1 Authority

This document is authoritative for design decisions in the Alaya project. When a pull request, issue discussion, or design proposal conflicts with this extract, this document wins unless the re-evaluation triggers in Section 6 have been formally activated.

### 7.2 Amendment Process

1. **Identify trigger.** A re-evaluation trigger from Section 6 has been activated, with evidence.
2. **Document evidence.** Write a brief (1 page max) analysis of what changed and why.
3. **Propose amendment.** Specify exactly which axiom, non-goal, pattern, or invariant is affected and what the proposed change is.
4. **Review.** At minimum, sleep on it for 48 hours. If there are other contributors, discuss in a GitHub issue.
5. **Update.** Amend this document, update the version number, and note the change in the changelog below.

### 7.3 Changelog

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-02-26 | Initial extract generated from Phases 1-3 (Brand Guidelines, North Star, Competitive Landscape) |

### 7.4 Cross-References

| Document | Phase | Key Inputs to This Extract |
|----------|-------|----------------------------|
| Brand Guidelines | 1 | Beliefs (5), kill list (7), voice attributes, terminology |
| North Star Specification | 2 | Metric (MACC), personas, phases, technology constraints, positioning |
| Competitive Landscape | 3 | Rejected moves (6), differentiators (6), gaps, monitoring signals |

---

*This extract encodes Alaya's design DNA. It is the shortest path to understanding what the project will and will not become. Read it before proposing a feature. Read it before writing a PR. Read it before arguing about architecture. If you disagree with something here, start with Section 6 -- the triggers exist for a reason.*
