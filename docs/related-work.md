# Related Work: AI Agent Memory Systems

A comparative analysis of Alaya against existing memory architectures, organized
around the CoALA taxonomy (Sumers et al., 2024) and recent survey literature on
RAG and agent memory.

## Table of Contents

- [Analytical Framework](#analytical-framework)
- [System-by-System Analysis](#system-by-system-analysis)
  - [Production Systems](#production-systems)
  - [Framework-Level Memory](#framework-level-memory)
  - [Vector Databases as Memory](#vector-databases-as-memory)
  - [Academic / Research Systems](#academic--research-systems)
- [Comparative Matrices](#comparative-matrices)
  - [CoALA Dimension Analysis](#coala-dimension-analysis)
  - [Storage and Infrastructure](#storage-and-infrastructure)
  - [Retrieval Pipeline](#retrieval-pipeline)
  - [Memory Lifecycle](#memory-lifecycle)
- [Landscape Analysis](#landscape-analysis)
- [Where Alaya Sits](#where-alaya-sits)
- [Tradeoffs: Embedded SQLite vs. External Infrastructure](#tradeoffs-embedded-sqlite-vs-external-infrastructure)
- [References](#references)

---

## Analytical Framework

This comparison uses three complementary frameworks:

**1. CoALA** (Sumers et al., 2024) classifies agent memory along three axes:
memory modules (working, episodic, semantic, procedural), action space (internal
reasoning/retrieval vs. external grounding), and decision-making procedure. It
draws on decades of cognitive architecture research (ACT-R, Soar) to provide a
principled vocabulary for comparing agent designs.

**2. RAG Survey Taxonomy** (Gao et al., 2023) classifies retrieval-augmented
systems as Naive, Advanced, or Modular RAG, with evaluation criteria including
context relevance, answer faithfulness, noise robustness, negative rejection,
and information integration.

**3. Agent Memory Survey** (Zhang et al., 2024) adds three dimensions: memory
sources (inside-trial vs. cross-trial), memory forms (natural language,
embeddings, databases, structured knowledge, parametric), and memory operations
(writing, management, reading).

Two additional surveys inform this analysis: "Memory in the Age of AI Agents"
(Hu et al., 2025), which proposes a Forms-Functions-Dynamics taxonomy, and "From
Human Memory to AI Memory" (Wu et al., 2025), which maps AI memory to human
cognitive structures via a 3D-8Q model (Object x Form x Time).

---

## System-by-System Analysis

### Production Systems

#### Mem0

- **Citation:** Choudhary et al. (2025). "Mem0: Building Production-Ready AI
  Agents with Scalable Long-Term Memory." arXiv:2504.19413.
- **Architecture:** Tiered memory with optional graph (Mem0g variant). Vector
  databases (Qdrant, Pinecone, Chroma) + relational DB for metadata + optional
  Neo4j for entity graphs.
- **CoALA mapping:** Episodic + semantic memory. Learning via LLM-driven
  extraction. Retrieval via vector similarity + graph traversal.
- **Retrieval:** Hybrid vector search + graph traversal with contextual tagging
  and priority scoring.
- **LLM dependency:** Required for memory extraction, conflict resolution, and
  update decisions.
- **Forgetting:** Exponential decay of low-relevance entries. Consolidation
  between short-term and long-term tiers.
- **Preference learning:** Yes — extracts user preferences and personality
  traits from interactions.
- **Key results:** 26% accuracy improvement over OpenAI memory on LOCOMO. 91%
  lower p95 latency vs. full-context approaches.

#### Zep / Graphiti

- **Citation:** Dempsey et al. (2025). "Graphiti: Building Real-Time,
  Multi-Layered Temporal Knowledge Graphs." arXiv:2501.13956.
- **Architecture:** Hierarchical temporal knowledge graph on Neo4j. Three
  subgraph layers: Community (cluster summaries), Entity (semantic entities),
  Episodic (raw conversation episodes).
- **CoALA mapping:** Episodic + semantic memory. Learning via LLM-driven entity
  extraction. Retrieval via triple hybrid (cosine + BM25 + graph traversal).
- **Retrieval:** Cosine similarity + Okapi BM25 + breadth-first graph traversal.
  Multi-stage reranking (episode-mentions, node-distance, RRF, MMR). p95 ~300ms.
- **LLM dependency:** Required for entity/relationship extraction (gpt-4o-mini).
  Retrieval itself is LLM-free.
- **Forgetting:** Bi-temporal invalidation — facts are never deleted, only
  marked with validity periods. No decay curves.
- **Preference learning:** Indirect — preferences captured as graph
  nodes/edges.
- **Key results:** 94.8% on DMR benchmark. Up to 18.5% accuracy improvement on
  LongMemEval.

#### Letta (formerly MemGPT)

- **Citation:** Packer et al. (2023). "MemGPT: Towards LLMs as Operating
  Systems." arXiv:2310.08560.
- **Architecture:** OS-inspired three-tier hierarchy. Core Memory ("RAM," always
  in-context), Recall Memory (conversation history), Archival Memory ("disk,"
  unbounded vector/graph DB).
- **CoALA mapping:** Working memory (core) + episodic (recall) + semantic
  (archival). The LLM is the memory manager — it decides what to store, evict,
  and retrieve via tool calls.
- **Retrieval:** Agent-driven via tool calls. Vector similarity for archival.
  Sequential scan for recall.
- **LLM dependency:** Required — the LLM IS the memory manager.
- **Forgetting:** Eviction-based — when context fills, summarize and store old
  messages (~70% eviction). Sleep-time agents (2025) handle asynchronous
  reorganization.
- **Preference learning:** Yes — core memory blocks store and track evolving
  user preferences.

#### Cognee

- **Citation:** cognee.ai. GitHub: topoteretes/cognee. $7.5M seed (2025).
- **Architecture:** Knowledge engine combining vector stores + graph databases
  (Neo4j, NetworkX). Transforms raw data into persistent, dynamic memory.
- **CoALA mapping:** Semantic memory with graph-structured knowledge.
- **Retrieval:** Hybrid vector + graph retrieval.
- **LLM dependency:** Required for knowledge extraction and graph construction.
- **Forgetting:** Not documented.
- **Key context:** 500x pipeline volume growth in 2025. Running in 70+
  companies. Backed by OpenAI and FAIR founders.

#### MemoryOS

- **Citation:** Kang et al. (2025). "MemoryOS: An Operating System-Like Memory
  Management Framework for LLM-Based Agents." arXiv:2506.06326. EMNLP 2025
  Oral.
- **Architecture:** Three-tier OS-inspired hierarchy (short-term, mid-term,
  long-term personal memory). Four modules: Storage, Updating, Retrieval,
  Generation.
- **CoALA mapping:** Working + episodic + semantic memory with tiered promotion.
- **Retrieval:** Hierarchical retrieval across tiers.
- **LLM dependency:** Required.
- **Forgetting:** FIFO eviction (short-to-mid), segmented page organization
  (mid-to-long).
- **Key results:** 49% F1 improvement and 46% BLEU-1 improvement on LoCoMo.

#### A-MEM

- **Citation:** Xu et al. (2025). "A-MEM: Agentic Memory for LLM Agents."
  arXiv:2502.12110. NeurIPS 2025.
- **Architecture:** Zettelkasten-inspired. Each memory is a structured "note"
  with contextual descriptions, keywords, and tags. Notes are dynamically linked
  via embedding similarity + LLM-driven relationship analysis.
- **CoALA mapping:** Semantic memory with emergent graph structure.
- **Retrieval:** Embedding-based query + linked note traversal.
- **LLM dependency:** Required for note construction, linking, and evolution.
- **Forgetting:** Evolution-based — notes are continuously updated, not deleted.
- **Key results:** Doubles multi-hop reasoning performance vs. baselines.

---

### Framework-Level Memory

#### LangChain Memory

- **Citation:** LangChain documentation. MIT License.
- **Architecture:** Multiple flat, conversation-centric memory classes:
  ConversationBufferMemory (full history), ConversationSummaryMemory (LLM
  rolling summaries), ConversationBufferWindowMemory (sliding window),
  ConversationEntityMemory (entity tracking).
- **CoALA mapping:** Working memory only. No long-term persistence by default.
- **Retrieval:** Direct injection into prompt template. No semantic search in
  base classes.
- **LLM dependency:** Optional — buffer/window classes need none; summary and
  entity classes require LLM.
- **Forgetting:** Truncation-based only (window drops, token buffer drops). No
  intelligent decay.
- **Key context:** Most widely adopted but also most basic. Most memory classes
  now deprecated in favor of RunnableWithMessageHistory.

#### LlamaIndex Memory

- **Citation:** LlamaIndex documentation. MIT License.
- **Architecture:** Composable short-term + long-term blocks. Short-term is FIFO
  queue. Long-term via pluggable Memory Blocks (StaticMemoryBlock,
  FactExtractionMemoryBlock, VectorMemoryBlock).
- **CoALA mapping:** Working memory + optional episodic/semantic via blocks.
- **Retrieval:** Depends on block type — structured lookup, vector similarity.
  Configurable token ratio (70% chat history / 30% long-term by default).
- **LLM dependency:** Optional — basic buffer needs none.
  FactExtractionMemoryBlock requires LLM.
- **Forgetting:** FIFO eviction when chat exceeds token ratio.

#### Haystack (deepset)

- **Citation:** deepset. GitHub: deepset-ai/haystack. Apache 2.0.
- **Architecture:** Pipeline-based, modular. Memory is a component within
  Haystack's explicit pipeline architecture, not a primary focus.
- **CoALA mapping:** Working memory. Long-term via external integrations (Mem0).
- **Retrieval:** Pipeline-based through Haystack retriever components.
- **LLM dependency:** Optional for base memory.
- **Forgetting:** No built-in mechanism.
- **Key context:** Advanced Agent Memory was P3 (low priority) on Q1 2025
  roadmap.

#### LangGraph

- **Citation:** LangChain team. Part of LangChain ecosystem.
- **Architecture:** Graph-based agent orchestration with state persistence.
  Memory is modeled as graph state that persists across interactions. Checkpoints
  enable conversation resumption.
- **CoALA mapping:** Working memory (graph state) + episodic (checkpoints).
- **Retrieval:** State-based — the graph state IS the memory.
- **LLM dependency:** Required for agent execution.
- **Forgetting:** No built-in mechanism beyond state management.
- **Key context:** Not a memory system per se — a stateful orchestration
  framework. Memory is an emergent property of persistent graph execution.

---

### Vector Databases as Memory

These systems are infrastructure components, not memory architectures. They
provide the storage and retrieval layer that memory systems build on.

| System | Language | Index Type | Hybrid Search | Managed Cloud | OSS License | GitHub Stars |
|--------|----------|-----------|---------------|---------------|-------------|-------------|
| **Pinecone** | Cloud-native | Proprietary adaptive | No native BM25 | Yes (only option) | Proprietary | N/A |
| **ChromaDB** | Python | HNSW | FTS + vector | Chroma Cloud | Apache 2.0 | ~23K |
| **Weaviate** | Go | HNSW + BM25/SPLADE | Yes (native) | Weaviate Cloud | BSD-3 | ~14K |
| **Milvus** | Go/C++ | HNSW, IVF, DiskANN, GPU | Dense + sparse | Zilliz Cloud | Apache 2.0 | ~40K |
| **Cloudflare Vectorize** | Cloud-native | Proprietary | Via Workers AI | Yes (only option) | Proprietary | N/A |

**Relevance to Alaya:** These are all potential future backends if alaya
outgrows SQLite's brute-force vector search. They do not provide memory
semantics (stores, lifecycle, graph, forgetting) — only vector storage and ANN
retrieval.

---

### Academic / Research Systems

#### Generative Agents (Park et al., 2023)

- **Citation:** Park et al. (2023). "Generative Agents: Interactive Simulacra of
  Human Behavior." ACM UIST 2023. arXiv:2304.03442.
- **Architecture:** Memory stream (chronological observations) + reflections
  (higher-order thoughts) + plans (future actions).
- **Retrieval:** Triple-scored: recency (exponential decay) x importance
  (LLM-rated 1-10) x relevance (cosine similarity). The foundational retrieval
  formula for agent memory.
- **Forgetting:** Exponential recency decay. Unaccessed memories score lower.
- **Significance:** The seminal paper that launched modern agent memory research.
  The recency/importance/relevance scoring is now widely copied.

#### Reflexion (Shinn et al., 2023)

- **Citation:** Shinn et al. (2023). "Reflexion: Language Agents with Verbal
  Reinforcement Learning." NeurIPS 2023. arXiv:2303.11366.
- **Architecture:** Episodic self-reflection buffer. After each trial, the agent
  generates a natural language self-reflection. Bounded to 1-3 entries.
- **Significance:** Reframes memory as "verbal reinforcement learning." 91%
  pass@1 on HumanEval vs. GPT-4's 80%.

#### Voyager (Wang et al., 2023)

- **Citation:** Wang et al. (2023). "Voyager: An Open-Ended Embodied Agent with
  Large Language Models." arXiv:2305.16291.
- **Architecture:** Executable skill library — memories are stored as reusable
  JavaScript code, not natural language. Append-only, verified skills.
- **CoALA mapping:** Pure procedural memory. The only system in this survey
  focused on procedural memory.
- **Significance:** First LLM-powered embodied lifelong learning agent. Memory
  as executable code. 15.3x faster tech tree progression vs. baselines.

#### MemoryBank (Zhong et al., 2024)

- **Citation:** Zhong et al. (2024). "MemoryBank: Enhancing Large Language
  Models with Long-Term Memory." AAAI 2024. arXiv:2305.10250.
- **Architecture:** Three-module (Writer + Retriever + Reader). Stores
  conversations, events, and user portraits.
- **Forgetting:** Ebbinghaus Forgetting Curve: R = e^(-t/S). Recalled memories
  strengthen (S increments, t resets). The first system to formally implement
  Ebbinghaus forgetting in LLM memory.
- **Preference learning:** Explicit user portrait construction.

#### Think-in-Memory (Liu et al., 2023)

- **Citation:** Liu et al. (2023). "Think-in-Memory: Recalling and
  Post-Thinking Enable LLMs with Long-Term Memory." arXiv:2311.08719.
- **Architecture:** Stores processed "thoughts" (reasoning conclusions) rather
  than raw events. Locality-Sensitive Hashing for O(1) retrieval.
- **Forgetting:** Explicit forget and merge operations.
- **Significance:** Unique approach — storing reasoning results rather than
  observations.

#### Second Me (Mindverse, 2025)

- **Citation:** Second Me (2025). "SecondMe: Building the AI Version of
  Yourself." arXiv:2503.08102.
- **Architecture:** Three-layer HMM: L0 (short-term context), L1 (mid-term
  abstracted), L2 (AI-Native Memory — long-term knowledge encoded directly into
  model parameters via fine-tuning).
- **Significance:** Unique "memory as model parameters" approach. Parametric
  memory eliminates retrieval latency for deeply learned knowledge.

---

## Comparative Matrices

### CoALA Dimension Analysis

How each system maps to CoALA's memory module taxonomy:

| System | Working Memory | Episodic | Semantic | Procedural | Cross-Memory Learning |
|--------|---------------|----------|----------|------------|----------------------|
| **Alaya** | Agent-managed (not in scope) | Yes (episodes) | Yes (consolidation) | No | Yes (episodic -> semantic via consolidation) |
| **Mem0** | Agent-managed | Yes (interactions) | Yes (extracted facts) | No | Yes (extraction pipeline) |
| **Zep / Graphiti** | Agent-managed | Yes (episodic subgraph) | Yes (entity subgraph) | No | Yes (episode -> entity extraction) |
| **Letta** | Yes (core memory) | Yes (recall) | Yes (archival) | No | Yes (LLM-driven promotion) |
| **LangChain** | Yes (buffer) | Partial (history) | No | No | No |
| **Generative Agents** | Yes (current context) | Yes (memory stream) | Yes (reflections) | No | Yes (observation -> reflection) |
| **Voyager** | Yes (current task) | No | No | Yes (skill library) | No |
| **MemoryBank** | Yes (current dialogue) | Yes (conversations) | Yes (user portraits) | No | Yes (Ebbinghaus-gated) |
| **A-MEM** | Agent-managed | No | Yes (Zettelkasten notes) | No | Evolution-based |

**Key observation:** Alaya and Generative Agents are the only systems that
implement principled cross-memory learning (episodic -> semantic consolidation).
Most others either store everything flat or rely on LLM extraction without a
formal consolidation model.

### Storage and Infrastructure

| System | Storage Backend | External Services | Deployment Complexity | Data Locality |
|--------|----------------|-------------------|----------------------|---------------|
| **Alaya** | SQLite (embedded, single file) | None | `cargo add alaya` | Fully local |
| **Mem0** | Qdrant/Pinecone + Postgres + optional Neo4j | 2-3 services | Docker compose or cloud | Cloud-dependent |
| **Zep / Graphiti** | Neo4j + vector embeddings + Lucene | 1-2 services | Neo4j instance required | Self-hosted or cloud |
| **Letta** | Postgres/SQLite + Chroma/Qdrant/Milvus | 1-2 services | Docker or cloud | Configurable |
| **LangChain** | In-memory / Redis / Postgres | 0-1 services | pip install | Configurable |
| **Generative Agents** | In-memory | 0 services | Research code | Local (ephemeral) |
| **MemoryBank** | External memory bank | 1 service | Research code | Configurable |
| **Motorhead** | Redis + Redisearch | 1 service | Docker + Redis | Redis-dependent |
| **ChromaDB** | Embedded HNSW | 0 services | pip install | Local |
| **Weaviate** | Custom engine | 0-1 services | Docker or cloud | Configurable |

### Retrieval Pipeline

Mapped to Gao et al.'s taxonomy (Naive / Advanced / Modular RAG):

| System | Sparse (BM25) | Dense (Vector) | Graph | Fusion Method | Reranking | RAG Category |
|--------|:------------:|:--------------:|:-----:|:-------------:|:---------:|:------------:|
| **Alaya** | FTS5 | Cosine | Spreading activation | RRF | Context-weighted | Modular |
| **Mem0** | No | Yes | Optional (Mem0g) | Priority scoring | Yes | Advanced |
| **Zep / Graphiti** | BM25 (Okapi) | Cosine | BFS traversal | RRF + MMR | Multi-stage | Modular |
| **Letta** | No | Yes | No | N/A (agent-driven) | N/A | Naive (agent-augmented) |
| **LangChain** | No | No | No | N/A | N/A | Naive |
| **Generative Agents** | No | Cosine | No | Weighted sum | Recency + importance | Advanced |
| **MemoryBank** | No | Likely | No | Not specified | Not specified | Naive |
| **A-MEM** | No | Yes | Link traversal | Not specified | Not specified | Advanced |

**Key observation:** Only Alaya and Zep/Graphiti implement full three-signal
retrieval (sparse + dense + graph) with principled fusion (RRF). Most systems
rely on vector similarity alone.

### Memory Lifecycle

Mapped to Zhang et al.'s memory operations taxonomy and Hu et al.'s dynamics
axis:

| System | Formation | Consolidation | Forgetting Model | Contradiction Resolution | Preference Crystallization |
|--------|-----------|---------------|-----------------|-------------------------|---------------------------|
| **Alaya** | Direct episode storage | CLS-inspired (episodic -> semantic) | Bjork dual-strength + RIF | Via transformation lifecycle | Vasana (impression accumulation) |
| **Mem0** | LLM extraction | Short-to-long promotion | Exponential decay | LLM-driven | LLM-extracted profiles |
| **Zep / Graphiti** | LLM entity extraction | Episode -> entity subgraph | Bi-temporal invalidation | Temporal versioning | Indirect (graph structure) |
| **Letta** | Agent-directed tool calls | Sleep-time reorganization | Eviction + summarization | Not built-in | Agent-edited core blocks |
| **LangChain** | Direct buffer append | ConversationSummaryMemory | Truncation / window drop | Not built-in | Not built-in |
| **Generative Agents** | Observation logging | Reflection generation | Recency decay | Not built-in | Emergent from reflections |
| **MemoryBank** | Writer module | Not built-in | Ebbinghaus curve (R = e^(-t/S)) | Not built-in | User portrait synthesis |
| **A-MEM** | LLM note construction | Evolution-based | Evolution (not deletion) | LLM-driven merge | Not built-in |

**Key observation:** Alaya is the only system that combines CLS-inspired
consolidation, dual-strength forgetting with RIF suppression, explicit
contradiction resolution, and preference crystallization in a single
architecture. Most systems implement at most one of these lifecycle processes.

---

## Landscape Analysis

### Dominant Paradigms

Three architectural paradigms have emerged in the field:

1. **Vector store** (flat semantic retrieval): ChromaDB, Pinecone, LangChain
   memory. Simple, fast, but no structural understanding of relationships.

2. **Knowledge graph** (structured relationships): Zep/Graphiti, Mem0g, Cognee.
   Rich relational reasoning, but requires external graph DB and LLM for
   construction.

3. **OS-inspired tiering** (RAM/disk metaphor): Letta, MemoryOS, SCM.
   Hierarchical management with promotion/eviction, but typically lacks graph
   structure and forgetting.

A fourth paradigm is emerging:

4. **Parametric memory** (embedded in model weights): Second Me's L2 layer.
   Zero retrieval latency for deeply learned knowledge, but requires
   fine-tuning infrastructure.

### Underserved Areas

The survey literature consistently identifies these gaps:

**Forgetting:** Only MemoryBank (Ebbinghaus), Mem0 (exponential decay),
Generative Agents (recency decay), and Alaya (Bjork dual-strength + RIF)
implement principled forgetting. Du et al. (2025) lists forgetting as one of
six fundamental memory operations, yet most systems treat it as an afterthought.

**Preference emergence:** Most systems either don't model preferences at all or
rely on LLM extraction (Mem0). Alaya's vasana/perfuming model — where
preferences crystallize from accumulated impressions without explicit extraction
— is unique in the landscape.

**Adaptive retrieval:** CoALA identifies this as a major underexplored
direction. Most systems use fixed retrieval strategies. Alaya's spreading
activation provides one form of adaptive retrieval (graph paths reshape through
Hebbian strengthening), though fully learned retrieval strategies remain an open
problem.

**Cross-memory consolidation:** Generative Agents' reflection mechanism and
Alaya's CLS-inspired consolidation are rare examples. Most systems treat
episodic and semantic stores as independent.

### The Graph Question

Graph memory is becoming table stakes (Mem0 added Mem0g, Zep/Graphiti built on
Neo4j, Cognee combines vectors + graph, A-MEM creates implicit link graphs).
Flat vector search alone is increasingly seen as insufficient for multi-hop
reasoning.

However, most graph implementations are **static knowledge graphs** — they
record relationships extracted by an LLM and don't change unless the LLM
updates them. Alaya's Hebbian graph is **dynamic** — links strengthen through
co-retrieval (LTP) and weaken through disuse (LTD), naturally developing
small-world topology without LLM intervention.

---

## Where Alaya Sits

In CoALA terms, Alaya provides:
- **Episodic memory** (episodes with full context)
- **Semantic memory** (consolidated knowledge with confidence scores)
- **Implicit memory** (not in CoALA — closest to procedural, but behavioral rather than skill-based)
- **Cross-memory learning** (episodic -> semantic via CLS consolidation; episodic -> implicit via vasana perfuming)

In Gao et al.'s RAG taxonomy, Alaya is **Modular RAG**: three parallel retrieval
signals (sparse + dense + graph) fused via RRF, with context-weighted reranking
and post-retrieval memory modification (strength updates, Hebbian co-retrieval).

In Zhang et al.'s memory operations taxonomy, Alaya covers all three:
- **Writing:** Episode storage, semantic node creation, impression accumulation
- **Management:** Consolidation, transformation (dedup, contradiction
  resolution, pruning), Bjork forgetting, vasana crystallization
- **Reading:** Hybrid retrieval with spreading activation

In Hu et al.'s Forms-Functions-Dynamics taxonomy:
- **Forms:** Token-level (natural language episodes and nodes) — flat within
  stores, graph across stores
- **Functions:** Factual (semantic store) + experiential (episodic store) +
  working memory is agent-managed
- **Dynamics:** Active formation (CLS consolidation), evolution (transformation
  lifecycle), principled forgetting (Bjork + RIF)

### Unique Contributions

| Capability | Alaya | Closest Alternative | Difference |
|-----------|-------|-------------------|------------|
| **Three-store architecture** | Episodic + semantic + implicit | Letta (core + recall + archival) | Alaya's stores are cognitively grounded; Letta's are operationally grounded (RAM/disk) |
| **Hebbian graph** | Dynamic, reshapes through use | Zep/Graphiti (static temporal KG) | Alaya's graph learns from retrieval patterns; Zep's graph records LLM extractions |
| **Bjork forgetting** | Dual-strength decay + RIF suppression | MemoryBank (Ebbinghaus curve) | Alaya models storage and retrieval strength independently; MemoryBank uses single-curve decay |
| **Vasana preferences** | Impressions crystallize into preferences | Mem0 (LLM-extracted profiles) | Alaya's preferences emerge from patterns; Mem0's are explicitly extracted by LLM |
| **CLS consolidation** | Episodic -> semantic pipeline | Generative Agents (observation -> reflection) | Alaya's is provider-driven and configurable; Gen Agents' is hardcoded threshold-based |
| **Zero external dependencies** | Single SQLite file | Engram (SQLite + FTS5) | Comparable simplicity, but Alaya adds vector, graph, lifecycle |
| **LLM-agnostic** | Agent provides via traits | Letta (model-agnostic) | Both agnostic, but Alaya also works with NO LLM (BM25-only) |

---

## Tradeoffs: Embedded SQLite vs. External Infrastructure

There is a reason most production memory systems use Postgres, Neo4j, Pinecone,
and similar infrastructure. The tradeoff is real.

### What External Infrastructure Provides

**Approximate Nearest Neighbor (ANN) search:** Dedicated vector databases
(Pinecone, Milvus, Weaviate) use HNSW, IVF, or DiskANN indexes that provide
sub-linear search time. At 1M+ vectors, ANN indexes are orders of magnitude
faster than brute-force. Alaya's current vector search is brute-force O(n)
cosine similarity — fine for thousands of memories, problematic for millions.

**Native graph traversal:** Neo4j provides Cypher queries, path-finding
algorithms, and optimized graph storage. Alaya implements graph operations on
top of SQLite's relational model, which means multi-hop traversals require
multiple SQL queries rather than native graph operations.

**Horizontal scaling:** Postgres, Neo4j, and managed vector databases scale
horizontally. SQLite is single-writer and single-file. If an agent needs to
serve thousands of concurrent users with millions of memories each, SQLite is
not the right choice.

**Ecosystem maturity:** Neo4j has decades of graph algorithm research. Pinecone
has optimized quantization and adaptive indexing. These are hard-won
optimizations that a SQLite-based system cannot replicate.

### What Embedded SQLite Provides

**Zero operational overhead:** No database servers to provision, monitor,
backup, or upgrade. The memory file is the entire system. This matters for
individual developers, edge deployments, and privacy-sensitive applications.

**Data locality and privacy:** The memory never leaves the process. No network
calls means no data exfiltration surface. For personal AI agents, this is a
significant advantage — your memories are a file on your disk.

**Deployment simplicity:** `cargo add alaya` and you have a working memory
system. Compare with standing up Postgres + Neo4j + Pinecone and managing
connection strings, credentials, and infrastructure lifecycle.

**Transactional consistency:** SQLite provides ACID transactions within a single
process. Multi-service architectures (vector DB + graph DB + relational DB)
require distributed transaction management or eventual consistency.

**Predictable performance:** No network latency variance. No cold starts. No
rate limits. The performance envelope is entirely determined by local I/O and
CPU.

### Honest Assessment of Alaya's Limitations

| Dimension | SQLite (Alaya) | External Infra (others) | Crossover Point |
|-----------|---------------|------------------------|-----------------|
| **Vector search** | O(n) brute-force | O(log n) ANN | ~10K-50K embeddings |
| **Graph traversal** | SQL joins per hop | Native graph engine | ~100K+ nodes, deep traversals |
| **Concurrent writes** | Single-writer | Multi-writer | >1 concurrent writer |
| **Total data volume** | Practical to ~10-100 GB | Petabyte-scale | ~100 GB |
| **Multi-user** | One file per user (simple) | Shared DB with tenancy | >1000 concurrent users |

### Design Position

Alaya is designed for the **personal AI agent** use case: one user, one agent,
thousands to tens of thousands of memories, running locally. This covers the
vast majority of conversational AI agent deployments today.

For the long tail — enterprise-scale multi-user deployments with millions of
memories — the architecture supports future extension:
- The `store_embedding` / `search_by_vector` functions could be backed by an
  external vector index via a trait
- The graph could be backed by an external graph DB via a trait
- The SQLite storage could be swapped for Postgres via rusqlite's API
  compatibility with libpq

But these are future extensions, not current priorities. The design principle
is: **start simple, extend when you must, not when you might.**

---

## References

### Surveys and Frameworks

- Sumers, T. R., Yao, S., Narasimhan, K., & Griffiths, T. L. (2024).
  Cognitive Architectures for Language Agents. *Transactions on Machine Learning
  Research*. arXiv:2309.02427.

- Gao, Y., Xiong, Y., Gao, X., Jia, K., Pan, J., Bi, Y., Dai, Y., Sun, J.,
  Guo, Q., Wang, M., & Wang, H. (2023). Retrieval-Augmented Generation for
  Large Language Models: A Survey. arXiv:2312.10997.

- Zhang, Z., Bo, X., Ma, C., Li, R., Chen, X., Dai, Q., Zhu, J., Dong, Z., &
  Wen, J.-R. (2024). A Survey on the Memory Mechanism of Large Language Model
  based Agents. *ACM Transactions on Information Systems*, 43(6).
  arXiv:2404.13501.

- Hu, Y., Liu, S., Yue, Y., Zhang, G., et al. (2025). Memory in the Age of AI
  Agents. arXiv:2512.13564.

- Wu, Y., Liang, S., Zhang, C., Wang, Y., Zhang, Y., Guo, H., Tang, R., &
  Liu, Y. (2025). From Human Memory to AI Memory: A Survey on Memory Mechanisms
  in the Era of LLMs. arXiv:2504.15965.

- Du, Y., et al. (2025). Rethinking Memory in AI: Taxonomy, Operations, Topics,
  and Future Directions. arXiv:2505.00675.

- Shan, L., et al. (2025). Cognitive Memory in Large Language Models.
  arXiv:2504.02441.

- Jiang, D., et al. (2026). Anatomy of Agentic Memory: Taxonomy and Empirical
  Analysis of Evaluation and System Limitations. arXiv:2602.19320.

### Production Systems

- Choudhary, T., et al. (2025). Mem0: Building Production-Ready AI Agents with
  Scalable Long-Term Memory. arXiv:2504.19413.

- Dempsey, D., et al. (2025). Graphiti: Building Real-Time, Multi-Layered
  Temporal Knowledge Graphs. arXiv:2501.13956.

- Packer, C., Fang, V., et al. (2023). MemGPT: Towards LLMs as Operating
  Systems. arXiv:2310.08560.

- Kang, et al. (2025). MemoryOS. arXiv:2506.06326.

- Xu, et al. (2025). A-MEM: Agentic Memory for LLM Agents. arXiv:2502.12110.

### Academic Research Systems

- Park, J. S., O'Brien, J. C., Cai, C. J., Morris, M. R., Liang, P., &
  Bernstein, M. S. (2023). Generative Agents: Interactive Simulacra of Human
  Behavior. *ACM UIST 2023*. arXiv:2304.03442.

- Shinn, N., Cassano, F., Gopinath, A., Narasimhan, K., & Yao, S. (2023).
  Reflexion: Language Agents with Verbal Reinforcement Learning. *NeurIPS 2023*.
  arXiv:2303.11366.

- Wang, G., Xie, Y., Jiang, Y., et al. (2023). Voyager: An Open-Ended Embodied
  Agent with Large Language Models. arXiv:2305.16291.

- Zhong, W., Guo, L., Gao, Q., Ye, H., & Wang, Y. (2024). MemoryBank:
  Enhancing Large Language Models with Long-Term Memory. *AAAI 2024*.
  arXiv:2305.10250.

- Liu, et al. (2023). Think-in-Memory: Recalling and Post-Thinking Enable LLMs
  with Long-Term Memory. arXiv:2311.08719.

### Neuroscience and Psychology

- Hebb, D. O. (1949). *The Organization of Behavior*. Wiley.

- Bliss, T. V. P., & Lomo, T. (1973). Long-lasting potentiation of synaptic
  transmission in the dentate area of the anaesthetized rabbit following
  stimulation of the perforant path. *Journal of Physiology*, 232(2), 331-356.

- Collins, A. M., & Loftus, E. F. (1975). A spreading-activation theory of
  semantic processing. *Psychological Review*, 82(6), 407-428.

- Tulving, E., & Thomson, D. M. (1973). Encoding specificity and retrieval
  processes in episodic memory. *Psychological Review*, 80(5), 352-373.

- McClelland, J. L., McNaughton, B. L., & O'Reilly, R. C. (1995). Why there
  are complementary learning systems in the hippocampus and neocortex.
  *Psychological Review*, 102(3), 419-457.

- Bjork, R. A., & Bjork, E. L. (1992). A new theory of disuse and an old
  theory of stimulus fluctuation. In *From Learning Processes to Cognitive
  Processes: Essays in Honor of William K. Estes* (Vol. 2, pp. 35-67).

- Anderson, M. C., Bjork, R. A., & Bjork, E. L. (1994). Remembering can cause
  forgetting: Retrieval dynamics in long-term memory. *Journal of Experimental
  Psychology: Learning, Memory, and Cognition*, 20(5), 1063-1087.

- Cowan, N. (2001). The magical number 4 in short-term memory: A
  reconsideration of mental storage capacity. *Behavioral and Brain Sciences*,
  24(1), 87-114.

- Ebbinghaus, H. (1885). *Uber das Gedachtnis*. Duncker & Humblot. Translated
  as *Memory: A Contribution to Experimental Psychology* (1913).

### Information Retrieval

- Cormack, G. V., Clarke, C. L. A., & Buettcher, S. (2009). Reciprocal Rank
  Fusion outperforms Condorcet and individual Rank Learning Methods.
  *SIGIR 2009*.
