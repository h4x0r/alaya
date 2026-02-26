# Strategic Recommendation: Alaya Memory Engine

**Generated:** 2026-02-26 | **Phase:** 11 (Strategy Synthesis)
**Status:** Active | **Review Date:** 2026-04-01

---

## How to Read This Document

This is the synthesis document for the Alaya project. It draws on every preceding phase of strategic analysis -- brand positioning, north star metrics, competitive landscape, axiom extraction, architecture blueprints, security posture, ADRs, post-publication operations, resilience patterns, implementation scaffolding, and testing strategy -- to produce a single recommended path forward.

The recommendation is directed at one person: the solo maintainer making daily build-sequence decisions under time pressure and resource constraints. Every section is designed to resolve ambiguity, not create it.

**Cross-reference index:**
- Brand: `brand.yml` (beliefs, kill list, voice)
- North Star: `northstar.yml` (MACC metric, phases, personas)
- Competition: `competitive.yml` (market shifts, differentiators)
- Axioms: `extract.yml` (axioms, non-goals, always/never lists)
- Architecture: `architecture.yml` (components, gaps, pipeline)
- Integration: `agent-prompts.yml` (consumer patterns, anti-patterns)
- Security: `security.yml` (threats, hardening roadmap)
- Decisions: `adr.yml` (ADR-001 through ADR-010)
- Operations: `post-deployment.yml` (release gates, community)
- Scaffold: `scaffold.yml` (module tree, known gaps)
- Resilience: `resilience.yml` (degradation, idempotency)
- Testing: `testing.yml` (coverage, benchmarks, CI)

---

## Part 1: Situation Summary

### 1.1 The Challenge

Alaya is an embeddable Rust memory library that implements a complete cognitive lifecycle -- consolidation, preference emergence, adaptive forgetting, and Hebbian graph dynamics -- in a single SQLite file with zero external dependencies. The library is functional, tested (43 passing tests), and architecturally sound (10 accepted ADRs grounding every design choice in cited research). But it has never been published, has zero users, and has 13 identified hardening gaps that stand between the current codebase and a responsible v0.1 release.

The challenge is sequencing. There are three plausible paths forward, each with genuine merit, and the solo-developer constraint means only one can be pursued at a time. Choosing wrong does not destroy the project -- Alaya's architecture is durable -- but it can waste the only non-renewable resource available: the maintainer's time during a specific window of ecosystem opportunity.

### 1.2 Context

**What exists today:**

The codebase is 4,064 lines across 25 files. It implements the full three-store architecture (episodic, semantic, implicit), a Hebbian graph overlay with LTP dynamics and spreading activation, a hybrid retrieval pipeline (BM25 + vector + graph -> RRF fusion -> reranking), and four lifecycle processes (consolidation via CLS replay, vasana perfuming for preference emergence, transformation for maintenance, and Bjork dual-strength forgetting). All 43 unit tests pass. The public API surface is 12 methods on `AlayaStore`, plus a `ConsolidationProvider` trait with `NoOpProvider` fallback enabling graceful degradation from full-capability to BM25-only to empty-result without error.

The dependency footprint is minimal: `rusqlite 0.32` (bundled SQLite), `serde`, `serde_json`, `thiserror`. No networking dependencies. No LLM dependencies. This is not an aspirational constraint -- it is a structural fact of the current codebase, verifiable by `cargo tree`.

**What does not exist:**

- Zero compilable doctests on public methods (GAP-005)
- No `#[non_exhaustive]` on public enums (GAP-001) -- adding enum variants without it is a semver break
- No `BEGIN IMMEDIATE` for write transactions (GAP-002) -- deadlock risk under concurrent access
- No input validation at API boundary (GAP-003)
- No tombstone mechanism for deleted nodes (GAP-008) -- memory resurrection risk
- No CI pipeline, no MSRV pinning, no benchmarks, no integration tests
- No published benchmark results (LoCoMo, LongMemEval)
- No documentation beyond doc comments
- No MCP server, no C FFI, no Python bindings
- Zero community (no stars, no dependents, no issues)

**Competitive position:**

Alaya occupies the only uncontested quadrant in the agent memory landscape: high cognitive completeness combined with high operational simplicity. Every competitor either requires cloud infrastructure (Mem0, Zep/Graphiti, Letta, Supermemory, Hindsight) or lacks cognitive depth (Engram, Memvid). No shipping system implements Bjork dual-strength forgetting. No system does implicit preference emergence without an LLM. No system has a dynamic Hebbian graph that reshapes through use without LLM involvement. The intersection of "complete cognitive lifecycle" and "zero external dependencies" contains exactly one occupant: Alaya.

This positioning advantage is not permanent. The SYNAPSE research project implements spreading activation with lateral inhibition. MAGMA demonstrates multi-graph decomposition with SOTA benchmarks. RL-trained memory systems (Memory-R1, Mem-alpha, AgeMem) may discover management strategies that hand-crafted processes miss. These are all Python research code today, not shipping libraries. But research code has a way of becoming shipping code within 12-18 months when the timing is right.

**Timing windows:**

1. **OpenClaw ecosystem (now through Q3 2026):** The open-source agent ecosystem is actively selecting components. Memory is an unresolved need. Being present during selection is more valuable than being perfect after it.

2. **DEF CON AI Village (August 2026):** A presentation slot provides credibility within the security-conscious, privacy-first developer community -- Alaya's exact target persona. This requires the library to exist, be usable, and have publishable results.

3. **Pre-RL productionization (12-18 months):** RL-trained memory systems are in research phase. Alaya's hand-crafted processes have a window before learned policies potentially outperform them. That window is best used to establish benchmark baselines and community presence, not to optimize in isolation.

4. **Edge AI memory gap (2026-2028):** On-device models are accelerating. No memory system is optimized for ARM/mobile/edge deployment. Alaya's single-file, zero-network architecture is already edge-ready by construction.

### 1.3 What Is at Stake

The immediate question is not whether Alaya should exist -- the architecture is sound, the positioning is defensible, and the research grounding is genuine. The question is whether Alaya will exist _in the world_ before the market either fills its quadrant or stops looking for occupants.

A library with zero users teaches nothing. It cannot validate its retrieval quality against real workloads. It cannot discover API ergonomics problems that only surface when someone outside the maintainer's head tries to use it. It cannot benefit from community contributions that close the gap between 43 tests and 150. And it cannot participate in ecosystem conversations that are happening now.

The stakes are not existential. The code is open-source, the architecture is clean, the MIT license means it is forkable by design. But the window for establishing first-mover presence in the "embedded cognitive memory" category is finite, and the solo-developer constraint means every week spent on one thing is a week not spent on another.

### 1.4 Key Insights from Prior Analysis

These insights emerge from synthesizing all 12 preceding outputs:

**Insight 1: The hardening gaps are real but bounded.** The scaffold analysis identifies 13 gaps in three priority tiers. The P0 tier (5 items: `#[non_exhaustive]`, `BEGIN IMMEDIATE`, input validation, `pub(crate)` visibility, doctests) is the minimum bar for responsible publication. These are well-defined tasks with clear acceptance criteria. They do not require architectural changes, external dependencies, or creative judgment. A focused sprint can close them in 1-2 weeks.

**Insight 2: Benchmarks require the library to be usable first.** Running LoCoMo requires ingesting multi-session conversation data, executing queries, and measuring precision/recall/nDCG. This requires a stable API surface, a working retrieval pipeline (which exists), and sufficient test infrastructure to produce reproducible results. But benchmark numbers have meaning only when external developers can reproduce them. A benchmark blog post pointing to an unpublished crate is an academic exercise, not a credibility play.

**Insight 3: The MCP server is an adoption multiplier, not a prerequisite.** The integration patterns analysis shows that the Rust API (`store_episode` + `query` + lifecycle methods) is the primary interface. The MCP server lowers the barrier for non-Rust agents, but the v0.1 target of 5 MACC is achievable through direct Rust consumers alone. The MCP server is correctly placed at v0.2.

**Insight 4: The OpenClaw window rewards presence over perfection.** Ecosystem component selection favors candidates that exist, are documented, and can be evaluated. A published crate with honest benchmark baselines and good documentation outcompetes a hypothetical crate with theoretically better numbers. The competitive landscape shows that Mem0 established its position with a 68.5% LoCoMo score -- a number that Alaya's architecture should match or exceed, but that did not need to be exceptional to be influential.

**Insight 5: Solo-developer constraints make sequencing critical.** With one person doing all implementation, documentation, testing, benchmarking, community engagement, and ecosystem outreach, the cost of context-switching is high. Each path (benchmarks, DX, OpenClaw) requires a different primary activity: measurement/analysis for benchmarks, writing/examples for DX, targeted integration for OpenClaw. Hybrid approaches that do a little of each tend to finish nothing.

**Insight 6: The axioms resolve the choice.** The project's own axiom hierarchy -- `Privacy > Features`, `Process > Storage`, `Correctness > Speed`, `Simplicity > Completeness`, `Honesty > Marketing` -- provides a decision framework. "Honesty > Marketing" says publish benchmarks even when numbers are bad. "Correctness > Speed" says research grounding before shipping velocity. But "Simplicity > Completeness" says a working, minimal release outweighs a comprehensive delayed one. The axioms do not unanimously support any single path, but they weight most heavily toward getting a correct, simple, honest release into the world.

---

## Part 2: Strategic Paths

### Path A: "Benchmark First"

**Description:** Focus implementation effort on running LoCoMo and LongMemEval benchmark suites, optimizing retrieval quality, and publishing results as a research-credibility play. Documentation and developer experience are secondary. Publication follows benchmark achievement.

**Sequencing:**
1. Implement benchmark harness (2-3 weeks)
2. Run LoCoMo baseline, identify retrieval pipeline weaknesses (1 week)
3. Optimize: wire RIF suppression, enrich semantic nodes in pipeline, tune RRF k and rerank weights (3-4 weeks)
4. Run LongMemEval (1 week)
5. Write benchmark blog post with reproducible methodology (1 week)
6. Close P0 gaps and publish to crates.io (1-2 weeks)
7. Submit to r/MachineLearning, arXiv preprint (optional)

**Timeline:** 10-13 weeks to publication

**Strengths:**
- Benchmark numbers provide concrete credibility (addresses Marcus persona directly)
- Research narrative aligns with DEF CON presentation track
- Identifies real retrieval quality issues before they become user-facing bugs
- Differentiates from competitors who do not publish reproducible numbers
- "Honesty > Marketing" axiom fully satisfied -- publish whatever numbers emerge

**Weaknesses:**
- Delays publication by 2-3 months (benchmark optimization is open-ended)
- Benchmark numbers without usable documentation are not actionable by developers
- Solo developer doing optimization work in isolation, with no user feedback loop
- Risk of benchmark-chasing: optimizing for LoCoMo at the expense of real-world retrieval quality
- OpenClaw window may close or narrow during optimization period
- No community engagement until publication; community traction starts from zero at a later date
- LoCoMo/LongMemEval may not fully exercise Alaya's unique features (vasana, Hebbian graph)

**Risk assessment:**
- **Execution risk:** Medium. Benchmark infrastructure is straightforward, but optimization cycles are unpredictable. The retrieval pipeline has known gaps (GAP-007: semantic node enrichment not implemented, GAP-009: RIF not wired) that affect benchmark scores but are not trivial to close.
- **Market risk:** High. Every week of delayed publication is a week where ecosystem decisions happen without Alaya present.
- **Axiom alignment:** Mixed. Satisfies "Honesty > Marketing" and "Correctness > Speed." Conflicts with "Simplicity > Completeness" (benchmarks add complexity before simplicity is validated by users).

**Expected MACC at 6 months:** 3-8 (late start but strong positioning for benchmark-aware consumers)

### Path B: "Developer Experience First"

**Description:** Focus on documentation, examples, onboarding flow, and a polished first-use experience. Close P0 hardening gaps, write comprehensive docs, create a quickstart example, and publish to crates.io as soon as the library is usable. Benchmark measurement follows publication when real usage informs optimization priorities.

**Sequencing:**
1. Close P0 hardening gaps: `#[non_exhaustive]`, `BEGIN IMMEDIATE`, input validation, `pub(crate)`, doctests (1-2 weeks)
2. Write README with quickstart, architecture overview, code examples (1 week)
3. Create `examples/` directory: basic agent, lifecycle demo, custom provider (1 week)
4. Close P1 quality gaps: LTD in transform, semantic enrichment, tombstones, RIF wiring (2-3 weeks)
5. Set up CI pipeline (1 week)
6. Publish v0.1.0 to crates.io (1 day)
7. Launch: r/rust, HackerNews, OpenClaw outreach, blog post (1 week)
8. Run LoCoMo baseline, publish results (2-3 weeks post-launch)
9. Collect feedback, iterate (ongoing)

**Timeline:** 6-8 weeks to publication

**Strengths:**
- Fastest path to publication and first MACC data point
- Satisfies "Simplicity > Completeness" axiom -- ship the simple version first
- Enables user feedback loop before optimization investment
- OpenClaw outreach happens during ecosystem selection window
- Documentation investment pays dividends across all future paths
- Identifies real DX problems (not hypothetical ones) through actual usage
- Aligns with v0.1 phase goal: "Publishable crate on crates.io with complete CRUD, cognitive lifecycle, LoCoMo baseline"
- Community engagement starts earliest, compounding over time

**Weaknesses:**
- Publishes without benchmark numbers (benchmark blog follows 2-3 weeks later)
- Marcus persona (performance-focused) may not engage without published numbers
- Competitors with published benchmarks (Mem0 68.5%, Hindsight 89.6%) have a credibility advantage
- Risk of premature negative impression if DX is good but retrieval quality disappoints
- Known retrieval gaps (GAP-007, GAP-009) mean benchmark numbers, when published, may be lower than optimized scores would be

**Risk assessment:**
- **Execution risk:** Low. P0 gaps are well-defined, documentation is within maintainer's control, publication process is mechanical. The highest risk is writing good docs (creative work with uncertain time estimates), but "good enough" docs are achievable in the timeline.
- **Market risk:** Low. Gets into the market earliest. Retrieval quality risk is mitigated by the "Honesty > Marketing" axiom -- publish baseline numbers honestly, improve in v0.1.x and v0.2.
- **Axiom alignment:** Strong. "Simplicity > Completeness" (ship the simple version). "Honesty > Marketing" (publish honest baselines). "Process > Storage" (the lifecycle is the value proposition, and users can experience it without benchmarks). Mild tension with "Correctness > Speed" (shipping before full optimization).

**Expected MACC at 6 months:** 5-15 (earliest start, longest feedback loop, OpenClaw window captured)

### Path C: "OpenClaw Window"

**Description:** Sprint to produce an OpenClaw-compatible integration as the primary publication vehicle. Scope features tightly to what the OpenClaw architecture requires. Broader ecosystem play follows.

**Sequencing:**
1. Research OpenClaw architecture requirements and memory interface expectations (1 week)
2. Close minimum P0 gaps required for integration (1-2 weeks)
3. Build OpenClaw-specific integration layer or example (2-3 weeks)
4. Publish to crates.io with OpenClaw-focused documentation (1 week)
5. Submit PR or proposal to OpenClaw project (1 week)
6. Broaden documentation and examples for general audience (2-3 weeks)
7. Run benchmarks, iterate (2-3 weeks)

**Timeline:** 8-11 weeks to publication (but publication is OpenClaw-coupled)

**Strengths:**
- Direct path to the most strategically valuable adoption event
- Concrete external deadline provides focus and prevents scope creep
- If accepted, OpenClaw integration is the strongest credibility signal possible
- Marcus persona (OpenClaw contributor) is directly served
- Validates API design against a real, complex consumer

**Weaknesses:**
- Couples Alaya's timeline to an external project's decisions and priorities
- OpenClaw may not be ready for memory component integration (their timeline is uncertain)
- If OpenClaw chooses differently, the sprint's specificity becomes a liability
- Narrowing focus to one consumer may produce an API shaped around their quirks rather than general utility
- OpenClaw-specific documentation is less useful for Priya (privacy-first companion agent) persona
- Risk of political rejection regardless of technical merit
- No fallback if OpenClaw integration does not materialize

**Risk assessment:**
- **Execution risk:** Medium-high. Depends on understanding OpenClaw's needs accurately without full specification. Integration work may surface API design issues requiring architectural changes.
- **Market risk:** Binary. If accepted, extremely high value. If rejected or deferred, the time investment has limited transferability.
- **Axiom alignment:** Weak. "Privacy > Features" is not particularly served (OpenClaw may have different privacy requirements). "Simplicity > Completeness" is violated (integration adds complexity for one consumer). "Correctness > Speed" tension is high (sprint timeline pressures correctness).

**Expected MACC at 6 months:** 0-1 (if rejected) or 10-30 (if accepted and OpenClaw drives adoption)

### Path Comparison Matrix

| Dimension | Path A: Benchmark First | Path B: DX First | Path C: OpenClaw Window |
|-----------|------------------------|-------------------|------------------------|
| **Time to crates.io** | 10-13 weeks | 6-8 weeks | 8-11 weeks |
| **MACC at 6 months** | 3-8 | 5-15 | 0-1 or 10-30 |
| **Axiom alignment** | Mixed | Strong | Weak |
| **Execution risk** | Medium | Low | Medium-high |
| **Market risk** | High (late) | Low (earliest) | Binary (external dep) |
| **Feedback loop** | Late, benchmark-focused | Early, broad | Late, narrow |
| **OpenClaw window** | Partially missed | Captured | Targeted |
| **DEF CON alignment** | Strong | Moderate | Weak |
| **Priya persona** | Deferred | Served directly | Indirectly |
| **Marcus persona** | Served directly | Deferred (2-3 wks) | Served directly |
| **Documentation quality** | Low initial | High initial | Narrow initial |
| **Solo-dev sustainability** | Grinding | Sustainable | Sprinting |
| **Recovery from failure** | Wasted optimization | Broad foundation | Narrow investment |

---

## Part 3: The Recommendation

### Recommended Path: B -- Developer Experience First, with Benchmark Fast-Follow

**Chosen path:** Path B, modified with an explicit benchmark commitment baked into the first 3 weeks post-publication.

**The reasoning is structural, not preferential.**

The recommendation emerges from three converging analyses:

**First, the axiom hierarchy resolves the tie.** Paths A and B both have genuine merit. The axioms are the tiebreaker. "Simplicity > Completeness" says ship the minimal correct version. "Honesty > Marketing" says publish honest numbers, not optimized ones. "Correctness > Speed" says do not skip hardening. Path B satisfies all three: harden first (Correctness), ship simple (Simplicity), benchmark honestly after (Honesty). Path A satisfies Honesty and Correctness but violates Simplicity by delaying publication for optimization.

**Second, the feedback loop argument is decisive for a solo developer.** The scaffold analysis shows 13 known gaps. The testing analysis shows 43 tests against a v0.1 target of 150. The security analysis identifies 10 threat vectors with 3 unmitigated. These are the _known_ gaps. A solo developer working in isolation discovers only the gaps that their own mental model predicts. The first external user discovers a different category of gap entirely: API ergonomics problems, confusing error messages, missing CRUD operations that seem obvious from outside, documentation assumptions that do not hold. Every week of isolated optimization delays the discovery of these qualitatively different problems.

**Third, the downside analysis favors B.** If Path B fails (the library gets published but nobody uses it), the result is: a published crate with good documentation, closed hardening gaps, and a foundation for benchmark work. If Path A fails (benchmarks take longer than expected, or numbers are disappointing), the result is: an unpublished crate with benchmark infrastructure but no documentation, no community presence, and a demoralized maintainer who spent 3 months optimizing in isolation. If Path C fails (OpenClaw declines), the result is: a crate shaped around one consumer's needs with narrow documentation. Path B's failure mode produces the most reusable assets.

### The Modified Sequence

The core modification to Path B is making the benchmark commitment explicit and time-boxed, not aspirational.

**Phase 1: P0 Hardening (Weeks 1-2)**

Close the five P0 hardening gaps that are non-negotiable for publication:

1. **`#[non_exhaustive]` on all public enums** (GAP-001). Files: `src/types.rs`, `src/error.rs`. This is a one-time change that prevents accidental semver breaks when adding variants. It must happen before v0.1.0 because adding it after publication is itself a breaking change.

2. **`BEGIN IMMEDIATE` for write transactions** (GAP-002). Files: `src/schema.rs`, `src/lib.rs`. Implement an `immediate_transaction()` helper and use it for all write paths: `store_episode`, `consolidate`, `perfume`, `transform`, `forget`, `purge`. This prevents deadlocks under concurrent access (threat T7 in security analysis).

3. **Input validation at API boundary** (GAP-003). File: `src/lib.rs`. Validate non-empty content in `store_episode()`, non-zero embedding dimensions, clamp confidence values to `[0.0, 1.0]`, reject NaN/infinity in embeddings. Return `AlayaError::InvalidInput` with descriptive messages.

4. **Internal modules to `pub(crate)`** (GAP-004). File: `src/lib.rs`. Change `pub mod store`, `pub mod graph`, `pub mod retrieval`, `pub mod lifecycle`, `pub mod schema` to `pub(crate) mod`. This prevents consumers from depending on internal APIs.

5. **Compilable doctests on all public methods** (GAP-005). File: `src/lib.rs`. Write a doctest for each of the 12 `AlayaStore` methods, plus key types and traits. Each doctest uses `open_in_memory()` and is self-contained. This directly addresses the docs.rs experience and is a v0.1 exit criterion.

**Phase 2: P1 Quality + Documentation (Weeks 3-5)**

Close the four P1 quality gaps and write publication-ready documentation:

6. **Wire LTD in `transform()`** (GAP-006). File: `src/lifecycle/transformation.rs`. Call `decay_links()` during transformation to implement the long-term depression half of the Hebbian cycle.

7. **Semantic/preference node enrichment in retrieval pipeline** (GAP-007). File: `src/retrieval/pipeline.rs`. Currently, non-episode nodes are dropped during enrichment. Retrieve semantic nodes and preferences by ID when they appear in RRF fusion results.

8. **Tombstone mechanism** (GAP-008). File: `src/schema.rs`. Add `tombstones` table. Check tombstones during consolidation to prevent resurrection of deleted content. Cascade deletion through episodes -> embeddings, links, strengths.

9. **RIF suppression in retrieval pipeline** (GAP-009). Files: `src/retrieval/pipeline.rs`, `src/store/strengths.rs`. Wire the existing `suppress()` function into post-retrieval effects so that competitors of retrieved memories have their retrieval strength reduced, implementing retrieval-induced forgetting.

10. **README with quickstart** (new). Write a README.md that gets a developer from `cargo add alaya` to a working store/query/consolidate example in under 2 minutes. Include architecture diagram (text-based), feature overview, and links to docs.rs.

11. **Examples directory** (new). Create `examples/basic_agent.rs` (minimal store/query loop), `examples/lifecycle_demo.rs` (full consolidation/forgetting cycle), and `examples/custom_provider.rs` (implementing `ConsolidationProvider` with a mock LLM).

12. **Cargo.toml metadata** (GAP-013 partial). Add `rust-version`, `readme`, `documentation`, `homepage`, `categories`, `keywords` fields. Configure `[lints]` and `[package.metadata.docs.rs]`.

**Phase 3: CI + Publication (Week 6)**

13. **CI pipeline** (GAP-012). File: `.github/workflows/ci.yml`. Implement the planned pipeline: test (ubuntu, macos, windows), clippy (pedantic), fmt, security audit, MSRV check, no-network-deps verification.

14. **`cargo publish --dry-run`** validation.

15. **`cargo publish`** v0.1.0 to crates.io.

16. **Launch communications:** r/rust post, HackerNews submission, blog post describing the cognitive lifecycle approach, direct outreach to OpenClaw contributors with a link to the crate and a brief explanation of how Alaya addresses their memory needs.

**Phase 4: Benchmark Fast-Follow (Weeks 7-9)**

17. **LoCoMo benchmark harness** (new). Implement a reproducible benchmark suite that ingests LoCoMo's multi-session conversation data, runs queries, and measures precision@5, recall@10, nDCG@5, and MRR. Measure both BM25-only and hybrid (with embeddings if applicable) configurations.

18. **Run baselines and publish results.** Write a blog post with the numbers, whatever they are. Include methodology, reproduction instructions, and honest analysis of where Alaya falls short. Per "Honesty > Marketing," the numbers are the numbers.

19. **LongMemEval baseline** (if time permits). Secondary benchmark, same honest reporting.

20. **OpenClaw follow-up.** If initial outreach generated interest, provide benchmark numbers and integration guidance. If not, the numbers still serve broader credibility.

**Phase 5: v0.1.x Iteration (Weeks 10+)**

21. **Respond to user feedback.** Close gaps discovered by first MACC consumers. Prioritize by axiom alignment.

22. **P2 polish.** WAL checkpoint management, content-hash integrity column, MSRV pinning via `rust-toolchain.toml`.

23. **Begin v0.2 planning.** Scope MCP server, feature flags (vec-sqlite, embed-ort, async), based on actual user needs rather than hypothetical ones.

### Why Not Path A

Path A is the stronger play _if Alaya already had users._ Benchmark leadership is a retention and credibility tool for an established project. For a project with zero users, it is an internal quality exercise with an audience of one. The "Honesty > Marketing" axiom does not say "publish great benchmarks" -- it says "publish honest benchmarks." Path B publishes honest baselines 3 weeks after launch, which satisfies the axiom without delaying the feedback loop.

The specific concern about Path A is that benchmark optimization is unbounded. The scaffold analysis shows two retrieval pipeline gaps (GAP-007, GAP-009) that directly affect benchmark scores. Closing them is included in Phase 2 of the recommended path. But benchmark optimization goes beyond gap closure: it involves tuning RRF k values, adjusting rerank weights, experimenting with spreading activation parameters, and potentially rethinking the enrichment strategy. This is valuable work, but it is work that benefits from knowing which real-world query patterns matter, and that knowledge comes from users, not from synthetic benchmarks.

Furthermore, the competitive analysis shows that Mem0's LoCoMo score is 68.5% and Hindsight claims 89.61%. Alaya's architecture (BM25 + vector + Hebbian graph + RRF) should be competitive, but the exact number is unknown. If it turns out to be 55% at baseline, the correct response is to publish that number honestly and improve in subsequent versions -- not to delay publication until the number is impressive. The brand voice is "quiet confidence," not "impressive statistics."

### Why Not Path C

Path C is the highest-variance option, and variance is costly for a solo developer. The OpenClaw integration is the single most valuable adoption event on the horizon, but coupling Alaya's entire timeline to an external project's decisions introduces dependency risk that is not commensurate with the certainty of payoff.

The recommended path captures most of Path C's value without the coupling: Phase 3 includes direct OpenClaw outreach with a published, documented, working crate. If OpenClaw is interested, the integration discussion happens from a position of strength (the library exists, is documented, has benchmarks forthcoming). If OpenClaw is not ready or chooses differently, nothing is lost.

The specific concern about Path C is the "narrow API" risk. Shaping the library around one consumer's needs before understanding the general case produces an API that serves that consumer well but may be awkward for everyone else. The brand positioning is "framework-agnostic" -- this is undermined by an OpenClaw-shaped API.

### The Synthesis

The recommended path is not a compromise. It is the only sequencing that satisfies all five axioms simultaneously:

- **Privacy > Features:** No features added that compromise privacy. The path is about hardening and documentation, not new capabilities.
- **Process > Storage:** The lifecycle is the value proposition, and users can experience it from day one. Benchmarks measure the process, but experiencing the process does not require benchmarks.
- **Correctness > Speed:** P0 hardening happens before publication. `BEGIN IMMEDIATE`, input validation, `#[non_exhaustive]` -- these are correctness gates, not speed gates.
- **Simplicity > Completeness:** v0.1.0 ships with the complete cognitive lifecycle and zero external dependencies. It does not wait for MCP, Python bindings, feature flags, or benchmark optimization.
- **Honesty > Marketing:** Benchmarks are published as fast-follow, with honest numbers, not delayed until numbers are impressive.

---

## Part 4: Focus Areas and Avoid List

### 4.1 Focus Areas (Weeks 1-9)

**Focus 1: Close the P0 hardening gaps without scope creep.**

The five P0 gaps are specific, testable, and finite. The temptation will be to "also fix" adjacent issues while touching those files. Resist. `#[non_exhaustive]` is a one-line addition per enum. `BEGIN IMMEDIATE` is a helper function and call-site updates. Input validation is a set of guard clauses. `pub(crate)` is a visibility change. Doctests are focused examples. Each gap has a clear "done" definition. Treat them as a checklist, not an exploration.

**Focus 2: Write documentation that serves Priya, not Marcus.**

The north star analysis identifies two personas. Priya (privacy-first companion agent developer) is the more accessible early adopter: she needs on-device memory, zero cloud dependencies, and a straightforward API. Marcus (performance-focused, benchmark-aware) is served by the fast-follow benchmarks. The README, quickstart, and examples should be written for Priya: simple, privacy-emphasizing, showing the path from `cargo add alaya` to a working agent memory in under 2 minutes.

This does not mean ignoring Marcus. It means the documentation layers progressively: Priya's quickstart first, then architecture deep-dive for Marcus, then benchmark results for technical evaluation. Progressive disclosure mirrors the library's own design philosophy (Day 1 methods -> Day 2 methods -> Week 1 methods).

**Focus 3: Make the launch communications specific, not promotional.**

The brand voice is "quiet confidence." The launch post on r/rust should describe what Alaya is, what it does differently, and what it does not do. Include a code snippet showing the store/query/consolidate cycle. Link to the docs.rs documentation and the examples. Do not use superlatives. Do not compare to competitors by name. Let the technical description speak.

The kill list applies to communications: not cloud-dependent, not enterprise, not LLM-coupled, not hype-driven. If the launch post reads like marketing copy, it is wrong.

**Focus 4: Run LoCoMo within 3 weeks of publication, regardless of results.**

The benchmark commitment is not optional and not contingent on the numbers being good. The "Honesty > Marketing" axiom is a commitment to transparency, not a commitment to excellence. If BM25-only baseline is 45%, publish 45% with an analysis of why and a roadmap for improvement. The competitive landscape analysis shows that publishing reproducible numbers -- any numbers -- is a differentiator in a field where most projects publish no benchmarks at all.

**Focus 5: Engage OpenClaw with a working crate, not a pitch.**

The difference between "we are building a memory library that could work for OpenClaw" and "here is a published memory library on crates.io, here are the docs, here is how it would integrate" is the difference between a pitch and a proposal. The latter is dramatically more effective. The recommended path ensures that OpenClaw outreach happens after publication, so the conversation begins with a working artifact.

### 4.2 Avoid List (Critical Missteps to Prevent)

**Avoid 1: Do not optimize retrieval quality before publication.**

The retrieval pipeline works. It has BM25, vector search, graph activation, RRF fusion, and reranking. It gracefully degrades. The known gaps (GAP-007, GAP-009) are included in Phase 2 because they are correctness issues, not optimization targets. But do not tune RRF k values, adjust rerank weights, experiment with activation parameters, or implement alternative fusion strategies before v0.1.0. Optimization without user data is guessing.

**Avoid 2: Do not build the MCP server before v0.1.0.**

The MCP server is correctly scoped to v0.2. It requires the Rust API to be stable (or at least published) before wrapping it. Building MCP before publication means maintaining two interfaces simultaneously during the highest-velocity period. The Rust API is the foundation; validate it first.

**Avoid 3: Do not add feature flags before v0.1.0.**

The architecture blueprint identifies four planned feature flags: `vec-sqlite`, `embed-ort`, `embed-fastembed`, `async`. All are v0.2 scope. Adding feature flags before publication increases the test matrix (each flag combination must work), increases compile times, and increases documentation burden. The v0.1 crate has zero optional features and that is a strength, not a limitation.

**Avoid 4: Do not chase GitHub stars or social media engagement.**

The brand analysis explicitly places "competing on GitHub stars or social media hype" on the rejected moves list. Stars follow utility; utility follows good documentation and real adoption. The launch communications should be one post per channel (r/rust, HackerNews, blog), honest and technical. Do not cross-post to 10 subreddits, do not create a Twitter/X account for the project, do not spend time on promotional graphics. The audience is developers who read technical posts and evaluate libraries by their docs.rs page.

**Avoid 5: Do not implement Python bindings, C FFI, or any cross-language interface before v0.2.**

The temptation to "reach more developers" by adding PyO3 bindings early is strong, especially when the north star targets 100 MACC at v0.3. But cross-language bindings are a maintenance multiplier. Every API change in the Rust crate must be propagated to every binding. The v0.1 Rust API is explicitly expected to evolve based on user feedback. Stabilize before binding.

**Avoid 6: Do not add a web site, logo, or branding assets before the library has users.**

These are multipliers on existing adoption, not drivers of new adoption. The brand guidelines document defines voice, terminology, and positioning -- these guide documentation writing. Visual branding is deferred until the project has enough presence to benefit from it.

**Avoid 7: Do not skip the CI pipeline.**

The temptation to "publish first, add CI later" is real when you are the only developer and all tests pass locally. But the CI pipeline (GAP-012) is included in Phase 3 (before publication) for a reason: it catches platform-specific issues (the pipeline tests ubuntu, macos, windows), enforces MSRV compatibility, and runs the network-dependency ban check. Publishing a crate that fails to compile on Windows or requires a newer Rust than declared is a first-impression failure that is hard to recover from.

**Avoid 8: Do not attempt to make the first version perfect.**

The v0.1.0 release will have known limitations. The LoCoMo score will be a baseline, not a leadership number. The documentation will cover the core path but not every edge case. The test count will be higher than 43 but lower than 150. Some consumers will request features that are on the kill list. This is expected and acceptable. The "Simplicity > Completeness" axiom is the permission to ship an honest, well-hardened, well-documented v0.1.0 that is not complete.

---

## Part 5: Confidence Assessment and Review Triggers

### 5.1 Confidence Assessment

**Overall confidence in recommendation: High (8/10)**

The confidence is high because the recommendation aligns with multiple independent analyses:

- **Axiom alignment:** All five axioms support the recommended path. No axiom is violated.
- **Risk profile:** Lowest downside risk of the three paths. Failure mode (no adoption) still produces reusable assets.
- **Precedent:** The competitive analysis shows that Mem0 and Engram both established positions with initial releases that were not benchmark-leading. Market presence preceded market dominance.
- **Solo-developer fit:** Path B requires the most common solo-developer skill (writing documentation and examples) rather than the most specialized (benchmark optimization infrastructure) or the most externally dependent (integration with an evolving partner project).

**Confidence-reducing factors:**

- **Unknown LoCoMo baseline (1 point).** If Alaya's BM25-only retrieval quality turns out to be below 40%, the "publish honest baselines" strategy has a harder path to credibility. This is unlikely given FTS5's quality and the reranking layer, but it is unknown until measured.
- **Unknown OpenClaw timeline (1 point).** If OpenClaw makes component decisions within the next 4 weeks (before v0.1 publication), Path B misses the window. The mitigation is that outreach can happen pre-publication with a link to the GitHub repo, but this is weaker than outreach with a published crate.

### 5.2 Review Triggers

The recommendation should be re-evaluated if any of these conditions occur:

**Trigger 1: OpenClaw announces component selection timeline within 4 weeks.**
- **Response:** Evaluate accelerating Phase 1-2 into 3 weeks total. If OpenClaw's timeline is 2-3 weeks away, consider a "preview release" approach: publish v0.1.0-alpha to crates.io with P0 gaps closed but P1 gaps open, alongside outreach. This deviates from the recommended sequence but is justified by the timing window.
- **Threshold:** OpenClaw publicly announces memory component evaluation or RFP.

**Trigger 2: A competitor ships a zero-dependency embedded memory library with cognitive lifecycle features.**
- **Response:** Accelerate to publication immediately. The competitive moat is being first in the quadrant, not being perfect in the quadrant. If the moat is threatened, close P0 gaps in emergency mode (1 week) and publish with minimal documentation.
- **Threshold:** A Rust or C library on crates.io or GitHub with >100 stars that implements both forgetting and preference emergence without external infrastructure.

**Trigger 3: LoCoMo baseline, when measured, is below 40% precision@5 in BM25-only mode.**
- **Response:** Do not delay publication, but adjust the benchmark blog post to be an analysis piece rather than a results announcement. Focus on explaining the architecture and the expected improvement trajectory (enrichment, RIF, hybrid with embeddings) rather than the baseline number. Consider running a "Bjork forgetting impact" benchmark that showcases Alaya's unique value (how retrieval quality changes over time with forgetting vs. without).
- **Threshold:** precision@5 < 0.40 on LoCoMo BM25-only configuration.

**Trigger 4: Zero MACC 90 days post-publication.**
- **Response:** Per the extract analysis trigger: "Full strategy review; likely positioning or DX, not architecture." Conduct 5 developer interviews (reach out to Rust agent developers on Discord and r/rust). The most likely causes, in order: (a) documentation does not convey value proposition, (b) quickstart is too slow, (c) API is confusing, (d) the library is in the wrong channel (needs MCP or Python to reach the audience). Adjust based on findings. Do not change architecture.
- **Threshold:** GitHub dependents + crates.io reverse dependencies = 0 at 90 days.

**Trigger 5: MACC exceeds v0.1 target (5) within 4 weeks of publication.**
- **Response:** Accelerate v0.2 planning. The feedback from early adopters should drive feature prioritization between MCP server, async API, sqlite-vec, and embedding providers. Consider publishing a v0.2 roadmap RFC to engage the emerging community.
- **Threshold:** 5+ unique projects using Alaya within 4 weeks.

**Trigger 6: Solo-developer capacity constraint becomes binding.**
- **Response:** If the maintainer is unable to complete the recommended 9-week sequence within 12 calendar weeks (accounting for part-time allocation), narrow scope: drop Phase 4 (benchmark fast-follow) from the initial sequence and defer it to v0.1.x. The minimum viable path is Phases 1-3 (harden, document, publish). Benchmarks can follow at any time; publication cannot be deferred indefinitely without cost.
- **Threshold:** Phase 2 incomplete after 6 calendar weeks.

**Trigger 7: Research invalidates a core mechanism.**
- **Response:** Per the extract analysis: "Update affected mechanism; research grounding is a two-way commitment." If Bjork dual-strength model is shown to be inferior to a simpler approach, update the forgetting mechanism. If CLS consolidation theory is revised, update consolidation. The architecture is modular enough (each lifecycle process is independent) to replace individual mechanisms without restructuring the library. Do not delay publication for theoretical concerns -- respond to published research with published updates.
- **Threshold:** Peer-reviewed paper at a top venue demonstrating clear failure of Bjork, CLS, or Hebbian mechanisms in the agent memory context.

### 5.3 Scheduled Reviews

| Review | Date | Focus |
|--------|------|-------|
| Phase 1 completion | End of Week 2 | Are P0 gaps closed? Any blockers? |
| Pre-publication gate | End of Week 6 | CI green? Docs adequate? Ship decision. |
| Post-launch check | 2 weeks post-publish | Any MACC? Feedback quality? Adjust? |
| Benchmark review | 3 weeks post-publish | LoCoMo numbers in. Strategy adjustment? |
| 90-day review | 90 days post-publish | MACC vs. target. Full strategy reassessment. |
| v0.2 scope decision | When MACC >= 5 | User-driven feature prioritization. |

### 5.4 Success Criteria

The recommended path succeeds if, 90 days after publication:

1. **v0.1.0 is published on crates.io** with zero `cargo clippy --pedantic` warnings, zero failing tests, and compilable doctests on all public methods.
2. **MACC >= 3** (60% of v0.1 target). Below 5 is acceptable if the trajectory is positive (month-over-month growth).
3. **LoCoMo baseline is measured and published.** The number itself does not define success; publishing it does.
4. **At least one external API review** has been received (GitHub issue, PR, or direct feedback from a non-maintainer developer).
5. **OpenClaw outreach has occurred** with a published crate as the backing artifact.
6. **No SEV-1 or SEV-2 incidents** (data corruption, semver violation, security vulnerability).

The recommended path fails if, 90 days after publication:

1. v0.1.0 has not been published (execution failure).
2. Zero external engagement of any kind (positioning failure).
3. A competitor occupies the quadrant first (timing failure).

---

## Appendix A: Detailed Timeline

```
Week  1: [P0] #[non_exhaustive] on all enums
         [P0] BEGIN IMMEDIATE helper + call-site migration
Week  2: [P0] Input validation at API boundary
         [P0] pub(crate) visibility for internal modules
         [P0] Compilable doctests on all pub methods
Week  3: [P1] Wire LTD in transform()
         [P1] Semantic/preference node enrichment in pipeline
         [DOC] README with quickstart and architecture overview
Week  4: [P1] Tombstone mechanism for deleted nodes
         [P1] RIF suppression in retrieval pipeline
         [DOC] examples/basic_agent.rs, examples/lifecycle_demo.rs
Week  5: [DOC] examples/custom_provider.rs
         [DOC] Cargo.toml metadata, docs.rs configuration
         [TEST] Expand unit tests toward 80+ (focus: types, error, provider)
Week  6: [CI] GitHub Actions pipeline (test, clippy, fmt, audit, msrv, no-network)
         [PUBLISH] cargo publish --dry-run, then cargo publish
         [LAUNCH] r/rust, HackerNews, blog post, OpenClaw outreach
Week  7: [BENCH] LoCoMo benchmark harness implementation
         [COMMUNITY] Respond to initial feedback, triage issues
Week  8: [BENCH] Run LoCoMo baseline (BM25-only and hybrid if embeddings available)
         [BENCH] Run Alaya-Internal benchmarks (Bjork impact, vasana accuracy)
Week  9: [BENCH] Benchmark blog post with honest results and methodology
         [PLAN] Begin v0.1.x patch planning based on user feedback
         [PLAN] Begin v0.2 scope based on MACC data and feedback themes
```

## Appendix B: Decision Audit Trail

This recommendation was made by synthesizing:

| Document | Key Input to Recommendation |
|----------|---------------------------|
| `brand.yml` | Kill list (constraints on what to avoid), voice rules (communication approach) |
| `northstar.yml` | MACC targets (5 for v0.1), persona priorities (Priya first, Marcus fast-follow) |
| `competitive.yml` | Unoccupied quadrant (urgency to occupy), Mem0 at 68.5% LoCoMo (benchmark bar) |
| `extract.yml` | Axiom hierarchy (decision framework), always/never lists (guardrails) |
| `architecture.yml` | Known gaps (P0/P1/P2 prioritization), pipeline completeness (retrieval works) |
| `agent-prompts.yml` | Day-1/Day-2/Week-1 method progression (documentation structure) |
| `security.yml` | Threat T7 deadlock (P0 urgency for BEGIN IMMEDIATE), threat T3 resurrection (P1 tombstones) |
| `adr.yml` | ADR-009 zero network (structural privacy), ADR-008 sync-first (simplicity) |
| `post-deployment.yml` | Pre-publication blockers (P0 gap list), release gates (CI requirements) |
| `scaffold.yml` | Module tree (scope of changes), test count (43, gap to 150) |
| `resilience.yml` | Degradation chain works (retrieval is safe to ship), transaction gaps (P0 urgency) |
| `testing.yml` | Benchmark framework choice (divan), golden datasets (LoCoMo targets) |

## Appendix C: What Changes If the Recommendation Is Wrong

If Path B proves to be the wrong choice, these are the recovery paths:

**Scenario: Benchmark numbers are embarrassingly low (< 30% P@5).**
- Recovery: The published crate still works. Reframe messaging around the cognitive lifecycle (vasana, Bjork, Hebbian) as the differentiator, not raw retrieval scores. Use the benchmark blog as a "what we learned" post. Accelerate pipeline optimization in v0.1.x. The honest publication still satisfies the axiom.

**Scenario: OpenClaw selects a different memory system before Alaya publishes.**
- Recovery: OpenClaw is not the only consumer. The library serves all agent developers building privacy-first agents. Redirect outreach to r/LocalLLaMA, companion agent builders, and edge AI developers. The documentation and hardening work is fully reusable.

**Scenario: A competitor ships an identical product faster.**
- Recovery: Alaya's differentiators (vasana, Bjork dual-strength, Hebbian LTP/LTD) are research-grounded and novel. If a competitor ships a "cognitive lifecycle" library but with simpler mechanisms, differentiate on depth. If they ship an identical architecture, the market is validated and there is room for two. MIT license means Alaya is forkable and composable.

**Scenario: Nobody cares.**
- Recovery: 5 developer interviews (extract.yml trigger). The most likely fix is documentation, not architecture. The recommended path produces the best documentation foundation of any path, making this recovery easiest from Path B.

## Appendix D: Relationship to Existing Roadmap

The recommended sequence maps to the existing v0.1 phase definition:

| v0.1 Exit Criterion (from northstar.yml) | Recommended Phase | Status |
|------------------------------------------|-------------------|--------|
| All 5 missing table-stakes operations implemented | Pre-existing (verify) | Needs verification |
| Every public method has compilable doctest | Phase 1 (Week 2) | GAP-005 |
| LoCoMo score measured | Phase 4 (Week 8) | Planned |
| cargo publish succeeds | Phase 3 (Week 6) | Planned |
| At least one external API review | Phase 3+ (post-launch) | Planned via outreach |

The recommended path achieves all v0.1 exit criteria within 9 weeks. The existing roadmap estimated 4-6 weeks; the 9-week timeline is more realistic because it includes the benchmark fast-follow that the original estimate deferred. The net effect is the same: v0.1 is complete when all exit criteria are met, which happens at Week 8-9.

---

_This document synthesizes all preceding analysis phases. It should be reviewed at each scheduled checkpoint and updated if any review trigger fires. The recommendation is not permanent -- it is the best available decision given current information, and new information changes the decision._
