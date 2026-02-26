# Alaya Developer Accessibility Strategy

Alaya is a Rust crate. There is no web UI, no screen readers, no ARIA attributes, no WCAG compliance. For a library, "accessibility" means: can the widest possible range of developers -- across skill levels, platforms, languages, and cognitive styles -- discover, understand, adopt, and productively use the API?

This document defines Alaya's developer accessibility strategy across seven dimensions: skill level accessibility, error message accessibility, documentation accessibility, platform accessibility, cognitive accessibility, diagnostic accessibility, and onboarding accessibility. Every section is grounded in the actual `AlayaStore` API and cross-referenced against the developer journeys (Phase 5a) and API design system (Phase 5b).

**Cross-references:** [Developer Journeys](USER_JOURNEYS.md) | [API Design System](UI_DESIGN_SYSTEM.md) | [Brand Guidelines](../BRAND_GUIDELINES.md) | [North Star](../NORTHSTAR.md) | [North Star Extract](../NORTHSTAR_EXTRACT.md)

---

## Table of Contents

1. [Developer Skill Level Accessibility](#1-developer-skill-level-accessibility)
2. [Error Message Accessibility](#2-error-message-accessibility)
3. [Documentation Accessibility Strategy](#3-documentation-accessibility-strategy)
4. [Platform Accessibility](#4-platform-accessibility)
5. [Cognitive Accessibility](#5-cognitive-accessibility)
6. [Diagnostic Accessibility](#6-diagnostic-accessibility)
7. [Onboarding Accessibility](#7-onboarding-accessibility)
8. [Testing and Validation](#8-testing-and-validation)
9. [Accessibility Metrics](#9-accessibility-metrics)
10. [Anti-Patterns](#10-anti-patterns)

---

## 1. Developer Skill Level Accessibility

Alaya's API must serve developers at four distinct skill levels without forcing any level to learn concepts intended for a different level. The key mechanism is **progressive API surface disclosure**: a beginner interacts with three methods, an expert with twenty. The same `AlayaStore` struct serves both.

### Skill Level Map

| Level | Who They Are | What They Need | API Surface | Documentation Entry Point |
|-------|-------------|---------------|-------------|---------------------------|
| **Beginner Rust** | New to Rust, building first agent. Copies examples, modifies parameters. | Copy-paste quickstart that compiles on first try. Clear error messages. Minimal imports. | `AlayaStore::open()`, `store_episode()`, `query()` (3 methods) | README quickstart, crate-level doc comment |
| **Intermediate** | Comfortable with Rust traits, error handling, lifetimes. Building a real agent. | Lifecycle integration, session management, context enrichment. | + `consolidate()`, `forget()`, `transform()`, `perfume()`, `status()`, `preferences()`, `knowledge()`, `purge()` (12 methods total) | examples/ directory, docs.rs module docs |
| **Advanced** | Has shipped Rust libraries. Understands trait objects, SQLite internals, embedding models. | Custom provider implementations, embedding integration, FFI wrapping. | + `ConsolidationProvider` trait (3 methods), `EmbeddingProvider` trait, `AlayaConfig::builder()` | Architecture docs, trait documentation, provider examples |
| **Expert** | Research background or systems engineering. Tuning retrieval, benchmarking, contributing to Alaya. | Graph weight tuning, spreading activation parameters, benchmark harnesses, diagnostic types. | + `neighbors()`, `NodeStrength`, `MemoryStatus` diagnostics, `QueryExplanation` (future) | Research citations, benchmark methodology, ARCHITECTURE.md |

### Progressive Disclosure Rules

1. **The README shows only Level 1.** Three methods, five imports, one code block. No mention of consolidation, forgetting, or providers until the developer scrolls past the quickstart.

2. **docs.rs organizes by level.** The crate-level documentation presents methods in order of complexity: write path first, read path second, lifecycle third, admin fourth. Each section links forward to the next level but does not require it.

3. **Examples directory stages complexity.**
   - `examples/quickstart.rs` -- Level 1: open, store, query (10 lines of meaningful code)
   - `examples/lifecycle.rs` -- Level 2: dream cycle with `NoOpProvider` (20 lines)
   - `examples/custom_provider.rs` -- Level 3: implement `ConsolidationProvider` (40 lines)
   - `examples/benchmark.rs` -- Level 4: load N episodes, measure query latency (30 lines)

4. **Import lists grow with skill level.**
   - Level 1: `use alaya::{AlayaStore, NewEpisode, Role, EpisodeContext, Query};`
   - Level 2: adds `ConsolidationReport, ForgettingReport, TransformationReport, MemoryStatus, PurgeFilter`
   - Level 3: adds `ConsolidationProvider, NoOpProvider, NewSemanticNode, NewImpression, Interaction`
   - Level 4: adds `NodeRef, NodeStrength, KnowledgeFilter, LinkType, SemanticType`

5. **Error messages target Level 1 comprehension.** Every error message assumes the developer may not know what FTS5 is, what WAL mode means, or what a Hebbian link does. The message uses plain language and provides a concrete next step.

### What Each Level Must Never Experience

| Level | Must Never Encounter | Why | Mitigation |
|-------|---------------------|-----|------------|
| Beginner | Lifetime errors from the API | Kills first impression; "Rust is hard" reinforced | All public methods take `&self` and borrow arguments; no lifetime annotations in public signatures |
| Beginner | Empty results on first query | #1 abandonment moment (from Journey 3, Flow B) | Quickstart example uses queries with lexical overlap to BM25; documentation sets expectations |
| Intermediate | Silent data loss from lifecycle processes | Erodes trust in the cognitive lifecycle | Every lifecycle method returns a typed report; no void mutations |
| Advanced | Opaque provider errors | Cannot debug their own trait implementation | `AlayaError::Provider(msg)` includes lifecycle phase context; stack traces point to provider code |
| Expert | Undocumented algorithm parameters | Cannot tune what they cannot see | All constants (`DECAY_FACTOR`, `RRF_K`, `ACTIVATION_THRESHOLD`) are documented with their research basis |

---

## 2. Error Message Accessibility

Every error message in Alaya must answer three questions:

1. **What happened?** -- the error condition in plain language
2. **Why?** -- the likely cause
3. **What to do?** -- the concrete next step

This is the error accessibility contract. No `AlayaError` variant is permitted to violate it.

### Current Error Variant Mapping

The current `AlayaError` enum has five variants. Each is mapped below to its accessible error message strategy.

#### `AlayaError::Db(rusqlite::Error)`

**Current display:** `"database error: {inner}"`

**Accessible strategy:** The raw SQLite error is insufficient for developers who do not know SQLite internals. Alaya should contextualize the error at the call site.

| SQLite Scenario | What the Developer Sees | What to Do |
|----------------|------------------------|------------|
| File not found / permission denied | "Cannot open database at '{path}': {os_error}. Check that the directory exists, the path is writable, and no other process holds a lock on the file." | Verify file path and permissions |
| Database locked (SQLITE_BUSY) | "Database is locked. Another process or thread is writing to '{path}'. Alaya uses WAL mode for concurrent reads, but only one writer is allowed at a time. If using multiple threads, wrap AlayaStore in Arc<Mutex<AlayaStore>>." | Serialize write access |
| Disk full | "Write failed: disk is full. The SQLite file at '{path}' cannot grow. Free disk space or move the database to a larger volume." | Free disk space |
| Corrupt database | "Database integrity check failed for '{path}'. This may indicate filesystem corruption. Recovery options: (1) restore from backup, (2) run `sqlite3 {path} '.recover'` to salvage data, (3) delete and recreate." | Restore from backup |
| FTS5 syntax error | Should never reach the developer -- all FTS5 input is sanitized (per Extract 4.2). If it does, the message says: "Internal error: FTS5 query sanitization failed. This is a bug in Alaya. Please report it at https://github.com/h4x0r/alaya/issues with the query text." | Report bug |
| Schema version mismatch | "Database schema version {found} does not match expected version {expected}. This usually means the database was created by a different version of Alaya. Options: (1) run `AlayaStore::migrate(path)?` to upgrade, (2) back up the file and recreate." | Migrate or recreate |

**Implementation note:** Error contextualization happens at the `AlayaStore` method level, not in the `From<rusqlite::Error>` impl. Each `AlayaStore` method that calls SQLite wraps the error with path and operation context before propagating.

#### `AlayaError::NotFound(String)`

**Current display:** `"not found: {0}"`

**Accessible strategy:**

```
Not found: episode with id 42.
This ID may have been deleted by purge() or the episode may not exist.
Use store.status() to check current counts, or query() to find episodes by content.
```

The message includes:
- The entity type and ID
- Why it might be missing (purge, wrong ID, never existed)
- Alternative approaches (status check, content-based query)

#### `AlayaError::InvalidInput(String)`

**Current display:** `"invalid input: {0}"`

**Accessible strategy:** Invalid input errors occur at the API boundary. Each validation point produces a specific message:

| Validation | Error Message |
|-----------|---------------|
| Empty episode content | "Episode content cannot be empty. Provide at least one character of text. If you need to store metadata-only episodes, put the metadata in the content field." |
| Zero-length embedding | "Embedding vector cannot be empty. Provide a non-empty Vec<f32> or set embedding to None for BM25-only retrieval." |
| Negative timestamp | "Timestamp {value} is negative. Alaya uses Unix timestamps (seconds since 1970-01-01). Use `chrono::Utc::now().timestamp()` or `std::time::SystemTime::now().duration_since(UNIX_EPOCH).as_secs() as i64`." |
| Empty session_id | "Session ID cannot be empty. Provide any non-empty string to group episodes by conversation. Example: a UUID, user ID, or descriptive label like 'onboarding-chat'." |
| max_results = 0 | "max_results cannot be 0. Set to at least 1, or use Query::simple() which defaults to 5." |

#### `AlayaError::Serialization(serde_json::Error)`

**Current display:** `"serialization error: {0}"`

**Accessible strategy:**

```
Failed to serialize/deserialize EpisodeContext: {serde_error}.
This usually means the context_json column in the database contains
malformed JSON. If this is a database created by an older version of Alaya,
run migrate() to fix the schema.
```

This error should be rare in normal usage because all JSON serialization is internal to Alaya. If the developer sees it, either the database was manually edited or there is a version mismatch.

#### `AlayaError::Provider(String)`

**Current display:** `"provider error: {0}"`

**Accessible strategy:** Provider errors originate from the developer's own `ConsolidationProvider` or `EmbeddingProvider` implementation. The boundary must be clear:

```
Provider error during consolidation.extract_knowledge(): {inner_error}

This error came from YOUR ConsolidationProvider implementation, not from Alaya.
The consolidation batch of {N} episodes was not processed.
Your data is safe -- episodes remain stored and will be included in the next
consolidation cycle.

To debug: check your extract_knowledge() implementation for the error above.
For a working example, see: https://docs.rs/alaya/latest/alaya/trait.ConsolidationProvider.html
```

Key elements:
- **Boundary attribution:** "This error came from YOUR ... implementation, not from Alaya."
- **Data safety assurance:** "Your data is safe."
- **Recovery path:** Episodes accumulate; next consolidation picks them up.
- **Debug pointer:** Link to trait documentation with example.

### Error Design Principles

These principles govern all error messages across the crate.

1. **Answer "what do I do?" before "what went wrong?"** Most developers read error messages to fix the problem, not to understand the internals. Lead with the fix when possible.

2. **Attribute the boundary.** When an error crosses the Alaya/provider boundary, say which side it came from. The developer should never wonder "is this my bug or Alaya's bug?"

3. **Communicate data safety.** After any error that might worry the developer about data loss, explicitly state whether data was affected. "Your data is safe" or "This operation may have partially completed" -- never silence.

4. **Prefer compilation errors over runtime errors.** The Rust type system is the first line of defense. Newtype IDs prevent wrong-ID-type bugs at compile time. `#[non_exhaustive]` prevents incomplete match arms. Strong typing on `Role`, `SemanticType`, `LinkType` prevents string-based errors.

5. **Never expose raw SQLite errors to Level 1 developers.** Wrap every `rusqlite::Error` with context about what operation was attempted and what the developer should do. The raw error can be in a `.source()` chain for Level 4 developers who want it, but the `.to_string()` representation must be self-contained.

6. **Silent failures are worse than loud errors.** `query()` returning an empty `Vec` is not an error, but it feels like one to a developer who just stored data. The diagnostic path (Section 6) addresses this, but the principle is: when something surprising happens, provide a way to understand why.

### Planned Error Improvements

| Improvement | Target Version | Description |
|------------|---------------|-------------|
| Contextual `Db` wrapping | v0.1 | Each `AlayaStore` method wraps SQLite errors with operation context and path |
| `SchemaVersion` variant | v0.1 | Dedicated error variant for version mismatch with migration instructions |
| `NotFound` structured fields | v0.1 | Replace `NotFound(String)` with `NotFound { entity: &'static str, id: i64 }` for programmatic handling |
| Provider phase context | v0.1 | `Provider` variant includes which lifecycle method triggered the error |
| `Capacity` variant | v0.2 | For disk-full and SQLite limit scenarios with specific recovery guidance |

---

## 3. Documentation Accessibility Strategy

Documentation accessibility means information is findable, understandable, and actionable for developers across skill levels and learning styles.

### Progressive Disclosure Layers

Documentation is organized into five layers, each deeper than the last. A developer should be able to stop at any layer and have a complete (if partial) understanding of Alaya.

```
Layer 1: README.md (30 seconds)
  "What is this? Does it solve my problem? How do I install it?"
    |
    v
Layer 2: Quickstart example (2 minutes)
  "Can I get it working? Does the API make sense?"
    |
    v
Layer 3: docs.rs / examples/ (10-30 minutes)
  "How do I integrate it into my project? What are the lifecycle methods?"
    |
    v
Layer 4: Architecture docs (1-2 hours)
  "How does the retrieval pipeline work? What is the Hebbian graph doing?"
    |
    v
Layer 5: Research citations (hours-days)
  "What is the Bjork dual-strength model? How does CLS consolidation work?"
```

### Layer Design

#### Layer 1: README.md

**Target:** Any developer, any skill level, 30 seconds.

**Structure:**
1. One-line description (crate-level metadata)
2. Three-sentence positioning statement (what, for whom, how it is different)
3. Quickstart code block (copy-paste, compiles, produces output)
4. Feature list (bullet points, not paragraphs)
5. Privacy guarantee (one sentence: "Zero network calls. Single SQLite file. No telemetry.")
6. Links to deeper documentation

**Rules:**
- Code appears above the fold (first screenful)
- No Yogacara/neuroscience terminology in the README (save for docs.rs)
- No feature flag discussion until after the quickstart
- Every claim is verifiable (dependency count, network calls, file count)

#### Layer 2: Quickstart Example

**Target:** Developer who just ran `cargo add alaya`.

**The quickstart must satisfy these constraints:**
- Compiles with `cargo run` and no additional flags
- Produces visible output (not just `()`)
- Uses queries with lexical overlap to stored content (avoids BM25 empty-result trap)
- Does not require an LLM API key, network access, or configuration file
- Fits in a single file under 30 lines of meaningful code
- Uses only five imports: `AlayaStore`, `NewEpisode`, `Role`, `EpisodeContext`, `Query`

**The quickstart is tested in CI.** Every release verifies that the quickstart code in README.md compiles and produces non-empty output. If the API changes and the quickstart breaks, the release is blocked.

#### Layer 3: docs.rs Module Documentation

**Target:** Developer integrating Alaya into their project.

**Structure per module:**

Each public type and method has:
- A one-sentence summary line
- A "Details" section explaining behavior and edge cases
- A compilable doctest example
- A "See also" section linking to related types and the lifecycle stage the type belongs to
- Performance characteristics (where relevant): "O(n) where n is episode count" or "< 1ms at 1K episodes"

**Cross-referencing rules:**
- Every type links to the lifecycle stage it participates in
- `NewEpisode` links to `Episode`, `AlayaStore::store_episode`, and `Query`
- `ConsolidationReport` links to `AlayaStore::consolidate` and `ConsolidationProvider`
- Lifecycle types link to the research citation for the underlying mechanism

**Keyword optimization:**
- Module docs include common search terms: "store", "query", "memory", "retrieve", "forget", "consolidate"
- Type docs include the plain-English equivalent: "ScoredMemory" docs mention "search result", "retrieval result", "ranked memory"

#### Layer 4: Architecture Documentation

**Target:** Advanced developer or contributor.

**Contents:**
- Retrieval pipeline diagram: BM25 -> vector -> graph activation -> RRF fusion -> reranking
- Schema design: tables, indices, FTS5 virtual table, triggers
- Lifecycle flow: ingest -> consolidate -> perfume -> transform -> forget
- Design decisions with rationale: why SQLite WAL mode, why `BEGIN IMMEDIATE`, why brute-force cosine before `sqlite-vec`

#### Layer 5: Research Citations

**Target:** Expert developer or academic.

**Every non-obvious mechanism cites its source:**
- Bjork dual-strength model: Bjork & Bjork (1992), "A new theory of disuse and an old theory of stimulus fluctuation"
- CLS consolidation: McClelland, McNaughton & O'Reilly (1995), "Why there are complementary learning systems in the hippocampus and neocortex"
- Hebbian LTP/LTD: Hebb (1949), "The Organization of Behavior"; Bi & Poo (2001) for spike-timing-dependent plasticity
- Vasana/perfuming: Yogacara tradition, Vasubandhu's Trimsika; Waldron (2003), "The Buddhist Unconscious"
- RRF fusion: Cormack, Clarke & Butt (2009), "Reciprocal rank fusion outperforms condorcet and individual rank learning methods"

### Terminology Glossary

Alaya uses specialized terms from neuroscience and Yogacara Buddhist psychology. Every specialized term has a plain-English equivalent documented in the glossary. The glossary appears in docs.rs at the crate level and is linked from every doc comment that uses a specialized term.

| Alaya Term | Plain-English Equivalent | Source Domain | Where It Appears |
|-----------|-------------------------|--------------|-----------------|
| Alaya-vijnana | Storehouse consciousness (the memory system as a whole) | Yogacara | Crate name, overview docs |
| Episode | A single conversation message with metadata | Cognitive science | `NewEpisode`, `Episode`, `store_episode()` |
| Consolidation | Compressing raw messages into structured knowledge | Neuroscience (CLS) | `consolidate()`, `ConsolidationReport` |
| Semantic node | A fact, relationship, event, or concept extracted from episodes | Neuroscience (neocortex) | `SemanticNode`, `NewSemanticNode` |
| Perfuming / Vasana | Implicit preference emergence from accumulated observations | Yogacara | `perfume()`, `PerfumingReport`, `Impression`, `Preference` |
| Impression | A single behavioral observation (raw data for preference extraction) | Yogacara (bija) | `Impression`, `NewImpression` |
| Preference | A crystallized pattern from accumulated impressions | Yogacara (vasana) | `Preference`, `preferences()` |
| Forgetting (Bjork) | Decay of retrieval accessibility while preserving storage strength | Bjork & Bjork (1992) | `forget()`, `ForgettingReport`, `NodeStrength` |
| Storage strength | How well-learned a memory is (increases monotonically) | Bjork & Bjork (1992) | `NodeStrength.storage_strength` |
| Retrieval strength | How accessible a memory is right now (decays without use) | Bjork & Bjork (1992) | `NodeStrength.retrieval_strength` |
| Hebbian link | A connection between memories that strengthens with co-activation | Hebb (1949) | `Link`, `LinkType`, `neighbors()` |
| LTP / LTD | Long-term potentiation / depression of graph links | Neuroscience | `graph::links`, link weight updates |
| Spreading activation | Graph traversal that activates neighbors of seed nodes | Collins & Loftus (1975) | `neighbors()`, retrieval pipeline |
| Transformation (Asraya-paravrtti) | Structural change of the memory system (dedup, prune) | Yogacara | `transform()`, `TransformationReport` |
| RRF | Reciprocal Rank Fusion for merging ranked result lists | Information retrieval | Retrieval pipeline (internal) |
| Dream cycle | Developer pattern: running consolidate + forget + transform between conversations | Alaya convention | Documentation, examples |

### Multiple Learning Styles

Different developers learn differently. Alaya's documentation addresses four learning styles.

| Learning Style | Format | Where Provided |
|---------------|--------|---------------|
| **Read-write** (textual) | Doc comments, README prose, architecture docs | docs.rs, README.md, ARCHITECTURE.md |
| **Example-first** (code) | Compilable doctests, examples/ directory, quickstart | Every pub method, examples/ directory |
| **Visual-spatial** (diagrams) | Pipeline diagrams, lifecycle flow, type hierarchy tree | Architecture docs, crate-level docs |
| **Reference** (lookup) | Glossary, type tables, method signature tables | Glossary section, API Design System |

---

## 4. Platform Accessibility

Platform accessibility means developers can use Alaya from languages and environments beyond Rust. Alaya's value should not be locked behind "you must write Rust." The strategy is tiered by version, with each tier reaching a larger developer audience.

### Platform Tier Map

| Tier | Access Method | Target Version | Developer Audience | Integration Effort |
|------|--------------|---------------|-------------------|-------------------|
| **Tier 1** | Rust native: `cargo add alaya` | v0.1 (now) | Rust developers | Minutes |
| **Tier 2a** | MCP server: `alaya-mcp` binary | v0.2 | Any MCP client (Claude, agents, IDEs) | Configuration only |
| **Tier 2b** | C/C++ via FFI: `libalaya.h` | v0.2 | C/C++ developers, any language with C FFI | Hours (FFI binding) |
| **Tier 3** | Python via PyO3: `pip install alaya` | v0.3 | Python developers | Minutes |
| **Tier 4** | Mobile/Edge: ARM cross-compilation | v0.3+ | Mobile and IoT developers | Hours (cross-compile setup) |

### Tier 1: Rust Native (v0.1)

**Access:**
```bash
cargo add alaya
```

**What works:** Full API surface, all lifecycle methods, all types, compile-time safety, zero-cost abstractions.

**Platform constraints:** Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x86_64). All platforms where `rusqlite` with `bundled` SQLite compiles. CI tests all three.

**MSRV (Minimum Supported Rust Version):** Documented in `Cargo.toml` and README. Tracked in CI.

### Tier 2a: MCP Server (v0.2)

**Access:**
```bash
cargo install alaya-mcp
# or download pre-built binary from GitHub releases
```

**What works:** Store, query, status, dream cycle. All operations available as MCP tools. JSON-serialized inputs and outputs.

**What it enables:** Any MCP-compatible client can use Alaya's memory without writing Rust. Claude Desktop, Claude Code, Python agents using an MCP client library, custom agent frameworks with MCP support.

**Platform reach expansion:** MCP turns Alaya from a Rust library into a language-agnostic memory service running locally. The developer does not need to know Rust -- they configure a binary and invoke tools.

**Limitations (honest):** MCP server uses `NoOpProvider` by default. Full consolidation requires either a configured LLM provider in the server or Rust-level integration. Documented clearly.

### Tier 2b: C/C++ via FFI (v0.2)

**Access:**
```c
#include "alaya.h"

AlayaStore* store = alaya_open("memory.db");
int64_t id = alaya_store_episode(store, "I prefer dark mode", "user", "session-1", time(NULL));
AlayaResults* results = alaya_query(store, "preferences");
alaya_free_results(results);
alaya_close(store);
```

**What works:** Core CRUD operations, query, status, lifecycle. C-compatible function signatures generated by `cbindgen`.

**What it enables:** Any language with C FFI can use Alaya: Swift, Kotlin, Go, Ruby, PHP, Zig, Nim, Lua, Julia. The C header is the universal adapter.

**Design rules for FFI accessibility:**
- All FFI functions return error codes, never panic across the FFI boundary
- Opaque pointer types (`AlayaStore*`, `AlayaResults*`) with explicit free functions
- String arguments are `const char*` (null-terminated UTF-8)
- Results are accessed via iteration functions, not direct struct access
- Memory ownership is always explicit: Alaya-allocated memory is freed by Alaya functions

### Tier 3: Python via PyO3 (v0.3)

**Access:**
```bash
pip install alaya
```

```python
from alaya import AlayaStore, Query

store = AlayaStore.open("memory.db")
store.store_episode("I prefer dark mode", role="user", session_id="s1")
results = store.query("preferences")
for r in results:
    print(f"[{r.score:.2f}] {r.content}")
```

**What works:** Full API with Pythonic wrappers. Snake_case methods, keyword arguments, context managers, exception-based error handling.

**What it enables:** Python is the dominant language for AI/ML development. PyO3 bindings remove the "I do not write Rust" barrier entirely. Developers can evaluate, prototype, and ship with Python while getting Rust-speed retrieval.

**Design rules for Python accessibility:**
- Pythonic naming and conventions (snake_case, keyword args, `with` statement support)
- Exceptions map to `AlayaError` variants (not raw Rust panics)
- Return Python-native types (`list`, `dict`) where possible, not opaque wrappers
- Type hints for all public functions (`.pyi` stub file)
- Docstrings that follow Python conventions, not Rust conventions

### Tier 4: Mobile/Edge (v0.3+)

**Access:** ARM cross-compilation for iOS (via staticlib + Swift bridge), Android (via JNI + Kotlin bridge), embedded Linux (aarch64-unknown-linux-musl).

**Feasibility assessment:**
- SQLite runs everywhere, including mobile and embedded
- Alaya's zero-network-dependency architecture is ideal for edge deployment
- `no_std` feasibility: Alaya depends on `rusqlite`, `serde`, `serde_json`, `thiserror` -- all require `std`. True `no_std` would require replacing these. Assessment: not feasible for v0.x. Instead, target `musl` for static linking.
- Binary size: with `rusqlite[bundled]`, the compiled library is approximately 2-4 MB. Acceptable for mobile, tight for bare-metal embedded.

### Platform Accessibility Matrix

| Capability | Rust | MCP | C FFI | Python | Mobile |
|-----------|------|-----|-------|--------|--------|
| store_episode | v0.1 | v0.2 | v0.2 | v0.3 | v0.3+ |
| query | v0.1 | v0.2 | v0.2 | v0.3 | v0.3+ |
| status | v0.1 | v0.2 | v0.2 | v0.3 | v0.3+ |
| consolidate | v0.1 | v0.2 (NoOp) | v0.2 | v0.3 | v0.3+ |
| forget | v0.1 | v0.2 | v0.2 | v0.3 | v0.3+ |
| transform | v0.1 | v0.2 | v0.2 | v0.3 | v0.3+ |
| perfume | v0.1 | v0.2 (NoOp) | v0.2 | v0.3 | v0.3+ |
| purge | v0.1 | v0.2 | v0.2 | v0.3 | v0.3+ |
| Custom provider | v0.1 | Partial | Callback-based | v0.3 | Limited |
| Embeddings | v0.1 | v0.2 | v0.2 | v0.3 | v0.3+ |
| Type safety | Full (compile-time) | Schema-validated | Runtime checks | Runtime + type hints | Runtime checks |

---

## 5. Cognitive Accessibility

Cognitive accessibility means the developer can build and maintain an accurate mental model of Alaya without excessive cognitive load. The API should be predictable: once the developer understands one pattern, they can predict how the rest of the API works.

### API Surface Size

**Target: fewer than 20 public methods on `AlayaStore`.**

Current public method count: 14.

| Category | Methods | Count |
|----------|---------|-------|
| Construction | `open()`, `open_in_memory()` | 2 |
| Write | `store_episode()` | 1 |
| Read | `query()`, `preferences()`, `knowledge()`, `neighbors()` | 4 |
| Lifecycle | `consolidate()`, `perfume()`, `transform()`, `forget()` | 4 |
| Admin | `status()`, `purge()` | 2 |
| **Total** | | **13** |

The planned `get_episode()`, `list_episodes()`, `list_sessions()`, `delete_episode()` additions (for CRUD completeness) bring the total to 17. Still under 20.

**Rule:** Every proposed public method must justify its existence. If it can be accomplished by combining existing methods, it does not get a dedicated method. The one exception is convenience constructors (`Query::simple()`) that dramatically reduce friction for Level 1 developers.

### CRUD Symmetry

Every entity follows the same pattern. Once the developer learns the pattern for episodes, they can predict the API for semantic nodes and preferences.

```
store_*()   -> create a new entity, return its ID
get_*()     -> retrieve a single entity by ID
list_*()    -> retrieve multiple entities with optional filter
delete_*()  -> remove an entity by ID
```

Current implementation status:

| Entity | store | get | list | delete |
|--------|-------|-----|------|--------|
| Episode | `store_episode()` | `pub(crate)` | planned | planned |
| SemanticNode | via `consolidate()` | `pub(crate)` | `knowledge()` | via `purge()` |
| Impression | via `perfume()` | `pub(crate)` | planned | via `purge()` |
| Preference | via `perfume()` | `pub(crate)` | `preferences()` | via `purge()` |

The CRUD symmetry is intentionally incomplete for entities created by lifecycle processes (semantic nodes, impressions, preferences). The developer stores episodes; the lifecycle creates everything else. This reinforces the mental model: "I store conversations. Alaya extracts knowledge."

### Sensible Defaults

Every configuration point has a default that produces working behavior without configuration.

| Configuration | Default | When to Override |
|--------------|---------|-----------------|
| Provider | `NoOpProvider` (no LLM) | When you have an LLM and want richer consolidation |
| Query results | 5 (via `Query::simple()`) | When you need more or fewer results for your context window |
| Database path | Required (no magic default) | N/A -- path is always explicit, per Simplicity axiom |
| Embedding | `None` (BM25-only) | When you have an embedding model and want semantic similarity |
| EpisodeContext | `Default::default()` (empty) | When you have topic, entity, or sentiment metadata |
| Feature flags | Default features = full working system | When you need embedding backends or async |

**Rule for defaults:** The default configuration produces the most useful behavior that is achievable without external dependencies. Defaults never require network access, LLM keys, or additional crates.

### Gradual Complexity Curve

Complexity is introduced in a defined sequence. Each step builds on the previous one.

```
Step 1: CRUD (open, store, query)
  "I can store and retrieve memories."
    |
    v
Step 2: Lifecycle (consolidate, forget, transform)
  "Memories evolve over time. The system gets smarter."
    |
    v
Step 3: Providers (ConsolidationProvider, EmbeddingProvider)
  "I can plug in my own LLM to enhance the lifecycle."
    |
    v
Step 4: Tuning (weights, thresholds, graph parameters)
  "I can optimize retrieval for my specific domain."
```

No step requires understanding the subsequent step. A developer who never moves past Step 1 has a working, useful memory system.

### Mental Model: The Storehouse Metaphor

The single unifying metaphor is the **storehouse** (from the Sanskrit *alaya-vijnana*). This metaphor maps to every Alaya concept.

| Metaphor Element | Alaya Concept | API Surface |
|-----------------|--------------|-------------|
| The storehouse | The SQLite file | `AlayaStore::open("memory.db")` |
| Putting something in | Storing an episode | `store_episode()` |
| Looking for something | Querying memories | `query()` |
| Organizing the storehouse | Consolidation | `consolidate()` |
| Forgetting where things are | Retrieval strength decay | `forget()` |
| Cleaning up the storehouse | Transformation / purge | `transform()`, `purge()` |
| Noticing patterns | Preference emergence | `perfume()` |
| Checking inventory | Status | `status()` |

The metaphor works at every skill level:
- Level 1: "I put things in and find them later."
- Level 2: "I organize periodically and the storehouse gets more useful."
- Level 3: "I have a helper (provider) who categorizes things for me."
- Level 4: "I can tune how the organization system works."

### Consistency Patterns

Consistency reduces cognitive load. Once a developer observes a pattern, they expect it everywhere.

| Pattern | Where Applied | Violations |
|---------|--------------|------------|
| All fallible methods return `Result<T, AlayaError>` | Every public method | None |
| Input types are `New*`, output types are the entity name | `NewEpisode`/`Episode`, `NewSemanticNode`/`SemanticNode`, `NewImpression`/`Impression` | `Interaction` is consumed by perfuming, not stored directly |
| Lifecycle methods return `*Report` types | `consolidate()` -> `ConsolidationReport`, `forget()` -> `ForgettingReport`, etc. | None |
| ID types are newtypes around `i64` | `EpisodeId`, `NodeId`, `PreferenceId`, `ImpressionId`, `LinkId` | None |
| All public types derive `Debug, Clone` | All types in `types.rs` | None |
| Output types derive `Serialize, Deserialize` | `Episode`, `SemanticNode`, `Preference`, `ScoredMemory`, all reports | Input types (`NewEpisode`, `Query`) do not, by design |
| `&self` on all methods (no `&mut self`) | All `AlayaStore` methods | None (SQLite handles interior mutability via WAL) |

---

## 6. Diagnostic Accessibility

Diagnostic accessibility means the developer can understand what Alaya is doing and why. When retrieval results are unexpected, when lifecycle processes do not seem to work, or when performance degrades, the developer needs tools to investigate.

### Diagnostic Levels

| Level | What It Shows | How to Access | Audience |
|-------|--------------|--------------|----------|
| **Status** | Counts of all entities | `store.status() -> MemoryStatus` | All levels |
| **Reports** | What each lifecycle process did | Return values from `consolidate()`, `forget()`, `transform()`, `perfume()` | Level 2+ |
| **Explanation** (planned) | Per-stage scores for a query | `store.explain_query(&query) -> QueryExplanation` | Level 3+ |
| **Tracing** (planned) | Structured log events for every operation | `tracing` integration with span-per-method | Level 4 / production |

### MemoryStatus

The simplest diagnostic. Tells the developer what is in the database.

```rust
let status = store.status()?;
println!("Episodes: {}", status.episode_count);
println!("Semantic nodes: {}", status.semantic_node_count);
println!("Preferences: {}", status.preference_count);
println!("Impressions: {}", status.impression_count);
println!("Links: {}", status.link_count);
println!("Embeddings: {}", status.embedding_count);
```

**What it diagnoses:**
- "I stored episodes but `semantic_node_count` is 0." -- You have not run `consolidate()` yet, or your provider returned no nodes.
- "I ran consolidate but `preference_count` is 0." -- Preferences come from `perfume()`, not `consolidate()`. Run `perfume()` with an interaction.
- "My `link_count` is 0 after storing many episodes." -- Links require either temporal linking (set `preceding_episode` in `EpisodeContext`) or co-retrieval (run queries that retrieve multiple results).
- "`embedding_count` is 0." -- You are not providing embeddings in `NewEpisode.embedding`. BM25-only retrieval is working, but semantic similarity is not available.

### Lifecycle Reports

Every lifecycle method returns a typed report. The developer can log, display, or assert on these.

```rust
let cr = store.consolidate(&provider)?;
println!("Processed {} episodes, created {} nodes, {} links",
    cr.episodes_processed, cr.nodes_created, cr.links_created);

let fr = store.forget()?;
println!("Decayed {} nodes, archived {} nodes",
    fr.nodes_decayed, fr.nodes_archived);

let tr = store.transform()?;
println!("Merged {} duplicates, pruned {} links, decayed {} preferences",
    tr.duplicates_merged, tr.links_pruned, tr.preferences_decayed);

let pr = store.perfume(&interaction, &provider)?;
println!("Stored {} impressions, crystallized {} preferences, reinforced {}",
    pr.impressions_stored, pr.preferences_crystallized, pr.preferences_reinforced);
```

**What reports diagnose:**
- "consolidate() returns episodes_processed: 0." -- Either no unconsolidated episodes exist, or consolidation has already processed all available episodes.
- "nodes_created is always 0 even with episodes." -- Your `ConsolidationProvider::extract_knowledge()` is returning an empty vector. Check your provider implementation.
- "forget() returns nodes_decayed: 0." -- All nodes still have high retrieval strength. Either they were recently accessed or not enough time has passed for decay.
- "transform() merged 0 duplicates." -- No duplicate semantic nodes detected. This is normal for small datasets.

### QueryExplanation (Planned, v0.2)

The most important diagnostic for retrieval quality. Shows exactly why a query returned (or did not return) specific results.

```rust
// Planned API
let explanation = store.explain_query(&Query::simple("editor preferences"))?;

println!("BM25 matches: {}", explanation.bm25_results.len());
for (node, score) in &explanation.bm25_results {
    println!("  {:?}: {:.4}", node, score);
}

println!("Vector matches: {}", explanation.vector_results.len());
println!("Graph activated: {}", explanation.graph_activated.len());
println!("After RRF fusion: {}", explanation.fused_results.len());
println!("After reranking: {}", explanation.final_results.len());
```

**What it diagnoses:**
- "BM25 returned 0 results." -- No lexical overlap between query terms and stored content. Either rephrase the query using terms that appear in stored episodes, or add embeddings for semantic matching.
- "BM25 found results but they scored low after reranking." -- Temporal recency or context mismatch is penalizing these results. Check `QueryContext` fields.
- "Vector search found results but BM25 did not." -- The semantic match exists but the words are different. This confirms embedding value.
- "Graph activation contributed results not found by BM25 or vector." -- The Hebbian graph is connecting related memories through co-retrieval links. This is the lifecycle payoff.

### Structured Logging (Planned, v0.2)

Integration with the `tracing` crate for production debugging. Each `AlayaStore` method emits a span with relevant context.

```rust
// Planned: tracing integration behind feature flag
// cargo add alaya --features tracing

// Developer configures their tracing subscriber as usual
// Alaya emits spans like:
// alaya::store_episode{session_id="s1" content_len=42}
// alaya::query{text="editor preferences" max_results=5}
//   alaya::retrieval::bm25{matches=3}
//   alaya::retrieval::vector{matches=0}
//   alaya::retrieval::graph{activated=2}
//   alaya::retrieval::fusion{fused=4}
//   alaya::retrieval::rerank{final=3}
// alaya::consolidate{episodes=5}
//   alaya::lifecycle::extract_knowledge{nodes=2}
// alaya::forget{decayed=12 archived=1}
```

**Design rules for tracing accessibility:**
- Span names follow the module path: `alaya::store_episode`, `alaya::retrieval::bm25`
- Field names are descriptive: `content_len`, not `cl`; `matches`, not `n`
- No sensitive data in spans: session IDs yes, episode content no
- Feature-gated: adds zero overhead when disabled

---

## 7. Onboarding Accessibility

Onboarding accessibility is measured by one number: **time to first success.** For Alaya, the target is under 2 minutes from `cargo add alaya` to a working query that returns relevant results.

### The 2-Minute Budget

| Step | Time Budget | Potential Failure |
|------|------------|-------------------|
| `cargo add alaya` | 5 seconds | Version conflict with existing `rusqlite` dep |
| `cargo build` (incremental) | 15-30 seconds | First build includes SQLite compilation (~45s). Documented honestly. |
| Copy quickstart from README | 15 seconds | Developer modifications break the example |
| `cargo run` | 5 seconds | Compilation error from typo or missing import |
| See query results in terminal | 0 seconds | Empty results (BM25 gap) |
| **Total** | **40-55 seconds** | |

The 2-minute budget includes one failure-and-retry cycle. If the quickstart compiles on second try after fixing a typo, the developer is still within budget.

### First Example Design Constraints

The quickstart example is the most important piece of documentation in the project. It is designed with the following constraints, each derived from the developer journey analysis (Phase 5a).

1. **Must compile without modification.** No placeholder values that need to be replaced. No `YOUR_API_KEY_HERE`. No commented-out lines that need uncommenting.

2. **Must produce visible output.** The example prints query results to stdout. A developer who runs `cargo run` and sees nothing is a developer who leaves.

3. **Must avoid the empty-results trap.** The stored content and the query share lexical tokens. "I prefer dark mode and Vim keybindings" queried with "dark mode" or "Vim keybindings" returns results. Not "What are the user's interface preferences?" which shares zero tokens with the stored content.

4. **Must use `EpisodeContext::default()`.** The developer should not need to understand context metadata to get their first result.

5. **Must not require external dependencies.** No `chrono` (use `std::time` or hardcoded timestamp), no `tokio`, no `uuid`. The only `use` statements are from the `alaya` crate.

6. **Must fit in a single screen.** Under 30 lines of meaningful code (excluding blank lines and comments). The developer should see the entire example without scrolling.

### Graceful Failure Guarantee

Even when misconfigured, Alaya must never panic in release mode. Every failure path returns a `Result::Err` with an actionable message.

| Misconfiguration | Behavior | What the Developer Sees |
|-----------------|----------|------------------------|
| Path to non-existent directory | `Err(AlayaError::Db(...))` | "Cannot open database: directory '/bad/path' does not exist" |
| Read-only filesystem | `Err(AlayaError::Db(...))` | "Cannot write to database at '{path}': permission denied" |
| Zero-length content | `Err(AlayaError::InvalidInput(...))` | "Episode content cannot be empty" |
| Query on empty database | `Ok(vec![])` | Empty results (not an error -- this is correct behavior) |
| Provider panics | Caught at FFI/MCP boundary; in Rust, `catch_unwind` consideration | Provider errors are `Result::Err`, not panics |
| Database file locked by another process | `Err(AlayaError::Db(...))` | "Database is locked" with explanation |

**The no-panic guarantee:**
- All `unwrap()` calls in public code paths have been audited. The only `unwrap()` in the public API is in `AlayaStore::open()` for the path conversion, which is being replaced with proper error handling.
- `#[cfg(test)]` code may use `unwrap()` for brevity.
- FFI boundary will use `catch_unwind` to prevent Rust panics from crossing into C callers.

### Migration Path Between Versions

Version upgrades must not be a barrier to continued use. The migration strategy:

1. **Schema versioning:** The SQLite database includes a schema version number (user_version pragma). Every `AlayaStore::open()` checks the version.

2. **Forward migration:** `AlayaStore::open()` detects old schemas and returns a clear error with migration instructions. `AlayaStore::open_and_migrate()` (planned) runs the migration automatically after creating a backup.

3. **Backup before migration:** Before any schema change, Alaya copies `memory.db` to `memory.db.backup-v{old_version}`. The developer can always roll back.

4. **Semver discipline:** Breaking API changes require a major version bump. Adding enum variants on `#[non_exhaustive]` enums is non-breaking. The developer's `match` arms do not break on minor upgrades.

5. **Changelog with migration notes:** Every release includes a "Migration" section that says either "No migration needed" or specifies the exact steps.

---

## 8. Testing and Validation

Accessibility is not a one-time effort. It is validated continuously through automated testing.

### Doctest Coverage

**Rule:** Every `pub` method has at least one compilable doctest.

**Validation:** `cargo test --doc` runs in CI on every commit. Zero warnings from `cargo doc`. Any public method without a doctest fails CI.

**Doctest accessibility rules:**
- Doctests use the simplest possible setup (prefer `open_in_memory()` over file-based)
- Doctests do not depend on external state or timing
- Doctests show the import line at the top (developers copy-paste from docs.rs)
- Doctests demonstrate the happy path first, then error handling

### Integration Test Suite

**Coverage requirements:**
- Every public `AlayaStore` method has at least one integration test
- Every `AlayaError` variant is exercised by at least one test
- Every lifecycle process is tested with both `NoOpProvider` and `MockProvider`
- Schema migration is tested for every version transition

**Test organization:**
```
tests/
  quickstart.rs       -- The README quickstart, verbatim, as a test
  crud.rs             -- Store/get/list/delete for all entities
  retrieval.rs        -- BM25, vector, graph, fusion, reranking
  lifecycle.rs        -- Consolidation, forgetting, transformation, perfuming
  errors.rs           -- Every error variant, every validation path
  migration.rs        -- Schema version upgrades
  concurrency.rs      -- Multi-threaded access patterns
```

### Error Path Testing

Every `AlayaError` variant is tested for:
1. The error is produced by the expected condition
2. The `.to_string()` output is actionable (contains "what to do")
3. The error does not leak internal implementation details at Level 1
4. The `.source()` chain preserves the original error for Level 4 debugging

### Cross-Platform CI

| Platform | Rust Version | Runs On |
|----------|-------------|---------|
| Ubuntu 22.04 (x86_64) | Stable, MSRV | Every commit |
| macOS 14 (aarch64) | Stable | Every commit |
| Windows Server 2022 (x86_64) | Stable | Every commit |
| Ubuntu 22.04 (aarch64) | Stable | Weekly |

### Quickstart Regression Test

A dedicated CI job that:
1. Creates a new Rust project with `cargo init`
2. Adds `alaya` as a dependency
3. Copies the quickstart code from README.md (extracted automatically)
4. Runs `cargo run` and verifies non-empty output
5. Fails the CI pipeline if the quickstart produces no output or does not compile

This prevents the README from drifting out of sync with the actual API.

---

## 9. Accessibility Metrics

These metrics measure whether Alaya's accessibility strategy is working. They are leading indicators for the North Star metric (MACC).

### Onboarding Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Time to first successful query | < 2 minutes | CI quickstart regression test; user interviews |
| Quickstart compilation errors | 0 per release | CI quickstart regression test |
| First-try success rate (in user tests) | > 80% | User interviews and testing sessions |

### Documentation Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Public API documentation coverage | 100% | `cargo doc` warnings = 0 in CI |
| Doctest pass rate | 100% | `cargo test --doc` in CI |
| Glossary term coverage | 100% of specialized terms | Manual audit per release |
| docs.rs page load (no broken links) | 0 broken links | CI link checker |

### Error Quality Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Error messages with "what to do" guidance | 100% of variants | Code review checklist |
| Self-service error resolution rate | > 90% | Issue tracker: ratio of error-related issues to download count |
| Provider error boundary attribution | 100% | Unit tests verify "your ... implementation" appears in Provider errors |

### Platform Reach Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Cross-platform CI pass rate | 100% (Linux, macOS, Windows) | CI dashboard |
| MCP server tool coverage | 100% of core CRUD + lifecycle | MCP server test suite |
| FFI function coverage | 100% of core CRUD + lifecycle | FFI test suite |
| Python binding coverage | 100% of core CRUD + lifecycle | PyO3 test suite |

### Cognitive Load Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Public method count on AlayaStore | < 20 | Automated count in CI |
| Import count for quickstart | <= 5 | Quickstart regression test |
| Required reading for Level 1 | < 1 page (README quickstart section) | Manual audit |
| CRUD pattern consistency | 100% (all entities follow same pattern) | Code review |

---

## 10. Anti-Patterns

These are specific accessibility failures that Alaya must avoid. Each is drawn from the competitive landscape, user journey analysis, or the Extract's constraints.

### Anti-Pattern 1: The Jargon Wall

**Description:** Documentation leads with specialized terminology ("vasana perfuming," "Bjork dual-strength," "asraya-paravrtti") before establishing what the library does in plain language.

**Why it kills accessibility:** A developer who encounters unfamiliar jargon before understanding the value proposition will leave. Jargon is a signal of exclusion, not sophistication.

**Alaya's rule:** The README and quickstart use zero specialized terms. Layer 1 and Layer 2 documentation use plain English exclusively. Specialized terms appear in Layer 3 (docs.rs) with immediate plain-English translations, and are fully explained in Layer 4 (architecture docs) with research citations.

**Test:** Can a Python developer with zero Rust experience read the README and understand what Alaya does? If no, the README has failed.

### Anti-Pattern 2: The Configuration Prerequisite

**Description:** The library requires the developer to configure providers, feature flags, or environment variables before basic operations work.

**Why it kills accessibility:** Configuration is a barrier. Every configuration step before "it works" is a moment where the developer can give up.

**Alaya's rule:** `AlayaStore::open("memory.db")` followed by `store_episode()` and `query()` works with zero configuration. No providers, no feature flags, no environment variables. `NoOpProvider` is the default. `BM25-only` retrieval is the default. The developer opts into complexity; they are never forced into it.

### Anti-Pattern 3: The Debugging Black Box

**Description:** The library returns results but provides no way to understand why those specific results were returned (or why results are empty).

**Why it kills accessibility:** "It doesn't work" is the developer's reaction to unexpected results. Without diagnostics, the developer cannot distinguish between "I am using it wrong" and "the library has a bug."

**Alaya's rule:** `MemoryStatus` is available from day one. Lifecycle reports ship with v0.1. `QueryExplanation` ships with v0.2. Structured tracing ships with v0.2. At every stage, the developer has tools proportional to their skill level.

### Anti-Pattern 4: The Platform Prison

**Description:** The library is excellent but only accessible from one language, locking out 90% of potential users.

**Why it kills accessibility:** Most AI agent developers write Python, not Rust. If Alaya is Rust-only forever, its addressable market is tiny.

**Alaya's rule:** Tier 2 (MCP + FFI) ships with v0.2, 6-8 weeks after v0.1. Tier 3 (Python) ships with v0.3. The library's value should not be gated by language choice. Every tier is documented and tested to the same standard as the Rust API.

### Anti-Pattern 5: The Stale Example

**Description:** Documentation examples do not compile because the API changed since they were written.

**Why it kills accessibility:** A broken example is worse than no example. The developer has invested time copying and trying to compile code that cannot work. Trust is damaged.

**Alaya's rule:** All examples are compiled in CI via doctests. The README quickstart is extracted and compiled as a standalone test. No documentation ships without passing compilation. If an API change breaks an example, the example is fixed before the release.

---

## Appendix: Accessibility Checklist Per Release

Before every release, the following checklist is verified:

### Documentation
- [ ] README quickstart compiles and produces output (CI verified)
- [ ] All public methods have doctest examples (cargo doc warnings = 0)
- [ ] All doctests pass (cargo test --doc)
- [ ] Glossary covers all specialized terms used in docs.rs
- [ ] CHANGELOG includes migration section

### Errors
- [ ] Every AlayaError variant has an actionable message
- [ ] Provider errors include boundary attribution
- [ ] Database errors include path context
- [ ] No raw SQLite errors exposed to Level 1 surface

### Onboarding
- [ ] Time-to-first-success measured (target: < 2 minutes)
- [ ] Quickstart uses no more than 5 imports
- [ ] No configuration required for basic CRUD + query

### Platform
- [ ] CI passes on Linux, macOS, Windows
- [ ] MCP server tests pass (when applicable)
- [ ] FFI tests pass (when applicable)
- [ ] Python tests pass (when applicable)

### Cognitive Load
- [ ] Public method count on AlayaStore < 20
- [ ] CRUD symmetry maintained for all entities
- [ ] All lifecycle methods return typed reports
- [ ] No new public types without documentation

---

*Generated: 2026-02-26 | Phase: 5c | Cross-references: Developer Journeys (Phase 5a), API Design System (Phase 5b), Brand Guidelines (Phase 1), North Star (Phase 2), Competitive Landscape (Phase 3), North Star Extract (Phase 4)*
