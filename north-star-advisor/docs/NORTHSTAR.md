# Alaya -- North Star Specification

> Memory is a process, not a database.

**Version:** 1.0
**Date:** 2026-02-26
**Status:** Active
**Owner:** Albert Hui

---

## PART 1: STRATEGIC FOUNDATION

### 1.1 North Star Metric

**Monthly Active Crate Consumers (MACC):** The count of unique Rust projects that call `alaya::AlayaStore::open()` or `alaya::AlayaStore::open_in_memory()` and execute at least one `store_episode()` + `query()` cycle within a 30-day period, as estimated by crates.io download telemetry and GitHub dependency graph.

This metric captures genuine adoption rather than curiosity downloads. A project that integrates Alaya, stores episodes, and retrieves memories is a project that has committed to the library. The metric filters out CI noise, abandoned experiments, and mirror downloads.

**Why MACC over downloads:**

| Metric | Signal | Noise |
|--------|--------|-------|
| Crates.io downloads | Broad interest | CI rebuilds, bots, mirrors |
| GitHub stars | Social proof | Drive-by attention |
| MACC | Genuine integration | Minimal -- requires real usage |

**Target trajectory:**

| Phase | Target MACC | Timeline |
|-------|-------------|----------|
| v0.1 (MVP) | 5 | First 60 days post-publish |
| v0.2 (MCP + benchmarks) | 25 | 90 days after v0.1 |
| v0.3 (Python bindings) | 100 | 6 months after v0.1 |
| v1.0 (stable API) | 500 | 12 months after v0.1 |

**Proxy metrics (when MACC is not directly measurable):** crates.io 90-day download count, GitHub dependents graph, issues opened by non-maintainer users, MCP server connection logs (self-reported, opt-in).

### 1.2 Input Metrics Hierarchy

The North Star breaks down into four input metric categories. Improving any one of these categories should move MACC upward. If it does not, the category is mis-specified and must be revised.

#### 1.2.1 API Completeness (Does the surface area meet developer expectations?)

| Metric | Current | Target (v0.2) | Measurement |
|--------|---------|---------------|-------------|
| Table-stakes operations coverage | 70% | 100% | Checklist: `get_episode()`, `delete_episode()`, `session_history()`, `user_id` scoping |
| Public method doctest coverage | 0% | 100% | `cargo test --doc` pass rate |
| Missing CRUD operations | 3 | 0 | Audit against Mem0 and Zep public APIs |
| Feature flag count | 0 | 4-6 | Cargo.toml optional features |

Missing table-stakes operations that block adoption (priority order):

1. `get_episode(id) -> Episode` -- retrieve a single episode by ID
2. `delete_episode(id)` -- remove a specific episode and cascade to FTS5, embeddings, links
3. `session_history(session_id) -> Vec<Episode>` -- retrieve full session transcript
4. `user_id` / `agent_id` scoping on all operations -- mandatory for multi-user agents
5. `dream()` convenience method -- chain consolidate + perfume + transform + forget

#### 1.2.2 Retrieval Quality (Does the library return the right memories?)

| Metric | Current | Target (v0.2) | Measurement |
|--------|---------|---------------|-------------|
| LoCoMo benchmark score | untested | >75% (beat Letta 74%) | Run standardized LoCoMo evaluation suite |
| LongMemEval benchmark score | untested | >70% (beat Mem0 68.5%) | Run standardized LongMemEval evaluation suite |
| BM25-only baseline (no embeddings) | untested | >55% LoCoMo | Prove graceful degradation works |
| Hybrid vs. BM25-only lift | unmeasured | >15 percentage points | A/B on same dataset |

LoCoMo and LongMemEval are the industry-standard benchmarks for conversational memory. Publishing scores establishes credibility and provides a concrete talking point for adoption.

#### 1.2.3 Developer Experience (How fast can a developer go from zero to working memory?)

| Metric | Current | Target (v0.1) | Measurement |
|--------|---------|---------------|-------------|
| Time to working example | ~3 min | <2 min | Stopwatch test: clone, `cargo add alaya`, copy example, run |
| Lines of code for basic integration | ~20 | <15 | Count LOC in minimal example |
| Compilation time (clean build) | ~25s | <20s | `cargo build --timings` on M1 Mac |
| Error message actionability | partial | every error has fix suggestion | Audit `AlayaError` variants for context |
| Examples directory completeness | 0 examples | 4 graduated examples | basic, lifecycle, custom provider, graph traversal |

#### 1.2.4 Ecosystem Integration (Can developers plug Alaya into their stack?)

| Metric | Current | Target (v0.2) | Measurement |
|--------|---------|---------------|-------------|
| MCP server available | no | yes | Working MCP server crate in workspace |
| Embedding providers | 0 shipped | 2 (ort, fastembed-rs) | Feature-flagged optional providers |
| FFI availability | none | C headers (cbindgen) | `alaya-ffi` crate published |
| Python bindings | none | PyO3 crate | `pip install alaya` works |
| Framework integrations documented | 0 | 2 (OpenClaw, generic) | Integration guides in docs |

### 1.3 Positioning

Alaya occupies a unique position in the agent memory landscape: the only library that combines a complete cognitive lifecycle with zero external dependencies. This is not a niche -- it is the intersection of two unserved demands (privacy-first architecture + neuroscience-grounded memory processes) that no competitor addresses simultaneously.

#### Positioning Matrix

| Dimension | Mem0 | Zep / Graphiti | Engram | LangChain Memory | **Alaya** |
|-----------|------|----------------|--------|------------------|-----------|
| Architecture | Cloud service, API-first | Temporal knowledge graph (Neo4j) | Local-first, simple KV | Framework module | Embedded library, single SQLite file |
| LLM dependency | Required (core to extraction) | Required (for graph construction) | Optional | Required (framework assumes LLM) | Optional (trait-based, graceful degradation) |
| Privacy model | Cloud-hosted, data leaves device | Neo4j instance required | Local file | Depends on framework | Architectural privacy (zero network calls) |
| Memory lifecycle | Store + retrieve | Store + temporal graph | Store + retrieve + basic consolidation | Store + retrieve (buffer/summary) | Four-stage cognitive lifecycle (consolidation, perfuming, transformation, forgetting) |
| Preference learning | None | None | None | None | Implicit emergence via vasana/perfuming |
| Graph model | None | Temporal knowledge graph | None | None | Hebbian graph with LTP/LTD dynamics |
| Forgetting | Manual deletion only | Temporal invalidation | Basic TTL | Buffer window | Bjork dual-strength model (storage vs. retrieval strength) |
| Language | Python | Python | Python/Rust | Python | Rust (FFI to C, Python, others) |
| External deps | Many (cloud infra) | Neo4j, LLM | Minimal | Heavy (framework) | Zero (single SQLite file) |
| Funding/stage | $24M, growth | $7.5M, growth | Solo/small | Backed by LangChain Inc. | Solo, self-funded |
| LoCoMo score | untested | untested | untested | untested | target >75% |

#### Positioning Statement

For agent developers who need conversational memory with privacy guarantees, Alaya is the embeddable Rust memory engine that provides a complete cognitive lifecycle -- consolidation, preference emergence, graph reshaping, and strategic forgetting -- in a single SQLite file with zero external dependencies. Unlike Mem0 (cloud-dependent, LLM-required), Zep (Neo4j-dependent), or Engram (simpler lifecycle), Alaya delivers neuroscience-grounded memory processes that work with or without an LLM, never phone home, and ship as `cargo add alaya`.

### 1.4 Target Users

#### Persona 1: The Privacy-First Agent Developer

**Name:** Priya
**Role:** Solo developer / small team building a companion or coaching agent
**Context:** Building an AI companion app (therapy assistant, personal coach, language tutor) where users share sensitive personal information. Runs on user devices or self-hosted infrastructure.

**Goals:**
- Ship a companion agent where the AI demonstrably remembers user context across sessions
- Guarantee to users that their data never leaves the device
- Avoid recurring cloud costs for memory infrastructure (Mem0 pricing, Neo4j hosting)
- Implement preference learning without writing custom ML pipelines

**Pains:**
- Every memory solution requires cloud calls, an LLM, or both
- Building memory from scratch means reinventing consolidation, deduplication, and forgetting
- Users ask "does my data leave my phone?" and the answer must be an unqualified "no"
- LLM API costs make per-user memory extraction prohibitively expensive at scale

**Alaya value:** Single SQLite file that never makes network calls. Cognitive lifecycle works without an LLM (graceful degradation). Preference emergence extracts user patterns without cloud-side processing. `cargo add alaya` and the privacy story writes itself.

**Success signal:** Priya ships her companion app with Alaya, and a user notices the AI "remembers" their preferences without being told twice. Priya writes about this in a blog post or conference talk.

#### Persona 2: The Performance-Focused Agent Developer

**Name:** Marcus
**Role:** Systems engineer building a coding agent, DevOps assistant, or conversational tool with deep session history (e.g., contributing to OpenClaw)
**Context:** Building agents that handle long conversations (100+ turns), maintain project context across sessions, and need fast retrieval from large memory stores. Performance and correctness matter more than ease of setup.

**Goals:**
- Sub-millisecond retrieval for context injection during streaming responses
- Memory that gets better over time (consolidation, not just accumulation)
- Benchmark-verified retrieval quality (LoCoMo, LongMemEval scores)
- Integration via MCP server or direct Rust API for maximum performance

**Pains:**
- Python memory libraries are too slow for latency-sensitive paths
- Existing solutions accumulate memories without consolidation, leading to context flooding
- No benchmarks exist for most memory libraries -- quality is a black box
- Framework lock-in (LangChain, LlamaIndex) constrains architectural decisions

**Alaya value:** Rust performance (no GC pauses, no Python overhead). Published LoCoMo/LongMemEval benchmark scores. Hebbian graph prevents context flooding through relevance-weighted retrieval. MCP server provides universal agent connectivity without framework lock-in. Cognitive lifecycle means the memory store improves through use rather than growing unbounded.

**Success signal:** Marcus integrates Alaya into OpenClaw (or a similar coding agent), measures retrieval latency under 1ms for 10K+ memories, and the agent's conversation quality noticeably improves compared to the previous memory approach.

### 1.5 Forces of Progress

The Forces of Progress framework explains why developers switch to Alaya. Adoption happens when push + pull forces exceed anxiety + habit.

#### Push Forces (pain with current solutions)

1. **Cloud dependency pain.** Mem0 requires API calls. Zep requires Neo4j. LangChain memory requires the LangChain framework. Every dependency is a failure mode, a cost center, and a privacy liability. Developers building for edge deployment, offline use, or privacy-regulated domains hit a wall.

2. **LLM cost at scale.** Memory libraries that require LLM calls for every extraction, consolidation, or retrieval operation impose per-interaction costs that scale linearly with users. A companion app with 10K daily active users and 50 interactions per user means 500K LLM calls per day for memory alone.

3. **Privacy as a feature request.** End users increasingly demand data sovereignty. Regulations (GDPR, state privacy laws) require data minimization. Developers cannot credibly promise privacy when their memory layer phones home to a cloud service.

4. **Context flooding.** Memory systems that only accumulate (never consolidate, never forget) eventually degrade agent performance. Developers resort to ad-hoc truncation, losing valuable long-term context.

#### Pull Forces (attraction to Alaya)

1. **Zero-ops deployment.** `cargo add alaya` and a single SQLite file. No database server, no cloud account, no API keys, no network configuration. The memory system deploys wherever the agent deploys.

2. **Cognitive lifecycle.** Four biologically-grounded processes (consolidation, perfuming, transformation, forgetting) that transform raw episodes into refined knowledge. The memory store improves through use -- a qualitative difference from append-only alternatives.

3. **Rust performance.** No garbage collection pauses. No Python GIL. Brute-force vector search at 10K vectors completes in microseconds. Retrieval latency stays predictable under load.

4. **Implicit preference emergence.** No other library extracts user preferences from observed behavior without explicit declaration. The vasana/perfuming process identifies behavioral patterns (communication style, topic preferences, schedule habits) that the agent can use without the user having to configure anything.

5. **Published benchmarks.** LoCoMo and LongMemEval scores provide objective, comparable evidence of retrieval quality. Developers can evaluate Alaya against alternatives on standardized tasks rather than trusting marketing claims.

#### Anxiety Forces (barriers to adoption)

1. **Rust learning curve.** Developers primarily working in Python face friction adopting a Rust library. Mitigation: Python bindings (v0.3), MCP server (v0.2), comprehensive examples, and `unsafe`-free public API.

2. **Solo maintainer risk.** Single developer, self-funded. Mitigation: MIT license (anyone can fork), clean architecture (readable codebase), comprehensive tests, and early community building.

3. **Unproven at scale.** No production deployments yet. Mitigation: published benchmarks, integration tests, documented scalability characteristics (vector search tiers, graph traversal limits).

#### Habit Forces (inertia keeping developers on current solutions)

1. **Existing Mem0/LangChain integration.** Switching costs are real when memory is wired into an agent. Mitigation: MCP server provides a non-invasive integration path that does not require rewriting the agent.

2. **Python ecosystem gravity.** Most agent developers work in Python. Mitigation: PyO3 bindings (v0.3), MCP server (language-agnostic), and the Rust crate itself for Rust-native projects.

---

## PART 2: SCOPE DEFINITION

### 2.1 Phase Boundaries

#### Phase 1 -- MVP (v0.1): Core Library

**Goal:** Publishable crate on crates.io with complete CRUD operations, working cognitive lifecycle, and LoCoMo benchmark baseline.

**Scope:**

| Component | Deliverable | Acceptance Criteria |
|-----------|-------------|---------------------|
| Episodic store | Complete CRUD: `store_episode()`, `get_episode()`, `delete_episode()`, `session_history()` | All operations work with cascading cleanup (FTS5, embeddings, links, strengths) |
| Semantic store | `store_node()`, `find_by_type()`, `count_nodes()` | Consolidation populates semantic store from episodes |
| Implicit store | Impressions + preferences CRUD | Perfuming process crystallizes preferences from impressions |
| Graph overlay | Hebbian links with LTP and LTD | Co-retrieval strengthens links; disuse decays them |
| Hybrid retrieval | BM25 + vector (optional) + graph activation + RRF fusion | Returns ranked `ScoredMemory` results |
| Cognitive lifecycle | `consolidate()`, `perfume()`, `transform()`, `forget()` | Each process produces a typed report |
| Forgetting | Bjork dual-strength decay | Retrieval strength decays inversely proportional to storage strength |
| User scoping | `user_id` and `agent_id` on all operations | Queries never leak across user boundaries |
| API polish | `AlayaConfig::builder()`, `NewEpisode::quick()`, `Query::with_embedding()` | Builder pattern with sensible defaults |
| Documentation | Compilable doctests on every public method, 4 graduated examples | `cargo test --doc` passes, examples compile and run |
| Benchmarks | LoCoMo evaluation suite | Published baseline score |
| Publish | `cargo publish` on crates.io | `cargo add alaya` works |

**Duration estimate:** 4-6 weeks from current state.

**Exit criteria:** crates.io publish succeeds, LoCoMo baseline score measured, at least one external user (OpenClaw) has evaluated the API surface.

#### Phase 2 -- Ecosystem (v0.2): MCP + Benchmarks + Config

**Goal:** Universal agent connectivity via MCP server, benchmark leadership, and production-grade configuration.

**Scope:**

| Component | Deliverable | Acceptance Criteria |
|-----------|-------------|---------------------|
| MCP server | `alaya-mcp` crate in workspace | Standard MCP protocol, all Alaya operations exposed as tools |
| LoCoMo benchmarks | Published scores >75% | Score reproducible from published benchmark harness |
| LongMemEval benchmarks | Published scores >70% | Score reproducible from published benchmark harness |
| Embedding providers | `ort` and `fastembed-rs` behind feature flags | `cargo add alaya --features=embeddings-ort` |
| Preference tradeoff resolution | Contextual tradeoff when preferences conflict | Given conflicting preferences, resolution considers recency, confidence, and domain context |
| Builder config | `AlayaConfig` with all tuning knobs | Decay rates, RRF k, graph depth, consolidation batch size -- all configurable |
| Async feature flag | `async` feature using `spawn_blocking` | `cargo add alaya --features=async` for Tokio compatibility |
| FFI | `alaya-ffi` crate with C headers via cbindgen | C consumers can link against Alaya |
| Security | FTS5 injection prevention, `BEGIN IMMEDIATE` for all writes, content validation hooks | Fuzz testing passes, no injection vectors |

**Duration estimate:** 6-8 weeks after v0.1.

**Exit criteria:** MCP server passes protocol conformance tests, LoCoMo >75%, at least one non-Rust integration (via MCP or FFI) demonstrated.

#### Phase 3 -- Growth (v0.3): Python + Community + Leadership

**Goal:** Python ecosystem access, community traction, benchmark leadership position.

**Scope:**

| Component | Deliverable | Acceptance Criteria |
|-----------|-------------|---------------------|
| Python bindings | `alaya-py` via PyO3, published on PyPI | `pip install alaya` works, Pythonic API with type hints |
| UniFFI bindings | Swift/Kotlin via UniFFI | Mobile agent developers can use Alaya |
| Community | 1000+ GitHub stars, active issue tracker | At least 5 external contributors |
| Benchmark leadership | Highest published LoCoMo and LongMemEval scores | Updated scores published with each release |
| DEF CON presentation | Talk or demo at DEF CON AI Village | Presentation delivered |
| Blog series | 3+ technical blog posts | Memory architecture, benchmark methodology, Yogacara mapping |
| Integration guides | OpenClaw, generic MCP, Python agent framework | Step-by-step guides with working code |

**Duration estimate:** 3-6 months after v0.2.

**Exit criteria:** 100 MACC, Python package published, DEF CON presentation delivered or scheduled.

### 2.2 Kill List

These items are explicitly out of scope. If a feature request maps to this list, the answer is "no" without further discussion.

1. **Not cloud-dependent.** Alaya makes zero network calls. Privacy is architectural, not policy-based. No telemetry, no analytics, no phone-home. The single SQLite file is the entire system.

2. **Not enterprise.** No multi-tenant isolation, no role-based access control, no horizontal scaling, no sharding. Alaya is an embedded library for single-agent use. If you need enterprise features, use Mem0 or Zep.

3. **Not LLM-coupled.** Alaya works without any LLM. The `ConsolidationProvider` trait is optional -- `NoOpProvider` ships as the default. Consolidation, perfuming, and contradiction detection are enhanced by an LLM but do not require one.

4. **Not hype-driven.** No "AI-powered" language. No "intelligent" claims without specifying the mechanism. Every capability maps to a named algorithm (Bjork, Hebbian, RRF, CLS) or a cited research concept (vasana, alaya-vijnana).

5. **Not a standalone service.** Alaya is a library. `cargo add alaya`. The MCP server (`alaya-mcp`) is a thin wrapper, not the primary interface. If you want a memory service, you host it yourself.

6. **Not procedural memory.** Alaya stores observations about what happened, not executable skills or procedures. It does not learn to perform tasks -- it learns what users care about.

7. **Not parametric memory.** Alaya operates in the non-parametric domain. No model fine-tuning, no weight updates, no training loops. Memory lives in structured data, not neural network parameters.

### 2.3 Licensing and Ethics

**License:** MIT. Alaya is and will remain fully open source. Anyone can fork, modify, and redistribute without restriction.

**Privacy commitments:**

- Zero network calls in the core library, now and always. This is not a configuration option -- it is an architectural invariant enforced by the absence of any networking dependency in `Cargo.toml`.
- No telemetry, no analytics, no usage tracking of any kind.
- Data never leaves the SQLite file unless the consuming agent explicitly reads and transmits it.
- The `purge()` API provides complete data deletion, including `VACUUM` to reclaim disk space. No ghost data.

**Ethical considerations:**

- Memory systems that learn preferences raise consent questions. Alaya provides the mechanism; the consuming agent is responsible for user consent, disclosure, and control. Documentation includes guidance on ethical preference use.
- The `forget()` API supports right-to-be-forgotten compliance. The `purge(PurgeFilter::All)` operation is a hard delete with no recovery, by design.
- Alaya does not evaluate, judge, or filter memory content. Content moderation is the agent's responsibility.

---

## PART 3: SUCCESS MEASUREMENT

### 3.1 Metrics Dashboard

The following metrics should be tracked continuously. The dashboard is a living artifact updated with each release.

#### Primary Metric

| Metric | Source | Frequency | v0.1 Target | v0.2 Target | v0.3 Target |
|--------|--------|-----------|-------------|-------------|-------------|
| MACC (Monthly Active Crate Consumers) | crates.io downloads + GitHub dependents | Monthly | 5 | 25 | 100 |

#### Input Metrics

| Category | Metric | Source | Frequency | Target |
|----------|--------|--------|-----------|--------|
| API Completeness | Missing table-stakes operations | Manual audit | Per release | 0 |
| API Completeness | Public method doctest coverage | `cargo test --doc` | Per commit (CI) | 100% |
| Retrieval Quality | LoCoMo score | Benchmark harness | Per release | >75% |
| Retrieval Quality | LongMemEval score | Benchmark harness | Per release | >70% |
| Retrieval Quality | BM25-only baseline | Benchmark harness | Per release | >55% |
| Developer Experience | Time to working example | Manual test | Per release | <2 min |
| Developer Experience | Clean build time | `cargo build --timings` | Per commit (CI) | <20s |
| Ecosystem | MCP server available | Binary check | Per release | v0.2 |
| Ecosystem | Python bindings available | PyPI check | Per release | v0.3 |

#### Health Metrics

| Metric | Source | Alert Threshold |
|--------|--------|-----------------|
| Open issues without response | GitHub | >10 for >7 days |
| CI pass rate | GitHub Actions | <95% over 7 days |
| Compile time regression | `cargo build --timings` | >20% increase vs. previous release |
| Dependency audit | `cargo audit` | Any known vulnerability |

### 3.2 Validation Gates

Each phase has hard gates that must pass before proceeding to the next phase. These are binary -- pass or fail, no exceptions.

#### v0.1 Gate (MVP -> Ecosystem)

| Gate | Criteria | Verification |
|------|----------|--------------|
| API completeness | All 5 missing table-stakes operations implemented | Integration test for each |
| Test coverage | All public methods have at least one integration test | `cargo test` passes |
| Documentation | Every public method has a compilable doctest | `cargo test --doc` passes |
| Benchmark baseline | LoCoMo score measured and recorded | Benchmark harness produces score |
| Publish | `cargo publish` succeeds | `cargo add alaya` installs v0.1 |
| External review | At least one developer outside the project has reviewed the API | Written feedback received |

#### v0.2 Gate (Ecosystem -> Growth)

| Gate | Criteria | Verification |
|------|----------|--------------|
| Benchmark leadership | LoCoMo >75% (beats Letta) | Published, reproducible score |
| MCP server | Passes MCP protocol conformance | Conformance test suite |
| Non-Rust integration | At least one working integration via MCP or FFI | Demo or test |
| Security | FTS5 injection fuzz test passes, `BEGIN IMMEDIATE` on all writes | Fuzz harness + code audit |
| Config completeness | All tuning parameters exposed via `AlayaConfig` | Builder API covers all knobs |

#### v0.3 Gate (Growth -> Stable)

| Gate | Criteria | Verification |
|------|----------|--------------|
| Python bindings | `pip install alaya` works, basic operations pass | PyPI publish + integration test |
| Community traction | 100+ GitHub stars, 5+ external issues | GitHub metrics |
| Benchmark publication | Both LoCoMo and LongMemEval scores published in README | README check |
| Presentation | DEF CON talk delivered or accepted | Confirmation |

### 3.3 Course Correction Triggers

These are signals that the current strategy is failing and requires reassessment. Each trigger has a specific response protocol.

| Trigger | Threshold | Response |
|---------|-----------|----------|
| Zero external users after 90 days on crates.io | MACC = 0, 90 days post-v0.1 | Conduct 5 developer interviews to identify blockers. Likely causes: API ergonomics, missing documentation, or positioning failure. |
| LoCoMo score below 60% | Benchmark score <60% | Retrieval pipeline needs fundamental rework. Investigate: embedding quality, RRF weighting, graph activation contribution. Consider adding reranking stage. |
| Compilation time exceeds 45s | Clean build >45s | Feature-gate heavy dependencies more aggressively. Profile with `cargo build --timings`. Consider splitting into workspace crates. |
| OpenClaw integration rejected | OpenClaw team evaluates and declines | Analyze rejection reasons. If API surface mismatch: adapt. If performance: optimize hot paths. If architectural: reassess MCP server approach. |
| Competitor ships zero-dep solution | Mem0, Zep, or new entrant ships embedded, zero-dep memory library | Accelerate v0.2 timeline. Differentiate on cognitive lifecycle and benchmark scores. The lifecycle (consolidation + perfuming + forgetting) is the moat. |
| Solo maintainer burnout | 30+ days without commits, no communication | Project design supports this. MIT license + clean architecture means anyone can fork and continue. Document bus factor mitigation in CONTRIBUTING.md. |

---

## PART 4: MVP ARCHITECTURE SUMMARY

### 4.1 Component Topology

Alaya's internal architecture consists of five layers. Each layer is a Rust module with a well-defined boundary. The layers compose vertically -- higher layers depend on lower layers but never the reverse.

```
                    +--------------------------------------------------+
                    |              AlayaStore (public API)              |
                    |  open() | store_episode() | query() | dream()    |
                    +--------------------------------------------------+
                           |                    |
              +------------+------------+       |
              |            |            |       |
     +--------v---+ +------v------+ +--v-------v--------+
     |  Episodic   | |  Semantic   | |     Implicit      |
     |   Store     | |   Store     | |      Store        |
     | (episodes)  | | (sem_nodes) | | (impressions +    |
     |             | |             | |  preferences)     |
     +------+------+ +------+------+ +--------+----------+
            |               |                  |
            +-------+-------+------------------+
                    |
           +--------v---------+
           |   Graph Overlay  |
           |  (Hebbian links, |
           |  LTP/LTD, spread |
           |   activation)    |
           +--------+---------+
                    |
           +--------v---------+      +---------------------+
           | Retrieval Pipeline|<---->|  Provider Traits    |
           | BM25 + Vector +  |      | ConsolidationProvider|
           | Graph -> RRF     |      | EmbeddingProvider   |
           +--------+---------+      +---------------------+
                    |
           +--------v---------+
           |  Lifecycle Layer  |
           | consolidate()    |
           | perfume()        |
           | transform()     |
           | forget()        |
           +--------+---------+
                    |
           +--------v---------+
           |   Storage Layer   |
           | SQLite (WAL mode) |
           | FTS5, embeddings, |
           | node_strengths    |
           +-------------------+
```

#### Layer Descriptions

**AlayaStore (public API).** The single entry point. Owns the SQLite connection. Exposes all operations through methods on this struct. Consumers never interact with internal modules directly.

**Three Stores.** The episodic store holds raw conversation turns (episodes) with timestamps, roles, session IDs, and context metadata. The semantic store holds consolidated knowledge nodes (facts, relationships, events, concepts) extracted from episodes via the consolidation process. The implicit store holds raw behavioral observations (impressions) and crystallized preferences that emerge from accumulated impressions via the perfuming process.

**Graph Overlay.** A Hebbian associative graph connecting nodes across all three stores. Links have typed relationships (temporal, topical, entity, causal, co-retrieval), directional weights (forward and backward), and activation history. Long-term potentiation (LTP) strengthens links on co-activation. Long-term depression (LTD) decays links on disuse. Spreading activation propagates relevance through the graph during retrieval using recursive CTEs with configurable depth (2-3 hops) and decay factor (0.5-0.7).

**Retrieval Pipeline.** Hybrid retrieval combining three signal types: BM25 full-text search (via FTS5), cosine similarity vector search (optional, requires embeddings), and graph neighbor activation. Signals are fused using Reciprocal Rank Fusion (RRF, k=60). The pipeline applies retrieval-induced forgetting (RIF) -- accessing a memory strengthens it while weakening competing memories, mirroring the human memory phenomenon.

**Provider Traits.** Extension points where the consuming agent plugs in LLM and embedding capabilities. `ConsolidationProvider` defines `extract_knowledge()`, `extract_impressions()`, and `detect_contradiction()`. `NoOpProvider` ships as the default, enabling the full API to work without any LLM. The agent owns the LLM connection -- Alaya never calls an LLM directly.

**Lifecycle Layer.** Four cognitive processes inspired by Complementary Learning Systems (CLS) theory and Yogacara Buddhist psychology:

- **Consolidation (CLS replay).** Replays recent episodes and extracts semantic knowledge nodes. Maps to hippocampal-neocortical memory consolidation. Requires `ConsolidationProvider` for LLM-based extraction; skips gracefully with `NoOpProvider`.
- **Perfuming (vasana).** Observes interactions, records behavioral impressions, and crystallizes preferences when impression clusters reach sufficient density. Named after the Yogacara concept of vasana (perfuming) -- repeated experiences leave traces that gradually shape disposition.
- **Transformation (asraya-paravrtti).** Deduplicates semantic nodes, prunes weak graph links, resolves contradictions, and decays stale preferences. Named after the Yogacara concept of transformation of the basis.
- **Forgetting (Bjork dual-strength).** Applies the Bjork & Bjork (1992) new theory of disuse. Each memory has two strengths: storage strength (how well-learned) and retrieval strength (how accessible now). Retrieval strength decays inversely proportional to storage strength -- well-learned memories become temporarily inaccessible but recover quickly on re-exposure. Weakly-learned memories that fall below threshold are archived.

**Storage Layer.** SQLite in WAL mode with `PRAGMA synchronous = NORMAL` for write performance. FTS5 external content tables with trigger-based synchronization for full-text search. Embeddings stored as BLOBs with a shared table indexed by (node_type, node_id). Node strengths table tracks Bjork dual-strength values. All write operations use `BEGIN IMMEDIATE` to prevent deferred transaction upgrade traps.

### 4.2 Technology Stack

| Layer | Technology | Version | Rationale |
|-------|-----------|---------|-----------|
| Language | Rust | 2021 edition | Zero GC, FFI-embeddable, single binary, memory safety without runtime cost |
| Database | rusqlite | 0.32 -> 0.38 (upgrade in v0.1) | Sync-first SQLite bindings with `bundled` feature (no system SQLite dependency) |
| Full-text search | SQLite FTS5 | (bundled with rusqlite) | External content tables, porter stemmer, trigger-based sync |
| Vector search | Brute-force (default), sqlite-vec (feature flag) | -- | Zero-dep default viable to ~10K vectors; sqlite-vec for SIMD acceleration to ~50K |
| Error handling | thiserror | 2.x | Derive macros for ergonomic error types; add `#[non_exhaustive]` to `AlayaError` |
| Serialization | serde + serde_json | 1.x | Already minimal, validated |
| Benchmarking | divan | latest | Attribute-based API, allocation profiling, better than criterion for this use case |
| FFI (v0.2) | cbindgen | -- | C header generation from Rust source |
| FFI (v0.3) | PyO3 | -- | Python bindings with native performance |
| CI | GitHub Actions | -- | `cargo test`, `cargo test --doc`, `cargo audit`, `cargo-semver-checks`, `cargo build --timings` |

### 4.3 Technology Constraints

These constraints are architectural invariants. Violating any of them requires a formal decision to revisit this document.

1. **Zero runtime dependencies beyond rusqlite, serde, serde_json, and thiserror.** Every additional dependency must justify itself against the zero-ops promise. Heavy dependencies (embedding runtimes, networking, async runtimes) must be behind feature flags.

2. **Sync-first.** The core library is synchronous. Async support (v0.2) is a feature flag that wraps sync operations in `spawn_blocking`. The async surface is a convenience layer, not a parallel implementation.

3. **Single SQLite file.** All data lives in one SQLite database. No sidecar files, no external indexes, no temporary directories. The SQLite file is the backup, the migration target, and the portability unit.

4. **No network calls.** The core library (`alaya` crate) contains no networking code. The MCP server (`alaya-mcp` crate) is a separate workspace member that depends on `alaya` but is not depended upon by it.

5. **Trait-based extension, not plugin architecture.** LLM integration, embedding generation, and custom storage backends are Rust traits that the consumer implements. No dynamic loading, no plugin discovery, no configuration files.

6. **`#[non_exhaustive]` on all public enums and error types.** New variants can be added in minor versions without breaking downstream code.

7. **Feature flags limited to 4-6 maximum.** Each flag gates a genuinely optional heavy dependency. Tentative list: `embeddings-ort`, `embeddings-fastembed`, `sqlite-vec`, `async`, `ffi-c`, `ffi-python`.

---

## PART 5: OPERATIONS

### 5.1 Launch Plan

#### v0.1 Launch (MVP)

| Step | Action | Channel | Timing |
|------|--------|---------|--------|
| 1 | Final API review with at least one external developer | Direct outreach | 1 week before publish |
| 2 | `cargo publish` on crates.io | crates.io | Day 0 |
| 3 | GitHub repository set to public (if not already) | GitHub | Day 0 |
| 4 | Announcement post on r/rust | Reddit | Day 0 |
| 5 | Announcement post on r/MachineLearning | Reddit | Day 0 |
| 6 | Technical blog post: "Building a neuroscience-grounded memory engine in Rust" | Personal blog / dev.to | Day 0-3 |
| 7 | HackerNews submission | HN | Day 1 (timed for US morning) |
| 8 | Direct outreach to OpenClaw team with integration guide | Email/Discord | Day 1 |

**Success criteria for v0.1 launch:** 50+ crates.io downloads in first week, 100+ GitHub stars in first month, at least 3 GitHub issues from non-maintainer users.

#### v0.2 Launch (MCP + Benchmarks)

| Step | Action | Channel | Timing |
|------|--------|---------|--------|
| 1 | Publish benchmark results in README with comparison table | GitHub | Day 0 |
| 2 | Blog post: "Alaya vs. Mem0 vs. Zep: LoCoMo benchmark results" | Personal blog | Day 0-3 |
| 3 | MCP server demo video (2 min) | YouTube / Twitter | Day 1 |
| 4 | Submit to MCP server registry / awesome-mcp list | GitHub PRs | Day 1 |
| 5 | r/LocalLLaMA post focusing on privacy + benchmarks | Reddit | Day 1 |
| 6 | OpenClaw integration PR or guide | GitHub / Discord | Day 1-7 |

#### v0.3 Launch (Python + DEF CON)

| Step | Action | Channel | Timing |
|------|--------|---------|--------|
| 1 | `pip install alaya` published on PyPI | PyPI | Day 0 |
| 2 | Python quickstart guide | GitHub docs | Day 0 |
| 3 | Blog post: "Alaya for Python developers" | Personal blog | Day 0-3 |
| 4 | DEF CON AI Village submission | DEF CON CFP | Aligned with CFP deadline |
| 5 | r/Python announcement | Reddit | Day 1 |
| 6 | Conference talk submissions (RustConf, PyCon, local meetups) | CFPs | Ongoing |

### 5.2 Risk Monitoring

| Risk | Likelihood | Impact | Mitigation | Monitor |
|------|-----------|--------|------------|---------|
| Solo maintainer burnout | Medium | Critical | MIT license enables forks. Clean architecture reduces bus factor. Pace development sustainably. | Commit frequency, personal check-ins |
| Competitor ships zero-dep solution | Low | High | Cognitive lifecycle + benchmarks are the moat. Accelerate v0.2 if detected. | Monitor Mem0/Zep/Engram releases, r/MachineLearning, HN |
| LoCoMo score disappoints (<60%) | Medium | High | Investigate retrieval pipeline: embedding quality, RRF weights, graph contribution. Add reranking stage. | Benchmark harness in CI |
| rusqlite breaking change | Low | Medium | Pin to minor version. Upstream is stable and well-maintained. | `cargo audit`, dependency bot |
| SQLite limitation at scale | Low | Medium | Vector brute-force fails at ~50K. Documented escape hatch: sqlite-vec feature flag. | Performance benchmarks in CI |
| OpenClaw window closes | Medium | Medium | MCP server provides universal connectivity. OpenClaw is a target, not a dependency. | OpenClaw community activity |
| Python bindings too slow via PyO3 | Low | Medium | PyO3 overhead is minimal for database operations. Benchmark Python vs. Rust for critical paths. | Python integration benchmarks |
| Memory poisoning attack | Low | High | Content validation hooks, quarantine API, content-hash integrity. Document threat model. | Security advisory monitoring |
| GDPR/privacy complaint | Low | High | Architectural privacy (no network calls). `purge()` API with `VACUUM`. Crypto-shredding option (v0.2). | Legal landscape monitoring |

---

## PART 6: DOCUMENT HIERARCHY

This document is one component of the North Star Advisor specification suite. The documents build on each other -- later documents reference earlier ones.

| # | Document | Status | Purpose |
|---|----------|--------|---------|
| 1 | BRAND_GUIDELINES.md | Complete | Product identity, beliefs, voice, terminology, kill list |
| 2 | **NORTHSTAR.md** | **This document** | **Strategic foundation, metrics, scope, architecture summary** |
| 3 | COMPETITIVE_LANDSCAPE.md | Planned | Detailed competitor analysis, feature comparison, market positioning |
| 4 | AXIOMS.md | Planned | Core engineering axioms derived from beliefs and research |
| 5 | USER_STORIES.md | Planned | Detailed user stories for each persona and phase |
| 6 | ARCHITECTURE_BLUEPRINT.md | Planned | Detailed technical architecture, data flow, schema design |
| 7 | INTEGRATION_PATTERNS.md | Planned | MCP server, direct Rust API, FFI consumption patterns |
| 8 | SECURITY_ARCHITECTURE.md | Planned | Threat model, memory poisoning, FTS5 injection, GDPR compliance |
| 9 | BENCHMARK_METHODOLOGY.md | Planned | LoCoMo/LongMemEval setup, scoring, reproducibility |
| 10 | OPS_RUNBOOK.md | Planned | WAL checkpointing, FTS5 maintenance, graph pruning, consolidation scheduling |
| 11 | CONTRIBUTING.md | Planned | Contributor guide, code style, review process |
| 12 | ROADMAP.md | Planned | Milestone timeline, phase dependencies, release planning |

**Cross-reference protocol:** Documents reference each other using the format `[DOCUMENT_NAME S1.2]` (e.g., `[BRAND_GUIDELINES S2.1]` refers to section 2.1 of BRAND_GUIDELINES.md). This allows precise traceability without fragile hyperlinks.

---

## APPENDIX: GLOSSARY

### Yogacara Terms

| Term | Sanskrit | Meaning in Alaya |
|------|----------|-------------------|
| Alaya-vijnana | alaya-vijnana (storehouse consciousness) | The foundational metaphor. Alaya is the persistent substrate that holds seeds (bija) of experience. Seeds are not static records -- they transform through interaction, strengthen through reinforcement, and fade through disuse. |
| Bija | bija (seed) | A unit of stored experience. In Alaya, this maps to any stored node: an episode, a semantic node, a preference. Seeds have potential energy (storage strength) that may or may not manifest (retrieval strength). |
| Vasana | vasana (perfuming) | The process by which repeated experiences leave traces that gradually shape disposition. In Alaya, this is the perfuming lifecycle stage: raw impressions accumulate and crystallize into preferences when patterns reach sufficient density. The name reflects that preferences are not declared -- they emerge from the residue of experience, like cloth absorbing the scent of incense. |
| Asraya-paravrtti | asraya-paravrtti (transformation of the basis) | The process of fundamental restructuring. In Alaya, this is the transformation lifecycle stage: deduplication, contradiction resolution, and graph reorganization. The knowledge base does not just grow -- it restructures. |

### Neuroscience Terms

| Term | Origin | Meaning in Alaya |
|------|--------|-------------------|
| CLS (Complementary Learning Systems) | McClelland et al. (1995) | The theory that memory involves two complementary systems: a fast-learning episodic system (hippocampus) and a slow-learning semantic system (neocortex). In Alaya, episodes are the fast system; consolidation gradually extracts knowledge into the semantic store (slow system). |
| Hebbian learning | Hebb (1949) | "Neurons that fire together wire together." In Alaya, nodes that are retrieved together strengthen their graph links (LTP). Nodes that are never co-retrieved see their links decay (LTD). The graph reshapes through use. |
| LTP / LTD | Bliss & Lomo (1973) / Dudek & Bear (1992) | Long-term potentiation (strengthening) and long-term depression (weakening) of synaptic connections. In Alaya, LTP increases link weights on co-activation; LTD applies multiplicative decay on disuse. |
| Bjork dual-strength | Bjork & Bjork (1992) | The new theory of disuse. Each memory has storage strength (how well-learned) and retrieval strength (how accessible right now). Storage strength only increases. Retrieval strength decays, but the rate of decay is inversely proportional to storage strength. A well-learned memory that becomes temporarily inaccessible recovers quickly when cued. |
| RIF (Retrieval-Induced Forgetting) | Anderson et al. (1994) | Retrieving a memory suppresses competing memories. In Alaya, successful retrieval of a node slightly decays the retrieval strength of neighbors that were not selected. This naturally curates the memory space toward relevance. |
| Spreading activation | Collins & Loftus (1975) | Activation propagates through an associative network from a starting node to connected nodes, with decay at each hop. In Alaya, spreading activation is implemented via recursive CTEs on the graph overlay, with configurable depth and decay factor. |

### Technical Terms

| Term | Meaning in Alaya |
|------|-------------------|
| RRF (Reciprocal Rank Fusion) | A rank aggregation method that combines ranked lists from multiple retrieval signals (BM25, vector, graph) into a single ranking. Score = sum(1 / (k + rank_i)) where k=60. Simple, effective, parameter-light. |
| BM25 | A probabilistic full-text search ranking function. In Alaya, implemented via SQLite FTS5. The baseline retrieval signal that works without embeddings. |
| FTS5 | SQLite's full-text search extension. Alaya uses external content tables with trigger-based synchronization to keep the FTS5 index consistent with the episodes table. |
| WAL (Write-Ahead Logging) | SQLite journal mode that allows concurrent reads during writes. Alaya enables WAL mode on database creation for write performance and read concurrency. |
| Cognitive lifecycle | The four lifecycle processes collectively: consolidation, perfuming, transformation, forgetting. This is Alaya's primary differentiator -- no competitor implements all four. |
| Preference emergence | The vasana/perfuming process by which implicit preferences are extracted from observed behavior. Impressions accumulate; when a cluster reaches sufficient density, a preference crystallizes. The agent does not ask the user for preferences -- it observes them. |
| Memory engine | The preferred term for Alaya as a whole. Emphasizes that Alaya is active (engine) rather than passive (database). Avoid "memory database" or "memory store." |
| Graceful degradation | Alaya's design principle that every feature works independently. No embeddings? BM25-only retrieval still works. No LLM? Episodes accumulate without consolidation. No graph? Retrieval falls back to text + vector signals. Each capability adds value but none is required. |

---

*Generated by North Star Advisor. Cross-references: [BRAND_GUIDELINES] for beliefs, kill list, voice, and terminology.*
