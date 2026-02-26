# Alaya

A neuroscience and Buddhist psychology-inspired memory engine for conversational AI agents.

**Alaya** (Sanskrit: *alaya-vijnana*, "storehouse consciousness") is a Rust
library that provides three-tier memory, a Hebbian graph overlay, hybrid
retrieval with spreading activation, and adaptive lifecycle processes. It is
headless and LLM-agnostic — the consuming agent owns identity, embeddings,
and prompt assembly.

## Why Alaya?

Most AI memory systems are Python libraries that require external infrastructure
(Postgres, Neo4j, Redis, Pinecone) and are tightly coupled to specific LLM
providers. Alaya takes a different approach.

**Key differentiators:**

- **Single-file deployment** — one SQLite database, no external services
- **Rust** — embed in any language via FFI, or use natively with zero GC pauses
- **LLM-agnostic** — no hardcoded provider; the agent supplies embeddings and consolidation logic via traits
- **No network calls** — fully local, privacy by architecture
- **Memory as process** — Hebbian graph reshaping, adaptive forgetting, and preference crystallization make memory a living system, not a static store
- **Principled foundations** — architecture grounded in CLS theory, Bjork forgetting, spreading activation, and Yogacara psychology, not ad-hoc heuristics

### Comparison with Alternatives

#### Memory Systems

| | **Alaya** | **mem0** | **Zep / Graphiti** | **Letta (MemGPT)** | **LangChain** | **LlamaIndex** |
|---|---|---|---|---|---|---|
| **Language** | Rust | Python | Python | Python | Python | Python |
| **Storage** | SQLite (single file) | Qdrant/Pinecone + Postgres + Neo4j | Neo4j + Lucene | Postgres + Chroma/Qdrant | In-memory / Redis | SQLite / Postgres |
| **External infra** | None | 2-3 services | 1-2 services | 1-2 services | 0-1 services | 0-1 services |
| **LLM coupling** | None — traits | Required | Required for extraction | Required (LLM = memory manager) | Optional | Optional |
| **Memory model** | Three-store (episodic, semantic, implicit) | Tiered + optional graph | Temporal knowledge graph | OS-inspired (core, recall, archival) | Buffer / summary / entity | Composable blocks |
| **Graph** | Hebbian — reshapes through use | Optional (Mem0g) | Static temporal KG | No | No | No |
| **Retrieval** | BM25 + vector + graph + RRF | Vector + graph (Mem0g) | Cosine + BM25 + graph + RRF | Agent-driven tool calls | Direct injection | Block-dependent |
| **Forgetting** | Bjork dual-strength + RIF | Exponential decay | Temporal invalidation | Eviction + summarization | Window / truncation | FIFO eviction |
| **Preferences** | Vasana (emergent) | LLM-extracted profiles | Indirect (graph) | Agent-edited blocks | Minimal | Basic (facts) |
| **Privacy** | Fully local | Cloud-dependent | Configurable | Configurable | Configurable | Configurable |

#### Rising Stars and Noteworthy Systems

| System | Type | Language | Storage | Key Idea |
|--------|------|----------|---------|----------|
| **Cognee** | Knowledge engine | Python | Neo4j + vectors | Vector + graph hybrid; 70+ companies in production |
| **A-MEM** | Research (NeurIPS 2025) | Python | Vector + note graph | Zettelkasten-inspired; 2x multi-hop reasoning |
| **MemoryOS** | Research (EMNLP 2025) | Python | Configurable | Three-tier OS hierarchy; 49% F1 improvement on LoCoMo |
| **Motorhead** | Memory server | Rust | Redis | Simple REST API; incremental summarization |
| **Engram** | MCP memory | Go | SQLite + FTS5 | Zero dependencies; agent-directed; single binary |
| **OpenViking** | Context DB | Python | VikingDB | Virtual filesystem paradigm; tiered loading (ByteDance) |
| **LangGraph** | Agent framework | Python | Configurable | Stateful graph orchestration; checkpoint-based memory |
| **Second Me** | AI identity | Python | Model parameters | Memory encoded into model weights via fine-tuning |
| **MemoryBank** | Research (AAAI 2024) | Python | External bank | Ebbinghaus forgetting curve; user portrait synthesis |
| **Generative Agents** | Research (UIST 2023) | Python | In-memory | Recency x importance x relevance (seminal paper) |

#### Vector Databases (Infrastructure Layer)

These provide storage and retrieval but not memory semantics (lifecycle,
forgetting, preference learning, graph dynamics).

| System | Language | Hybrid Search | Managed Cloud | Open Source |
|--------|----------|:------------:|:-------------:|:----------:|
| **Pinecone** | Cloud-native | No native BM25 | Yes (only option) | No |
| **ChromaDB** | Python | FTS + vector | Chroma Cloud | Yes |
| **Weaviate** | Go | BM25 + vector | Weaviate Cloud | Yes |
| **Milvus** | Go/C++ | Dense + sparse | Zilliz Cloud | Yes |
| **Cloudflare Vectorize** | Cloud-native | Via Workers AI | Yes (only option) | No |

For a comprehensive analysis grounded in the CoALA taxonomy (Sumers et al.,
2024) and RAG survey literature (Gao et al., 2023; Zhang et al., 2024), see
[docs/related-work.md](docs/related-work.md).

## Architecture

```mermaid
graph LR
    subgraph Alaya["ALAYA (memory crate)"]
        ES[Episodic Store]
        SS[Semantic Store]
        IS[Implicit Store]
        GO[Graph Overlay]
        RE[Retrieval Engine]
        LP[Lifecycle Processes]

        ES --- GO
        SS --- GO
        IS --- GO
        GO --- RE
        RE --- LP
    end

    subgraph Agent["AGENT"]
        SOUL[Identity / SOUL.md]
        CTX[Context Assembly]
        LLM[LLM Provider]
        EMB[Embedding Provider]
    end

    Agent <-->|query · store · lifecycle| Alaya
```

### Three Stores

| Store | Analog | Purpose |
|-------|--------|---------|
| **Episodic** | Hippocampus | Raw conversation events with full context |
| **Semantic** | Neocortex | Distilled knowledge extracted through consolidation |
| **Implicit** | Alaya-vijnana | Preferences and habits that emerge through perfuming |

### Graph Overlay

A Hebbian weighted directed graph spans all three stores. Links strengthen on
co-retrieval (LTP) and weaken through disuse (LTD), naturally developing
small-world topology.

### Retrieval Pipeline

```mermaid
flowchart LR
    Q[Query] --> BM25[BM25 / FTS5]
    Q --> VEC[Vector / Cosine]
    Q --> GR[Graph Neighbors]

    BM25 --> RRF[Reciprocal Rank Fusion]
    VEC --> RRF
    GR --> RRF

    RRF --> RR[Context-Weighted Reranking]
    RR --> SA[Spreading Activation]
    SA --> RIF[Retrieval-Induced Forgetting]
    RIF --> OUT[Top 3-5 Results]
```

### Lifecycle Processes

| Process | Inspiration | What it does |
|---------|-------------|--------------|
| **Consolidation** | CLS theory (McClelland et al.) | Distills episodes into semantic knowledge |
| **Perfuming** | Vasana (Yogacara Buddhist psychology) | Accumulates impressions, crystallizes preferences |
| **Transformation** | Asraya-paravrtti | Deduplicates, resolves contradictions, prunes |
| **Forgetting** | Bjork & Bjork (1992) | Decays retrieval strength, archives weak nodes |

## Quick Start

```rust
use alaya::{AlayaStore, NewEpisode, Role, EpisodeContext, Query, NoOpProvider};

// Open a persistent database
let store = AlayaStore::open("memory.db")?;

// Store a conversation episode
store.store_episode(&NewEpisode {
    content: "I've been learning Rust for about six months now".into(),
    role: Role::User,
    session_id: "session-1".into(),
    timestamp: 1740000000,
    context: EpisodeContext::default(),
    embedding: None, // pass Some(vec![...]) if you have embeddings
})?;

// Query with hybrid retrieval
let results = store.query(&Query::simple("Rust experience"))?;
for mem in &results {
    println!("[{:.2}] {}", mem.score, mem.content);
}

// Get crystallized preferences
let prefs = store.preferences(Some("communication_style"))?;

// Run lifecycle with a no-op provider (or implement ConsolidationProvider)
let noop = NoOpProvider;
store.consolidate(&noop)?;
store.transform()?;
store.forget()?;
```

## Research Foundations

### Neuroscience

- **Hebbian LTP/LTD** — synapses strengthen on co-activation (Hebb 1949, Bliss & Lomo 1973)
- **Complementary Learning Systems** — fast hippocampus + slow neocortex (McClelland et al. 1995)
- **Spreading Activation** — associative retrieval beyond embedding similarity (Collins & Loftus 1975)
- **Encoding Specificity** — context-dependent retrieval (Tulving & Thomson 1973)
- **Dual-Strength Forgetting** — storage vs retrieval strength (Bjork & Bjork 1992)
- **Retrieval-Induced Forgetting** — retrieving some memories suppresses competitors (Anderson et al. 1994)
- **Working Memory Limits** — 4 +/- 1 chunks (Cowan 2001)

### Yogacara Buddhist psychology

- **Alaya-vijnana** — the storehouse consciousness, persistent substrate for all seeds
- **Bija (seeds)** — living potentials that ripen when conditions align
- **Vasana (perfuming)** — gradual accumulation of impressions that shape behavior
- **Asraya-paravrtti** — periodic transformation toward clarity
- **Vijnaptimatrata** — memory is perspective-relative, not objective

### Information Retrieval

- **Reciprocal Rank Fusion** — merging multiple ranked result sets (Cormack et al. 2009)
- **BM25 via FTS5** — keyword matching with relevance scoring
- **Cosine Similarity** — semantic vector search

## Design Principles

1. **Memory is a process, not a database.** Every retrieval changes what is
   remembered. The graph reshapes through use.

2. **Forgetting is a feature.** Strategic decay and suppression improve
   retrieval quality over time.

3. **Preferences emerge, they are not declared.** The vasana/perfuming model
   lets behavioral patterns crystallize from accumulated observations.

4. **The agent owns identity.** Alaya stores seeds. The agent decides which
   seeds matter and how to present them.

5. **Graceful degradation.** No embeddings? BM25-only. No LLM for
   consolidation? Episodes accumulate. Every feature works independently.

## API Overview

```rust
impl AlayaStore {
    // Write
    pub fn store_episode(&self, episode: &NewEpisode) -> Result<EpisodeId>;

    // Read
    pub fn query(&self, q: &Query) -> Result<Vec<ScoredMemory>>;
    pub fn preferences(&self, domain: Option<&str>) -> Result<Vec<Preference>>;
    pub fn knowledge(&self, filter: Option<KnowledgeFilter>) -> Result<Vec<SemanticNode>>;
    pub fn neighbors(&self, node: NodeRef, depth: u32) -> Result<Vec<(NodeRef, f32)>>;

    // Lifecycle
    pub fn consolidate(&self, provider: &dyn ConsolidationProvider) -> Result<ConsolidationReport>;
    pub fn perfume(&self, interaction: &Interaction, provider: &dyn ConsolidationProvider) -> Result<PerfumingReport>;
    pub fn transform(&self) -> Result<TransformationReport>;
    pub fn forget(&self) -> Result<ForgettingReport>;

    // Admin
    pub fn status(&self) -> Result<MemoryStatus>;
    pub fn purge(&self, filter: PurgeFilter) -> Result<PurgeReport>;
}
```

## License

MIT
