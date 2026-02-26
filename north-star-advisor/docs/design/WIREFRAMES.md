# Alaya API Surface Wireframes

Alaya is a Rust crate. There are no screens, no layouts, no responsive breakpoints. For a library, the "wireframes" are the concrete developer-facing surfaces that synthesize all previous design work into what developers actually see: the README they scan in 30 seconds, the docs.rs pages they browse, the example code they copy-paste, the error output they read in their terminal, and the diagnostic reports they use to debug.

This document specifies the exact structure and content of each surface. Every wireframe is grounded in the actual `AlayaStore` API, cross-referenced against the developer journeys (Phase 5a), API design system (Phase 5b), and accessibility strategy (Phase 5c).

**Cross-references:** [Developer Journeys](USER_JOURNEYS.md) | [API Design System](UI_DESIGN_SYSTEM.md) | [Accessibility](ACCESSIBILITY.md) | [Brand Guidelines](../BRAND_GUIDELINES.md) | [North Star](../NORTHSTAR.md) | [North Star Extract](../NORTHSTAR_EXTRACT.md)

---

## Table of Contents

1. [README Wireframe (Landing Page)](#1-readme-wireframe)
2. [docs.rs Module Layout Wireframe](#2-docsrs-module-layout-wireframe)
3. [Example Code Templates (The "Screens")](#3-example-code-templates)
4. [Error Output Wireframe](#4-error-output-wireframe)
5. [MCP Server Interface Wireframe](#5-mcp-server-interface-wireframe)
6. [Diagnostic Output Wireframe](#6-diagnostic-output-wireframe)
7. [Integration Gaps Analysis](#7-integration-gaps-analysis)

---

## 1. README Wireframe

The README is the landing page. A developer arriving from crates.io, GitHub search, or a blog post link makes their `cargo add` decision within 3 minutes of landing here (Journey 1, Phase 2). The README must answer six questions in order: What is this? How do I use it? Why is it different? How does it work? Who else exists? Where do I learn more?

### Structure Specification

```
+=====================================================================+
| # Alaya                                                              |
|                                                                      |
| **Memory is a process, not a database.**                             |
|                                                                      |
| [crates.io badge] [docs.rs badge] [CI badge] [license badge]        |
|                                                                      |
| Embeddable Rust memory engine with cognitive lifecycle processes      |
| and implicit preference emergence for privacy-first AI agents.       |
| Single SQLite file. Zero external dependencies. No network calls.    |
|                                                                      |
+=====================================================================+
| ## Quick Start                                                       |
|                                                                      |
| ```bash                                                              |
| cargo add alaya                                                      |
| ```                                                                  |
|                                                                      |
| ```rust                                                              |
| use alaya::{AlayaStore, NewEpisode, Role, EpisodeContext, Query};    |
|                                                                      |
| fn main() -> alaya::Result<()> {                                     |
|     let store = AlayaStore::open("memory.db")?;                      |
|                                                                      |
|     store.store_episode(&NewEpisode {                                |
|         content: "I prefer dark mode and Vim keybindings".into(),    |
|         role: Role::User,                                            |
|         session_id: "session-1".into(),                              |
|         timestamp: 1740000000,                                       |
|         context: EpisodeContext::default(),                          |
|         embedding: None,                                             |
|     })?;                                                             |
|                                                                      |
|     let results = store.query(&Query::simple("editor preferences"))? |
|     for mem in &results {                                            |
|         println!("[{:.2}] {}", mem.score, mem.content);              |
|     }                                                                |
|     Ok(())                                                           |
| }                                                                    |
| ```                                                                  |
|                                                                      |
+=====================================================================+
| ## Why Alaya?                                                        |
|                                                                      |
| [2-3 sentence positioning statement]                                 |
|                                                                      |
| - **Single-file deployment** -- one SQLite database, no external     |
|   services                                                           |
| - **Zero network calls** -- privacy by architecture, not policy      |
| - **LLM-agnostic** -- no hardcoded provider; traits for extension    |
| - **Memory as process** -- Hebbian graph, adaptive forgetting,       |
|   preference emergence                                               |
| - **Principled foundations** -- CLS theory, Bjork forgetting,        |
|   spreading activation, Yogacara psychology                          |
| - **Rust** -- embed in any language via FFI, zero GC pauses          |
|                                                                      |
+=====================================================================+
| ## How It Works                                                      |
|                                                                      |
| [Architecture diagram: Mermaid graph showing three stores,           |
|  graph overlay, retrieval pipeline, lifecycle processes]              |
|                                                                      |
| [Three Stores table: Episodic / Semantic / Implicit]                 |
| [Retrieval Pipeline diagram: BM25 + Vector + Graph -> RRF -> Rerank] |
| [Lifecycle table: Consolidation / Perfuming / Transformation /       |
|  Forgetting]                                                         |
|                                                                      |
+=====================================================================+
| ## API Overview                                                      |
|                                                                      |
| ```rust                                                              |
| impl AlayaStore {                                                    |
|     // Write                                                         |
|     pub fn store_episode(&self, ep: &NewEpisode) -> Result<EpisodeId>|
|                                                                      |
|     // Read                                                          |
|     pub fn query(&self, q: &Query) -> Result<Vec<ScoredMemory>>;     |
|     pub fn preferences(&self, d: Option<&str>) -> ...;               |
|     pub fn knowledge(&self, f: Option<KnowledgeFilter>) -> ...;      |
|     pub fn neighbors(&self, n: NodeRef, depth: u32) -> ...;          |
|                                                                      |
|     // Lifecycle                                                     |
|     pub fn consolidate(&self, p: &dyn ConsolidationProvider) -> ...; |
|     pub fn perfume(&self, i: &Interaction, p: &dyn ...) -> ...;      |
|     pub fn transform(&self) -> Result<TransformationReport>;         |
|     pub fn forget(&self) -> Result<ForgettingReport>;                |
|                                                                      |
|     // Admin                                                         |
|     pub fn status(&self) -> Result<MemoryStatus>;                    |
|     pub fn purge(&self, f: PurgeFilter) -> Result<PurgeReport>;      |
| }                                                                    |
| ```                                                                  |
|                                                                      |
+=====================================================================+
| ## Comparison                                                        |
|                                                                      |
| [Feature comparison table: Alaya vs Mem0 vs Zep vs Letta vs         |
|  Supermemory vs Memvid vs Engram]                                    |
| [Columns: Storage, Infra, LLM, Graph, Forgetting, Preferences]      |
|                                                                      |
+=====================================================================+
| ## Documentation                                                     |
|                                                                      |
| - [API Reference (docs.rs)](https://docs.rs/alaya)                   |
| - [Examples](examples/)                                              |
| - [Architecture Guide](docs/design.md)                               |
| - [Research Foundations](#research-foundations)                        |
| - [Related Work](docs/related-work.md)                               |
|                                                                      |
+=====================================================================+
| ## Research Foundations                                               |
|                                                                      |
| [Neuroscience citations: Hebbian, CLS, Spreading Activation,        |
|  Bjork, RIF, Encoding Specificity, Working Memory]                   |
| [Yogacara citations: Alaya-vijnana, Bija, Vasana, Asraya-paravrtti]  |
| [IR citations: RRF, BM25, Cosine Similarity]                        |
|                                                                      |
+=====================================================================+
| ## License                                                           |
|                                                                      |
| MIT                                                                  |
+=====================================================================+
```

### Section-by-Section Specification

#### Header Block (Lines 1-6)

**Purpose:** Identity and trust signals. Must be scannable in under 5 seconds.

| Element | Content | Rationale |
|---------|---------|-----------|
| Title | `# Alaya` | Product name only. No subtitle clutter. |
| Tagline | `**Memory is a process, not a database.**` | Brand Belief 1. Immediately differentiates from "just another vector store." |
| Badges | crates.io version, docs.rs, CI status, MIT license | Four trust signals in one line. Order matters: version (active?), docs (documented?), CI (tested?), license (can I use it?). |
| One-liner | "Embeddable Rust memory engine with cognitive lifecycle processes and implicit preference emergence for privacy-first AI agents." | From Brand positioning. Three differentiators in one sentence: cognitive lifecycle, preference emergence, privacy-first. |
| Constraints line | "Single SQLite file. Zero external dependencies. No network calls." | Three short sentences. Each is verifiable. This is what Priya (privacy persona) reads first. |

**Rules:**
- No specialized terminology in the first 6 lines. "Yogacara," "Hebbian," "vasana" appear later.
- Code must be visible without scrolling on a standard 1080p display.
- Badge links must resolve (do not add badges until crates.io publish).

#### Quick Start Section (Lines 7-30)

**Purpose:** 2-minute path from zero to working query results. This is the most important section in the entire README.

**Content specification:**

1. **Installation command:** `cargo add alaya` -- one line, no feature flags for basic use.
2. **Code example:** 15-20 lines maximum.
3. **Imports:** Exactly 5 types: `AlayaStore`, `NewEpisode`, `Role`, `EpisodeContext`, `Query`.
4. **No chrono dependency:** Use a literal timestamp (`1740000000`), not `chrono::Utc::now()`. The quickstart must not require dependencies beyond alaya.
5. **Query must have lexical overlap with stored content:** "editor preferences" overlaps with "Vim keybindings" through "prefer" and "dark mode" being semantically adjacent. To guard against the BM25 empty-result trap (Journey 3, Flow B), the example uses content and query text that share enough tokens for BM25 to match. Specifically: store "I prefer dark mode and Vim keybindings", query "dark mode preferences" -- "dark" and "mode" overlap directly.

**Exact quickstart code:**

```rust
use alaya::{AlayaStore, NewEpisode, Role, EpisodeContext, Query};

fn main() -> alaya::Result<()> {
    let store = AlayaStore::open("memory.db")?;

    store.store_episode(&NewEpisode {
        content: "I prefer dark mode and Vim keybindings".into(),
        role: Role::User,
        session_id: "session-1".into(),
        timestamp: 1740000000,
        context: EpisodeContext::default(),
        embedding: None,
    })?;

    let results = store.query(&Query::simple("dark mode preferences"))?;
    for mem in &results {
        println!("[{:.2}] {}", mem.score, mem.content);
    }

    Ok(())
}
```

**Expected output:**

```
[0.42] I prefer dark mode and Vim keybindings
```

**Rules:**
- The quickstart compiles without modification against the current API.
- CI regression test extracts this code block and compiles it as a standalone project.
- If this example ever requires an LLM API key, Alaya has violated its own axioms.
- No `unwrap()` -- use `?` throughout to model good error handling.
- The `EpisodeContext::default()` call must not confuse beginners. No comment needed; the type name is self-documenting.

#### Why Alaya Section (Lines 31-45)

**Purpose:** Differentiation for the evaluation phase. Developer has seen the quickstart and is now deciding whether to commit.

**Content specification:**

Opening paragraph (2-3 sentences):
> Most AI memory systems are Python libraries that require external infrastructure (Postgres, Neo4j, Redis, Pinecone) and are tightly coupled to specific LLM providers. Alaya is a Rust library that provides a complete cognitive lifecycle -- consolidation, forgetting, preference emergence, and graph reshaping -- in a single SQLite file with zero external dependencies.

Followed by 6 bullet points. Each starts with a bold phrase and a double-dash explanation. Order is deliberate:

| Position | Bullet | Addresses |
|----------|--------|-----------|
| 1 | Single-file deployment | Operational simplicity (vs. Zep/Mem0 infrastructure) |
| 2 | Zero network calls | Privacy (Priya's first check) |
| 3 | LLM-agnostic | No coupling (vs. Mem0/Letta LLM requirement) |
| 4 | Memory as process | Core differentiator (Belief 1) |
| 5 | Principled foundations | Research grounding (Honesty > Marketing) |
| 6 | Rust | Performance and embeddability (Marcus's check) |

**Rules:**
- No superlatives without evidence ("blazing fast" is banned; "sub-millisecond BM25 retrieval at 1K episodes" is allowed when benchmarks exist).
- Each bullet is verifiable from the source code or Cargo.toml.

#### How It Works Section (Lines 46-65)

**Purpose:** Architecture overview for developers who want to understand the system before using it (Marcus persona, and deepening integration developers).

**Content specification:**

1. **Architecture diagram** (Mermaid): Shows Agent <-> Alaya relationship. Alaya contains three stores, graph overlay, retrieval engine, lifecycle processes. Agent contains identity, context assembly, LLM provider, embedding provider.

2. **Three Stores table:**

| Store | Analog | Purpose |
|-------|--------|---------|
| Episodic | Hippocampus | Raw conversation events with full context |
| Semantic | Neocortex | Distilled knowledge extracted through consolidation |
| Implicit | Alaya-vijnana | Preferences and habits that emerge through perfuming |

3. **Retrieval Pipeline diagram** (Mermaid): `Query -> [BM25, Vector, Graph] -> RRF -> Rerank -> RIF -> Results`

4. **Lifecycle Processes table:**

| Process | Inspiration | What It Does |
|---------|-------------|--------------|
| Consolidation | CLS theory | Distills episodes into semantic knowledge |
| Perfuming | Vasana (Yogacara) | Accumulates impressions, crystallizes preferences |
| Transformation | Asraya-paravrtti | Deduplicates, resolves contradictions, prunes |
| Forgetting | Bjork & Bjork | Decays retrieval strength, archives weak nodes |

**Rules:**
- Diagrams use Mermaid (renders on GitHub, docs.rs).
- Terminology uses plain English first, research term in parentheses.
- This section is optional reading for beginners -- the quickstart works without understanding any of this.

#### API Overview Section (Lines 66-85)

**Purpose:** Complete method surface at a glance. Developer decides "is this API surface manageable?" (Accessibility target: <20 public methods).

**Content specification:**

Show the full `impl AlayaStore` block with all 12 public methods, grouped by category:
- Write (1 method): `store_episode`
- Read (4 methods): `query`, `preferences`, `knowledge`, `neighbors`
- Lifecycle (4 methods): `consolidate`, `perfume`, `transform`, `forget`
- Admin (2 methods): `status`, `purge`

Plus the provider trait:
- `ConsolidationProvider` (3 methods): `extract_knowledge`, `extract_impressions`, `detect_contradiction`

**Rules:**
- Method signatures must match the actual codebase exactly.
- Group comments (`// Write`, `// Read`, etc.) provide visual structure.
- No method documentation here -- that is docs.rs territory. This is a surface scan.

#### Comparison Table Section (Lines 86-100)

**Purpose:** Competitive positioning. The developer has evaluated the API and now wants to see how Alaya compares.

**Content specification:**

Table with 8+ systems, 8 columns:

| Column | What It Shows |
|--------|---------------|
| System | Name, language if not Python |
| Storage | Infrastructure requirement |
| Infra | Number of external services needed |
| LLM | Required / Optional / None |
| Memory Model | Architecture description |
| Graph | Graph type or None |
| Forgetting | Forgetting mechanism or None |
| Preferences | Preference learning or None |

**Systems to include (minimum):**
1. Alaya (Rust) -- first row, bold
2. Mem0 -- cloud SaaS, LLM-required
3. Zep / Graphiti -- Neo4j, LLM-required
4. Letta (MemGPT) -- LLM-dependent
5. Supermemory (TypeScript) -- VC-backed
6. Memvid (Rust) -- adjacent competitor, same deployment model
7. Engram -- adjacent competitor, zero-dep philosophy

**Rules:**
- Factual only. No marketing language about competitors.
- Link to `docs/related-work.md` for the comprehensive analysis.
- Update with each competitive landscape review (quarterly cadence per Phase 3).

#### Documentation Section (Lines 101-110)

**Purpose:** Progressive disclosure. Developer knows what Alaya is and wants to go deeper.

**Content specification:**

5 links in order of depth:
1. API Reference (docs.rs) -- complete reference
2. Examples directory -- copy-paste code
3. Architecture Guide -- design decisions
4. Research Foundations -- academic grounding
5. Related Work -- competitive analysis

**Rules:**
- Links must resolve. Do not add links to nonexistent pages.
- Order matches documentation layers from Accessibility (Phase 5c): README -> examples -> docs.rs -> architecture -> research.

#### Research Foundations Section (Lines 111-130)

**Purpose:** Credibility for the "correctness over speed" axiom. This is where research-grounded terminology is appropriate.

**Content specification:**

Three subsections:
1. **Neuroscience:** Hebbian LTP/LTD, CLS theory, Spreading Activation, Encoding Specificity, Dual-Strength Forgetting, RIF, Working Memory Limits. Each with one-line description and citation (Author Year).
2. **Yogacara Buddhist Psychology:** Alaya-vijnana, Bija, Vasana, Asraya-paravrtti, Vijnaptimatrata. Each with plain English explanation.
3. **Information Retrieval:** RRF, BM25 via FTS5, Cosine Similarity.

**Rules:**
- Research terms are permitted here (this is Level 5 documentation per Accessibility).
- Each term has a plain English gloss before the technical detail.
- Citations include author and year, not full bibliographic entries.

---

## 2. docs.rs Module Layout Wireframe

docs.rs is the reference documentation. It is generated from doc comments in the source code. This wireframe specifies what each module page should look like when a developer browses docs.rs/alaya.

### Crate Root Page (`alaya`)

```
+=====================================================================+
| Crate alaya                                                          |
|                                                                      |
| Embeddable Rust memory engine with cognitive lifecycle processes      |
| and implicit preference emergence for privacy-first AI agents.       |
|                                                                      |
| ## Quick Start                                                       |
| [Same 15-line quickstart as README]                                  |
|                                                                      |
| ## Architecture                                                      |
| [Text description of three stores, graph, retrieval, lifecycle]      |
| [ASCII diagram of data flow -- Mermaid does not render on docs.rs]   |
|                                                                      |
| ## Modules                                                           |
| - error -- Error types and Result alias                              |
| - provider -- ConsolidationProvider trait and NoOpProvider            |
| - types -- All public types (IDs, enums, structs, reports)           |
|                                                                      |
| ## Structs                                                           |
| - AlayaStore -- The main entry point                                 |
|                                                                      |
| ## Re-exports                                                        |
| - pub use error::{AlayaError, Result}                                |
| - pub use provider::{ConsolidationProvider, NoOpProvider}             |
| - pub use types::*                                                   |
+=====================================================================+
```

**Specification for the `//! ...` crate doc comment:**

```rust
//! # Alaya
//!
//! Embeddable Rust memory engine with cognitive lifecycle processes
//! and implicit preference emergence for privacy-first AI agents.
//!
//! Alaya provides three memory stores (episodic, semantic, implicit),
//! a Hebbian graph overlay, hybrid retrieval with spreading activation,
//! and adaptive lifecycle processes -- all in a single SQLite file with
//! zero external dependencies.
//!
//! ## Quick Start
//!
//! ```rust
//! # use alaya::{AlayaStore, NewEpisode, Role, EpisodeContext, Query};
//! # fn main() -> alaya::Result<()> {
//! let store = AlayaStore::open_in_memory()?;
//!
//! store.store_episode(&NewEpisode {
//!     content: "I prefer dark mode and Vim keybindings".into(),
//!     role: Role::User,
//!     session_id: "session-1".into(),
//!     timestamp: 1740000000,
//!     context: EpisodeContext::default(),
//!     embedding: None,
//! })?;
//!
//! let results = store.query(&Query::simple("dark mode preferences"))?;
//! assert!(!results.is_empty());
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! Alaya organizes memory into three stores, connected by a graph overlay:
//!
//! - **Episodic store** -- Raw conversation episodes (hippocampal analog).
//!   Fast write path, BM25 full-text index, optional vector embeddings.
//!
//! - **Semantic store** -- Distilled knowledge extracted through
//!   [`AlayaStore::consolidate`] (neocortical analog). Facts, relationships,
//!   events, and concepts with confidence scores.
//!
//! - **Implicit store** -- Impressions and crystallized preferences from
//!   [`AlayaStore::perfume`] (alaya-vijnana analog). Behavioral patterns
//!   emerge without explicit declaration.
//!
//! The **graph overlay** spans all three stores with Hebbian weighted links
//! that strengthen on co-retrieval (LTP) and weaken through disuse (LTD).
//!
//! The **retrieval pipeline** combines BM25 + vector + graph activation,
//! fuses via Reciprocal Rank Fusion, reranks by context, and applies
//! Retrieval-Induced Forgetting to retrieved results.
//!
//! ## Design Principles
//!
//! 1. **Memory is a process, not a database.** Every retrieval changes
//!    what is remembered.
//! 2. **Forgetting is a feature.** Strategic decay improves retrieval quality.
//! 3. **Preferences emerge, they are not declared.** Vasana/perfuming
//!    crystallizes patterns from observations.
//! 4. **The agent owns identity.** Alaya stores seeds; the agent decides
//!    which matter.
//! 5. **Graceful degradation.** No embeddings? BM25-only. No LLM?
//!    Episodes accumulate. Everything works independently.
```

**Rules:**
- The crate doc comment uses `open_in_memory()` in the doctest (not a file path) so the test runs without filesystem side effects.
- The doctest must compile and pass with `cargo test --doc`.
- Mermaid diagrams do not render on docs.rs; use text descriptions and link to the GitHub README for visual diagrams.
- Cross-references use `[`AlayaStore::consolidate`]` syntax for docs.rs linking.

### Module: `error`

```
+=====================================================================+
| Module alaya::error                                                  |
|                                                                      |
| Error types for the Alaya memory engine.                             |
|                                                                      |
| [`AlayaError`] is the primary error type. All fallible operations    |
| in Alaya return `Result<T, AlayaError>`. The [`Result`] type alias   |
| is provided for convenience.                                         |
|                                                                      |
| ## Error Variants                                                    |
|                                                                      |
| | Variant | Source | Recovery |                                      |
| |---------|--------|----------|                                      |
| | Db | SQLite operation | Check path, permissions, disk space |      |
| | NotFound | Entity lookup | Verify ID exists with list methods |    |
| | InvalidInput | API boundary | Check field constraints in docs |    |
| | Serialization | JSON round-trip | Check for schema version mismatch|
| | Provider | Your ConsolidationProvider | Fix your impl, data safe | |
|                                                                      |
| ## Enums                                                             |
| - AlayaError -- All error variants                                   |
|                                                                      |
| ## Type Aliases                                                      |
| - Result<T> = std::result::Result<T, AlayaError>                     |
+=====================================================================+
```

**Specification for `error.rs` module doc comment:**

```rust
//! Error types for the Alaya memory engine.
//!
//! [`AlayaError`] is the primary error type returned by all fallible
//! operations. Each variant includes enough context to diagnose the
//! problem and determine the next step.
//!
//! # Error Recovery Guide
//!
//! | Variant | Origin | What To Do |
//! |---------|--------|------------|
//! | [`AlayaError::Db`] | SQLite layer | Check file path, permissions, disk space. Your data is safe if the error occurred during a write (transactions roll back). |
//! | [`AlayaError::NotFound`] | Entity lookup | The requested ID does not exist. Use `list_*` methods to see available entities. |
//! | [`AlayaError::InvalidInput`] | API boundary | A field value failed validation. The error message includes the constraint that was violated. |
//! | [`AlayaError::Serialization`] | JSON round-trip | Internal serialization failed. This may indicate a schema version mismatch after an upgrade. |
//! | [`AlayaError::Provider`] | Your `ConsolidationProvider` | Your provider implementation returned an error. Alaya's data is not affected. Check your LLM client or extraction logic. |
```

**Rules:**
- The recovery table appears in the module doc, not just on individual variants.
- "Your data is safe" or "data is not affected" is stated explicitly where true.
- Provider errors are attributed to developer code with "your" language.

### Module: `types`

```
+=====================================================================+
| Module alaya::types                                                  |
|                                                                      |
| All public types for the Alaya memory engine.                        |
|                                                                      |
| Types are organized by role in the cognitive lifecycle:               |
|                                                                      |
| ## ID Types                                                          |
| Newtypes around i64 for type-safe entity references.                 |
| - EpisodeId, NodeId, PreferenceId, ImpressionId, LinkId              |
|                                                                      |
| ## Polymorphic Reference                                             |
| - NodeRef -- points into any store (Episode, Semantic, Preference)   |
|                                                                      |
| ## Input Types (New*)                                                |
| What you pass to Alaya. Fields are developer-controlled.             |
| - NewEpisode, NewSemanticNode, NewImpression, Interaction, Query     |
|                                                                      |
| ## Output Types                                                      |
| What Alaya returns. Includes system-generated fields (id, timestamps)|
| - Episode, SemanticNode, Impression, Preference, ScoredMemory, Link, |
|   NodeStrength                                                       |
|                                                                      |
| ## Report Types                                                      |
| Returned by lifecycle methods. Every field is a count you can log.   |
| - ConsolidationReport, PerfumingReport, TransformationReport,        |
|   ForgettingReport, PurgeReport, MemoryStatus                        |
|                                                                      |
| ## Enum Types                                                        |
| - Role, SemanticType, LinkType, PurgeFilter                          |
|                                                                      |
| ## Context Types                                                     |
| Optional metadata that improves retrieval quality.                   |
| - EpisodeContext, QueryContext, KnowledgeFilter                      |
+=====================================================================+
```

**Specification for `types.rs` module doc comment:**

```rust
//! All public types for the Alaya memory engine.
//!
//! Types are organized by their role in the cognitive lifecycle:
//!
//! - **ID types** ([`EpisodeId`], [`NodeId`], [`PreferenceId`],
//!   [`ImpressionId`], [`LinkId`]): Newtypes around `i64` for
//!   type-safe entity references.
//!
//! - **Input types** ([`NewEpisode`], [`NewSemanticNode`],
//!   [`NewImpression`], [`Interaction`], [`Query`]): What you pass
//!   to Alaya. All fields are developer-controlled.
//!
//! - **Output types** ([`Episode`], [`SemanticNode`], [`Impression`],
//!   [`Preference`], [`ScoredMemory`], [`Link`], [`NodeStrength`]):
//!   What Alaya returns. Includes system-generated fields like `id`
//!   and timestamps.
//!
//! - **Report types** ([`ConsolidationReport`], [`PerfumingReport`],
//!   [`TransformationReport`], [`ForgettingReport`], [`PurgeReport`],
//!   [`MemoryStatus`]): Returned by lifecycle and admin methods.
//!   Every field is a count or status you can log, display, or act on.
//!
//! - **Enum types** ([`Role`], [`SemanticType`], [`LinkType`],
//!   [`PurgeFilter`]): Closed sets of valid values.
//!
//! - **Context types** ([`EpisodeContext`], [`QueryContext`],
//!   [`KnowledgeFilter`]): Optional metadata that improves retrieval
//!   quality. All implement `Default` for zero-config use.
//!
//! # Naming Convention
//!
//! - `New*` prefix = input type (you create it)
//! - No prefix = output type (Alaya creates it, includes `id` and timestamps)
//! - `*Report` suffix = lifecycle return value (all fields are counts)
//! - `*Filter` suffix = query constraint (all fields are `Option`)
```

**Rules:**
- Every type links to its docs.rs page via `[`TypeName`]` syntax.
- The "Naming Convention" section helps developers predict type names before looking them up (cognitive accessibility).
- Input vs. output distinction is made explicit (New*/Entity Split pattern from Phase 5b).

### Module: `provider`

```
+=====================================================================+
| Module alaya::provider                                               |
|                                                                      |
| Extension traits for LLM-powered memory processes.                   |
|                                                                      |
| Alaya never calls an LLM directly. The agent owns the LLM           |
| connection and implements these traits to enable intelligent          |
| consolidation and impression extraction.                             |
|                                                                      |
| ## Getting Started Without a Provider                                |
|                                                                      |
| [`NoOpProvider`] works out of the box. Basic operations              |
| (store, query, forget, transform) work without any provider.         |
| Consolidation and perfuming skip LLM-dependent steps.                |
|                                                                      |
| ## Implementing a Provider                                           |
|                                                                      |
| [Example: 20-line ConsolidationProvider implementation]              |
|                                                                      |
| ## Traits                                                            |
| - ConsolidationProvider -- 3 methods for knowledge extraction        |
|                                                                      |
| ## Structs                                                           |
| - NoOpProvider -- Default provider, returns empty results            |
+=====================================================================+
```

**Specification for `provider.rs` module doc comment:**

```rust
//! Extension traits for LLM-powered memory processes.
//!
//! Alaya never calls an LLM directly. The agent owns the LLM connection
//! and implements [`ConsolidationProvider`] to enable intelligent
//! consolidation and impression extraction.
//!
//! # Getting Started Without a Provider
//!
//! [`NoOpProvider`] works out of the box:
//!
//! ```rust
//! # use alaya::{AlayaStore, NoOpProvider};
//! # fn main() -> alaya::Result<()> {
//! let store = AlayaStore::open_in_memory()?;
//! let noop = NoOpProvider;
//! let report = store.consolidate(&noop)?;
//! // NoOpProvider skips LLM-dependent extraction.
//! // Episodes accumulate; consolidation runs but creates no nodes.
//! assert_eq!(report.nodes_created, 0);
//! # Ok(())
//! # }
//! ```
//!
//! # Implementing a Custom Provider
//!
//! ```rust,no_run
//! use alaya::*;
//!
//! struct MyProvider { /* your LLM client */ }
//!
//! impl ConsolidationProvider for MyProvider {
//!     fn extract_knowledge(
//!         &self, episodes: &[Episode],
//!     ) -> Result<Vec<NewSemanticNode>> {
//!         // Send episodes to your LLM, parse structured output.
//!         // Return extracted facts, relationships, events, concepts.
//!         todo!("Implement with your LLM client")
//!     }
//!
//!     fn extract_impressions(
//!         &self, interaction: &Interaction,
//!     ) -> Result<Vec<NewImpression>> {
//!         // Analyze interaction for implicit behavioral signals.
//!         // Return domain/observation/valence triples.
//!         todo!("Implement with your LLM client")
//!     }
//!
//!     fn detect_contradiction(
//!         &self, a: &SemanticNode, b: &SemanticNode,
//!     ) -> Result<bool> {
//!         // Ask your LLM if two knowledge nodes contradict.
//!         todo!("Implement with your LLM client")
//!     }
//! }
//! ```
//!
//! When your provider returns an error, Alaya wraps it in
//! [`AlayaError::Provider`] and skips the current batch. Your data
//! is never affected by provider failures.
```

**Rules:**
- The module doc includes both NoOpProvider usage and custom implementation.
- `no_run` on the custom provider example (it uses `todo!()`).
- "Your data is never affected by provider failures" -- explicit safety guarantee.
- Provider error attribution is clear.

---

## 3. Example Code Templates

The `examples/` directory contains standalone programs that developers copy-paste as starting points. Each example maps to a skill level from the Accessibility strategy (Phase 5c) and a phase in the Deepening Integration journey (Phase 5a, Journey 2).

### Example Inventory

| File | Lines | Skill Level | Journey Phase | Purpose |
|------|-------|-------------|---------------|---------|
| `basic.rs` | 25 | Level 1 (Beginner) | First Code | Store episodes, query, print results |
| `lifecycle.rs` | 40 | Level 2 (Intermediate) | Dream Cycle | Run the full cognitive lifecycle |
| `custom_provider.rs` | 60 | Level 3 (Advanced) | Custom Providers | Implement ConsolidationProvider |
| `advanced_retrieval.rs` | 45 | Level 3 (Advanced) | Tuning | QueryContext, embeddings, graph, knowledge |
| `production.rs` | 55 | Level 4 (Expert) | Production | Arc, backup, periodic tasks, monitoring |

### Example: `basic.rs`

**Purpose:** Identical outcome to README quickstart, but as a standalone runnable file. Covers store, query, and status.

```rust
//! Basic Alaya usage: store episodes and query memories.
//!
//! Run with: cargo run --example basic

use alaya::{AlayaStore, EpisodeContext, NewEpisode, Query, Role};

fn main() -> alaya::Result<()> {
    // Open a persistent database. One line. One file.
    let store = AlayaStore::open("basic_memory.db")?;

    // Store some conversation episodes
    let episodes = vec![
        ("I prefer dark mode and monospace fonts", Role::User),
        ("Noted! I'll keep that in mind for recommendations.", Role::Assistant),
        ("I've been learning Rust for about six months now", Role::User),
        ("That's great progress! What areas interest you most?", Role::Assistant),
        ("Systems programming and embedded devices", Role::User),
    ];

    for (i, (content, role)) in episodes.iter().enumerate() {
        store.store_episode(&NewEpisode {
            content: content.to_string(),
            role: *role,
            session_id: "demo-session".into(),
            timestamp: 1740000000 + (i as i64) * 60,
            context: EpisodeContext::default(),
            embedding: None,
        })?;
    }

    // Query with natural language -- BM25 full-text retrieval
    println!("Query: 'programming experience'");
    let results = store.query(&Query::simple("programming experience"))?;
    for mem in &results {
        println!("  [{:.3}] {}", mem.score, mem.content);
    }

    println!();

    // Check system status
    let status = store.status()?;
    println!("Memory status:");
    println!("  Episodes:       {}", status.episode_count);
    println!("  Semantic nodes: {}", status.semantic_node_count);
    println!("  Preferences:    {}", status.preference_count);
    println!("  Graph links:    {}", status.link_count);

    Ok(())
}
```

**Rules:**
- Uses `basic_memory.db` (not `memory.db`) to avoid filename collision with README quickstart.
- Stores 5 episodes (enough for BM25 to have interesting ranking behavior).
- Query term "programming experience" has lexical overlap with stored content.
- Prints status to show the full store state.
- No cleanup of the `.db` file -- developer can inspect it with `sqlite3`.

### Example: `lifecycle.rs`

**Purpose:** Demonstrate the "dream cycle" pattern. Consolidation, forgetting, and transformation with reports.

```rust
//! Alaya lifecycle: the "dream cycle" that transforms raw episodes
//! into structured knowledge and refined recall.
//!
//! Run with: cargo run --example lifecycle

use alaya::{
    AlayaStore, EpisodeContext, NewEpisode, NoOpProvider, Query, Role,
};

fn main() -> alaya::Result<()> {
    let store = AlayaStore::open("lifecycle_memory.db")?;

    // Simulate a multi-session conversation
    let sessions = vec![
        vec![
            "I always use dark mode in every application",
            "My favorite editor is Neovim with custom Lua config",
            "I prefer terminal-based tools over GUIs",
        ],
        vec![
            "Dark backgrounds reduce eye strain for me",
            "I switched from VS Code to Neovim last year",
            "CLI tools are faster and more composable",
        ],
    ];

    for (s, session) in sessions.iter().enumerate() {
        for (i, content) in session.iter().enumerate() {
            store.store_episode(&NewEpisode {
                content: content.to_string(),
                role: Role::User,
                session_id: format!("session-{}", s + 1),
                timestamp: 1740000000 + (s as i64) * 86400 + (i as i64) * 60,
                context: EpisodeContext::default(),
                embedding: None,
            })?;
        }
    }

    println!("Before dream cycle:");
    let status = store.status()?;
    println!("  Episodes: {}, Nodes: {}, Preferences: {}",
        status.episode_count, status.semantic_node_count, status.preference_count);

    // === The Dream Cycle ===
    // Run between conversations, like the brain consolidating during sleep.
    let provider = NoOpProvider; // Replace with your LLM provider for real extraction

    // 1. Consolidation: episodes -> semantic knowledge (CLS replay)
    let cr = store.consolidate(&provider)?;
    println!("\nConsolidation: {} episodes processed, {} nodes created",
        cr.episodes_processed, cr.nodes_created);

    // 2. Forgetting: decay retrieval strengths, archive weak nodes (Bjork)
    let fr = store.forget()?;
    println!("Forgetting:    {} decayed, {} archived",
        fr.nodes_decayed, fr.nodes_archived);

    // 3. Transformation: dedup, prune, structural cleanup (asraya-paravrtti)
    let tr = store.transform()?;
    println!("Transform:     {} merged, {} links pruned",
        tr.duplicates_merged, tr.links_pruned);

    println!("\nAfter dream cycle:");
    let status = store.status()?;
    println!("  Episodes: {}, Nodes: {}, Preferences: {}",
        status.episode_count, status.semantic_node_count, status.preference_count);

    // Query after lifecycle -- results may differ from pre-lifecycle query
    println!("\nQuery: 'editor preferences'");
    let results = store.query(&Query::simple("editor preferences"))?;
    for mem in &results {
        println!("  [{:.3}] {}", mem.score, mem.content);
    }

    Ok(())
}
```

**Rules:**
- Uses `NoOpProvider` so the example runs without an LLM.
- Shows before/after status to make lifecycle effects visible.
- Comments explain each lifecycle phase with the plain English name first, research term in parentheses.
- With NoOpProvider, consolidation creates no nodes (this is expected and the example should not be confusing about it). The comment "Replace with your LLM provider for real extraction" sets the right expectation.

### Example: `custom_provider.rs`

**Purpose:** Demonstrate implementing `ConsolidationProvider` with a mock LLM. This is the Level 3 example that bridges "using Alaya" to "extending Alaya."

```rust
//! Implementing a custom ConsolidationProvider.
//!
//! This example shows how to connect your LLM to Alaya's lifecycle.
//! The mock provider below simulates LLM responses; replace with
//! your actual LLM client.
//!
//! Run with: cargo run --example custom_provider

use alaya::*;

/// A mock provider that simulates LLM-powered knowledge extraction.
/// In production, replace this with calls to your LLM API.
struct MockLLMProvider;

impl ConsolidationProvider for MockLLMProvider {
    fn extract_knowledge(
        &self,
        episodes: &[Episode],
    ) -> Result<Vec<NewSemanticNode>> {
        // In production: send episodes to LLM with extraction prompt.
        // Here we do simple keyword-based extraction as a demo.
        let mut nodes = Vec::new();

        let combined: String = episodes.iter()
            .map(|e| e.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        if combined.contains("Rust") || combined.contains("programming") {
            nodes.push(NewSemanticNode {
                content: "User is learning Rust programming".into(),
                node_type: SemanticType::Fact,
                confidence: 0.8,
                source_episodes: episodes.iter().map(|e| e.id).collect(),
                embedding: None,
            });
        }

        Ok(nodes)
    }

    fn extract_impressions(
        &self,
        interaction: &Interaction,
    ) -> Result<Vec<NewImpression>> {
        // In production: analyze interaction for implicit signals.
        let mut impressions = Vec::new();

        if interaction.text.to_lowercase().contains("prefer")
            || interaction.text.to_lowercase().contains("like")
        {
            impressions.push(NewImpression {
                domain: "user_preferences".into(),
                observation: format!("Expressed preference: {}", interaction.text),
                valence: 0.7,
            });
        }

        Ok(impressions)
    }

    fn detect_contradiction(
        &self,
        a: &SemanticNode,
        b: &SemanticNode,
    ) -> Result<bool> {
        // In production: ask LLM to compare two knowledge statements.
        // Simple heuristic for demo: same topic but different valence.
        Ok(false)
    }
}

fn main() -> alaya::Result<()> {
    let store = AlayaStore::open("custom_provider_memory.db")?;

    // Store episodes
    for content in &[
        "I prefer Rust over C++ for new projects",
        "Rust's ownership model prevents memory bugs",
        "I like that Rust has no garbage collector",
    ] {
        store.store_episode(&NewEpisode {
            content: content.to_string(),
            role: Role::User,
            session_id: "provider-demo".into(),
            timestamp: 1740000000,
            context: EpisodeContext::default(),
            embedding: None,
        })?;
    }

    // Run consolidation with our custom provider
    let provider = MockLLMProvider;
    let report = store.consolidate(&provider)?;
    println!("Consolidation: {} episodes -> {} nodes, {} links",
        report.episodes_processed, report.nodes_created, report.links_created);

    // Check what knowledge was extracted
    let knowledge = store.knowledge(None)?;
    for node in &knowledge {
        println!("Knowledge: [{}] {} (confidence: {:.1})",
            node.node_type.as_str(), node.content, node.confidence);
    }

    // Check preferences
    let prefs = store.preferences(None)?;
    for pref in &prefs {
        println!("Preference: [{}] {} (confidence: {:.1}, evidence: {})",
            pref.domain, pref.preference, pref.confidence, pref.evidence_count);
    }

    Ok(())
}
```

**Rules:**
- The mock provider uses simple keyword matching, not actual LLM calls.
- Comments explain what production code would do differently.
- Shows the full flow: store -> consolidate with custom provider -> inspect knowledge and preferences.
- Error handling uses `?` throughout (models good practice).

### Example: `advanced_retrieval.rs`

**Purpose:** Demonstrate QueryContext, knowledge queries, graph neighbors, and preferences. Level 3 usage.

```rust
//! Advanced retrieval: context-weighted queries, knowledge graphs,
//! and preference inspection.
//!
//! Run with: cargo run --example advanced_retrieval

use alaya::*;

fn main() -> alaya::Result<()> {
    let store = AlayaStore::open("advanced_memory.db")?;

    // Store episodes with rich context
    let ep1 = store.store_episode(&NewEpisode {
        content: "I always order oat milk lattes, never cow milk".into(),
        role: Role::User,
        session_id: "morning-chat".into(),
        timestamp: 1740000000,
        context: EpisodeContext {
            topics: vec!["coffee".into(), "dietary preferences".into()],
            sentiment: 0.6,
            mentioned_entities: vec!["oat milk".into(), "latte".into()],
            ..Default::default()
        },
        embedding: None,
    })?;

    let _ep2 = store.store_episode(&NewEpisode {
        content: "My favorite cafe is Blue Bottle on Valencia Street".into(),
        role: Role::User,
        session_id: "morning-chat".into(),
        timestamp: 1740000060,
        context: EpisodeContext {
            topics: vec!["coffee".into(), "locations".into()],
            mentioned_entities: vec!["Blue Bottle".into(), "Valencia Street".into()],
            preceding_episode: Some(ep1),
            ..Default::default()
        },
        embedding: None,
    })?;

    // Query with full context -- topics and entities improve graph activation
    let results = store.query(&Query {
        text: "coffee preferences".into(),
        embedding: None,
        context: QueryContext {
            topics: vec!["coffee".into()],
            mentioned_entities: vec!["oat milk".into()],
            current_timestamp: Some(1740001000),
            ..Default::default()
        },
        max_results: 5,
    })?;

    println!("Context-weighted query: 'coffee preferences'");
    for mem in &results {
        println!("  [{:.4}] {}", mem.score, mem.content);
    }

    // Explore graph neighbors of the first result
    if let Some(first) = results.first() {
        let neighbors = store.neighbors(first.node, 1)?;
        println!("\nGraph neighbors of top result:");
        for (node, weight) in &neighbors {
            println!("  {:?} (weight: {:.3})", node, weight);
        }
    }

    // Query knowledge store directly
    let facts = store.knowledge(Some(KnowledgeFilter {
        node_type: Some(SemanticType::Fact),
        min_confidence: Some(0.5),
        limit: Some(10),
    }))?;
    println!("\nKnowledge (facts with confidence >= 0.5):");
    for node in &facts {
        println!("  {} (confidence: {:.2})", node.content, node.confidence);
    }

    // Query preferences
    let prefs = store.preferences(Some("coffee"))?;
    println!("\nPreferences (coffee domain):");
    for pref in &prefs {
        println!("  {} (confidence: {:.2}, evidence: {})",
            pref.preference, pref.confidence, pref.evidence_count);
    }

    Ok(())
}
```

**Rules:**
- Shows EpisodeContext with all fields populated (topics, sentiment, entities, preceding_episode).
- Shows QueryContext with all fields populated.
- Demonstrates `neighbors()`, `knowledge()`, and `preferences()` -- the Level 3 API surface.
- Uses `preceding_episode` to create a temporal chain between episodes.

### Example: `production.rs`

**Purpose:** Production deployment patterns. Arc for thread safety, backup, periodic lifecycle, monitoring.

```rust
//! Production deployment patterns for Alaya.
//!
//! Demonstrates: thread-safe sharing, backup, periodic lifecycle,
//! and status monitoring.
//!
//! Run with: cargo run --example production

use alaya::{AlayaStore, NewEpisode, NoOpProvider, Query, Role, EpisodeContext};
use std::sync::Arc;

fn main() -> alaya::Result<()> {
    // Production path: use platform-appropriate data directory
    let db_path = "production_memory.db";
    let store = Arc::new(AlayaStore::open(db_path)?);

    // --- Thread-safe access ---
    // AlayaStore uses SQLite WAL mode, which allows concurrent readers.
    // For single-writer patterns, Arc<AlayaStore> is sufficient.
    // For multi-writer, use Arc<Mutex<AlayaStore>> or connection pooling.

    let store_writer = Arc::clone(&store);
    let store_reader = Arc::clone(&store);

    // Simulate write from one context
    store_writer.store_episode(&NewEpisode {
        content: "Production episode from writer thread".into(),
        role: Role::User,
        session_id: "prod-session-1".into(),
        timestamp: 1740000000,
        context: EpisodeContext::default(),
        embedding: None,
    })?;

    // Simulate read from another context
    let results = store_reader.query(&Query::simple("production"))?;
    println!("Query results: {} found", results.len());

    // --- Backup ---
    // Alaya stores everything in a single SQLite file.
    // Backup is a file copy. For live backups, use SQLite's backup API
    // or copy the file while the WAL is checkpointed.
    println!("\nBackup: cp {} {}.backup", db_path, db_path);

    // --- Periodic lifecycle ("dream cycle") ---
    // Run between conversations or on a timer. Not after every message.
    let provider = NoOpProvider; // Replace with your LLM provider
    let cr = store.consolidate(&provider)?;
    let fr = store.forget()?;
    let tr = store.transform()?;

    println!("\nDream cycle complete:");
    println!("  Consolidated: {} episodes -> {} nodes", cr.episodes_processed, cr.nodes_created);
    println!("  Forgotten:    {} decayed, {} archived", fr.nodes_decayed, fr.nodes_archived);
    println!("  Transformed:  {} merged, {} pruned", tr.duplicates_merged, tr.links_pruned);

    // --- Monitoring ---
    let status = store.status()?;
    println!("\nMemory health:");
    println!("  Episodes:    {}", status.episode_count);
    println!("  Knowledge:   {}", status.semantic_node_count);
    println!("  Preferences: {}", status.preference_count);
    println!("  Impressions: {}", status.impression_count);
    println!("  Graph links: {}", status.link_count);
    println!("  Embeddings:  {}", status.embedding_count);

    // --- Alerting thresholds ---
    if status.episode_count > 10_000 && status.embedding_count == 0 {
        println!("\nWARNING: >10K episodes without embeddings.");
        println!("  BM25-only retrieval degrades at scale.");
        println!("  Consider implementing EmbeddingProvider.");
    }

    Ok(())
}
```

**Rules:**
- Demonstrates `Arc<AlayaStore>` for thread safety (SQLite WAL mode supports concurrent readers).
- Backup is presented as a simple file copy (single-file invariant).
- Dream cycle is shown as a batch operation, not per-message.
- Status check includes a monitoring threshold example.
- No async -- Alaya is sync-first (async via feature flag in v0.2).

---

## 4. Error Output Wireframe

When something goes wrong, the terminal output is the only "UI" the developer sees. Error messages must answer three questions: what happened, why it happened, and what to do next.

### Error Output Format Specification

Every `AlayaError` variant produces output in this structure:

```
Error: AlayaError::<Variant> { <fields> }
  -> <What happened in plain English>
  -> <Why this likely occurred>
  -> <What to do next>
```

### Variant-by-Variant Wireframes

#### AlayaError::Db (Database Error)

**Scenario: File not found**
```
Error: database error: unable to open database file
  -> Could not open database at "/tmp/nonexistent/memory.db"
  -> The parent directory does not exist or is not writable
  -> Create the directory first, or use AlayaStore::open_in_memory() for testing
  -> Docs: https://docs.rs/alaya/latest/alaya/struct.AlayaStore.html#method.open
```

**Scenario: Permission denied**
```
Error: database error: unable to open database file
  -> Could not open database at "/etc/memory.db"
  -> Permission denied. The process does not have write access to this path
  -> Choose a path in a writable directory (e.g., ~/.local/share/myagent/memory.db)
```

**Scenario: Database locked**
```
Error: database error: database is locked
  -> Another process holds a write lock on "memory.db"
  -> SQLite allows one writer at a time. Another instance may be running
  -> If using multiple threads, wrap AlayaStore in Arc<Mutex<AlayaStore>>
  -> Your data is safe. The attempted write was not committed.
```

**Scenario: Corrupt database**
```
Error: database error: database disk image is malformed
  -> The database file "memory.db" is corrupt
  -> Possible causes: incomplete write during power loss, filesystem error, manual file editing
  -> Recovery steps:
     1. Check for a backup: ls memory.db.backup*
     2. Try SQLite recovery: sqlite3 memory.db ".recover" | sqlite3 memory_recovered.db
     3. If no backup exists, the data may be unrecoverable. Start fresh with a new file.
  -> Prevention: enable filesystem journaling, use UPS, never edit the .db file directly
```

#### AlayaError::NotFound (Entity Not Found)

**Current format (v0.1):**
```
Error: not found: episode with id 42
```

**Target format (v0.1 improvement):**
```
Error: not found: Episode with id 42
  -> No episode exists with EpisodeId(42)
  -> The episode may have been deleted by purge() or may never have existed
  -> Use store.status() to check episode count, or verify the ID from store_episode() return value
```

**With structured fields (v0.2 planned):**
```
Error: not found: Episode { id: 42 }
  -> No episode exists with EpisodeId(42)
  -> Available IDs: use store operations that return IDs (store_episode, query)
  -> Docs: https://docs.rs/alaya/latest/alaya/error/enum.AlayaError.html#variant.NotFound
```

#### AlayaError::InvalidInput (Validation Error)

**Scenario: Empty content**
```
Error: invalid input: episode content must not be empty
  -> NewEpisode.content was an empty string
  -> Every episode must contain at least one character of content
  -> Check your input before calling store_episode()
```

**Scenario: Zero-length embedding**
```
Error: invalid input: embedding must have at least 1 dimension
  -> NewEpisode.embedding was Some(vec![]) (empty vector)
  -> Either provide None (skip embeddings) or a non-empty Vec<f32>
  -> For BM25-only retrieval, set embedding: None
```

**Scenario: max_results = 0**
```
Error: invalid input: max_results must be at least 1
  -> Query.max_results was 0
  -> Set max_results to 1 or higher, or use Query::simple() which defaults to 5
```

#### AlayaError::Serialization (JSON Error)

```
Error: serialization error: invalid type: integer `3`, expected a string
  -> Internal JSON deserialization failed on EpisodeContext
  -> This may indicate a schema version mismatch after upgrading Alaya
  -> If you recently upgraded, check the migration guide
  -> Your episodes are safe. The raw data is intact in SQLite.
```

#### AlayaError::Provider (Developer's Code)

```
Error: provider error: HTTP 429 Too Many Requests
  -> Your ConsolidationProvider returned an error during extract_knowledge()
  -> This error came from YOUR provider implementation, not from Alaya
  -> The consolidation batch was skipped. Your data is not affected.
  -> Fix: check your LLM API key, rate limits, and network connection
  -> The skipped episodes will be retried on the next consolidate() call
```

### Error Output Rules

1. **Attribution boundary is always explicit.** "Your ConsolidationProvider" or "YOUR provider implementation" for Provider errors. "Alaya internal error" for unexpected Db/Serialization errors.

2. **Data safety is always stated.** "Your data is not affected," "the attempted write was not committed," "your episodes are safe."

3. **Recovery steps use imperative verbs.** "Create the directory," "check your API key," "use store.status()." Not "you might want to consider..."

4. **doc links point to specific items.** Not the crate root, but the specific error variant or method.

5. **No stack traces in Display output.** The `Debug` representation includes the full chain; `Display` (what the user sees with `{}`) is the actionable message.

6. **FTS5 syntax errors never reach the developer.** Input sanitization prevents them. If one somehow leaks through, the error message says "this is a bug in Alaya, please report it."

### Error Format for Level 1 (Beginner) vs Level 4 (Expert)

Level 1 developers see the `Display` output (3-4 lines, actionable).

Level 4 developers use the `Debug` output (full error chain, SQLite error codes, stack context):

```
// Display (for beginners -- what println!("{}", err) shows):
database error: unable to open database file

// Debug (for experts -- what println!("{:?}", err) shows):
Db(SqliteError { code: 14, message: "unable to open database file" })
    at AlayaStore::open("memory.db")
    path resolved to: /Users/dev/projects/agent/memory.db
    parent directory exists: false
```

---

## 5. MCP Server Interface Wireframe

The MCP (Model Context Protocol) server wraps `AlayaStore` and exposes it as tools callable by Claude, AI agents, or any MCP-compatible client. This is the v0.2 adoption wedge that lowers the barrier from "must write Rust" to "must configure an MCP server."

### MCP Server Configuration

**Installation:**
```bash
cargo install alaya-mcp
```

**Client configuration (Claude Desktop / Claude Code):**
```json
{
  "mcpServers": {
    "alaya": {
      "command": "alaya-mcp",
      "args": ["--db", "~/.alaya/memory.db"]
    }
  }
}
```

**Startup output:**
```
alaya-mcp v0.2.0
  Database: /Users/dev/.alaya/memory.db
  Status:   247 episodes, 18 semantic nodes, 5 preferences
  Provider: NoOp (configure --provider for LLM-powered lifecycle)
  Ready.
```

### MCP Tool Surface

#### Tool: `alaya_store_episode`

**Purpose:** Store a conversation episode.

```yaml
name: alaya_store_episode
description: "Store a conversation episode in Alaya's memory"
inputSchema:
  type: object
  required: [content, role, session_id]
  properties:
    content:
      type: string
      description: "The text content of the episode"
    role:
      type: string
      enum: [user, assistant, system]
      description: "Who produced this content"
    session_id:
      type: string
      description: "Session identifier for grouping episodes"
    topics:
      type: array
      items: { type: string }
      description: "Optional topic tags for improved retrieval"
    entities:
      type: array
      items: { type: string }
      description: "Optional named entities mentioned"
```

**Example invocation:**
```json
{
  "content": "I prefer dark mode and monospace fonts for coding",
  "role": "user",
  "session_id": "chat-2026-02-26"
}
```

**Example response:**
```json
{
  "episode_id": 42,
  "status": "stored",
  "timestamp": 1740000000
}
```

#### Tool: `alaya_query`

**Purpose:** Query memories with hybrid retrieval.

```yaml
name: alaya_query
description: "Query Alaya's memory using hybrid retrieval (BM25 + graph)"
inputSchema:
  type: object
  required: [text]
  properties:
    text:
      type: string
      description: "Natural language query"
    max_results:
      type: integer
      default: 5
      description: "Maximum number of results to return"
    topics:
      type: array
      items: { type: string }
      description: "Optional topic context for improved graph activation"
    entities:
      type: array
      items: { type: string }
      description: "Optional entity context for improved retrieval"
```

**Example invocation:**
```json
{
  "text": "What are the user's coding environment preferences?",
  "max_results": 3
}
```

**Example response:**
```json
{
  "results": [
    {
      "score": 0.823,
      "content": "I prefer dark mode and monospace fonts for coding",
      "role": "user",
      "timestamp": 1740000000,
      "node_type": "episode"
    },
    {
      "score": 0.412,
      "content": "I use Neovim with a custom Lua configuration",
      "role": "user",
      "timestamp": 1739990000,
      "node_type": "episode"
    }
  ],
  "query_time_ms": 1.2
}
```

#### Tool: `alaya_get_episode`

**Purpose:** Retrieve a specific episode by ID.

```yaml
name: alaya_get_episode
description: "Get a specific episode by ID"
inputSchema:
  type: object
  required: [episode_id]
  properties:
    episode_id:
      type: integer
      description: "The episode ID returned from alaya_store_episode"
```

**Example response:**
```json
{
  "id": 42,
  "content": "I prefer dark mode and monospace fonts for coding",
  "role": "user",
  "session_id": "chat-2026-02-26",
  "timestamp": 1740000000,
  "context": {
    "topics": [],
    "sentiment": 0.0,
    "mentioned_entities": []
  }
}
```

#### Tool: `alaya_dream`

**Purpose:** Run the cognitive lifecycle (consolidation + forgetting + transformation).

```yaml
name: alaya_dream
description: "Run Alaya's cognitive lifecycle processes (consolidation, forgetting, transformation). Best run between conversations, not after every message."
inputSchema:
  type: object
  properties: {}
```

**Example response:**
```json
{
  "consolidation": {
    "episodes_processed": 15,
    "nodes_created": 3,
    "links_created": 7
  },
  "forgetting": {
    "nodes_decayed": 42,
    "nodes_archived": 2
  },
  "transformation": {
    "duplicates_merged": 1,
    "links_pruned": 3,
    "preferences_decayed": 0,
    "impressions_pruned": 5
  },
  "note": "Using NoOp provider. Configure --provider for LLM-powered consolidation."
}
```

#### Tool: `alaya_status`

**Purpose:** Get memory system health and statistics.

```yaml
name: alaya_status
description: "Get Alaya memory system status and statistics"
inputSchema:
  type: object
  properties: {}
```

**Example response:**
```json
{
  "episode_count": 247,
  "semantic_node_count": 18,
  "preference_count": 5,
  "impression_count": 42,
  "link_count": 93,
  "embedding_count": 0,
  "database_path": "/Users/dev/.alaya/memory.db",
  "database_size_bytes": 524288,
  "provider": "NoOp"
}
```

#### Tool: `alaya_preferences`

**Purpose:** Get crystallized preferences, optionally filtered by domain.

```yaml
name: alaya_preferences
description: "Get Alaya's crystallized preferences (patterns emerged from accumulated observations)"
inputSchema:
  type: object
  properties:
    domain:
      type: string
      description: "Optional domain filter (e.g., 'coding', 'communication')"
```

**Example response:**
```json
{
  "preferences": [
    {
      "domain": "coding_environment",
      "preference": "Prefers dark mode and monospace fonts",
      "confidence": 0.82,
      "evidence_count": 7,
      "first_observed": 1738000000,
      "last_reinforced": 1740000000
    }
  ]
}
```

### MCP Tool Mapping to AlayaStore API

| MCP Tool | AlayaStore Method | Notes |
|----------|------------------|-------|
| `alaya_store_episode` | `store_episode()` | Timestamp auto-generated if not provided |
| `alaya_query` | `query()` | Topics/entities mapped to QueryContext |
| `alaya_get_episode` | (new: `get_episode()`) | **Gap**: method not yet in public API |
| `alaya_dream` | `consolidate()` + `forget()` + `transform()` | Bundled as single tool |
| `alaya_status` | `status()` | Extended with db path and size |
| `alaya_preferences` | `preferences()` | Direct mapping |

### MCP Interface Rules

1. **Tool names use `alaya_` prefix** to avoid collision with other MCP servers.
2. **Timestamps are Unix seconds (i64)** in all responses, matching the Rust API.
3. **Scores are f64** in all responses, matching `ScoredMemory.score`.
4. **Errors return structured JSON**, not raw error strings:
   ```json
   {
     "error": "not_found",
     "message": "No episode exists with id 999",
     "recovery": "Use alaya_status to check episode count"
   }
   ```
5. **NoOp provider is the default.** Full lifecycle requires configuring `--provider` flag at startup.
6. **No authentication.** MCP server runs locally. Security is filesystem-level (who can access the binary and database file).

---

## 6. Diagnostic Output Wireframe

When developers debug retrieval quality or lifecycle behavior, they need structured diagnostic output. This section specifies what `MemoryStatus`, lifecycle reports, and the planned `QueryExplanation` look like.

### MemoryStatus Output (v0.1)

**API:** `store.status() -> Result<MemoryStatus>`

**Serialized output:**
```json
{
  "episode_count": 247,
  "semantic_node_count": 18,
  "preference_count": 5,
  "impression_count": 42,
  "link_count": 93,
  "embedding_count": 0
}
```

**Pretty-printed for terminal logging:**
```
Alaya Memory Status
  Episodic:    247 episodes
  Semantic:    18 nodes (12 facts, 3 relationships, 2 events, 1 concept)
  Implicit:    42 impressions -> 5 preferences
  Graph:       93 links
  Embeddings:  0 (BM25-only mode)
```

**Rules:**
- `embedding_count: 0` is not an error. It means BM25-only mode (graceful degradation).
- Node counts by type are a v0.2 enhancement (current MemoryStatus does not break down by SemanticType).

### Lifecycle Reports (v0.1)

**ConsolidationReport:**
```json
{
  "episodes_processed": 15,
  "nodes_created": 3,
  "links_created": 7
}
```

**Terminal rendering:**
```
Consolidation complete:
  Episodes processed: 15
  Semantic nodes created: 3
  Graph links created: 7
```

**PerfumingReport:**
```json
{
  "impressions_stored": 4,
  "preferences_crystallized": 1,
  "preferences_reinforced": 2
}
```

**Terminal rendering:**
```
Perfuming complete:
  Impressions stored: 4
  Preferences crystallized: 1 (new)
  Preferences reinforced: 2 (existing, confidence increased)
```

**ForgettingReport:**
```json
{
  "nodes_decayed": 42,
  "nodes_archived": 2
}
```

**Terminal rendering:**
```
Forgetting complete:
  Retrieval strength decayed: 42 nodes
  Archived (below threshold): 2 nodes
```

**TransformationReport:**
```json
{
  "duplicates_merged": 1,
  "links_pruned": 3,
  "preferences_decayed": 0,
  "impressions_pruned": 5
}
```

**Terminal rendering:**
```
Transformation complete:
  Duplicates merged: 1
  Links pruned: 3
  Preferences decayed: 0
  Impressions pruned: 5
```

### QueryExplanation (v0.2 Planned)

**API (planned):** `store.explain_query(q: &Query) -> Result<QueryExplanation>`

**Purpose:** Debug empty or unexpected query results. This is the primary tool for the "empty results" failure mode identified in Journey 3, Flow B.

**Full diagnostic output:**

```
QueryExplanation {
  query: "coffee preferences",

  bm25_stage: {
    input_tokens: ["coffee", "preferences"],
    matches: [
      { node: Episode(7),  score: 0.82, matched_tokens: ["coffee"] },
      { node: Episode(3),  score: 0.45, matched_tokens: ["preferences"] },
    ],
    total_candidates: 2,
    note: "2 of 2 query tokens matched at least one document"
  },

  vector_stage: {
    enabled: true,
    embedding_dimensions: 384,
    matches: [
      { node: Episode(7),  score: 0.91 },
      { node: Episode(12), score: 0.67 },
      { node: Episode(3),  score: 0.58 },
    ],
    total_candidates: 3
  },

  graph_stage: {
    seed_nodes: [Episode(7), Episode(3), Episode(12)],
    activated: [
      { node: Semantic(5), activation: 0.38, hops: 1, path: "Episode(7) -> Semantic(5)" },
      { node: Episode(15), activation: 0.22, hops: 1, path: "Episode(3) -> Episode(15)" },
    ],
    total_activated: 2
  },

  fusion_stage: {
    algorithm: "RRF",
    k: 60,
    fused: [
      { node: Episode(7),  rrf_score: 0.041 },
      { node: Episode(3),  rrf_score: 0.029 },
      { node: Episode(12), rrf_score: 0.021 },
      { node: Semantic(5), rrf_score: 0.016 },
      { node: Episode(15), rrf_score: 0.011 },
    ]
  },

  rerank_stage: {
    factors: {
      topic_jaccard_weight: 0.50,
      entity_jaccard_weight: 0.25,
      sentiment_weight: 0.25,
      recency_half_life_days: 30
    },
    reranked: [
      { node: Episode(7),  final_score: 0.823, adjustments: "+0.12 topic, +0.05 recency" },
      { node: Episode(3),  final_score: 0.412, adjustments: "+0.08 entity" },
      { node: Episode(12), final_score: 0.389, adjustments: "+0.15 recency" },
    ]
  },

  post_retrieval: {
    rif_applied: true,
    strengths_boosted: [Episode(7), Episode(3), Episode(12)],
    co_retrieval_links_strengthened: 3
  },

  final_results: [Episode(7), Episode(3), Episode(12)],
  total_time_us: 1247
}
```

**For the "empty results" case:**

```
QueryExplanation {
  query: "text editor preferences",

  bm25_stage: {
    input_tokens: ["text", "editor", "preferences"],
    matches: [],
    total_candidates: 0,
    note: "WARNING: Zero BM25 matches. No stored episode contains 'text', 'editor', or 'preferences'."
  },

  vector_stage: {
    enabled: false,
    note: "No embedding provided with query. BM25 is the only retrieval source."
  },

  graph_stage: {
    seed_nodes: [],
    note: "No seeds from BM25 or vector. Graph activation skipped."
  },

  diagnosis: [
    "BM25 requires lexical overlap between query tokens and stored content.",
    "Your stored episodes do not contain 'text', 'editor', or 'preferences'.",
    "Try querying with terms that appear in your stored episodes (e.g., 'Vim', 'dark mode').",
    "For semantic matching beyond lexical overlap, provide embeddings via the embedding field.",
  ],

  final_results: [],
  total_time_us: 89
}
```

**QueryExplanation Rules:**

1. **Diagnosis field is mandatory when results are empty.** It must explain why and suggest fixes.
2. **Every stage shows its input and output.** Developer can trace exactly where the pipeline produced or lost candidates.
3. **Token matching is explicit.** BM25 stage shows which query tokens matched which documents.
4. **Performance is included.** `total_time_us` lets Marcus (performance persona) benchmark retrieval.
5. **Stage-skipping is explained.** "No embedding provided" and "graph activation skipped" are not failures; they are the graceful degradation chain working as designed.

### Diagnostic Output Hierarchy

| Level | API | Audience | What It Shows |
|-------|-----|----------|---------------|
| 1 | `status()` | All developers | Aggregate counts across all stores |
| 2 | Lifecycle reports | Intermediate+ | Per-process counts from consolidate/forget/transform/perfume |
| 3 | `explain_query()` (v0.2) | Advanced+ | Full pipeline trace with per-stage scoring |
| 4 | `tracing` integration (v0.2) | Expert / production | Structured logging for every internal operation |

---

## 7. Integration Gaps Analysis

This section identifies concrete gaps between the design documents (Phases 5a-5c) and the current codebase, then specifies the fix for each gap.

### Gap 1: Missing `get_episode()` Public Method

**Source:** MCP Journey (Phase 5a, Journey 4) requires `alaya_get_episode` tool.
**Current state:** `store::episodic::get_episode()` exists as `pub(crate)` but is not exposed on `AlayaStore`.
**Impact:** MCP server cannot retrieve a specific episode by ID. Developers cannot inspect individual episodes.

**Fix specification:**
```rust
// Add to AlayaStore impl block, under Read path:

/// Get a specific episode by its ID.
///
/// Returns `AlayaError::NotFound` if the episode does not exist.
///
/// # Examples
///
/// ```rust
/// # use alaya::{AlayaStore, NewEpisode, Role, EpisodeContext};
/// # fn main() -> alaya::Result<()> {
/// let store = AlayaStore::open_in_memory()?;
/// let id = store.store_episode(&NewEpisode {
///     content: "test episode".into(),
///     role: Role::User,
///     session_id: "s1".into(),
///     timestamp: 1000,
///     context: EpisodeContext::default(),
///     embedding: None,
/// })?;
///
/// let episode = store.get_episode(id)?;
/// assert_eq!(episode.content, "test episode");
/// # Ok(())
/// # }
/// ```
pub fn get_episode(&self, id: EpisodeId) -> Result<Episode> {
    store::episodic::get_episode(&self.conn, id)
}
```

**Priority:** v0.1 (required for CRUD completeness per North Star Phase 1 exit criteria).

### Gap 2: Missing `session_history()` Public Method

**Source:** Journey 2 (Deepening Integration) pattern shows agents wiring memory into conversation handlers. Developers need to retrieve all episodes from a session.
**Current state:** `store::episodic::get_episodes_by_session()` exists as `pub(crate)` but is not exposed on `AlayaStore`.
**Impact:** Developers cannot reconstruct conversation history for a session.

**Fix specification:**
```rust
/// Get all episodes from a session, ordered by timestamp.
///
/// # Examples
///
/// ```rust
/// # use alaya::{AlayaStore, NewEpisode, Role, EpisodeContext};
/// # fn main() -> alaya::Result<()> {
/// let store = AlayaStore::open_in_memory()?;
/// // ... store episodes with session_id "s1" ...
/// let history = store.session_history("s1")?;
/// // Episodes are ordered by timestamp (ascending)
/// # Ok(())
/// # }
/// ```
pub fn session_history(&self, session_id: &str) -> Result<Vec<Episode>> {
    store::episodic::get_episodes_by_session(&self.conn, session_id)
}
```

**Priority:** v0.1 (table-stakes operation for agent integration).

### Gap 3: Missing `delete_episode()` Public Method

**Source:** API Design System (Phase 5b) specifies CRUD symmetry: `store_*/get_*/list_*/delete_*` for every entity.
**Current state:** `purge()` exists for bulk deletion, but no single-episode delete.
**Impact:** CRUD pattern is incomplete. Developers cannot delete individual episodes.

**Fix specification:**
```rust
/// Delete a specific episode and its associated data (FTS5 entry,
/// embedding, graph links, node strength).
///
/// Returns `AlayaError::NotFound` if the episode does not exist.
pub fn delete_episode(&self, id: EpisodeId) -> Result<()> {
    store::episodic::delete_episode(&self.conn, id)
}
```

**Priority:** v0.1 (CRUD completeness).

### Gap 4: Missing `list_episodes()` Public Method

**Source:** API Design System (Phase 5b) CRUD symmetry pattern.
**Current state:** No list operation for episodes beyond `session_history()`.
**Impact:** Developers cannot enumerate all episodes (useful for debugging, export, migration).

**Fix specification:**
```rust
/// List episodes with optional limit and offset for pagination.
///
/// Returns episodes ordered by timestamp (most recent first).
pub fn list_episodes(&self, limit: usize, offset: usize) -> Result<Vec<Episode>> {
    store::episodic::list_episodes(&self.conn, limit, offset)
}
```

**Priority:** v0.1 (CRUD completeness, also needed for export/backup workflows).

### Gap 5: `#[non_exhaustive]` Not Applied to Public Enums

**Source:** Extract (Phase 4) specifies "#[non_exhaustive] on all public enums." API Design System (Phase 5b) lists this as a consistency rule.
**Current state:** `Role`, `SemanticType`, `LinkType`, `PurgeFilter`, `NodeRef`, `AlayaError` are all missing `#[non_exhaustive]`.
**Impact:** Adding variants in a minor version bump would be a breaking change.

**Fix specification:**
Add `#[non_exhaustive]` to all public enums:
- `Role`
- `SemanticType`
- `LinkType`
- `PurgeFilter`
- `NodeRef`
- `AlayaError`

**Priority:** v0.1 (must be applied before first crates.io publish, or adding variants later is a semver break).

### Gap 6: No Input Validation at API Boundary

**Source:** Accessibility (Phase 5c) specifies validation for empty content, zero-length embeddings, negative timestamps, empty session_id, max_results = 0. Extract (Phase 4) "always" list includes "validate content at API boundary."
**Current state:** `store_episode()` accepts empty content strings. No validation on embeddings, timestamps, or session_id.
**Impact:** Invalid data enters SQLite silently. FTS5 indexes empty strings. Developers discover problems late.

**Fix specification:**
Add validation at the top of `store_episode()`:
```rust
pub fn store_episode(&self, episode: &NewEpisode) -> Result<EpisodeId> {
    if episode.content.is_empty() {
        return Err(AlayaError::InvalidInput(
            "episode content must not be empty".into()
        ));
    }
    if episode.session_id.is_empty() {
        return Err(AlayaError::InvalidInput(
            "session_id must not be empty".into()
        ));
    }
    if let Some(ref emb) = episode.embedding {
        if emb.is_empty() {
            return Err(AlayaError::InvalidInput(
                "embedding must have at least 1 dimension, or use None".into()
            ));
        }
    }
    // ... existing implementation
}
```

Add validation at the top of `query()`:
```rust
pub fn query(&self, q: &Query) -> Result<Vec<ScoredMemory>> {
    if q.max_results == 0 {
        return Err(AlayaError::InvalidInput(
            "max_results must be at least 1".into()
        ));
    }
    // ... existing implementation
}
```

**Priority:** v0.1 (must be in place before crates.io publish).

### Gap 7: No Compilable Doctests on Public Methods

**Source:** Extract (Phase 4) "always" list: "compilable doctests on every public method." Phase 5b doc patterns specify first line imperative mood, examples section.
**Current state:** `AlayaStore` methods have one-line doc comments but no `# Examples` sections with compilable code.
**Impact:** docs.rs pages show methods without usage examples. Doctest coverage target (100%) is not met.

**Fix specification:** Add compilable doctests to all 12 public methods on `AlayaStore` plus `Query::simple()`. The examples in Section 3 of this document provide the code patterns; each method's doctest should be a minimal version of the relevant example.

**Priority:** v0.1 (exit criteria: "every public method has compilable doctest").

### Gap 8: QueryExplanation Type Not Yet Defined

**Source:** Accessibility (Phase 5c) diagnostic levels, Error Recovery (Phase 5a, Journey 3, Flow B).
**Current state:** No `QueryExplanation` type exists. The `explain_query()` method is not implemented.
**Impact:** Developers cannot debug empty or poor query results. This is the highest-trust-risk failure mode.

**Fix specification:** Design specified in Section 6 of this document. Implementation deferred to v0.2 per the accessibility diagnostic levels table.

**Priority:** v0.2 (complex internal change; BM25 behavior documentation and lexical-overlap quickstart mitigate in v0.1).

### Gap 9: Missing Derive Macros on Input Types

**Source:** API Design System (Phase 5b) consistency rules: "output types derive Serialize, Deserialize."
**Current state:** `NewEpisode`, `NewSemanticNode`, `NewImpression`, `Interaction`, `Query`, `QueryContext` do not derive `Serialize, Deserialize`.
**Impact:** MCP server cannot deserialize these types from JSON. Developers cannot log or serialize input types.

**Fix specification:** Add `#[derive(Serialize, Deserialize)]` to all input types:
- `NewEpisode`
- `NewSemanticNode`
- `NewImpression`
- `Interaction`
- `Query`
- `QueryContext`

**Priority:** v0.1 for `NewEpisode`, `Query`, `QueryContext` (needed for logging and future MCP). v0.2 for the rest.

### Gap 10: FTS5 Input Not Sanitized

**Source:** Extract (Phase 4) "always" list: "sanitize all FTS5 MATCH input." Journey 3 error taxonomy lists FTS5 syntax error.
**Current state:** `bm25::search_bm25()` passes query text directly to FTS5 MATCH. Special characters (`*`, `"`, `(`, `)`, `NEAR`, `OR`, `AND`, `NOT`, `:`) can cause syntax errors.
**Impact:** Certain query strings cause `AlayaError::Db` with a confusing FTS5 syntax error message.

**Fix specification:** Add sanitization in `bm25::search_bm25()`:
```rust
fn sanitize_fts5(input: &str) -> String {
    // Wrap each token in double quotes to escape special characters
    input.split_whitespace()
        .map(|token| {
            let escaped = token.replace('"', "\"\"");
            format!("\"{}\"", escaped)
        })
        .collect::<Vec<_>>()
        .join(" ")
}
```

**Priority:** v0.1 (security and DX requirement).

### Gap Summary Table

| # | Gap | Phase Source | Priority | Effort |
|---|-----|-------------|----------|--------|
| 1 | `get_episode()` not public | 5a (MCP Journey) | v0.1 | Small |
| 2 | `session_history()` not public | 5a (Deepening Journey) | v0.1 | Small |
| 3 | `delete_episode()` missing | 5b (CRUD symmetry) | v0.1 | Medium |
| 4 | `list_episodes()` missing | 5b (CRUD symmetry) | v0.1 | Small |
| 5 | `#[non_exhaustive]` missing | 4 (Extract), 5b | v0.1 | Small |
| 6 | No input validation | 5c (Accessibility), 4 | v0.1 | Medium |
| 7 | No compilable doctests | 4 (Extract), 5b | v0.1 | Medium |
| 8 | QueryExplanation not implemented | 5c (Diagnostics) | v0.2 | Large |
| 9 | Input types missing Serialize | 5b (Consistency) | v0.1/v0.2 | Small |
| 10 | FTS5 input not sanitized | 4 (Extract), 5a | v0.1 | Small |

### Gap Resolution Priority

**Must-fix before `cargo publish` (v0.1):** Gaps 1, 2, 3, 4, 5, 6, 7, 9 (partial), 10.

These gaps represent the delta between the designed API surface and the current codebase. Closing them is the primary work of the v0.1 MVP sprint.

**v0.2 deferred:** Gaps 8, 9 (remaining types).

---

## Appendix A: Surface-to-Journey Mapping

| Surface | Journey 1 (First-Time) | Journey 2 (Deepening) | Journey 3 (Error) | Journey 4 (MCP) | Journey 5 (Personas) |
|---------|:-----:|:-----:|:-----:|:-----:|:-----:|
| README | Primary | Reference | Recovery links | Discovery | Priya: privacy section; Marcus: benchmark section |
| docs.rs | Browse after quickstart | Primary reference | Error module | Tool schema reference | Module doc quality |
| examples/ | `basic.rs` | `lifecycle.rs`, `custom_provider.rs` | Error handling patterns | N/A | `production.rs` for both |
| Error output | First compilation error | Provider errors | Primary surface | MCP error responses | Attribution clarity |
| MCP interface | N/A | N/A | MCP error format | Primary surface | N/A |
| Diagnostics | `status()` | Lifecycle reports | `explain_query()` | `alaya_status` tool | Marcus: performance metrics |

## Appendix B: Skill Level to Surface Mapping

| Skill Level | Primary Surfaces | Methods Known | Time Investment |
|-------------|-----------------|---------------|-----------------|
| Level 1 (Beginner) | README, `basic.rs` | 3: `open`, `store_episode`, `query` | < 2 minutes |
| Level 2 (Intermediate) | `lifecycle.rs`, docs.rs modules | 12: +lifecycle +status +preferences +knowledge +purge | < 30 minutes |
| Level 3 (Advanced) | `custom_provider.rs`, `advanced_retrieval.rs`, provider module docs | 15+: +ConsolidationProvider trait +neighbors | < 2 hours |
| Level 4 (Expert) | `production.rs`, architecture docs, QueryExplanation, research citations | 17+: +explain_query +NodeStrength +tuning | 2+ hours |

---

*Generated: 2026-02-26 | Phase: 5d | Cross-references: Developer Journeys (Phase 5a), API Design System (Phase 5b), Accessibility (Phase 5c), Brand Guidelines (Phase 1), North Star (Phase 2), North Star Extract (Phase 4), Competitive Landscape (Phase 3)*
