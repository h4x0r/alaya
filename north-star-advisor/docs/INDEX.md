# Alaya Documentation Index

> Complete strategic documentation for Alaya -- embeddable Rust memory library with cognitive lifecycle processes and implicit preference emergence for privacy-first AI agents.

**Generated:** 2026-02-26 | **Phase:** 13 of 13 (Documentation Index)
**Total documents:** 22
**Total word count:** ~155,000
**Status:** Complete

---

## Table of Contents

1. [Document Hierarchy](#1-document-hierarchy)
2. [Reading Order by Audience](#2-reading-order-by-audience)
3. [Document Summaries](#3-document-summaries)
4. [Cross-Reference Map](#4-cross-reference-map)
5. [Additional Outputs](#5-additional-outputs)
6. [Version History](#6-version-history)

---

## 1. Document Hierarchy

```
north-star-advisor/
|
+-- docs/
|   |
|   +-- INDEX.md ............................. This document (Phase 13)
|   |
|   +-- [Tier 1: Strategy & Identity]
|   |   +-- BRAND_GUIDELINES.md .............. Phase 1  -- 4,536 words
|   |   +-- NORTHSTAR.md .................... Phase 2  -- 6,742 words
|   |   +-- COMPETITIVE_LANDSCAPE.md ........ Phase 3  -- 5,749 words
|   |   +-- NORTHSTAR_EXTRACT.md ............ Phase 4  -- 3,938 words
|   |
|   +-- [Tier 2: Architecture & Security]
|   |   +-- ARCHITECTURE_BLUEPRINT.md ....... Phase 6  -- 5,975 words
|   |   +-- SECURITY_ARCHITECTURE.md ........ Phase 8  -- 7,064 words
|   |   +-- ADR.md .......................... Phase 9  -- 7,925 words
|   |
|   +-- [Tier 3: Synthesis & Action]
|   |   +-- POST_DEPLOYMENT.md .............. Phase 10 -- 9,136 words
|   |   +-- STRATEGIC_RECOMMENDATION.md ..... Phase 11 -- 7,220 words
|   |   +-- ACTION_ROADMAP.md ............... Phase 12 -- 9,577 words
|   |
|   +-- design/
|   |   +-- [Tier 2: Developer Experience]
|   |   +-- USER_JOURNEYS.md ................ Phase 5a -- 5,900 words
|   |   +-- UI_DESIGN_SYSTEM.md ............. Phase 5b -- 6,185 words
|   |   +-- ACCESSIBILITY.md ................ Phase 5c -- 7,813 words
|   |   +-- WIREFRAMES.md ................... Phase 5d -- 10,266 words
|   |
|   +-- architecture/
|       +-- [Tier 2: Deep Architecture]
|       +-- AGENT_PROMPTS.md ................ Phase 7  -- 6,281 words
|       +-- PIPELINE_ORCHESTRATION.md ....... Phase 10p -- 7,602 words
|       +-- RESILIENCE_PATTERNS.md .......... Phase 10r -- 9,105 words
|       +-- IMPLEMENTATION_SCAFFOLD.md ...... Phase 10s -- 8,445 words
|       +-- OBSERVABILITY.md ................ Phase 10d -- 6,987 words
|       +-- TESTING_STRATEGY.md ............. Phase 10t -- 9,708 words
|       +-- HANDOFF_PROTOCOL.md ............. Phase 11h -- 7,548 words
|
+-- ai-context.yml .......................... Progressive strategic context
+-- .work-in-progress/
    +-- research/
        +-- summary.md ...................... Research synthesis
```

### Tier Summary

| Tier | Purpose | Documents | Total Words |
|------|---------|-----------|-------------|
| Tier 1: Strategy & Identity | Why Alaya exists, who it serves, where it competes | 4 | 20,965 |
| Tier 2: Architecture & Security | How it is built, how it stays safe | 3 | 20,964 |
| Tier 2: Developer Experience | How developers discover, learn, and use the API | 4 | 30,164 |
| Tier 2: Deep Architecture | Implementation details for each subsystem | 6 | 47,969 |
| Tier 3: Synthesis & Action | What to do and when | 3 | 25,933 |
| Meta | This index | 1 | -- |
| **Total** | | **22** | **~155,000** |

---

## 2. Reading Order by Audience

### New Contributor

Someone joining the project who needs to understand what Alaya is, why it exists, and how the code is organized.

| Step | Document | What You Learn |
|------|----------|----------------|
| 1 | [Brand Guidelines](BRAND_GUIDELINES.md) | Etymology, positioning, voice, beliefs, kill list |
| 2 | [North Star](NORTHSTAR.md) | MACC metric, personas (Priya, Marcus), success phases |
| 3 | [Architecture Blueprint](ARCHITECTURE_BLUEPRINT.md) | Three-store model, Hebbian graph, retrieval pipeline, provider traits |
| 4 | [Implementation Scaffold](architecture/IMPLEMENTATION_SCAFFOLD.md) | Module map, build commands, hardening gaps, workspace plan |
| 5 | [Testing Strategy](architecture/TESTING_STRATEGY.md) | Test pyramid, CI gates, benchmark harness, coverage targets |

### Integration Developer

A developer building an AI agent who wants to use Alaya as their memory layer.

| Step | Document | What You Learn |
|------|----------|----------------|
| 1 | [Consumer Integration Patterns](architecture/AGENT_PROMPTS.md) | Direct Rust API usage, MCP server integration, provider implementation guides |
| 2 | [Provider Contract Protocol](architecture/HANDOFF_PROTOCOL.md) | Trait signatures, context passing, error handling, NoOp fallback |
| 3 | [Retrieval Pipeline](architecture/PIPELINE_ORCHESTRATION.md) | BM25 + Vector + Graph stages, RRF fusion, degradation chain |
| 4 | [API Design System](design/UI_DESIGN_SYSTEM.md) | Naming conventions, type hierarchy, method patterns |
| 5 | [API Surface Specification](design/WIREFRAMES.md) | README structure, docs.rs wireframes, example code, error output |

### Strategic Reviewer

A stakeholder evaluating the project's viability, market position, and execution plan.

| Step | Document | What You Learn |
|------|----------|----------------|
| 1 | [North Star](NORTHSTAR.md) | Success metric, target personas, phased milestones |
| 2 | [Competitive Landscape](COMPETITIVE_LANDSCAPE.md) | 7 competitors analyzed, whitespace map, timing windows |
| 3 | [Strategic Recommendation](STRATEGIC_RECOMMENDATION.md) | Path B rationale, 3 paths evaluated, trade-off analysis, pivot triggers |
| 4 | [Action Roadmap](ACTION_ROADMAP.md) | Week-by-week task plan, success criteria, 90-day review gates |

### Security Reviewer

Someone auditing the library's threat model, data handling, and hardening roadmap.

| Step | Document | What You Learn |
|------|----------|----------------|
| 1 | [Security Architecture](SECURITY_ARCHITECTURE.md) | OWASP mapping, 10 threat vectors, guardrails, compliance posture |
| 2 | [Architecture Decision Records](ADR.md) | 10 ADRs with security implications (SQLite, zero network, FTS5 sanitization) |
| 3 | [Resilience Patterns](architecture/RESILIENCE_PATTERNS.md) | Degradation chain, transaction safety, idempotency analysis, resource budgets |
| 4 | [Provider Contract Protocol](architecture/HANDOFF_PROTOCOL.md) | Provider output validation, error propagation, injection surface |

### Operations / Release Manager

Someone responsible for CI, publishing, and post-release maintenance.

| Step | Document | What You Learn |
|------|----------|----------------|
| 1 | [Post-Publication Operations](POST_DEPLOYMENT.md) | CI-as-infrastructure, quality gates, alert escalation, semver policy |
| 2 | [Library Instrumentation](architecture/OBSERVABILITY.md) | Tracing spans, PII policy, consumer integration, metrics derivation |
| 3 | [Testing Strategy](architecture/TESTING_STRATEGY.md) | CI matrix, coverage targets, benchmark regression gates |
| 4 | [Action Roadmap](ACTION_ROADMAP.md) | Publication checklist, launch communications, post-launch monitoring |

---

## 3. Document Summaries

### Tier 1: Strategy & Identity

#### 1. [Brand Guidelines](BRAND_GUIDELINES.md)
**Phase 1 | 4,536 words**

Establishes Alaya's identity rooted in the Sanskrit concept of storehouse consciousness (alaya-vijnana). Defines the positioning statement ("For agent developers who need conversational memory with privacy guarantees..."), five core beliefs (memory is a process, forgetting is a feature, preferences emerge, the agent owns identity, graceful degradation), the kill list (not cloud, not enterprise, not LLM-coupled, not a service), voice guidelines (technical but accessible, research-grounded, honest about tradeoffs), and preferred terminology. Key output: the vocabulary and philosophical grounding that all subsequent documents reference.

#### 2. [North Star Specification](NORTHSTAR.md)
**Phase 2 | 6,742 words**

Defines the single success metric -- Monthly Active Crate Consumers (MACC) -- measured as unique projects calling `AlayaStore::open()` and executing `store_episode()` + `query()` in a 30-day period. Introduces two personas: Priya (privacy-first agent developer, wants on-device memory) and Marcus (performance-focused, wants sub-ms retrieval and benchmarks). Lays out progressive targets from 5 MACC at v0.1 to 500 at v1.0, with phased milestones (MVP, Ecosystem, Growth). Key output: the metric every other document optimizes toward and the personas whose needs drive all design decisions.

#### 3. [Competitive Landscape](COMPETITIVE_LANDSCAPE.md)
**Phase 3 | 5,749 words**

Analyzes seven direct competitors (Mem0, Zep/Graphiti, Letta/MemGPT, Supermemory, Hindsight/Vectorize, Memvid, Engram) across dimensions of cognitive completeness, operational simplicity, privacy guarantees, and LLM independence. Maps the competitive whitespace to identify Alaya's unoccupied quadrant: high cognitive completeness combined with high operational simplicity. Identifies four timing windows (OpenClaw ecosystem, DEF CON credibility, pre-RL-productionization, edge AI memory gap). Key output: the differentiation claims and market timing that inform the strategic recommendation.

#### 4. [North Star Extract](NORTHSTAR_EXTRACT.md)
**Phase 4 | 3,938 words**

Distills the design DNA into non-negotiable axioms with a strict conflict-resolution hierarchy: Safety > Privacy > Correctness > Simplicity > Performance > Features. Extracts five axioms (Privacy > Features, Process > Storage, Correctness > Speed, Simplicity > Completeness, Honesty > Marketing) and four architectural constraints (zero runtime deps beyond core four, single SQLite file, no network calls, typed reports from all lifecycle processes). Key output: the decision framework that resolves every ambiguity in subsequent phases without re-litigation.

### Tier 2: Architecture & Security

#### 5. [Architecture Blueprint](ARCHITECTURE_BLUEPRINT.md)
**Phase 6 | 5,975 words**

Defines Alaya's internal architecture: three stores (episodic, semantic, implicit), Hebbian graph overlay with LTP/LTD, hybrid retrieval pipeline (BM25 + vector + graph activation through RRF fusion and contextual reranking), four lifecycle processes (consolidation, perfuming, transformation, forgetting), and the trait-based provider model with NoOp fallback. Specifies the SQLite schema (7 tables, FTS5, WAL mode), the `AlayaStore` public API surface (12 methods), and known gaps targeted for v0.1 hardening. Key output: the technical reference architecture that all implementation documents elaborate.

#### 6. [Security Architecture](SECURITY_ARCHITECTURE.md)
**Phase 8 | 7,064 words**

Maps Alaya's threat model against the OWASP Top 10 for Agentic Applications (2025 draft). Identifies 10 threat vectors specific to an embeddable memory library: memory poisoning, FTS5 MATCH injection, memory resurrection, cross-user leakage, PII persistence, embedding poisoning, WAL corruption, transaction deadlocks, provider output injection, and file theft. Defines guardrails (FTS5 sanitization, parameterized queries, BEGIN IMMEDIATE, tombstones, surrogate keys) and a phased hardening roadmap from v0.1 through v0.3. Covers compliance posture for GDPR, CCPA, SOC2, and HIPAA. Key output: the security constraints that gate every feature decision.

#### 7. [Architecture Decision Records](ADR.md)
**Phase 9 | 7,925 words**

Documents 10 architectural decisions with full context, alternatives considered, and consequences: ADR-001 (SQLite as sole storage), ADR-002 (three-store architecture), ADR-003 (Hebbian graph overlay), ADR-004 (trait-based extension), ADR-005 (Bjork dual-strength forgetting), ADR-006 (RRF fusion), ADR-007 (vasana preference emergence), ADR-008 (sync-first API), ADR-009 (zero network calls), ADR-010 (FTS5 for full-text search). Each ADR records the specific tradeoff made and the axiom that resolved it. Key output: the immutable decision log that prevents re-litigation and explains "why" to future contributors.

### Tier 2: Developer Experience

#### 8. [Developer Experience Journeys](design/USER_JOURNEYS.md)
**Phase 5a | 5,900 words**

Maps five complete developer journeys through Alaya adoption: First-Time Developer (discovery through first working code), Deepening Integration (basic CRUD through production tuning), Error Recovery (compilation errors through performance debugging), MCP Integration (v0.2 planned server flow), and Persona Variations (Priya's privacy audit path, Marcus's benchmark-first path). Each journey is grounded in the actual `AlayaStore` API with compilable code examples. Key output: the end-to-end developer experience map that drives API design, documentation structure, and error message strategy.

#### 9. [API Design System](design/UI_DESIGN_SYSTEM.md)
**Phase 5b | 6,185 words**

Defines the complete API design language: naming conventions (snake_case functions, CamelCase types), the New*/Entity input/output split pattern, builder configuration (`AlayaConfig::builder()`), Result-everywhere error handling, typed lifecycle reports, two-level query API (`Query::simple()` for quickstart, full struct for advanced), and consistency rules (timestamps as i64, IDs as newtypes, scores as f64, weights as f32). Specifies the type hierarchy across IDs, polymorphic references, inputs, outputs, reports, and enums. Key output: the design tokens and patterns that make every public method feel like it belongs to the same API.

#### 10. [Developer Accessibility Strategy](design/ACCESSIBILITY.md)
**Phase 5c | 7,813 words**

Reframes accessibility for a Rust library across seven dimensions: skill level (beginner through expert with progressive disclosure), error messages (what happened + why + what to do), documentation (README to architecture docs), platform reach (Rust T1, C FFI + MCP T2, Python T3), cognitive load (fewer than 20 public methods, CRUD symmetry, sensible defaults), diagnostics (typed reports, tracing spans), and onboarding (under 2 minutes to first success). Defines the 3-method beginner surface (`open`, `store_episode`, `query`) and the tiered escalation to provider traits and tuning parameters. Key output: the accessibility constraints that keep the API approachable as feature count grows.

#### 11. [API Surface Specification](design/WIREFRAMES.md)
**Phase 5d | 10,266 words**

Specifies the exact developer-facing surfaces: README structure (30-second scan, quickstart, feature table), docs.rs page layout (module organization, method documentation, cross-links), example code (3 graduated examples from basic to provider implementation), terminal error output format, diagnostic report structure, and CHANGELOG format. Every wireframe is grounded in the actual API and cross-referenced against developer journeys and accessibility requirements. Key output: the concrete templates that translate design principles into what developers actually see.

### Tier 2: Deep Architecture

#### 12. [Consumer Integration Patterns](architecture/AGENT_PROMPTS.md)
**Phase 7 | 6,281 words**

Provides four integration guides for consumers: Direct Rust API (store after turns, query before response, dream() periodically), MCP Server (v0.2 universal agent integration), Provider Implementation (ConsolidationProvider and EmbeddingProvider trait guides with example code), and Memory-Aware Prompt Engineering (how to inject memory context, preferences, and guidelines into consumer system prompts). Includes anti-patterns (never skip lifecycle, never store system messages, never ignore preferences) and integration checklists. Key output: the practical guides that turn architecture into working agent integrations.

#### 13. [Retrieval Pipeline Implementation](architecture/PIPELINE_ORCHESTRATION.md)
**Phase 10p | 7,602 words**

Details the seven-stage retrieval pipeline: BM25 (FTS5 with sanitization), Vector (brute-force cosine, degrading gracefully when embeddings absent), Graph Activation (Collins & Loftus spreading activation, depth=1, threshold=0.1, decay=0.6), RRF Fusion (k=60, variable-length input), Enrichment (candidate hydration), Rerank (base * context * recency scoring), and Post-Retrieval Effects (RS reset, co-retrieval LTP). Specifies exact module paths, input/output types, skip conditions, and the degradation chain from full 3-way fusion down to empty database. Key output: the stage-by-stage reference for retrieval implementation and optimization.

#### 14. [Graceful Degradation Patterns](architecture/RESILIENCE_PATTERNS.md)
**Phase 10r | 9,105 words**

Defines Alaya's resilience architecture across five domains: capability-based degradation (6 levels from full to empty, self-healing via link formation), transaction safety (WAL mode, BEGIN IMMEDIATE, busy handling, consumer-side Arc<Mutex<AlayaStore>>), lifecycle idempotency (convergent consolidation, non-idempotent perfuming, partial forgetting), resource budget enforcement (embedding scan ceiling, graph traversal limits, pipeline amplification factor), and cascade integrity (tombstone mechanism, orphan detection, VACUUM safety). Key output: the fault-tolerance contracts that prevent silent data corruption and ensure the library never panics on degraded input.

#### 15. [Module Structure & Build System](architecture/IMPLEMENTATION_SCAFFOLD.md)
**Phase 10s | 8,445 words**

Maps the complete codebase: 25 source files across 4,064 lines, organized into entry point (lib.rs), types (types.rs, error.rs, provider.rs), schema (schema.rs), stores (episodic, semantic, implicit, embeddings, strengths), graph (links, activation), retrieval (bm25, vector, fusion, rerank, pipeline), and lifecycle (consolidation, perfuming, transformation, forgetting). Specifies build commands, visibility plan (public vs pub(crate)), hardening gaps by priority (P0 through P2), workspace evolution plan (single crate to alaya + alaya-ffi + alaya-py), and planned feature flags. Key output: the implementation-ready map from architecture to file paths.

#### 16. [Library Instrumentation](architecture/OBSERVABILITY.md)
**Phase 10d | 6,987 words**

Specifies library-native observability through the Rust `tracing` ecosystem: span hierarchy (9 public method spans, 7 pipeline stage spans, 9 lifecycle stage spans), 51 structured events, PII-safe logging policy (never log content, queries, observations, embeddings, or entity names), degradation events, and consumer integration patterns (fmt, json, OpenTelemetry, tokio-console). Defines the optional `tracing` feature flag, the AlayaMetrics trait planned for v0.2, typed reports as the always-available observability layer, and compile-time/binary-size budgets. Key output: the instrumentation specification that gives consumers visibility without leaking private data.

#### 17. [Rust Testing Framework](architecture/TESTING_STRATEGY.md)
**Phase 10t | 9,708 words**

Defines the complete testing strategy: unit tests (current 43, target 150 at v0.1, 250 at v0.2), integration test suites (7 suites, 20 target at v0.1), property-based tests (proptest for 19 lifecycle invariants), golden datasets (LoCoMo, LongMemEval, Alaya-Internal), fuzz targets (5 targets for injection and deserialization), benchmarks (divan framework, performance targets including sub-ms BM25), doc tests (100% public method coverage), and CI pipeline (3 platforms, 3 Rust versions, 7 quality gates including network dependency ban). Key output: the quality infrastructure that enforces correctness from local development through CI.

#### 18. [Provider Contract Protocol](architecture/HANDOFF_PROTOCOL.md)
**Phase 11h | 7,548 words**

Specifies the trait-based inversion of control between Alaya and consumer code: ConsolidationProvider (extract_knowledge, extract_impressions, detect_contradiction), planned EmbeddingProvider (embed, embed_batch, dimension), context passing rules (what Alaya sends to providers, what it expects back), NoOpProvider fallback behavior, error propagation (AlayaError::Provider), and post-processing rules (link creation, strength initialization, crystallization thresholds). Documents the exact validation gaps in v0.1 and the planned v0.1.x hardening. Key output: the contract that consumer trait implementations must satisfy, and the guarantees Alaya makes in return.

### Tier 3: Synthesis & Action

#### 19. [Library Operations & Release Management](POST_DEPLOYMENT.md)
**Phase 10 | 9,136 words**

Reframes traditional post-deployment operations for a published Rust crate: CI pipeline as infrastructure (7 quality gates per PR), crates.io as production, semver as SLA, GitHub Issues as incident management. Defines severity levels (SEV-1 data corruption through SEV-4 performance regression), escalation timelines, post-mortem requirements, release checklist, yanking policy, compile-time and binary-size budgets, proxy metrics (MACC, crates.io downloads, issue velocity), and the monitoring-through-CI model. Key output: the operational playbook for maintaining a published crate without cloud infrastructure.

#### 20. [Strategic Recommendation](STRATEGIC_RECOMMENDATION.md)
**Phase 11 | 7,220 words**

Synthesizes all preceding phases into a single recommended path: Path B -- Developer Experience First, with Benchmark Fast-Follow. Evaluates three candidate paths (A: Benchmark First, B: DX First, C: OpenClaw Window) against the axiom hierarchy and downside analysis. Recommends 6-week time to publication, 8/10 confidence, with expected 5-15 MACC at 6 months. Documents the specific trade-offs (publishing without benchmark numbers, deferring Marcus persona 2-3 weeks), key review triggers (OpenClaw timeline, competitor moves, LoCoMo baseline results), and an explicit avoid list. Key output: the decision document that resolves "what do we do next" with full reasoning.

#### 21. [Action Roadmap](ACTION_ROADMAP.md)
**Phase 12 | 9,577 words**

Translates the strategic recommendation into a week-by-week execution plan across five phases: P0 Hardening (weeks 1-2, closing semver safety and transaction integrity gaps), P1 Quality & Docs (weeks 3-4, lifecycle correctness, README, examples, test expansion), CI & Publication (weeks 5-6, GitHub Actions, cargo publish v0.1.0, launch communications), Benchmark Fast-Follow (weeks 7-9, LoCoMo harness, honest blog post), and v0.1.x Iteration (weeks 10-12, feedback response, v0.2 planning). Includes specific task assignments with file paths, review gates at each phase boundary, success criteria at 90 days, and pivot triggers for strategy adjustment. Key output: the actionable checklist that turns strategy into daily work.

#### 22. [Documentation Index](INDEX.md)
**Phase 13 | This document**

The document you are reading. Provides the complete document hierarchy, audience-specific reading orders, one-paragraph summaries of all 22 documents, cross-reference map, and version history.

---

## 4. Cross-Reference Map

This matrix shows which documents reference which. A reference means the document actively draws on or cites content from the referenced document.

### Upstream Dependencies (what each document reads from)

| Document | Reads From |
|----------|------------|
| BRAND_GUIDELINES | (root document -- no upstream) |
| NORTHSTAR | BRAND_GUIDELINES |
| COMPETITIVE_LANDSCAPE | BRAND_GUIDELINES, NORTHSTAR |
| NORTHSTAR_EXTRACT | BRAND_GUIDELINES, NORTHSTAR, COMPETITIVE_LANDSCAPE |
| USER_JOURNEYS | BRAND_GUIDELINES, NORTHSTAR, COMPETITIVE_LANDSCAPE, NORTHSTAR_EXTRACT |
| UI_DESIGN_SYSTEM | BRAND_GUIDELINES, NORTHSTAR, NORTHSTAR_EXTRACT, USER_JOURNEYS, COMPETITIVE_LANDSCAPE |
| ACCESSIBILITY | USER_JOURNEYS, UI_DESIGN_SYSTEM, BRAND_GUIDELINES, NORTHSTAR, NORTHSTAR_EXTRACT |
| WIREFRAMES | USER_JOURNEYS, UI_DESIGN_SYSTEM, ACCESSIBILITY, BRAND_GUIDELINES, NORTHSTAR, NORTHSTAR_EXTRACT |
| ARCHITECTURE_BLUEPRINT | NORTHSTAR, NORTHSTAR_EXTRACT, COMPETITIVE_LANDSCAPE |
| AGENT_PROMPTS | ARCHITECTURE_BLUEPRINT, NORTHSTAR, NORTHSTAR_EXTRACT, USER_JOURNEYS |
| SECURITY_ARCHITECTURE | ARCHITECTURE_BLUEPRINT, NORTHSTAR_EXTRACT, ADR |
| ADR | ARCHITECTURE_BLUEPRINT, NORTHSTAR_EXTRACT, SECURITY_ARCHITECTURE |
| POST_DEPLOYMENT | ARCHITECTURE_BLUEPRINT, SECURITY_ARCHITECTURE, TESTING_STRATEGY |
| PIPELINE_ORCHESTRATION | ARCHITECTURE_BLUEPRINT, NORTHSTAR_EXTRACT |
| RESILIENCE_PATTERNS | ARCHITECTURE_BLUEPRINT, SECURITY_ARCHITECTURE, ADR |
| IMPLEMENTATION_SCAFFOLD | ARCHITECTURE_BLUEPRINT, NORTHSTAR_EXTRACT, TESTING_STRATEGY |
| OBSERVABILITY | ARCHITECTURE_BLUEPRINT, PIPELINE_ORCHESTRATION, SECURITY_ARCHITECTURE |
| TESTING_STRATEGY | ARCHITECTURE_BLUEPRINT, IMPLEMENTATION_SCAFFOLD, PIPELINE_ORCHESTRATION |
| HANDOFF_PROTOCOL | ARCHITECTURE_BLUEPRINT, AGENT_PROMPTS, RESILIENCE_PATTERNS |
| STRATEGIC_RECOMMENDATION | All Tier 1 + Tier 2 documents |
| ACTION_ROADMAP | STRATEGIC_RECOMMENDATION, IMPLEMENTATION_SCAFFOLD, TESTING_STRATEGY |
| INDEX | All documents |

### Downstream Impact (what reads from each document)

| Document | Read By | Impact Level |
|----------|---------|--------------|
| BRAND_GUIDELINES | 10 documents | Foundational -- terminology and voice propagate everywhere |
| NORTHSTAR | 9 documents | Foundational -- metric and personas drive all design |
| NORTHSTAR_EXTRACT | 9 documents | Foundational -- axioms resolve conflicts in every document |
| COMPETITIVE_LANDSCAPE | 4 documents | Strategic -- informs positioning and differentiation claims |
| ARCHITECTURE_BLUEPRINT | 12 documents | Structural -- every implementation document depends on this |
| USER_JOURNEYS | 5 documents | Design -- shapes API, docs, and accessibility decisions |
| UI_DESIGN_SYSTEM | 4 documents | Design -- naming and type patterns propagate to implementation |
| SECURITY_ARCHITECTURE | 5 documents | Constraint -- security requirements gate feature decisions |
| ADR | 3 documents | Constraint -- immutable decisions prevent re-litigation |
| TESTING_STRATEGY | 3 documents | Quality -- CI gates affect publication and operations |
| IMPLEMENTATION_SCAFFOLD | 3 documents | Practical -- file paths and module map used for task planning |
| PIPELINE_ORCHESTRATION | 3 documents | Technical -- stage details used by observability and testing |
| STRATEGIC_RECOMMENDATION | 2 documents | Directive -- the decision that drives the roadmap |

### Critical Reference Chains

These are the dependency chains where a change in an upstream document cascades through multiple layers:

```
BRAND_GUIDELINES -> NORTHSTAR -> NORTHSTAR_EXTRACT -> ARCHITECTURE_BLUEPRINT -> [all implementation docs]
                                                   |
                                                   +-> STRATEGIC_RECOMMENDATION -> ACTION_ROADMAP

SECURITY_ARCHITECTURE -> RESILIENCE_PATTERNS -> HANDOFF_PROTOCOL
                      |
                      +-> POST_DEPLOYMENT

USER_JOURNEYS -> UI_DESIGN_SYSTEM -> ACCESSIBILITY -> WIREFRAMES
```

---

## 5. Additional Outputs

### ai-context.yml

**Path:** `north-star-advisor/ai-context.yml`
**Size:** 40 KB
**Purpose:** Machine-readable progressive strategic context. Updated after each generation phase. Contains structured YAML representations of all key decisions, metrics, architecture details, security posture, testing strategy, implementation scaffold, and roadmap. Designed for AI agents to consume as context when working on Alaya.

**Sections:** `_meta`, `project`, `northstar`, `strategy`, `brand`, `market`, `design`, `architecture`, `decisions`, `security`, `operations`, `testing`, `implementation`, `roadmap`, `references`

### Research Summary

**Path:** `north-star-advisor/.work-in-progress/research/summary.md`
**Size:** 8.6 KB
**Purpose:** Synthesis of competitive research conducted during Phase 0 (pre-generation). Summarizes findings from analyzing Mem0, Zep/Graphiti, Letta/MemGPT, Supermemory, Hindsight/Vectorize, Memvid, Engram, and the broader agent memory landscape. Fed into the Competitive Landscape document as primary source material.

---

## 6. Version History

All documents were generated on 2026-02-26 during a single 13-phase strategic documentation process.

| Phase | Document | Generated | Words |
|-------|----------|-----------|-------|
| 0 | Research Summary | 2026-02-26 16:46 | -- |
| 1 | [Brand Guidelines](BRAND_GUIDELINES.md) | 2026-02-26 16:51 | 4,536 |
| 2 | [North Star Specification](NORTHSTAR.md) | 2026-02-26 17:14 | 6,742 |
| 3 | [Competitive Landscape](COMPETITIVE_LANDSCAPE.md) | 2026-02-26 17:23 | 5,749 |
| 4 | [North Star Extract](NORTHSTAR_EXTRACT.md) | 2026-02-26 17:30 | 3,938 |
| 5a | [Developer Experience Journeys](design/USER_JOURNEYS.md) | 2026-02-26 17:39 | 5,900 |
| 5b | [API Design System](design/UI_DESIGN_SYSTEM.md) | 2026-02-26 17:47 | 6,185 |
| 5c | [Developer Accessibility Strategy](design/ACCESSIBILITY.md) | 2026-02-26 17:56 | 7,813 |
| 5d | [API Surface Specification](design/WIREFRAMES.md) | 2026-02-26 18:07 | 10,266 |
| 6 | [Architecture Blueprint](ARCHITECTURE_BLUEPRINT.md) | 2026-02-26 18:27 | 5,975 |
| 7 | [Consumer Integration Patterns](architecture/AGENT_PROMPTS.md) | 2026-02-26 18:36 | 6,281 |
| 8 | [Security Architecture](SECURITY_ARCHITECTURE.md) | 2026-02-26 18:48 | 7,064 |
| 9 | [Architecture Decision Records](ADR.md) | 2026-02-26 18:58 | 7,925 |
| 10 | [Library Operations & Release Management](POST_DEPLOYMENT.md) | 2026-02-26 19:16 | 9,136 |
| 10p | [Retrieval Pipeline Implementation](architecture/PIPELINE_ORCHESTRATION.md) | 2026-02-26 19:15 | 7,602 |
| 10r | [Graceful Degradation Patterns](architecture/RESILIENCE_PATTERNS.md) | 2026-02-26 19:16 | 9,105 |
| 10s | [Module Structure & Build System](architecture/IMPLEMENTATION_SCAFFOLD.md) | 2026-02-26 19:16 | 8,445 |
| 10d | [Library Instrumentation](architecture/OBSERVABILITY.md) | 2026-02-26 19:15 | 6,987 |
| 10t | [Rust Testing Framework](architecture/TESTING_STRATEGY.md) | 2026-02-26 19:17 | 9,708 |
| 11 | [Strategic Recommendation](STRATEGIC_RECOMMENDATION.md) | 2026-02-26 21:15 | 7,220 |
| 11h | [Provider Contract Protocol](architecture/HANDOFF_PROTOCOL.md) | 2026-02-26 19:15 | 7,548 |
| 12 | [Action Roadmap](ACTION_ROADMAP.md) | 2026-02-26 21:25 | 9,577 |
| 13 | [Documentation Index](INDEX.md) | 2026-02-26 | -- |

### Generation Statistics

- **Total generation time:** ~5 hours (16:46 to 21:25, plus Phase 13)
- **Total word count:** ~155,000 words across 22 documents
- **Average document size:** ~7,300 words
- **Largest document:** API Surface Specification (WIREFRAMES.md) at 10,266 words
- **Smallest document:** North Star Extract (NORTHSTAR_EXTRACT.md) at 3,938 words

---

## How to Use This Documentation

**If you are new to the project**, start with the [New Contributor reading order](#new-contributor). It takes approximately 60-90 minutes to read through the five recommended documents.

**If you need a specific answer**, use the [Document Summaries](#3-document-summaries) to identify which document covers your topic, then navigate directly via the links.

**If you are updating a document**, consult the [Cross-Reference Map](#4-cross-reference-map) to understand which downstream documents may need corresponding updates.

**If you are an AI agent** working on Alaya, load `north-star-advisor/ai-context.yml` as structured context. It contains the machine-readable distillation of all 22 documents.

---

*Generated by North Star Advisor, Phase 13 of 13.*
