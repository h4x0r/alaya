# Alaya Developer Journeys

This document maps how Rust developers discover, evaluate, adopt, integrate, and deepen their usage of the Alaya memory library. Alaya is a crate, not a web application. There is no signup flow, no dashboard, no authentication. The "users" are developers. The "product" is the API surface, the documentation, and the compile-test-ship loop.

Every journey below is grounded in the actual `AlayaStore` API, the `ConsolidationProvider` trait system, and the single-SQLite-file architecture. Code examples use the current public API.

**Cross-references:** [Brand Guidelines](../BRAND_GUIDELINES.md) | [North Star](../NORTHSTAR.md) | [Competitive Landscape](../COMPETITIVE_LANDSCAPE.md) | [North Star Extract](../NORTHSTAR_EXTRACT.md)

---

## Table of Contents

1. [Journey 1: First-Time Developer (Discovery to First Value)](#journey-1-first-time-developer)
2. [Journey 2: Deepening Integration (Basic to Production)](#journey-2-deepening-integration)
3. [Journey 3: Error Recovery](#journey-3-error-recovery)
4. [Journey 4: MCP Integration](#journey-4-mcp-integration)
5. [Journey 5: Persona Variations](#journey-5-persona-variations)
6. [Journey Metrics](#journey-metrics)
7. [DX Anti-Patterns to Avoid](#dx-anti-patterns-to-avoid)
8. [Implications for API Design](#implications-for-api-design)

---

## Journey 1: First-Time Developer

**Narrative:** A Rust developer building an AI agent needs conversational memory. They find Alaya through crates.io, a blog post, or a mention in the OpenClaw community. They evaluate the README, run `cargo add alaya`, paste the quickstart example, and see relevant retrieval results within two minutes. They decide to integrate Alaya into their own project.

### Phase Map

```
DISCOVERY        EVALUATION       INSTALLATION      FIRST CODE        VALUE
    |                |                |                 |              |
    v                v                v                 v              v
+--------+     +--------+      +--------+       +--------+     +--------+
|crates.io|---->| README |----->|cargo   |------>|Quick-  |---->|Working |
|/GitHub  |     |/docs.rs|      |add     |       |start   |     |Memory  |
|/blog    |     |        |      |alaya   |       |Example |     |System  |
+--------+     +--------+      +--------+       +--------+     +--------+

EMOTION:        EMOTION:        EMOTION:         EMOTION:       EMOTION:
Skeptical       Evaluating      Committed        Invested       Delighted
"Another         "Does this     "Let's try it"   "Does this    "It remembers
memory lib?"     actually                         compile?"     context"
                 work without
                 an LLM?"
```

### Phase 1: Discovery

**Entry points:**
- crates.io search for "memory agent" or "conversational memory"
- GitHub search or trending Rust repositories
- Blog post: "Building agent memory without cloud dependencies"
- r/rust or r/MachineLearning discussion thread
- OpenClaw contributor mentions Alaya in a design doc
- Conference talk or DEF CON AI Village presentation

**Developer's mental state:** Skeptical. They have tried Mem0 (requires LLM for every write), looked at Zep (requires Neo4j), considered LangChain Memory (Python, framework lock-in). They want something that works without infrastructure.

**What they see first:**
- crates.io: One-line description, download count, dependency list (just `rusqlite`, `serde`, `serde_json`, `thiserror`)
- GitHub: README with clear positioning statement, quickstart code block, architecture diagram
- Blog: Concrete benchmarks, honest about what works and what does not

**Success criteria:** The developer clicks through to the README within 60 seconds.

**Failure modes:**
- Unclear crate description on crates.io (too abstract, uses "AI-powered" language)
- README opens with philosophy instead of code
- No quickstart visible above the fold
- Dependency list looks heavy or includes unexpected crates

### Phase 2: Evaluation

**Duration target:** Under 3 minutes from README to installation decision.

**What the developer evaluates:**
1. **Dependencies** -- scrolls to `Cargo.toml` or checks crates.io sidebar. Sees four dependencies total. This is the first trust signal.
2. **API surface** -- scans the README quickstart. Sees `AlayaStore::open()`, `store_episode()`, `query()`. Recognizes familiar Rust patterns (builder, `Result<T>`, `impl AsRef<Path>`).
3. **Privacy claims** -- checks for network calls. Reads "zero network calls in core crate." Verifies by scanning `Cargo.toml` for `reqwest`, `hyper`, `tokio` in required dependencies. Finds none.
4. **LLM coupling** -- reads that basic operations work with `NoOpProvider`. No LLM needed for store/query/forget.
5. **Maturity signals** -- checks test count, CI status, documentation coverage, open issues.

**Key evaluation questions the README must answer:**

| Question | Where Answered | Target Impression |
|----------|---------------|-------------------|
| What does this do? | First paragraph | "Memory engine for AI agents, not a database" |
| What makes it different? | Positioning statement | "Cognitive lifecycle + zero deps + privacy" |
| Does it need an LLM? | Quickstart + NoOpProvider | "No -- works without one, enhanced with one" |
| Does it phone home? | Privacy section | "Zero network calls, single SQLite file" |
| Is the API sane? | Quickstart code | "Looks like idiomatic Rust" |
| Is it maintained? | CI badge, recent commits | "Active development, tests pass" |

**Success criteria:** Developer runs `cargo add alaya`.

**Failure modes:**
- README is a wall of theory with no code in the first screenful
- Claims that cannot be verified from the source (e.g., "blazing fast" with no benchmarks)
- Quickstart example requires an LLM API key
- API looks non-idiomatic (raw SQL exposure, unsafe blocks, C-style error codes)

### Phase 3: Installation

**Duration target:** Under 30 seconds.

**Steps:**
```bash
cargo add alaya
```

**What can go wrong:**
- Compilation errors from version conflicts with `rusqlite` (if developer already depends on a different version)
- Feature flag confusion (which flags do they need?)
- Long compile times from `libsqlite3-sys` building SQLite from source on first compile

**Mitigation:**
- Document minimum supported Rust version (MSRV)
- Keep feature flags minimal (4-6 maximum, per Extract constraints)
- Document compile time expectations honestly: first build includes SQLite compilation, subsequent builds are incremental
- Pin `rusqlite` version range carefully to minimize version conflict surface

**Success criteria:** `cargo build` succeeds without warnings on the developer's first try.

### Phase 4: First Code

**Duration target:** Under 2 minutes from successful build to running example.

**The quickstart the developer copies:**

```rust
use alaya::{AlayaStore, NewEpisode, Role, EpisodeContext, Query};

fn main() -> alaya::Result<()> {
    // One line to open. One file for everything.
    let store = AlayaStore::open("memory.db")?;

    // Store a conversation episode
    store.store_episode(&NewEpisode {
        content: "I prefer dark mode and Vim keybindings".to_string(),
        role: Role::User,
        session_id: "session-1".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        context: EpisodeContext::default(),
        embedding: None,
    })?;

    // Query by natural language -- BM25 retrieval, no LLM needed
    let results = store.query(&Query::simple("What editor settings?"))?;

    for result in &results {
        println!("[{:.2}] {}", result.score, result.content);
    }

    Ok(())
}
```

**What the developer expects:** The query returns the stored episode with a nonzero relevance score. They see that "Vim keybindings" was retrieved by a query about "editor settings" through BM25 text matching.

**Emotional arc through first code:**
1. Types the `use` statement -- sees clean module structure, no glob imports needed
2. `AlayaStore::open("memory.db")` -- recognizes `open` pattern from `File::open`, feels familiar
3. `NewEpisode { ... }` -- struct fields are self-documenting, no builder chain mystery
4. `Query::simple("...")` -- convenience constructor, not a 10-field struct
5. Results print -- nonzero score, content matches. "It works."

**Success criteria:** Query returns relevant results. Developer understands why.

**Failure modes:**
- Example does not compile due to missing imports or API changes since README was written
- `Query::simple()` returns empty results (BM25 needs enough token overlap)
- Error messages are opaque (`Db(SqliteError(...))` instead of actionable text)
- Developer does not understand what `EpisodeContext::default()` means or whether they need to fill it in

### Phase 5: Value Realization

**The moment:** The developer stores 10-20 episodes across two sessions, then queries. Results come back ranked by relevance, spanning both sessions. They realize this is not just key-value storage -- it is contextual retrieval across time.

**Follow-up actions the developer takes:**
1. Checks `store.status()` to see what is in the database
2. Tries `store.query()` with different phrasings to test retrieval quality
3. Opens `memory.db` with `sqlite3` CLI to inspect the schema (curiosity, not requirement)
4. Reads docs.rs for `AlayaStore` to see what else is available
5. Notices `consolidate()`, `forget()`, `perfume()` -- realizes there is a lifecycle beyond CRUD

**Value confirmed when:** The developer starts planning how to integrate Alaya into their own agent project, not just running examples.

---

## Journey 2: Deepening Integration

**Narrative:** The developer has a working quickstart. Now they integrate Alaya into their agent, add lifecycle processes, implement custom providers for LLM-powered consolidation, tune retrieval for their domain, and deploy to production.

### Phase Map

```
BASIC CRUD      LIFECYCLE       CUSTOM PROVIDERS   TUNING         PRODUCTION
    |               |                |                |              |
    v               v                v                v              v
+--------+     +--------+      +--------+       +--------+     +--------+
|store/  |---->|dream() |----->|Implement|------>|Bench-  |---->|Ship    |
|query   |     |cycle   |      |Provider |       |mark    |     |Agent   |
|/get    |     |        |      |traits   |       |& Tune  |     |        |
+--------+     +--------+      +--------+       +--------+     +--------+

TIMEFRAME:     TIMEFRAME:      TIMEFRAME:       TIMEFRAME:     TIMEFRAME:
Day 1          Day 2-3         Week 1           Week 2-3       Week 4+
```

### Phase 1: Basic CRUD Integration

**What happens:** Developer replaces their ad-hoc memory (Vec of strings, JSON files, raw SQLite) with Alaya. They wire `store_episode()` into their agent's conversation handler and `query()` into their context assembly pipeline.

**Integration pattern:**
```rust
// In the agent's conversation handler
fn handle_message(&self, msg: &UserMessage) -> Result<AgentResponse> {
    // Store the incoming message
    let ep_id = self.memory.store_episode(&NewEpisode {
        content: msg.text.clone(),
        role: Role::User,
        session_id: msg.session_id.clone(),
        timestamp: msg.timestamp,
        context: EpisodeContext {
            preceding_episode: self.last_episode_id,
            ..Default::default()
        },
        embedding: None,
    })?;

    // Retrieve relevant context for the response
    let context = self.memory.query(&Query::simple(&msg.text))?;

    // Build prompt with retrieved memories
    let prompt = self.build_prompt(msg, &context);
    let response = self.llm.generate(&prompt)?;

    // Store the agent's response too
    self.memory.store_episode(&NewEpisode {
        content: response.text.clone(),
        role: Role::Assistant,
        session_id: msg.session_id.clone(),
        timestamp: now(),
        context: EpisodeContext {
            preceding_episode: Some(ep_id),
            ..Default::default()
        },
        embedding: None,
    })?;

    Ok(response)
}
```

**Developer questions at this stage:**
- "Should I store every message or just important ones?" -- Store everything. Forgetting handles the rest.
- "What goes in `EpisodeContext`?" -- Preceding episode for temporal linking. Topics and entities if available.
- "How many results should I retrieve?" -- Start with 5 (`Query::simple` default). Tune later.
- "Do I need embeddings?" -- Not initially. BM25-only works. Add embeddings when you want semantic similarity beyond lexical match.

**Success criteria:** Agent retrieves relevant memories from previous conversations. User notices the agent "remembers."

### Phase 2: Lifecycle Integration (The "Dream Cycle")

**What happens:** Developer adds periodic lifecycle calls. This is where Alaya goes from "memory database" to "memory engine." The cognitive lifecycle runs between conversations, like the brain consolidating during sleep.

**The dream cycle pattern:**
```rust
/// Run between conversations or on a timer.
/// This is where memory transforms from raw episodes
/// into structured knowledge, preferences, and refined recall.
fn dream(&self) -> Result<DreamReport> {
    let provider = &self.consolidation_provider;

    // CLS replay: episodes -> semantic knowledge
    let consolidation = self.memory.consolidate(provider)?;

    // Bjork forgetting: decay retrieval strengths, archive weak nodes
    let forgetting = self.memory.forget()?;

    // Asraya-paravrtti: dedup, prune, structural transformation
    let transformation = self.memory.transform()?;

    Ok(DreamReport {
        nodes_created: consolidation.nodes_created,
        nodes_decayed: forgetting.nodes_decayed,
        nodes_archived: forgetting.nodes_archived,
        dedup_merged: transformation.nodes_merged,
    })
}
```

**Developer realizes at this stage:**
- Consolidation extracts knowledge from episodes ("user prefers dark mode" becomes a semantic node, not just a stored message)
- Forgetting is not data loss -- it is retrieval strength decay. High-storage, low-retrieval nodes become latent, not deleted
- Transformation deduplicates and prunes, keeping the knowledge graph clean
- Each process returns a typed report -- the agent can log, display, or act on results

**Common mistake:** Running lifecycle processes after every single message. They are designed for batch operation. Consolidation needs at least 3 episodes to find corroboration. Forgetting is a sweep, not a per-node operation.

**Success criteria:** After running dream cycles, `store.status()` shows semantic nodes and preferences emerging alongside episodes. Query results improve because the graph has structure.

### Phase 3: Custom Providers

**What happens:** The developer implements `ConsolidationProvider` to use their LLM for knowledge extraction and impression analysis. This is where Alaya's trait-based extension pattern pays off.

**The trait they implement:**
```rust
use alaya::{ConsolidationProvider, Episode, NewSemanticNode, NewImpression, Interaction};

struct MyLLMProvider {
    client: MyLLMClient,
}

impl ConsolidationProvider for MyLLMProvider {
    fn extract_knowledge(
        &self,
        episodes: &[Episode],
    ) -> alaya::Result<Vec<NewSemanticNode>> {
        // Send episodes to LLM with a structured extraction prompt.
        // Return facts, relationships, events, concepts.
        let prompt = build_extraction_prompt(episodes);
        let response = self.client.generate(&prompt)
            .map_err(|e| alaya::AlayaError::Provider(e.to_string()))?;
        parse_knowledge_nodes(&response)
    }

    fn extract_impressions(
        &self,
        interaction: &Interaction,
    ) -> alaya::Result<Vec<NewImpression>> {
        // Analyze interaction for implicit preferences.
        // "User asked for dark mode twice" -> Impression { domain: "ui", ... }
        let prompt = build_impression_prompt(interaction);
        let response = self.client.generate(&prompt)
            .map_err(|e| alaya::AlayaError::Provider(e.to_string()))?;
        parse_impressions(&response)
    }
}
```

**Key insight the developer gains:** Alaya does not own the LLM connection. The developer chooses which model, which API, which prompt format. Alaya provides the lifecycle orchestration and the storage. This is the Trait Extension Pattern from the Extract: core behavior in the library, optional enhancement via trait, agent provides implementation.

**Graceful degradation in action:** If the developer's LLM provider fails (API down, rate limited, timeout), Alaya does not crash. `NoOpProvider` returns empty knowledge and impressions. Episodes still accumulate. Consolidation runs again next cycle with more data.

**Success criteria:** LLM-extracted semantic nodes appear after consolidation. Preferences crystallize from accumulated impressions. Query results improve because the semantic layer now participates in retrieval.

### Phase 4: Tuning and Benchmarking

**What happens:** The developer measures retrieval quality against their domain, tunes parameters, and validates performance at scale.

**Tuning dimensions:**
- **Retrieval count:** Adjust `max_results` in `Query` based on context window budget
- **Query context:** Populate `QueryContext.topics` and `mentioned_entities` for better graph activation
- **Embedding quality:** Swap embedding models via `EmbeddingProvider` trait (when available)
- **Lifecycle frequency:** How often to run dream cycles (every N conversations, daily, on idle)
- **Forgetting aggressiveness:** How quickly retrieval strength decays (currently hardcoded at 0.95 decay factor -- this becomes configurable)

**Performance baseline the developer establishes:**
```
Query latency at 100 episodes:   < 1ms
Query latency at 1,000 episodes: < 5ms
Query latency at 10,000 episodes: < 50ms (BM25-only)
Memory footprint at 10,000 episodes: ~ 10-20 MB SQLite file
Cold open time: < 10ms
```

**Developer's checklist before production:**
- [ ] Query returns relevant results for their domain (manual spot-check, 20+ queries)
- [ ] Lifecycle processes run without error on their data shape
- [ ] SQLite file size is acceptable for their deployment target
- [ ] Error handling covers all `AlayaError` variants in their agent
- [ ] No panics in release mode (fuzzing or extended testing)

**Success criteria:** Developer has quantified retrieval quality and latency for their specific use case. Numbers are acceptable for their deployment target.

### Phase 5: Production Deployment

**What happens:** The developer ships their agent with Alaya as the memory backend. Their end users interact with the agent, and Alaya manages memory transparently.

**Deployment considerations:**
- **File location:** Where does `memory.db` live? Developer chooses: app data directory, user home, XDG path
- **Backup:** Single file to back up. `cp memory.db memory.db.bak` works
- **Migration:** Schema versioning handles upgrades when the developer bumps Alaya version in `Cargo.toml`
- **Concurrency:** Single-writer (SQLite WAL mode). If the agent is single-threaded per user, this is not a constraint. Multi-threaded access needs `Arc<Mutex<AlayaStore>>` or connection pooling
- **Monitoring:** `store.status()` provides counts. Lifecycle reports provide operational telemetry

**What production teaches the developer:**
- Real users generate messier data than test fixtures
- Forgetting becomes essential at scale (without it, query quality degrades from noise accumulation)
- Consolidation quality depends on LLM prompt engineering -- the developer iterates on their `ConsolidationProvider`
- Users notice when the agent remembers preferences unprompted -- this is the vasana payoff

**Success criteria:** End user says "it remembered that I prefer X" without being told to remember. This is the North Star signal for MACC (the end user does not know about Alaya, but the developer knows their user just validated the integration).

---

## Journey 3: Error Recovery

Every error the developer encounters is an opportunity to build or destroy trust. Alaya's error messages must be actionable, not just descriptive. The developer should be able to fix the problem from the error message alone in 90% of cases.

### Error Taxonomy

| Category | Error Type | Developer Sees | Emotional Impact | Recovery Path |
|----------|-----------|----------------|------------------|---------------|
| **Compilation** | Trait bound not satisfied | `the trait ConsolidationProvider is not implemented for MyProvider` | Confused -- "I thought I implemented it" | Clear trait docs with required method signatures and example implementation |
| **Compilation** | Missing field in struct | `missing field content in NewEpisode` | Minor frustration | Struct docs list all fields with descriptions; IDE autocomplete helps |
| **Compilation** | Version conflict | `rusqlite version conflict` | Annoyed -- "not my fault" | Document supported rusqlite version range; consider re-exporting rusqlite types |
| **Runtime** | Database open failure | `AlayaError::Db(...)` wrapping SQLite error | Worried -- "is my data gone?" | Error message includes path attempted, permission check suggestion, and note that existing data is not affected |
| **Runtime** | Schema migration | `SchemaVersionMismatch { found: 2, expected: 3 }` | Worried about data loss | Migration guide with backup instructions; automatic migration when safe; explicit opt-in for breaking migrations |
| **Runtime** | FTS5 syntax error | `AlayaError::Db(...)` from malformed MATCH | Annoyed -- "I just passed a string" | Auto-sanitization of FTS5 input (per Extract: "sanitize all FTS5 MATCH input"). Developer should never see this error |
| **Semantic** | Empty query results | `query() returns Vec::new()` | Disappointed -- "it doesn't work" | Not an error but feels like one. Debug mode with `QueryExplanation` showing BM25 scores, graph activation, fusion weights |
| **Semantic** | Irrelevant results | Top results have low scores | Frustrated -- "it returns garbage" | Score thresholds in documentation; guidance on when to add embeddings vs. BM25-only |
| **Performance** | Slow queries | >10ms at moderate scale | Frustrated -- "Rust is supposed to be fast" | Performance tuning guide: check embedding count, graph density, FTS5 index health. `ANALYZE` command |
| **Lifecycle** | Provider error | `AlayaError::Provider(msg)` | Confused -- "is this my code or Alaya's?" | Error message wraps the provider's error with context about which lifecycle phase failed and what the provider was asked to do |
| **Data** | Corrupt database | SQLite integrity check failure | Panicked -- "I lost everything" | Recovery guide: `.backup` command, WAL checkpoint, integrity check. Emphasize single-file backup strategy |

### Error Recovery Flows

#### Flow A: First Compilation Error

```
Developer writes:  let store = AlayaStore::open("test.db");
                                                            ^
Compiler says:     expected Result<AlayaStore>, found AlayaStore
                   help: consider using `?` or `.unwrap()`

Developer action:  Adds `?` operator, wraps main in Result
Time to fix:       30 seconds
Trust impact:      Neutral (standard Rust, not Alaya-specific)
```

#### Flow B: Empty Query Results

```
Developer stores:  "I like Vim keybindings"
Developer queries: "text editor preferences"
Results:           [] (empty)

Developer reaction: "It doesn't work."

Root cause: BM25 requires lexical overlap. "text editor preferences"
shares no tokens with "Vim keybindings."

Recovery options:
1. Query "Vim" or "keybindings" -- immediate results (teaches BM25 behavior)
2. Add embeddings via EmbeddingProvider -- semantic similarity bridges lexical gap
3. Future: QueryExplanation shows "BM25 score: 0.0, no token overlap"

Trust impact: HIGH RISK. This is the #1 moment developers abandon a memory library.
Mitigation: Documentation must set expectations about BM25 behavior upfront.
Quickstart example must use queries with lexical overlap to avoid this on first try.
```

#### Flow C: Provider Implementation Error

```
Developer implements ConsolidationProvider.
extract_knowledge() panics on unexpected LLM response format.

AlayaError::Provider("called `Result::unwrap()` on an `Err` value: ...")

Developer reaction: "Where did this panic come from?"

Recovery:
1. Error message includes "during consolidation.extract_knowledge()"
2. Stack trace points to developer's provider code, not Alaya internals
3. Alaya docs show example provider with proper error handling
4. NoOpProvider fallback means data is safe -- consolidation just skipped this batch

Trust impact: Medium. Developer understands it's their code, but the error path
needs to be clear about the boundary.
```

#### Flow D: Schema Migration on Version Upgrade

```
Developer bumps alaya from 0.1.0 to 0.2.0 in Cargo.toml.
Runs their agent. AlayaStore::open() returns:

AlayaError::SchemaVersionMismatch {
    found: 1,
    expected: 2,
    migration_available: true,
    backup_recommended: true,
}

Developer reaction: "Will this destroy my data?"

Recovery:
1. Error message says migration is available and reversible
2. Developer runs: store.migrate()? or AlayaStore::open_and_migrate(path)?
3. Alaya creates backup before migrating: memory.db.backup-v1
4. Migration runs in a transaction -- rolls back on failure
5. If migration is not available (breaking change), error says so explicitly
   with manual migration instructions

Trust impact: Critical moment. Handled well, builds deep trust.
Handled poorly (silent migration or data loss), developer never returns.
```

### Error Design Principles

1. **Every error message answers "what do I do now?"** -- not just "what went wrong"
2. **Boundary errors are labeled** -- "this error came from your ConsolidationProvider" vs. "this is an Alaya internal error"
3. **Data safety is always communicated** -- "your data is not affected" when true; explicit warning when it might be
4. **Compilation errors are preferable to runtime errors** -- use the Rust type system to prevent invalid states
5. **Silent failures are worse than loud errors** -- empty results from `query()` need a diagnostic path, not silence

---

## Journey 4: MCP Integration

**Narrative:** A developer wants to use Alaya's memory capabilities from Claude, a Python agent, or any MCP-compatible client -- without writing Rust. The MCP server wraps AlayaStore and exposes it over the Model Context Protocol.

This journey is the v0.2 adoption wedge. It lowers the barrier from "must write Rust" to "must configure an MCP server."

### Phase Map

```
DISCOVERY        CONFIGURATION    FIRST MEMORY      CROSS-SESSION    DEEP USAGE
    |                |                |                 |              |
    v                v                v                 v              v
+--------+     +--------+      +--------+       +--------+     +--------+
|MCP     |---->|Config  |----->|Store   |------>|Query   |---->|Life-   |
|registry|     |in agent|      |via MCP |       |across  |     |cycle   |
|/README |     |config  |      |tool    |       |sessions|     |via MCP |
+--------+     +--------+      +--------+       +--------+     +--------+

EMOTION:        EMOTION:        EMOTION:         EMOTION:       EMOTION:
Curious          Cautious       Testing          Impressed      Committed
"Memory via     "Will this     "Does it          "It            "I'll build
MCP tool?"      just work?"    persist?"         remembers!"    on this"
```

### Phase 1: Discovery

**Entry points:**
- MCP server registry / directory listing
- Alaya README mentions MCP server as a usage mode
- Blog post: "Give Claude persistent memory with a single SQLite file"
- Developer already uses MCP for other tools and searches for memory capabilities

**What the developer evaluates:**
- Does the MCP server require infrastructure? (No -- single binary, single file)
- Can it run locally? (Yes -- same privacy guarantees as the library)
- What tools does it expose? (`store_episode`, `query`, `status`, `dream`)

### Phase 2: Configuration

**Steps for Claude Desktop / Claude Code:**
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

**What can go wrong:**
- Binary not installed or not on PATH
- Database path permissions
- Config syntax errors (JSON vs. YAML confusion in different MCP clients)
- Port conflicts if using stdio vs. HTTP transport

**Mitigation:**
- `cargo install alaya-mcp` with clear instructions
- Default path (`~/.alaya/memory.db`) works without configuration
- Helpful error messages on startup: "Database opened at /path/to/memory.db, N episodes stored"

### Phase 3: First Memory via MCP

**What the developer does:** In their MCP client, they invoke the `store_episode` tool:

```
Tool: alaya.store_episode
Input: {
  "content": "User prefers dark mode and monospace fonts",
  "role": "user",
  "session_id": "mcp-session-1"
}
Response: { "episode_id": 1, "status": "stored" }
```

Then queries:
```
Tool: alaya.query
Input: { "text": "What are the user's UI preferences?" }
Response: {
  "results": [
    { "score": 0.82, "content": "User prefers dark mode and monospace fonts" }
  ]
}
```

**Success criteria:** Round-trip works. Store and retrieve through MCP with no Rust code.

### Phase 4: Cross-Session Queries

**The moment:** The developer closes their MCP client, reopens it the next day, queries "UI preferences," and gets results from yesterday's session. The SQLite file persisted everything.

This is where MCP users first experience the "it remembers" moment that Rust API users get in Journey 1, Phase 5.

**What deepens engagement:**
- Querying across sessions reveals temporal context
- `alaya.status` shows growing memory store
- Results from weeks ago surface when relevant

### Phase 5: Lifecycle via MCP

**Advanced usage:** The developer exposes `dream` as an MCP tool or runs it on a schedule:

```
Tool: alaya.dream
Response: {
  "consolidation": { "episodes_processed": 15, "nodes_created": 3 },
  "forgetting": { "nodes_decayed": 42, "nodes_archived": 2 },
  "transformation": { "nodes_merged": 1 }
}
```

**Limitation:** MCP server uses `NoOpProvider` for consolidation by default (no LLM access from server side). Developer must configure a provider or accept rule-based-only extraction. This is an honest tradeoff: MCP server provides storage + retrieval + forgetting, but full cognitive lifecycle requires Rust API integration or a configured LLM provider in the MCP server.

---

## Journey 5: Persona Variations

The two primary personas (Priya and Marcus) take the same journeys above but with different priorities, evaluation criteria, and integration depths.

### Priya: Privacy-First Agent Developer

**Context:** Solo developer building a companion/coaching agent. Users trust the agent with personal information. Data must never leave the device.

**Journey variations:**

| Standard Phase | Priya's Variation | Additional Steps |
|---------------|-------------------|------------------|
| **Evaluation** | Privacy audit | Checks `Cargo.toml` for network crates. Reads source for `reqwest`, `hyper`, `tokio`. Finds none in required deps. Searches for `connect`, `socket`, `http` in crate source |
| **Evaluation** | Data locality check | Verifies single-file storage. Opens SQLite file to inspect schema. Confirms no temp files written elsewhere |
| **Installation** | Offline build test | Disconnects from internet. Runs `cargo build` with vendored deps. Confirms it compiles offline |
| **First Code** | Encryption check | Asks: "Can I encrypt the SQLite file?" Answer: yes, via SQLCipher feature flag or filesystem-level encryption. Alaya does not implement its own crypto (Simplicity > Completeness) |
| **Production** | Threat modeling | Maps data flow: user input -> AlayaStore -> SQLite file. No network egress. No telemetry. No crash reporting. Single attack surface: filesystem access to the .db file |
| **Production** | Compliance documentation | Documents for users: "Your data stays on this device. Memory is stored in [path]. Delete this file to erase all memory." |

**Priya's trust signals (in order of importance):**
1. Zero network dependencies in `Cargo.toml` (verifiable in 10 seconds)
2. No telemetry code in source (verifiable by grep)
3. Single SQLite file (verifiable by running and checking filesystem)
4. MIT license (verifiable in LICENSE file)
5. Clear data deletion path (`PurgeFilter::All` or delete the file)

**Priya's dealbreakers:**
- Any crate in the dependency tree that makes network calls
- Telemetry or analytics of any kind
- Data written to locations other than the specified database path
- Unclear data deletion guarantees

### Marcus: Performance-Focused Agent Developer

**Context:** Systems engineer building a coding agent or conversational tool for the OpenClaw ecosystem. Needs sub-millisecond retrieval, verifiable quality, and no framework lock-in.

**Journey variations:**

| Standard Phase | Marcus's Variation | Additional Steps |
|---------------|-------------------|------------------|
| **Evaluation** | Benchmark check | Looks for published LoCoMo/LongMemEval scores. Checks if benchmarks are reproducible. Runs them locally |
| **Evaluation** | Dependency audit | Not for privacy but for binary size and compile time. Counts transitive deps. Checks for proc macros that slow compilation |
| **First Code** | Performance baseline | Writes a microbenchmark before integrating: store 1000 episodes, query 100 times, measure p50/p99 latency |
| **Integration** | Scale test | Loads 10,000+ episodes. Measures query latency, memory footprint, SQLite file size. Tests with and without embeddings |
| **Integration** | Retrieval quality measurement | Builds a domain-specific test set (coding agent queries). Measures precision@5, recall@10. Compares BM25-only vs. BM25+embeddings |
| **Tuning** | Graph analysis | Examines Hebbian link weights. Tests spreading activation depth. Measures graph's impact on retrieval quality |
| **Production** | Continuous benchmarking | Integrates Alaya benchmarks into CI. Regression alerts on latency or quality degradation |

**Marcus's benchmark expectations:**

```
Operation                          Target       Acceptable
store_episode (single)             < 0.5ms      < 2ms
query (BM25-only, 1K episodes)     < 1ms        < 5ms
query (BM25+vec, 1K episodes)      < 5ms        < 20ms
query (BM25-only, 10K episodes)    < 5ms        < 20ms
consolidate (10 episodes)          < 10ms       < 50ms (excl. LLM)
forget sweep (10K nodes)           < 50ms       < 200ms
AlayaStore::open (cold, 10K eps)   < 10ms       < 50ms
SQLite file size per 1K episodes   < 500KB      < 2MB
```

**Marcus's integration decision tree:**
```
                    Does it compile in < 30s?
                           |
                    yes ---|--- no --> REJECT
                           |
                    Does query return in < 5ms at 1K episodes?
                           |
                    yes ---|--- no --> REJECT
                           |
                    Is LoCoMo > 60% (BM25-only)?
                           |
                    yes ---|--- no --> EVALUATE (might still use for speed)
                           |
                    Does it run without LLM?
                           |
                    yes ---|--- no --> REJECT (Mem0/Zep territory)
                           |
                    INTEGRATE
```

**Marcus's dealbreakers:**
- No published benchmarks (claims without evidence)
- Required LLM for basic operations
- Framework lock-in (must use specific agent framework)
- Significant performance regression between versions

---

## Journey Metrics

These metrics measure how well Alaya's developer experience supports each journey phase. They are leading indicators for the North Star metric (MACC).

### Funnel Metrics

| Phase | Metric | Target | Measurement Method |
|-------|--------|--------|-------------------|
| Discovery -> Evaluation | README click-through | >50% of crates.io visitors | crates.io analytics (if available) |
| Evaluation -> Installation | `cargo add` within 5 min of README | >50% of evaluators | Cannot measure directly; proxy: GitHub clone-to-star ratio |
| Installation -> First Example | Working example in <2 min | >80% of installers | Measure in user interviews; quickstart test in CI |
| First Example -> Integration | Uses in own project within 1 week | >30% of first-time users | GitHub dependents count; community survey |
| Integration -> Production | Ships to end users within 1 month | >10% of integrators | Community reports; GitHub issues from production use |
| Error -> Resolution | Self-service resolution (no issue filed) | >90% | Issue volume relative to download count |

### Quality Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Time to first successful query | < 2 minutes | CI test: clone, build, run quickstart, verify output |
| Compilation errors in quickstart | 0 | CI test: compile quickstart examples on each release |
| Documentation coverage (public API) | 100% | `cargo doc` warnings = 0 |
| Doctest pass rate | 100% | `cargo test --doc` in CI |
| Error message actionability | >80% self-service | Track issue resolution: filed vs. self-resolved |

### Persona-Specific Metrics

| Persona | Metric | Target |
|---------|--------|--------|
| Priya | Privacy audit pass (zero network deps) | 100% -- this is a hard constraint, not a target |
| Priya | Time to verify privacy claims | < 5 minutes |
| Marcus | Published benchmark reproducibility | 100% (anyone can run and get similar results) |
| Marcus | Query latency at 1K episodes (BM25) | < 1ms p99 |
| Marcus | LoCoMo score (published and honest) | >75% at v0.2 |

---

## DX Anti-Patterns to Avoid

These are specific failure modes that destroy developer trust. Each is derived from the competitive landscape analysis and the Extract's "never" list.

### 1. The "Hello World That Isn't"

**Anti-pattern:** Quickstart example that requires configuration, API keys, or multiple files to run.

**Alaya's commitment:** The quickstart is a single file with `cargo add alaya` and no external dependencies. It compiles, runs, and returns results. If the quickstart ever requires an LLM API key, Alaya has failed its own axioms.

### 2. The Opaque Error

**Anti-pattern:** Error message says what went wrong but not what to do.

**Example of bad:**
```
Error: Db(SqliteError(Some("fts5: syntax error")))
```

**Example of good:**
```
Error: FTS5 query syntax error in "user:preferences"
  The colon character is special in FTS5 syntax.
  Alaya auto-sanitizes queries, but this error suggests
  a bug in sanitization. Please report this at:
  https://github.com/h4x0r/alaya/issues
```

### 3. The Mandatory Migration

**Anti-pattern:** Version upgrade silently changes the database schema or breaks the file format without clear communication.

**Alaya's commitment:** Schema changes are versioned. Migration is explicit. Backup is created before migration. Developer is never surprised by data shape changes.

### 4. The Feature Flag Maze

**Anti-pattern:** Library requires the developer to understand 15 feature flags before they can use it.

**Alaya's commitment:** 4-6 feature flags maximum. Default features give a complete working system. Flags add capabilities (embeddings, async, etc.), they do not gate basic functionality.

### 5. The "Works on My Machine" Benchmark

**Anti-pattern:** Claiming performance numbers without reproducible methodology.

**Alaya's commitment:** Every published benchmark includes the hardware, dataset, methodology, and a script to reproduce. If numbers are bad, publish them anyway (Honesty > Marketing).

---

## Implications for API Design

The journey analysis reveals specific requirements for the Alaya API surface.

### From Journey 1 (First-Time Developer)

| Finding | API Implication |
|---------|----------------|
| Quickstart must compile with zero config | `AlayaStore::open(path)` with no builder required for basic use |
| Query must return results on first try | `Query::simple(text)` convenience constructor with sensible defaults |
| Developer expects familiar Rust patterns | `Result<T>` everywhere, `impl AsRef<Path>`, no custom error conventions |
| Types must be self-documenting | All struct fields documented, no stringly-typed parameters |
| Import list must be short | Re-export commonly used types from crate root |

### From Journey 2 (Deepening Integration)

| Finding | API Implication |
|---------|----------------|
| Lifecycle should be opt-in, not automatic | Explicit `consolidate()`, `forget()`, `transform()` calls |
| Provider trait must be easy to implement | Two required methods, clear input/output types, example in docs |
| Reports must be useful | Typed lifecycle reports with counts, not just `()` returns |
| Status must be inspectable | `store.status()` returning `MemoryStatus` with all counts |
| Dream cycle is a pattern, not a method | Document the pattern; do not prescribe scheduling |

### From Journey 3 (Error Recovery)

| Finding | API Implication |
|---------|----------------|
| Error messages must be actionable | `AlayaError` variants include context, not just wrapped SQLite errors |
| Provider errors must be attributed | `AlayaError::Provider(msg)` with phase context |
| Schema migration must be explicit | `migrate()` method, not silent auto-migration |
| Empty results need explanation | Future: `QueryExplanation` type showing scoring breakdown |
| FTS5 injection must be impossible | All query input sanitized before reaching SQLite |

### From Journey 4 (MCP Integration)

| Finding | API Implication |
|---------|----------------|
| MCP tools map 1:1 to AlayaStore methods | API methods must be individually callable, not pipeline-only |
| JSON serialization is required for MCP | All public types derive `Serialize, Deserialize` |
| Default provider must work without config | `NoOpProvider` provides baseline functionality |
| Status reporting over MCP needs structure | `MemoryStatus` and lifecycle reports must serialize cleanly |

### From Journey 5 (Persona Variations)

| Finding | API Implication |
|---------|----------------|
| Priya needs to verify no network calls | Keep dependency list minimal and auditable |
| Marcus needs benchmarkable operations | Each public method should be independently benchmarkable (no hidden setup) |
| Both need honest documentation | Every public method documents its performance characteristics |
| Both need clean error boundaries | Errors clearly attributed to Alaya vs. provider vs. SQLite |

---

## Appendix: Journey-to-Phase Mapping

| Journey | Relevant Alaya Phase | Key Deliverables |
|---------|---------------------|------------------|
| Journey 1 (First-Time) | v0.1 MVP | Quickstart, doctests, clean README, crates.io publish |
| Journey 2 (Deepening) | v0.1 MVP + v0.2 | Provider docs, lifecycle guide, tuning guide |
| Journey 3 (Error Recovery) | v0.1 MVP | Error message quality, migration system, FTS5 sanitization |
| Journey 4 (MCP) | v0.2 Ecosystem | MCP server binary, MCP tool definitions, configuration guide |
| Journey 5 (Priya) | v0.1 MVP | Privacy documentation, dependency audit, offline verification |
| Journey 5 (Marcus) | v0.1 + v0.2 | Published benchmarks, reproducible methodology, performance guide |

---

*Generated: 2026-02-26 | Phase: 5a | Cross-references: Brand Guidelines (Phase 1), North Star (Phase 2), Competitive Landscape (Phase 3), North Star Extract (Phase 4)*
