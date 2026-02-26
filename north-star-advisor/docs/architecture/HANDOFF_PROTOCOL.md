# Provider Contract Protocol

> How Alaya calls consumer-provided trait implementations, what it passes, what it expects back, and what happens when things go wrong.

**Status**: Living document, tracking `alaya 0.1.0`
**Last Updated**: 2026-02-26
**Source of Truth**: `/src/provider.rs`, `/src/lifecycle/consolidation.rs`, `/src/lifecycle/perfuming.rs`

---

## Table of Contents

1. [Provider Contract Philosophy](#1-provider-contract-philosophy)
2. [ConsolidationProvider Trait](#2-consolidationprovider-trait)
3. [extract_knowledge() Contract](#3-extract_knowledge-contract)
4. [extract_impressions() Contract](#4-extract_impressions-contract)
5. [detect_contradiction() Contract](#5-detect_contradiction-contract)
6. [NoOpProvider Reference Implementation](#6-noopprovider-reference-implementation)
7. [Provider Lifecycle: When Each Method Is Called](#7-provider-lifecycle-when-each-method-is-called)
8. [Error Handling Contract](#8-error-handling-contract)
9. [Contract Validation](#9-contract-validation)
10. [EmbeddingProvider Design (Planned, v0.2)](#10-embeddingprovider-design-planned-v02)
11. [Testing Provider Implementations](#11-testing-provider-implementations)
12. [Migration Guide](#12-migration-guide)

---

## 1. Provider Contract Philosophy

### The Boundary

Alaya is a library. It has no LLM connection, no HTTP client, no API keys. The consumer (your agent application) owns the intelligence. Alaya owns the storage, retrieval, and lifecycle orchestration. The provider trait is the boundary where these two domains meet.

The relationship is **inversion of control**: Alaya calls your code, not the other way around. You implement a trait. Alaya calls its methods at specific points during lifecycle processes. Your implementation can do anything -- call an LLM, run a local model, apply heuristics, return hardcoded values. Alaya does not care how you produce the answer. It cares only that you return the right types.

```
+---------------------------+         +---------------------------+
|      Your Agent           |         |        Alaya              |
|                           |         |                           |
|  - Owns the LLM           |         |  - Owns the SQLite DB     |
|  - Owns the API keys      |         |  - Owns the graph          |
|  - Owns the prompt logic   |         |  - Owns the lifecycle      |
|  - Implements provider     | <------ |  - Calls provider methods  |
|    trait methods           |         |    during consolidate()    |
|                           |         |    and perfume()           |
+---------------------------+         +---------------------------+
```

### Three Axioms

**Axiom 1: Alaya never calls an LLM directly.** The library has zero network dependencies. If a lifecycle process requires intelligence (extracting facts from conversation, detecting contradictions, classifying behavioral impressions), that intelligence comes through a provider trait method. This is not a limitation. It is a design choice rooted in the project's privacy-by-architecture principle. The consumer controls when LLM calls happen, to which model, at what cost, with what prompt.

**Axiom 2: NoOpProvider is always valid.** Every provider method has a sensible no-op return value: empty vectors, `false`. A consumer can pass `&NoOpProvider` to any lifecycle method and receive a valid (if empty) result. The library never panics, never errors, never degrades in a way that requires recovery. This means lifecycle processes are safe to call even when no LLM is available -- they simply produce no new semantic nodes, no new impressions, no contradiction detections. The episodic store continues to accumulate, BM25 retrieval continues to work, the Hebbian graph continues to reshape through co-retrieval. The system gracefully degrades along a well-defined chain:

| Provider State | What Works | What Doesn't |
|---|---|---|
| Full provider | Everything: consolidation, perfuming, contradiction detection | Nothing missing |
| NoOpProvider | Store, query, transform, forget, BM25, graph, embeddings | No new semantic nodes, no impressions, no contradiction checks |
| No lifecycle calls at all | Store, query (BM25-only if no embeddings) | No semantic layer, no preferences, no forgetting, no graph pruning |

**Axiom 3: The agent controls the clock.** Lifecycle processes (`consolidate()`, `perfume()`, `transform()`, `forget()`) are explicit function calls, not background threads or timers. The consumer decides when to run them, how often, and in what order. This means:

- No surprising state changes. The database does not mutate except when you call a mutating method.
- No surprising LLM costs. Provider methods are called only when the consumer explicitly triggers a lifecycle process.
- Deterministic testing. You can run lifecycle processes in a specific order with specific inputs and assert specific outputs.

### What This Document Covers

This document specifies the **contract** between Alaya and a provider implementation. For each trait method, it defines:

- **Input**: What Alaya passes to the method, and what each field means.
- **Output**: What the consumer must return, and what Alaya does with each field.
- **Semantic contract**: What the method *means*, beyond the type signature.
- **When called**: The precise lifecycle context in which Alaya invokes the method.
- **Error behavior**: What happens when the method returns `Err`, panics, or returns structurally valid but semantically garbage output.
- **NoOp behavior**: What the no-op implementation returns and why it is safe.

---

## 2. ConsolidationProvider Trait

### Trait Definition (Source of Truth: `src/provider.rs`)

```rust
/// The agent provides this trait to support intelligent consolidation.
/// Alaya never calls an LLM directly -- the agent owns the LLM connection.
pub trait ConsolidationProvider {
    /// Extract semantic knowledge from a batch of episodes.
    fn extract_knowledge(&self, episodes: &[Episode]) -> Result<Vec<NewSemanticNode>>;

    /// Extract behavioral impressions from an interaction.
    fn extract_impressions(&self, interaction: &Interaction) -> Result<Vec<NewImpression>>;

    /// Detect whether two semantic nodes contradict each other.
    fn detect_contradiction(&self, a: &SemanticNode, b: &SemanticNode) -> Result<bool>;
}
```

### Object Safety

`ConsolidationProvider` is used as `&dyn ConsolidationProvider` in Alaya's lifecycle methods. This means:

- It is object-safe (no generic methods, no `Self` in return position).
- The consumer passes a reference, not an owned value.
- The consumer can implement it on any type: a struct holding an HTTP client, a struct holding a local model handle, a unit struct for heuristic extraction.

### Lifetime and Ownership

All method parameters are borrowed references. The consumer never needs to take ownership of Alaya data. All return values are owned (`Vec<NewSemanticNode>`, `Vec<NewImpression>`, `bool`). Alaya takes ownership of the returned data and stores it.

```rust
// Alaya calls provider methods with borrowed data:
pub fn consolidate(&self, provider: &dyn ConsolidationProvider) -> Result<ConsolidationReport>
pub fn perfume(&self, interaction: &Interaction, provider: &dyn ConsolidationProvider) -> Result<PerfumingReport>
```

The `&self` on `ConsolidationProvider` methods means the provider itself is borrowed. If your provider holds a connection pool, a rate limiter, or cached state, it can use interior mutability (`Mutex`, `RefCell`) as needed. Alaya never takes ownership of the provider.

### Thread Safety

`AlayaStore` is `Send` but not `Sync`. The typical multi-threaded pattern is `Arc<Mutex<AlayaStore>>`. The provider is received as `&dyn ConsolidationProvider`, meaning it must be valid for the duration of the `consolidate()` or `perfume()` call, but there are no `Send` or `Sync` bounds on the trait itself. If you need to call your provider from multiple threads, the `Sync` bound is your responsibility at the call site.

---

## 3. extract_knowledge() Contract

### When Called

`extract_knowledge()` is called exactly once per `consolidate()` invocation, after Alaya has fetched a batch of unconsolidated episodes from the episodic store.

```
consolidate() called by consumer
  |
  +-> Fetch unconsolidated episodes (batch_size = 10, minimum = 3)
  |     If fewer than 3 episodes: return empty report, provider never called
  |
  +-> provider.extract_knowledge(&episodes)    <--- YOUR CODE RUNS HERE
  |
  +-> For each returned NewSemanticNode:
        +-> Store in semantic_nodes table
        +-> Create Causal links to source episodes (weight 0.7)
        +-> Initialize node strength (SS=0.5, RS=1.0)
```

### Input: `&[Episode]`

Alaya passes a slice of `Episode` structs. These are the raw conversation turns that have not yet been consolidated.

```rust
pub struct Episode {
    pub id: EpisodeId,          // Unique ID (newtype around i64)
    pub content: String,        // The raw conversation text
    pub role: Role,             // User, Assistant, or System
    pub session_id: String,     // Groups episodes into conversations
    pub timestamp: i64,         // Unix seconds
    pub context: EpisodeContext, // Extracted metadata
}

pub struct EpisodeContext {
    pub topics: Vec<String>,           // Topic tags (may be empty)
    pub sentiment: f32,                // Sentiment score (may be 0.0)
    pub conversation_turn: u32,        // Turn number within session (may be 0)
    pub mentioned_entities: Vec<String>, // Named entities (may be empty)
    pub preceding_episode: Option<EpisodeId>, // Link to previous turn
}
```

**Batch characteristics:**

- **Size**: Up to 10 episodes per batch (`CONSOLIDATION_BATCH_SIZE = 10`).
- **Minimum**: At least 3 episodes (the corroboration threshold). If fewer exist, `extract_knowledge()` is never called.
- **Ordering**: Episodes are fetched in insertion order (ascending `id`), which typically corresponds to chronological order.
- **Content**: Raw conversation text, exactly as stored by `store_episode()`. No preprocessing, no sanitization beyond what the consumer applied before storage.
- **Context fields**: May be populated or may be empty defaults. Your provider should handle both cases.

### Output: `Vec<NewSemanticNode>`

Your implementation returns zero or more semantic nodes extracted from the episodes.

```rust
pub struct NewSemanticNode {
    pub content: String,                // The extracted knowledge statement
    pub node_type: SemanticType,        // Fact, Relationship, Event, or Concept
    pub confidence: f32,                // How confident the extraction is [0.0, 1.0]
    pub source_episodes: Vec<EpisodeId>, // Which episodes support this knowledge
    pub embedding: Option<Vec<f32>>,    // Optional embedding vector
}

pub enum SemanticType {
    Fact,          // "User works at Acme Corp"
    Relationship,  // "User's sister is named Sarah"
    Event,         // "User graduated in 2019"
    Concept,       // "User values work-life balance"
}
```

**Semantic contract for each field:**

| Field | Contract | Example |
|---|---|---|
| `content` | A natural-language statement of extracted knowledge. Should be self-contained (readable without the source episodes). Should be specific (not "User discussed programming" but "User is building a CLI tool in Rust for log analysis"). | `"User prefers tabs over spaces in Rust code"` |
| `node_type` | Classify the knowledge. `Fact` for declarative statements. `Relationship` for connections between entities. `Event` for time-bound occurrences. `Concept` for abstract ideas or values. | `SemanticType::Fact` |
| `confidence` | A float in `[0.0, 1.0]` representing extraction confidence. `0.8+` means the episodes clearly state this. `0.5-0.8` means it is implied. `<0.5` means it is speculative. Alaya stores this value directly; it is used in retrieval ranking and transformation-phase pruning. | `0.85` |
| `source_episodes` | The IDs of episodes that support this knowledge node. Must reference episodes that actually exist in the batch. Alaya uses these to create Causal links in the graph overlay. | `vec![EpisodeId(1), EpisodeId(3)]` |
| `embedding` | An optional embedding vector. If provided, stored in the embeddings table and used for vector retrieval and deduplication. If `None`, the node is only reachable via BM25 and graph traversal. | `Some(vec![0.1, 0.2, ...])` or `None` |

**What Alaya does with each returned node (source: `src/lifecycle/consolidation.rs`):**

1. Stores the node in the `semantic_nodes` table via `semantic::store_semantic_node()`.
2. For each `EpisodeId` in `source_episodes`, creates a `Causal` link with initial weight `0.7` via `links::create_link()`.
3. Initializes Bjork dual-strength tracking via `strengths::init_strength()` (storage strength = 0.5, retrieval strength = 1.0).
4. If `embedding` is `Some`, stores it in the `embeddings` table (handled at the store layer).
5. Increments `nodes_created` and `links_created` in the `ConsolidationReport`.

**Returning an empty vector is always valid.** It means the provider found no extractable knowledge in this batch. This is functionally identical to using `NoOpProvider` for this particular batch.

### Guidance for LLM-Based Implementations

If your provider wraps an LLM call, consider this prompt structure:

```
Given the following conversation episodes, extract factual knowledge about the user.

Rules:
- Each fact should be a single, self-contained statement
- Classify each as: fact, relationship, event, or concept
- Rate your confidence from 0.0 to 1.0
- Reference the episode IDs that support each fact
- Only extract knowledge that is clearly stated or strongly implied
- Do NOT fabricate or infer beyond what the text supports

Episodes:
[serialize episodes here]
```

Parse the LLM response into `Vec<NewSemanticNode>`. Handle LLM failures by returning `Err(AlayaError::Provider("LLM call failed: ...".into()))`.

### Guidance for Heuristic Implementations

Not every provider needs an LLM. For domain-specific agents, heuristic extraction can be effective:

```rust
impl ConsolidationProvider for RegexProvider {
    fn extract_knowledge(&self, episodes: &[Episode]) -> Result<Vec<NewSemanticNode>> {
        let mut nodes = Vec::new();
        for ep in episodes {
            // Extract email addresses as facts
            for capture in EMAIL_REGEX.captures_iter(&ep.content) {
                nodes.push(NewSemanticNode {
                    content: format!("User's email is {}", &capture[0]),
                    node_type: SemanticType::Fact,
                    confidence: 0.95,
                    source_episodes: vec![ep.id],
                    embedding: None,
                });
            }
        }
        Ok(nodes)
    }
    // ...
}
```

---

## 4. extract_impressions() Contract

### When Called

`extract_impressions()` is called exactly once per `perfume()` invocation, at the start of the perfuming process.

```
perfume(interaction, provider) called by consumer
  |
  +-> provider.extract_impressions(&interaction)   <--- YOUR CODE RUNS HERE
  |
  +-> For each returned NewImpression:
  |     +-> Store in impressions table
  |
  +-> For each affected domain:
        +-> Count impressions in this domain
        +-> If count >= 5 (CRYSTALLIZATION_THRESHOLD):
              +-> If no existing preference for this domain:
              |     +-> Crystallize new preference from recent impressions
              |     +-> Initialize node strength
              +-> If existing preference:
                    +-> Reinforce (increment evidence_count)
```

### Input: `&Interaction`

Alaya passes a single interaction. This is a lightweight view of a conversation turn, constructed by the consumer specifically for the perfuming process.

```rust
pub struct Interaction {
    pub text: String,            // The conversation text
    pub role: Role,              // User, Assistant, or System
    pub session_id: String,      // Session identifier
    pub timestamp: i64,          // Unix seconds
    pub context: EpisodeContext, // Metadata (same structure as episodes)
}
```

**Key distinction from Episode**: An `Interaction` is not stored in the episodic store. It is a transient input to the perfuming process. The consumer constructs it from whatever interaction format their agent uses (Signal message, Discord message, HTTP request, MCP tool call). Typically the consumer calls `perfume()` after `store_episode()`, passing the same content in a different wrapper:

```rust
// Consumer's conversation loop:
let episode = NewEpisode { content: msg.clone(), role: Role::User, /* ... */ };
store.store_episode(&episode)?;

let interaction = Interaction { text: msg, role: Role::User, /* ... */ };
store.perfume(&interaction, &provider)?;
```

### Output: `Vec<NewImpression>`

Your implementation returns zero or more behavioral impressions extracted from the interaction.

```rust
pub struct NewImpression {
    pub domain: String,      // The behavioral domain
    pub observation: String,  // What was observed
    pub valence: f32,         // Positive (1.0) to negative (-1.0) signal
}
```

**Semantic contract for each field:**

| Field | Contract | Example |
|---|---|---|
| `domain` | A category string that groups related impressions. Alaya uses this for crystallization thresholds: when 5+ impressions accumulate in the same domain, a preference crystallizes. Choose domains that represent meaningful behavioral dimensions. Use consistent naming. | `"communication_style"`, `"code_formatting"`, `"topic_interest"` |
| `observation` | A natural-language statement of what was observed. Should describe the user's behavior or preference, not the content of the conversation. This text becomes the basis for crystallized preferences. | `"prefers concise answers over detailed explanations"` |
| `valence` | A float in `[-1.0, 1.0]`. Positive valence means the user expressed preference *for* something. Negative means preference *against*. Zero means neutral observation. Currently stored but not used in crystallization logic (the `avg_valence` is computed but reserved for future valence-aware preference scoring). | `1.0` (strong positive), `-0.5` (mild negative), `0.0` (neutral) |

**What Alaya does with each returned impression (source: `src/lifecycle/perfuming.rs`):**

1. Stores each impression in the `impressions` table via `implicit::store_impression()`.
2. Collects the unique domains from all returned impressions.
3. For each domain, counts total impressions via `implicit::count_impressions_by_domain()`.
4. If count >= 5 (`CRYSTALLIZATION_THRESHOLD`):
   - If no existing preference for this domain: crystallizes a new preference from the 20 most recent impressions, with confidence calculated as `min(count / 20, 0.9)`. Initializes node strength tracking.
   - If an existing preference exists: reinforces it by incrementing `evidence_count` via `implicit::reinforce_preference()`.

**The crystallization process**: When enough impressions accumulate, Alaya calls `summarize_impressions()` (currently a simple heuristic that picks the most recent observation). The resulting text becomes the `preference` field of a new `Preference` record. This is an internal Alaya function, not a provider method -- the provider's role is extracting impressions, not summarizing them.

**Returning an empty vector is always valid.** It means the provider found no behavioral signal in this interaction. No impressions are stored, no domains are checked for crystallization.

### Domain Naming Strategy

Domains are freeform strings. Alaya does not validate them. The consumer is responsible for consistent naming. Inconsistent domains prevent crystallization:

```rust
// BAD: These are all different domains, will never reach threshold of 5
NewImpression { domain: "style".into(), observation: "prefers bullet points".into(), valence: 1.0 }
NewImpression { domain: "formatting".into(), observation: "likes bullet points".into(), valence: 1.0 }
NewImpression { domain: "response_style".into(), observation: "wants bullet points".into(), valence: 1.0 }

// GOOD: Consistent domain, will crystallize after 5 impressions
NewImpression { domain: "response_format".into(), observation: "prefers bullet points".into(), valence: 1.0 }
NewImpression { domain: "response_format".into(), observation: "likes structured output".into(), valence: 1.0 }
NewImpression { domain: "response_format".into(), observation: "asks for lists over paragraphs".into(), valence: 1.0 }
```

A recommended domain taxonomy for general-purpose agents:

| Domain | What It Captures | Example Observations |
|---|---|---|
| `communication_style` | How the user wants to be spoken to | "prefers casual tone", "dislikes jargon" |
| `response_format` | Structural preferences | "prefers code blocks", "wants bullet points" |
| `response_length` | Verbosity preferences | "prefers concise answers", "wants detailed explanations" |
| `topic_interest` | What the user cares about | "interested in Rust", "follows ML research" |
| `workflow` | How the user works | "uses vim", "tests before committing" |
| `values` | Personal or professional values | "values privacy", "prefers open source" |

---

## 5. detect_contradiction() Contract

### When Called

In the current implementation (`alaya 0.1.0`), `detect_contradiction()` is defined on the trait but **not called by any lifecycle process**. It exists in the trait definition as a forward-looking contract for planned contradiction resolution during the transformation phase.

The intended call site (planned for a near-future release):

```
transform() called by consumer
  |
  +-> Deduplicate semantic nodes (existing, uses embedding similarity)
  |
  +-> [PLANNED] Contradiction detection:
  |     For each pair of semantic nodes with overlapping source episodes:
  |       +-> provider.detect_contradiction(&node_a, &node_b)
  |       +-> If true: resolve contradiction (keep higher confidence, archive lower)
  |
  +-> Prune weak links
  +-> Decay preferences
  +-> Prune old impressions
```

**Important**: Even though this method is not currently called by Alaya, consumers implementing `ConsolidationProvider` must provide it (it is a required trait method, not a default method). The `NoOpProvider` returns `Ok(false)` (no contradiction detected), which is the safe default.

### Input: `(&SemanticNode, &SemanticNode)`

Two semantic nodes that Alaya suspects might contradict each other.

```rust
pub struct SemanticNode {
    pub id: NodeId,                     // Unique ID
    pub content: String,                // The knowledge statement
    pub node_type: SemanticType,        // Fact, Relationship, Event, Concept
    pub confidence: f32,                // Extraction confidence
    pub source_episodes: Vec<EpisodeId>, // Supporting episodes
    pub created_at: i64,                // When extracted
    pub last_corroborated: i64,         // When last corroborated
    pub corroboration_count: u32,       // How many times corroborated
}
```

### Output: `bool`

Return `true` if the two nodes contradict each other, `false` otherwise.

**Semantic contract**: A contradiction means the two statements cannot both be true simultaneously. Examples:

| Node A | Node B | Contradiction? |
|---|---|---|
| "User works at Acme Corp" | "User works at Globex Inc" | Yes (if these are current-state facts) |
| "User prefers Python" | "User is learning Rust" | No (preferences and activities can coexist) |
| "User graduated in 2019" | "User graduated in 2020" | Yes (single event, two dates) |
| "User likes coffee" | "User prefers tea" | Depends on interpretation -- consumer decides |

**Returning `false` is always safe.** It means no contradiction is detected, and both nodes are kept. False negatives result in redundant nodes (handled by deduplication). False positives would result in node deletion (currently no deletion path exists, but will when contradiction resolution is implemented).

---

## 6. NoOpProvider Reference Implementation

### Source of Truth: `src/provider.rs`

```rust
/// A no-op provider for when no LLM is available.
/// Consolidation and perfuming simply skip the LLM-dependent steps.
pub struct NoOpProvider;

impl ConsolidationProvider for NoOpProvider {
    fn extract_knowledge(&self, _episodes: &[Episode]) -> Result<Vec<NewSemanticNode>> {
        Ok(vec![])
    }

    fn extract_impressions(&self, _interaction: &Interaction) -> Result<Vec<NewImpression>> {
        Ok(vec![])
    }

    fn detect_contradiction(&self, _a: &SemanticNode, _b: &SemanticNode) -> Result<bool> {
        Ok(false)
    }
}
```

### Why NoOpProvider Exists

NoOpProvider is not a testing convenience. It is a first-class citizen of the architecture, serving three purposes:

1. **Graceful degradation**: Consumers who cannot or do not want to provide LLM-backed extraction can still call `consolidate()` and `perfume()` safely. The calls succeed, return typed reports with zero counts, and the system operates in BM25-only mode.

2. **Incremental adoption**: A consumer can start with `NoOpProvider`, get store/query working, then swap in a real provider later. The upgrade path is: implement the trait, pass a reference instead of `&NoOpProvider`.

3. **MCP server default**: The planned MCP server (v0.2) exposes lifecycle tools (`alaya_dream`) that internally call `consolidate()`. When no LLM provider is configured, `NoOpProvider` is used, and the MCP tool still succeeds.

### Behavioral Guarantees

| Method | NoOp Return | Effect on System |
|---|---|---|
| `extract_knowledge()` | `Ok(vec![])` | No semantic nodes created. No links created. `ConsolidationReport { episodes_processed: N, nodes_created: 0, links_created: 0 }` |
| `extract_impressions()` | `Ok(vec![])` | No impressions stored. No domains checked for crystallization. `PerfumingReport { impressions_stored: 0, preferences_crystallized: 0, preferences_reinforced: 0 }` |
| `detect_contradiction()` | `Ok(false)` | No contradiction detected. Both nodes kept. |

### When to Use NoOpProvider

- During development and testing
- When no LLM is available (offline mode, edge deployment)
- When you want lifecycle processes to run (for forgetting and transformation benefits) without the cost of LLM calls
- As a fallback when your real provider fails (see [Error Handling Contract](#8-error-handling-contract))

---

## 7. Provider Lifecycle: When Each Method Is Called

### The Full Lifecycle Sequence

Alaya defines four lifecycle processes. Only two of them call provider methods:

| Process | Method | Calls Provider? | What It Does |
|---|---|---|---|
| `consolidate(provider)` | `AlayaStore::consolidate()` | Yes: `extract_knowledge()` | Episodic to semantic transfer (CLS replay) |
| `perfume(interaction, provider)` | `AlayaStore::perfume()` | Yes: `extract_impressions()` | Impression extraction and preference crystallization |
| `transform()` | `AlayaStore::transform()` | No (planned: `detect_contradiction()`) | Dedup, prune, decay |
| `forget()` | `AlayaStore::forget()` | No | Bjork RS decay, archive weak nodes |

### consolidate() Process Flow

```
Consumer calls: store.consolidate(&provider)
                         |
                         v
            +----------------------------+
            | Fetch unconsolidated       |
            | episodes (LIMIT 10)        |
            +----------------------------+
                         |
                    episodes.len() < 3?
                    /              \
                  Yes               No
                   |                 |
            Return empty       Call provider
            report             .extract_knowledge(&episodes)
                                     |
                                     v
                         +----------------------------+
                         | For each NewSemanticNode:  |
                         |   1. Store in semantic_nodes|
                         |   2. Create Causal links   |
                         |      to source episodes    |
                         |      (weight = 0.7)        |
                         |   3. Init node strength    |
                         |      (SS=0.5, RS=1.0)      |
                         +----------------------------+
                                     |
                                     v
                          Return ConsolidationReport
```

**Key details from `src/lifecycle/consolidation.rs`:**

- `CONSOLIDATION_BATCH_SIZE = 10`: Maximum episodes fetched per call.
- Minimum 3 episodes required. This ensures corroboration -- knowledge extracted from a single episode is less reliable.
- Episodes are fetched via `episodic::get_unconsolidated_episodes()`, which returns episodes not yet processed by consolidation.
- Links are created with `LinkType::Causal` and initial weight `0.7`.
- Node strength is initialized with storage strength `0.5` and retrieval strength `1.0`.

**Calling consolidate() multiple times**: Each call processes the *next* batch of unconsolidated episodes. If you have 25 unconsolidated episodes and call `consolidate()` three times, it will process batches of 10, 10, and 5 (assuming the third batch meets the minimum of 3).

### perfume() Process Flow

```
Consumer calls: store.perfume(&interaction, &provider)
                         |
                         v
            +----------------------------+
            | Call provider              |
            | .extract_impressions(      |
            |   &interaction)            |
            +----------------------------+
                         |
                         v
            +----------------------------+
            | Store each impression      |
            | in impressions table       |
            +----------------------------+
                         |
                         v
            +----------------------------+
            | Collect unique domains     |
            | from returned impressions  |
            +----------------------------+
                         |
                         v
            For each domain:
                |
            count_impressions_by_domain(domain)
                |
            count >= 5?
            /         \
          No           Yes
           |             |
         skip      existing preference?
                   /              \
                 Yes               No
                  |                 |
            Reinforce         Crystallize new
            existing          preference from
            preference        recent impressions
```

**Key details from `src/lifecycle/perfuming.rs`:**

- `CRYSTALLIZATION_THRESHOLD = 5`: Five impressions in a domain before crystallization.
- Crystallization fetches the 20 most recent impressions in the domain.
- New preference confidence is calculated as `min(count / 20, 0.9)` -- it asymptotically approaches `0.9` as evidence accumulates but never reaches `1.0`.
- The `summarize_impressions()` function is an internal Alaya heuristic (currently picks the most recent observation). This is NOT a provider method -- your provider extracts raw impressions, Alaya handles crystallization.
- Once a preference exists in a domain, subsequent perfuming calls for that domain reinforce rather than create new preferences.
- Preference reinforcement increments `evidence_count` and updates `last_reinforced`.

### transform() Process Flow (No Provider, but Planned)

```
Consumer calls: store.transform()
                         |
                         v
        1. Dedup semantic nodes (embedding similarity >= 0.95)
              - Keep older node, delete duplicate
              - Transfer links from duplicate to kept node
              - Increment corroboration_count on kept node
                         |
                         v
        2. [PLANNED] Contradiction detection via provider
                         |
                         v
        3. Prune weak graph links (forward_weight < 0.02)
                         |
                         v
        4. Decay un-reinforced preferences (half-life = 30 days)
                         |
                         v
        5. Prune weak preferences (confidence < 0.05)
                         |
                         v
        6. Prune old impressions (age > 90 days)
                         |
                         v
                  Return TransformationReport
```

### forget() Process Flow (No Provider)

```
Consumer calls: store.forget()
                         |
                         v
        1. Decay all retrieval strengths (RS *= 0.95)
                         |
                         v
        2. Find archivable nodes:
              SS < 0.1 AND RS < 0.05
                         |
                         v
        3. Delete archivable episodes and semantic nodes
              (preferences excluded from forgetting)
                         |
                         v
        4. Clean up strength records
                         |
                         v
                  Return ForgettingReport
```

### Recommended Calling Patterns

The consumer controls when lifecycle processes run. Here are recommended patterns based on agent type:

**Pattern 1: Post-Interaction Perfuming, Periodic Consolidation**

```rust
// Every conversation turn:
store.store_episode(&new_episode)?;
store.perfume(&interaction, &provider)?;  // Extract impressions immediately

// Every N episodes (e.g., 10-50):
store.consolidate(&provider)?;  // Batch knowledge extraction
store.forget()?;                // Decay retrieval strengths
store.transform()?;             // Dedup, prune, clean
```

**Pattern 2: Dream Cycle (Batch Everything)**

```rust
// During idle period (end of session, scheduled task):
fn dream(store: &AlayaStore, provider: &dyn ConsolidationProvider) -> Result<()> {
    loop {
        let report = store.consolidate(provider)?;
        if report.episodes_processed == 0 { break; }
    }
    store.forget()?;
    store.transform()?;
    Ok(())
}
```

**Pattern 3: Cost-Controlled (Minimize LLM Calls)**

```rust
// Only consolidate when explicitly requested or on schedule:
if should_consolidate() {
    store.consolidate(&llm_provider)?;
} else {
    store.consolidate(&NoOpProvider)?;  // Still processes lifecycle, no LLM cost
}

// Always run non-provider lifecycle:
store.forget()?;
store.transform()?;
```

---

## 8. Error Handling Contract

### The AlayaError::Provider Variant

```rust
#[derive(Debug, Error)]
pub enum AlayaError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("provider error: {0}")]
    Provider(String),
}
```

The `Provider` variant is the designated error type for provider failures. When your implementation encounters an error (LLM API timeout, rate limit, parsing failure), return `Err(AlayaError::Provider("descriptive message".into()))`.

### Error Propagation

Provider errors propagate through the lifecycle function via `?`. When `extract_knowledge()` returns `Err`, `consolidate()` immediately returns that same `Err` to the consumer. **No episodes are marked as consolidated.** The batch remains unconsolidated and will be retried on the next `consolidate()` call.

```
Provider returns Err(AlayaError::Provider("LLM timeout"))
  |
  +-> consolidate() returns Err(AlayaError::Provider("LLM timeout"))
        |
        +-> Consumer decides: retry, fall back to NoOp, or log and continue
```

**No partial writes**: If `extract_knowledge()` returns a successful result but Alaya fails to store one of the nodes (e.g., database error), the function returns `Err(AlayaError::Db(...))`. In the current implementation, this means earlier nodes in the batch may have been stored while later ones were not. This is a known limitation -- see [Known Gaps](#known-gaps) below.

### What Happens When a Provider Panics

If your provider panics (e.g., `unwrap()` on `None`, index out of bounds), the panic unwinds through Alaya's call stack normally. Alaya does not catch panics. The `consolidate()` or `perfume()` call panics at the consumer's call site.

**Recommendation**: Provider implementations should never panic. Use `?` with `AlayaError::Provider` for all fallible operations. If you call code that might panic, wrap it in `std::panic::catch_unwind()` and convert to an error:

```rust
fn extract_knowledge(&self, episodes: &[Episode]) -> Result<Vec<NewSemanticNode>> {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        self.do_extraction(episodes)
    }));
    match result {
        Ok(inner) => inner,
        Err(_) => Err(AlayaError::Provider("extraction panicked".into())),
    }
}
```

### What Happens When a Provider Returns Garbage

Alaya does **not** currently validate provider output at runtime. If your provider returns a `NewSemanticNode` with:

- Empty `content`: stored as-is. Retrievable via BM25 (matching empty string) and graph traversal. Semantically useless but not harmful.
- `confidence` of `5.0` (out of `[0.0, 1.0]` range): stored as-is. May distort retrieval ranking. Will not be pruned by transformation (which checks `< 0.05`).
- `source_episodes` referencing nonexistent IDs: Alaya creates `Causal` links to those IDs. The links are structurally valid in the graph but point to nothing. They will never be traversed because the target node is absent from all stores.
- `NaN` confidence: stored as-is. `NaN` comparisons behave unexpectedly in SQLite and Rust; all ordering operations with `NaN` produce undefined-but-not-crashing behavior.

**Planned provider output validation (v0.1 hardening):**

The security architecture specifies these validations as planned for v0.1:

| Validation | Rule | Effect |
|---|---|---|
| Non-empty content | `content.trim().is_empty()` returns error | Reject node |
| Content length | `content.len() > MAX_CONTENT_LENGTH` returns error | Reject node |
| Source episode existence | Verify each `EpisodeId` exists in DB | Reject or strip invalid refs |
| Confidence clamping | `confidence.clamp(0.0, 1.0)` | Silently clamp |
| Max nodes per batch | `nodes.len() > MAX_NODES_PER_BATCH` | Truncate |
| Non-empty domain | `domain.trim().is_empty()` returns error | Reject impression |
| Valence clamping | `valence.clamp(-1.0, 1.0)` | Silently clamp |

**Until these validations are implemented, the provider is responsible for its own output quality.** Treat this as a contract: Alaya trusts the provider to return valid data, and the provider trusts Alaya to store and use it correctly.

### Fallback Strategy

A production provider should implement a fallback strategy:

```rust
struct FallbackProvider {
    primary: Box<dyn ConsolidationProvider>,
    fallback: NoOpProvider,
}

impl ConsolidationProvider for FallbackProvider {
    fn extract_knowledge(&self, episodes: &[Episode]) -> Result<Vec<NewSemanticNode>> {
        match self.primary.extract_knowledge(episodes) {
            Ok(nodes) => Ok(nodes),
            Err(e) => {
                log::warn!("Primary provider failed: {}, falling back to NoOp", e);
                self.fallback.extract_knowledge(episodes)
            }
        }
    }
    // ... same pattern for other methods
}
```

---

## 9. Contract Validation

### Current State (v0.1.0)

Alaya performs **no runtime validation** of provider output. This section documents both the current behavior and the planned validation regime.

### Structural Validity (Checked by Rust Type System)

The Rust type system enforces structural validity at compile time:

| Constraint | Enforcement |
|---|---|
| `content` is a `String` | Cannot be null (Option-less) |
| `node_type` is a `SemanticType` enum | Must be one of four variants |
| `confidence` is `f32` | Must be a valid float (but `NaN` and `Inf` are valid `f32` values) |
| `source_episodes` is `Vec<EpisodeId>` | Must be a vector of typed IDs |
| `domain` is a `String` | Cannot be null |
| `valence` is `f32` | Must be a valid float |

### Semantic Validity (Not Currently Checked)

These constraints are part of the contract but are not enforced by Alaya at runtime:

| Constraint | Contract | Current Behavior if Violated |
|---|---|---|
| `content` is non-empty | Should contain meaningful text | Stored as empty string; pollutes semantic store |
| `confidence` in `[0.0, 1.0]` | Should represent valid probability | Stored as-is; may distort ranking |
| `source_episodes` reference real IDs | Should reference episodes in the batch | Dangling links created in graph |
| `domain` is non-empty | Should name a behavioral domain | Stored as empty string; impression unreachable for crystallization |
| `valence` in `[-1.0, 1.0]` | Should represent positive/negative signal | Stored as-is; currently unused in logic |
| Embedding dimensions consistent | All vectors same length | Cosine similarity returns 0.0 for mismatched lengths |

### Defensive Validation Pattern

Until Alaya implements runtime validation, consumers should validate their own output:

```rust
impl ConsolidationProvider for MyProvider {
    fn extract_knowledge(&self, episodes: &[Episode]) -> Result<Vec<NewSemanticNode>> {
        let raw_nodes = self.llm_extract(episodes)?;

        // Validate and clean provider output
        let valid_ids: HashSet<EpisodeId> = episodes.iter().map(|e| e.id).collect();
        let validated: Vec<NewSemanticNode> = raw_nodes
            .into_iter()
            .filter(|n| !n.content.trim().is_empty())
            .map(|mut n| {
                n.confidence = n.confidence.clamp(0.0, 1.0);
                n.source_episodes.retain(|id| valid_ids.contains(id));
                if n.source_episodes.is_empty() {
                    // Use all episodes as source if LLM didn't specify
                    n.source_episodes = episodes.iter().map(|e| e.id).collect();
                }
                n
            })
            .collect();

        Ok(validated)
    }
    // ...
}
```

### Embedding Dimension Validation

If your provider returns embeddings, ensure dimensional consistency:

```rust
// All embeddings should use the same model and dimension
const EMBEDDING_DIM: usize = 384; // e.g., all-MiniLM-L6-v2

fn validate_embedding(emb: &Option<Vec<f32>>) -> bool {
    match emb {
        None => true,  // No embedding is always valid
        Some(v) => {
            v.len() == EMBEDDING_DIM
            && v.iter().all(|f| f.is_finite())       // No NaN or Inf
            && v.iter().any(|f| *f != 0.0)           // Not a zero vector
        }
    }
}
```

Mixed-dimension embeddings in the same database will not cause errors (cosine similarity returns `0.0` for mismatched lengths), but they will silently break vector retrieval and deduplication. All embeddings in a single Alaya database should come from the same model.

---

## 10. EmbeddingProvider Design (Planned, v0.2)

### Motivation

In v0.1, embeddings are manual: the consumer passes `Option<Vec<f32>>` in `NewEpisode.embedding` and `NewSemanticNode.embedding`. This works but requires the consumer to manage embedding generation, model loading, and dimensional consistency themselves.

In v0.2, a new `EmbeddingProvider` trait will allow Alaya to generate embeddings automatically during `store_episode()` and `consolidate()`.

### Planned Trait Design

```rust
/// The agent provides this trait to enable vector search and deduplication.
/// Like ConsolidationProvider, Alaya never loads an embedding model directly.
pub trait EmbeddingProvider {
    /// Generate an embedding vector for the given text.
    fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for a batch of texts (for efficiency).
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        // Default implementation: call embed() in a loop
        texts.iter().map(|t| self.embed(t)).collect()
    }

    /// Return the dimensionality of the embedding model.
    fn dimension(&self) -> usize;

    /// Return the model identifier (for metadata tracking).
    fn model_name(&self) -> &str;
}
```

### Planned Usage Points

| Call Site | When | What Gets Embedded |
|---|---|---|
| `store_episode()` | After storing episode in DB | Episode content |
| `consolidate()` | After creating semantic node | Semantic node content |
| `query()` | Before vector search | Query text (if no embedding provided) |
| `store_episode()` | After storing (if auto-embed enabled) | Episode content, even without manual embedding |

### Planned Feature Flags

```toml
# Cargo.toml (planned)
[features]
embed-ort = ["ort"]           # ONNX Runtime backend
embed-fastembed = ["fastembed"] # Turnkey fastembed-rs backend
vec-sqlite = ["sqlite-vec"]    # SIMD vector search (replaces brute-force)
```

### NoOpEmbeddingProvider

Following the same pattern as `NoOpProvider`:

```rust
pub struct NoOpEmbeddingProvider;

impl EmbeddingProvider for NoOpEmbeddingProvider {
    fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![])  // Empty embedding: vector search returns no results
    }

    fn dimension(&self) -> usize { 0 }
    fn model_name(&self) -> &str { "noop" }
}
```

### AlayaConfig Integration (Planned)

```rust
// Planned v0.2 API:
let config = AlayaConfig::builder()
    .path("memory.db")
    .consolidation_provider(my_llm_provider)
    .embedding_provider(my_embedding_provider)
    .build();

let store = AlayaStore::from_config(config)?;

// Or keep v0.1 style:
let store = AlayaStore::open("memory.db")?;
```

### Semver Implications

Adding `EmbeddingProvider` as a separate trait (not adding methods to `ConsolidationProvider`) means it is a **non-breaking change**. Existing consumers who implement `ConsolidationProvider` do not need to change anything. Consumers who want embeddings implement the new trait in addition. This is a minor version bump (`0.1 -> 0.2`).

---

## 11. Testing Provider Implementations

### MockProvider (Source of Truth: `src/provider.rs`)

Alaya provides a `MockProvider` under `#[cfg(test)]` for testing:

```rust
#[cfg(test)]
pub struct MockProvider {
    pub knowledge: Vec<NewSemanticNode>,
    pub impressions: Vec<NewImpression>,
}

#[cfg(test)]
impl MockProvider {
    pub fn empty() -> Self {
        Self { knowledge: vec![], impressions: vec![] }
    }

    pub fn with_knowledge(knowledge: Vec<NewSemanticNode>) -> Self {
        Self { knowledge, impressions: vec![] }
    }

    pub fn with_impressions(impressions: Vec<NewImpression>) -> Self {
        Self { knowledge: vec![], impressions }
    }
}

#[cfg(test)]
impl ConsolidationProvider for MockProvider {
    fn extract_knowledge(&self, _episodes: &[Episode]) -> Result<Vec<NewSemanticNode>> {
        Ok(self.knowledge.clone())
    }

    fn extract_impressions(&self, _interaction: &Interaction) -> Result<Vec<NewImpression>> {
        Ok(self.impressions.clone())
    }

    fn detect_contradiction(&self, _a: &SemanticNode, _b: &SemanticNode) -> Result<bool> {
        Ok(false)
    }
}
```

`MockProvider` returns canned results regardless of input. It is useful for testing Alaya's lifecycle logic but not for testing your provider's extraction logic.

### Testing Your Provider Implementation

When testing your own `ConsolidationProvider`, you need to verify:

1. **Structural validity**: Returned types are well-formed.
2. **Semantic validity**: Extracted knowledge is relevant and accurate.
3. **Error handling**: Provider handles LLM failures gracefully.
4. **Edge cases**: Empty episodes, very long content, non-English text, adversarial input.

```rust
#[cfg(test)]
mod provider_tests {
    use super::*;

    fn sample_episodes() -> Vec<Episode> {
        vec![
            Episode {
                id: EpisodeId(1),
                content: "I've been working with Rust for about two years now.".into(),
                role: Role::User,
                session_id: "test".into(),
                timestamp: 1000,
                context: EpisodeContext::default(),
            },
            Episode {
                id: EpisodeId(2),
                content: "My main project is a CLI tool for log analysis.".into(),
                role: Role::User,
                session_id: "test".into(),
                timestamp: 1100,
                context: EpisodeContext::default(),
            },
            Episode {
                id: EpisodeId(3),
                content: "I prefer using async Rust with tokio.".into(),
                role: Role::User,
                session_id: "test".into(),
                timestamp: 1200,
                context: EpisodeContext::default(),
            },
        ]
    }

    #[test]
    fn test_knowledge_extraction_returns_valid_nodes() {
        let provider = MyProvider::new(/* test config */);
        let episodes = sample_episodes();
        let nodes = provider.extract_knowledge(&episodes).unwrap();

        for node in &nodes {
            // Content is non-empty
            assert!(!node.content.trim().is_empty(), "node content must not be empty");

            // Confidence is in valid range
            assert!(
                (0.0..=1.0).contains(&node.confidence),
                "confidence {} out of range", node.confidence
            );

            // Source episodes reference actual episodes
            let valid_ids: HashSet<EpisodeId> = episodes.iter().map(|e| e.id).collect();
            for src in &node.source_episodes {
                assert!(
                    valid_ids.contains(src),
                    "source episode {:?} not in batch", src
                );
            }

            // Embedding dimensions are consistent (if provided)
            if let Some(ref emb) = node.embedding {
                assert!(
                    emb.iter().all(|f| f.is_finite()),
                    "embedding contains NaN or Inf"
                );
            }
        }
    }

    #[test]
    fn test_empty_episodes_returns_empty() {
        let provider = MyProvider::new(/* test config */);
        let nodes = provider.extract_knowledge(&[]).unwrap();
        assert!(nodes.is_empty(), "empty input should produce empty output");
    }

    #[test]
    fn test_impression_extraction_has_valid_domains() {
        let provider = MyProvider::new(/* test config */);
        let interaction = Interaction {
            text: "Can you please be more concise?".into(),
            role: Role::User,
            session_id: "test".into(),
            timestamp: 1000,
            context: EpisodeContext::default(),
        };

        let impressions = provider.extract_impressions(&interaction).unwrap();
        for imp in &impressions {
            assert!(!imp.domain.trim().is_empty(), "domain must not be empty");
            assert!(!imp.observation.trim().is_empty(), "observation must not be empty");
            assert!(
                (-1.0..=1.0).contains(&imp.valence),
                "valence {} out of range", imp.valence
            );
        }
    }

    #[test]
    fn test_provider_error_handling() {
        let provider = MyProvider::new_with_broken_llm(/* ... */);
        let result = provider.extract_knowledge(&sample_episodes());

        // Should return Err, not panic
        assert!(result.is_err());
        match result.unwrap_err() {
            AlayaError::Provider(msg) => {
                assert!(!msg.is_empty(), "error message should be descriptive");
            }
            other => panic!("expected AlayaError::Provider, got: {:?}", other),
        }
    }
}
```

### Integration Testing with AlayaStore

Test your provider through the full lifecycle to verify end-to-end behavior:

```rust
#[test]
fn test_full_lifecycle_with_provider() {
    let store = AlayaStore::open_in_memory().unwrap();
    let provider = MyProvider::new(/* ... */);

    // Store enough episodes to trigger consolidation (minimum 3)
    for i in 0..5 {
        store.store_episode(&NewEpisode {
            content: format!("test message {}", i),
            role: Role::User,
            session_id: "s1".into(),
            timestamp: 1000 + i * 100,
            context: EpisodeContext::default(),
            embedding: None,
        }).unwrap();
    }

    // Consolidate should create semantic nodes via provider
    let cr = store.consolidate(&provider).unwrap();
    assert!(cr.episodes_processed >= 3);
    // Whether nodes_created > 0 depends on your provider

    // Verify semantic nodes exist
    let status = store.status().unwrap();
    assert_eq!(status.semantic_node_count, cr.nodes_created as u64);

    // Verify knowledge is retrievable
    if cr.nodes_created > 0 {
        let knowledge = store.knowledge(None).unwrap();
        assert!(!knowledge.is_empty());
    }

    // Test perfuming
    let interaction = Interaction {
        text: "Please use code examples in your responses".into(),
        role: Role::User,
        session_id: "s1".into(),
        timestamp: 2000,
        context: EpisodeContext::default(),
    };
    let pr = store.perfume(&interaction, &provider).unwrap();
    // Whether impressions_stored > 0 depends on your provider

    // Run non-provider lifecycle
    store.forget().unwrap();
    store.transform().unwrap();
}
```

### Property-Based Testing

For providers backed by deterministic logic (not LLM), property-based testing with `proptest` or `quickcheck` can verify invariants:

```rust
// Example with proptest (not included in Alaya's dependencies)
proptest! {
    #[test]
    fn knowledge_confidence_is_valid(
        content in "[a-zA-Z ]{10,200}",
    ) {
        let provider = MyHeuristicProvider::new();
        let episodes = vec![Episode {
            id: EpisodeId(1),
            content,
            role: Role::User,
            session_id: "test".into(),
            timestamp: 1000,
            context: EpisodeContext::default(),
        }];

        if let Ok(nodes) = provider.extract_knowledge(&episodes) {
            for node in nodes {
                prop_assert!(node.confidence >= 0.0 && node.confidence <= 1.0);
                prop_assert!(!node.content.trim().is_empty());
            }
        }
    }
}
```

---

## 12. Migration Guide

### Semver Policy

Alaya follows semantic versioning. The `ConsolidationProvider` trait is part of the public API. Changes to the trait follow these rules:

| Change Type | Semver | Example |
|---|---|---|
| New required method on trait | **Major** (breaking) | Adding `summarize()` to `ConsolidationProvider` |
| New trait (separate) | Minor (non-breaking) | Adding `EmbeddingProvider` trait |
| Change method signature | **Major** (breaking) | Changing `extract_knowledge(&[Episode])` to `extract_knowledge(&[Episode], &Config)` |
| New default method on trait | Minor (non-breaking) | Adding `fn priority(&self) -> u8 { 0 }` |
| New field on input struct | Minor (non-breaking, if struct is `#[non_exhaustive]`) | Adding `metadata` field to `Episode` |
| New enum variant | Minor (non-breaking, if enum is `#[non_exhaustive]`) | Adding `SemanticType::Procedure` |

### Current Stability Guarantees (v0.1.0)

Alaya is pre-1.0. The trait API may change between minor versions. However, the team commits to:

1. **Changelog documentation** for all trait changes.
2. **Migration examples** for every breaking change.
3. **Deprecation warnings** before removal (when possible).

### Known Planned Changes

**v0.1.x (patch releases):**

- No trait changes. Bug fixes and validation only.
- `#[non_exhaustive]` will be added to `Role`, `SemanticType`, `LinkType`, `PurgeFilter`, `AlayaError`. This is technically a breaking change (exhaustive `match` arms will need `_ =>` wildcards), but it is a correctness improvement that aligns with Rust API guidelines.

**v0.2.0 (minor release):**

- New `EmbeddingProvider` trait (non-breaking: new trait, not a change to existing trait).
- `AlayaConfig` builder pattern (non-breaking: new entry point alongside `AlayaStore::open()`).
- Possible: `detect_contradiction()` called from `transform()` (non-breaking: method already exists on trait, Alaya simply starts calling it).

**v0.3.0 and beyond:**

- Possible: change `detect_contradiction()` to return richer information (e.g., `ContradictionResult` struct instead of `bool`). This would be a breaking change.
- Possible: change `extract_knowledge()` to accept a `ConsolidationContext` struct alongside episodes, providing batch metadata. This would be a breaking change.

### Upgrading Your Provider

When Alaya releases a version with trait changes:

1. Check the changelog for the specific change.
2. Update your `Cargo.toml` dependency.
3. Run `cargo build` -- the compiler will tell you exactly what needs to change.
4. Update method signatures to match the new trait definition.
5. Run your provider tests (see [Section 11](#11-testing-provider-implementations)).

---

## Appendix A: Type Reference

### Input Types (What Alaya Passes to Providers)

```rust
// Passed to extract_knowledge()
pub struct Episode {
    pub id: EpisodeId,           // i64 newtype
    pub content: String,
    pub role: Role,              // User | Assistant | System
    pub session_id: String,
    pub timestamp: i64,          // Unix seconds
    pub context: EpisodeContext,
}

pub struct EpisodeContext {
    pub topics: Vec<String>,
    pub sentiment: f32,
    pub conversation_turn: u32,
    pub mentioned_entities: Vec<String>,
    pub preceding_episode: Option<EpisodeId>,
}

// Passed to extract_impressions()
pub struct Interaction {
    pub text: String,
    pub role: Role,
    pub session_id: String,
    pub timestamp: i64,
    pub context: EpisodeContext,
}

// Passed to detect_contradiction()
pub struct SemanticNode {
    pub id: NodeId,              // i64 newtype
    pub content: String,
    pub node_type: SemanticType, // Fact | Relationship | Event | Concept
    pub confidence: f32,
    pub source_episodes: Vec<EpisodeId>,
    pub created_at: i64,
    pub last_corroborated: i64,
    pub corroboration_count: u32,
}
```

### Output Types (What Providers Return to Alaya)

```rust
// Returned by extract_knowledge()
pub struct NewSemanticNode {
    pub content: String,
    pub node_type: SemanticType,
    pub confidence: f32,
    pub source_episodes: Vec<EpisodeId>,
    pub embedding: Option<Vec<f32>>,
}

// Returned by extract_impressions()
pub struct NewImpression {
    pub domain: String,
    pub observation: String,
    pub valence: f32,
}

// Returned by detect_contradiction()
// bool: true = contradiction, false = no contradiction
```

### Report Types (Lifecycle Process Results)

```rust
pub struct ConsolidationReport {
    pub episodes_processed: u32,
    pub nodes_created: u32,
    pub links_created: u32,
}

pub struct PerfumingReport {
    pub impressions_stored: u32,
    pub preferences_crystallized: u32,
    pub preferences_reinforced: u32,
}

pub struct TransformationReport {
    pub duplicates_merged: u32,
    pub links_pruned: u32,
    pub preferences_decayed: u32,
    pub impressions_pruned: u32,
}

pub struct ForgettingReport {
    pub nodes_decayed: u32,
    pub nodes_archived: u32,
}
```

---

## Appendix B: Known Gaps

These are acknowledged limitations in the current provider contract implementation, tracked for resolution:

| Gap | Impact | Planned Fix | Version |
|---|---|---|---|
| No provider output validation | Garbage in, garbage stored | Validation layer in lifecycle functions | v0.1.x |
| `detect_contradiction()` never called | Dead code in trait | Wire into `transform()` | v0.2.0 |
| No transactional batch processing in `consolidate()` | Partial writes on mid-batch DB error | Wrap node storage loop in a transaction | v0.1.x |
| `summarize_impressions()` is a trivial heuristic | Preference text quality depends on most-recent impression | Improve heuristic or delegate to provider | v0.2.0 |
| No provider call metrics | Consumers cannot measure extraction latency | Add timing to lifecycle reports | v0.2.0 |
| Unconsolidated episode tracking unclear | Consumer cannot tell which episodes have been consolidated | Add `consolidated` flag or tracking table | v0.1.x |
| No `#[non_exhaustive]` on public enums | Adding variants is breaking | Add attribute | v0.1.x |
| LTD (long-term depression) not called from retrieval | Weak links not weakened on irrelevant retrieval | Wire into retrieval pipeline | v0.2.0 |

---

## Appendix C: Quick Reference Card

```
TRAIT: ConsolidationProvider
  |
  +-- extract_knowledge(&[Episode]) -> Result<Vec<NewSemanticNode>>
  |     Called by: consolidate()
  |     When: Batch of 3-10 unconsolidated episodes available
  |     NoOp: returns Ok(vec![])
  |     Error: propagates to caller, batch retried on next call
  |
  +-- extract_impressions(&Interaction) -> Result<Vec<NewImpression>>
  |     Called by: perfume()
  |     When: Every perfume() call
  |     NoOp: returns Ok(vec![])
  |     Error: propagates to caller, interaction not processed
  |
  +-- detect_contradiction(&SemanticNode, &SemanticNode) -> Result<bool>
        Called by: [not currently called, planned for transform()]
        NoOp: returns Ok(false)
        Error: would propagate to caller

LIFECYCLE FLOW:
  store_episode() --> accumulate episodes
  consolidate(&provider) --> extract_knowledge() --> semantic nodes + graph links
  perfume(&interaction, &provider) --> extract_impressions() --> impressions --> preferences
  transform() --> dedup, prune, decay (no provider, planned: detect_contradiction)
  forget() --> RS decay, archive weak nodes (no provider)

CONSTANTS:
  CONSOLIDATION_BATCH_SIZE = 10
  MINIMUM_EPISODES_FOR_CONSOLIDATION = 3
  CRYSTALLIZATION_THRESHOLD = 5 impressions per domain
  INITIAL_LINK_WEIGHT (causal, from consolidation) = 0.7
  INITIAL_STORAGE_STRENGTH = 0.5
  INITIAL_RETRIEVAL_STRENGTH = 1.0
  PREFERENCE_CONFIDENCE_CAP = 0.9
  RS_DECAY_FACTOR = 0.95
  ARCHIVE_SS_THRESHOLD = 0.1
  ARCHIVE_RS_THRESHOLD = 0.05
  LINK_PRUNE_THRESHOLD = 0.02
  DEDUP_SIMILARITY_THRESHOLD = 0.95
  PREFERENCE_HALF_LIFE = 30 days
  MAX_IMPRESSION_AGE = 90 days
  MIN_PREFERENCE_CONFIDENCE = 0.05
```
