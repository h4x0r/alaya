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

#### Supermemory

- **Citation:** Supermemory. GitHub: supermemoryai/supermemory. $2.6M seed
  (Susa Ventures, backed by Google AI chief Jeff Dean).
- **Architecture:** Knowledge graph + vector store + graph database. Brain-
  inspired with smart forgetting, decay curves, recency bias, and context
  rewriting. Memories are indexed into both vector store and graph database.
  Memories can update, extend, derive, and expire.
- **CoALA mapping:** Episodic + semantic memory with graph overlay.
- **Retrieval:** Hybrid vector + graph retrieval.
- **LLM dependency:** Required for memory extraction and context rewriting.
- **Forgetting:** Yes — decay curves with recency bias. Memories expire.
- **Key context:** 16.6K GitHub stars. Claims 10x faster than Zep, 25x faster
  than Mem0. Founded by 19-year-old Dhravya Shah. Customers include Cluely
  (a16z-backed). TypeScript.

#### Hindsight (Vectorize)

- **Citation:** Latimer et al. (2025). "Hindsight is 20/20: Building Agent
  Memory that Retains, Recalls, and Reflects." arXiv:2512.12818.
- **Architecture:** Four logical memory networks: world (objective facts),
  experience (agent experiences), opinion (subjective beliefs with confidence
  scores that evolve), observation (preference-neutral entity summaries). Tempr
  for temporal entity memory priming retrieval. Cara for coherent adaptive
  reasoning.
- **CoALA mapping:** Episodic + semantic + opinion memory (novel category).
- **Retrieval:** Temporal entity priming + adaptive reasoning.
- **LLM dependency:** Required. Supports OpenAI, Anthropic, Gemini, Groq,
  Ollama.
- **Forgetting:** Not explicit — belief confidence scores evolve over time.
- **Key results:** Claims SOTA: 91.4% LongMemEval, 89.61% LoCoMo.
  Distinguishes facts from opinions with confidence-scored belief evolution.

#### Cortex-Mem

- **Citation:** sopaco. GitHub: sopaco/cortex-mem. cortexmemory.dev.
- **Architecture:** Production-ready memory framework. Automatic extraction,
  vector search, deduplication, optimization. REST API + MCP server + CLI +
  insights dashboard.
- **CoALA mapping:** Semantic memory (extracted facts).
- **Storage:** Configurable. **Written in Rust.**
- **Retrieval:** Vector similarity search.
- **LLM dependency:** Required for fact extraction.
- **Forgetting:** Not documented.
- **Key context:** The closest Rust-based competitor. Same language as Alaya,
  but positioned as a standalone service (REST/MCP) rather than an embeddable
  library. Claims 60-90% storage savings via deduplication.

#### Memvid

- **Citation:** Memvid. GitHub: memvid/memvid. memvid.com.
- **Architecture:** Video-encoding-inspired architecture with Smart Frames as
  append-only immutable units. Embedded WAL for crash recovery. Single `.mv2`
  binary format file. ONNX local embeddings (bge-small, nomic-embed).
- **CoALA mapping:** Episodic memory (append-only).
- **Storage:** Single `.mv2` file. **Written in Rust** (V2 rewrite).
- **Retrieval:** Tantivy full-text + HNSW vector + chronological time indexing.
- **LLM dependency:** None for storage/retrieval. Local ONNX embeddings.
- **Forgetting:** None — append-only, immutable.
- **Key results:** Claims SOTA on LoCoMo (+35%). 0.025ms P50 / 0.075ms P99.
  1,372x throughput vs. standard approaches. 13.2K GitHub stars. Python + Node
  + Rust + CLI + MCP bindings.
- **Key context:** The closest deployment model to Alaya (Rust, single-file,
  zero dependencies). Key difference: Memvid is append-only with no forgetting
  or consolidation; Alaya has cognitive lifecycle processes.

#### Redis Agent Memory Server

- **Citation:** Redis Labs. GitHub: redis/agent-memory-server.
- **Architecture:** Auto-extracts, organizes, deduplicates memories. REST API +
  MCP interfaces. Topic extraction (LLM or BERTopic), NER, HNSW vector
  indexing. Multi-tenancy.
- **CoALA mapping:** Semantic memory (extracted topics and entities).
- **Storage:** Redis (default) + Pinecone/Chroma/PostgreSQL backends.
- **Retrieval:** HNSW vector search + topic filtering.
- **LLM dependency:** Required for extraction. 100+ providers via LiteLLM.
- **Forgetting:** Not documented.
- **Key context:** Official Redis project (v0.13.1). Docker deployment.
  Represents a major infrastructure company entering the memory space.

#### LangMem SDK

- **Citation:** LangChain team. GitHub: langchain-ai/langmem.
- **Architecture:** Dedicated long-term memory SDK, distinct from the deprecated
  LangChain Memory classes. Semantic memory (facts/preferences) + procedural
  memory (saved as updated prompt instructions). Functional primitives +
  LangGraph storage integration. Namespace-based multi-tenancy.
- **CoALA mapping:** Semantic + procedural memory.
- **Retrieval:** Vector similarity via LangGraph store.
- **LLM dependency:** Required for extraction.
- **Forgetting:** Not documented.
- **Key context:** LangChain's actual memory engine for production use.
  48.72 F1 on LoCoMo. 1.3K GitHub stars.

---

### Standalone Memory Servers

#### Motorhead (Metal)

- **Citation:** Metal. GitHub: getmetal/motorhead. Apache 2.0. YC-backed.
- **Architecture:** Flat conversation buffer with incremental summarization.
  Stores messages per session with a configurable window size (default 12). When
  the window fills, the oldest half is summarized and the summary is
  incrementally updated.
- **CoALA mapping:** Working memory (buffer) with rudimentary episodic
  (summaries).
- **Storage:** Redis (required). Redisearch VSS for long-term retrieval.
- **Language:** Rust server with Python/JS client libraries.
- **Retrieval:** Session-based retrieval (GET messages). Vector similarity via
  Redisearch VSS. Three simple REST endpoints.
- **LLM dependency:** Required for incremental summarization (default:
  gpt-3.5-turbo).
- **Forgetting:** Window-based eviction — oldest half summarized and removed.
  No time-based decay.
- **Key context:** Written in Rust for performance. Extremely simple API.
  LangChain integration via MotorheadMemory class. Less actively maintained as
  of 2025.

#### Engram

- **Citation:** Gentleman-Programming. GitHub: Gentleman-Programming/engram.
- **Architecture:** Flat, agent-directed memory. The agent (Claude Code, Gemini
  CLI, etc.) decides what to save. Session-based with automatic context
  injection on session start and summary generation on session end.
- **CoALA mapping:** Working memory + agent-directed episodic writes.
- **Storage:** SQLite + FTS5 (full-text search). Single binary, single file.
- **Language:** Go.
- **Retrieval:** Full-text search via FTS5. No vector search. Agent proactively
  saves relevant memories.
- **LLM dependency:** None for storage/retrieval. The LLM client decides what to
  remember.
- **Forgetting:** No built-in mechanism.
- **Key context:** Zero dependencies — single Go binary. MCP server + HTTP API +
  CLI + TUI. Designed for coding agents. Agent-trusting philosophy (agent
  decides what is worth remembering). Closest in spirit to Alaya's
  zero-dependency approach, though without vector search, graph, or lifecycle
  processes.

#### OpenViking (ByteDance / Volcengine)

- **Citation:** Volcengine. GitHub: volcengine/OpenViking. Open-sourced January
  2026.
- **Architecture:** Virtual filesystem paradigm. Abandons fragmented vector
  storage in favor of a `viking://` protocol that maps all context (memories,
  resources, skills) to virtual directories. Three-tier structure: L0 (immediate
  context), L1 (session-level), L2 (persistent/archival).
- **CoALA mapping:** Working memory (L0) + episodic (L1) + semantic (L2).
- **Storage:** Custom virtual filesystem built on VikingDB vector database
  infrastructure.
- **Retrieval:** Directory recursive retrieval combining directory positioning
  with semantic search. Hierarchical context delivery.
- **LLM dependency:** Required for context processing and agent interaction.
- **Forgetting:** Not explicitly documented. Tiered loading implicitly
  deprioritizes unused context.
- **Key context:** Novel filesystem metaphor for context management. Built by
  the team behind VikingDB (serves all ByteDance production workloads since
  2019). Designed for coding agents (e.g., OpenClaw). Strongly opinionated
  against traditional RAG fragmentation. ~2.9K GitHub stars.

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

#### HippoRAG / HippoRAG 2 (OSU NLP, 2024-2025)

- **Citation:** Gutierrez et al. (2024). "HippoRAG: Neurobiologically Inspired
  Long-Term Memory for Large Language Models." NeurIPS 2024. arXiv:2405.14831.
  Gutierrez et al. (2025). "From RAG to Memory." ICML 2025. arXiv:2502.14802.
- **Architecture:** Inspired by hippocampal indexing theory. LLM acts as
  neocortex, PHR encoder detects synonymy, open knowledge graph acts as
  hippocampus. Retrieval via Personalized PageRank on the KG.
- **Forgetting:** Not built-in.
- **Key results:** 20% improvement on multi-hop QA. 10-30x cheaper than
  iterative retrieval. ~3.7K GitHub stars.
- **Significance:** Top-venue neuroscience-inspired memory. PPR on KG is
  comparable to Alaya's spreading activation on Hebbian graph — different
  mechanisms, similar cognitive inspiration.

#### SYNAPSE (Jiang et al., 2026)

- **Citation:** Jiang et al. (2026). "SYNAPSE: Empowering LLM Agents with
  Episodic-Semantic Memory via Spreading Activation." arXiv:2601.02744.
- **Architecture:** Unified episodic-semantic graph with spreading activation
  AND lateral inhibition (biological mechanisms). Triple hybrid retrieval fusing
  geometric embeddings with activation-based graph traversal. Temporal decay.
- **Key results:** +7.2 F1 on LoCoMo (SOTA at publication). 23% improvement on
  multi-hop reasoning. 95% token reduction vs. full context.
- **Significance:** The most directly comparable system to Alaya's retrieval
  approach. Both use spreading activation over episodic-semantic graphs. SYNAPSE
  adds lateral inhibition (analogous to Alaya's RIF suppression).

#### Mem-alpha (Wang et al., 2025)

- **Citation:** Wang et al. (2025). "Mem-alpha: Learning Memory Construction
  via Reinforcement Learning." arXiv:2509.25911.
- **Architecture:** RL framework training agents to manage core, episodic, and
  semantic memory. Reward signal from downstream QA accuracy. Trained on 30K
  tokens, generalizes to 400K+ (13x extrapolation).
- **Key results:** Apache 2.0. GitHub: wangyu-ustc/Mem-alpha.
- **Significance:** Nearly identical three-component memory decomposition to
  Alaya (core + episodic + semantic). Key difference: Mem-alpha learns
  management via RL; Alaya uses cognitive principles.

#### MAGMA (Jiang et al., 2026)

- **Citation:** Jiang et al. (2026). "MAGMA: A Multi-Graph based Agentic Memory
  Architecture." arXiv:2601.03236.
- **Architecture:** Four orthogonal graphs per memory item: semantic, temporal,
  causal, and entity. Adaptive traversal policy routes retrieval based on query
  intent. Dual-stream write: fast ingestion + async consolidation.
- **Key results:** SOTA on LoCoMo and LongMemEval.
- **Significance:** Multi-graph decomposition is a different approach from
  Alaya's single Hebbian graph with multiple edge types. MAGMA separates
  graph types; Alaya unifies them with typed, weighted edges.

#### LightMem (Fang et al., 2025)

- **Citation:** Fang et al. (2025). "LightMem: Lightweight and Efficient
  Memory-Augmented Generation." ICLR 2026. arXiv:2510.18866.
- **Architecture:** Atkinson-Shiffrin-inspired three-stage pipeline: sensory
  memory (compression + topic grouping), short-term (topic-aware consolidation),
  long-term with "sleep-time" offline consolidation.
- **Key results:** 38x token reduction, 30x fewer API calls, 12.4x faster.
- **Significance:** Best efficiency profile. Sleep-time consolidation pattern
  parallels Alaya's offline lifecycle processes. Both inspired by
  complementary learning systems.

#### MemTree (Rezazadeh et al., 2025)

- **Citation:** Rezazadeh et al. (2025). "MemTree: Dynamic Tree Memory." ICLR
  2025. arXiv:2410.14052.
- **Architecture:** Dynamic tree-structured memory mimicking cognitive schemas.
  Hierarchical nodes at varying abstraction levels. New information routes from
  root to matching leaf. Ancestor nodes integrate via summarization.
- **Significance:** Tree vs. graph is a fundamental structural difference from
  Alaya. MemTree's hierarchical schemas enable top-down reasoning; Alaya's flat
  stores + graph overlay enable lateral associative reasoning.

#### RMM (Tan et al., 2025)

- **Citation:** Tan et al. (2025). "In Prospect and Retrospect: Reflective
  Memory Management for Long-term Personalized Dialogue Agents." ACL 2025.
  arXiv:2503.08026.
- **Architecture:** Prospective reflection (dynamic summarization across
  utterance/turn/session granularities) + retrospective reflection (online RL
  refinement of retrieval based on cited evidence).
- **Key results:** 10%+ accuracy improvement on LongMemEval.
- **Significance:** Explicitly designed for personalized conversational agents
  — the same target as Alaya.

### Additional Notable Systems

Systems with significant community traction or novel ideas, presented in
summary form:

| System | Type | Key Idea | Stars / Venue |
|--------|------|----------|---------------|
| **EverMemOS** | Memory OS | Self-organizing memory; encoding/consolidation/retrieval pipeline; 93% LoCoMo | 2.2K stars |
| **MemOS** | Memory OS | Textual + activation (KV cache) + parametric (LoRA) memory types | 5.9K stars |
| **OpenMemory** | Cognitive engine | Hierarchical Memory Decomposition + temporal graph; MCP native | 3.4K stars |
| **SimpleMem** | Compression | Semantic lossless compression via implicit density gating | 3.0K stars |
| **Memobase** | User profiling | Extracts user profiles from conversations; top LoCoMo scores | 2.6K stars |
| **IronClaw** | Rust agent | OpenClaw-inspired Rust agent with persistent hybrid-search memory | 3.5K stars |
| **LightRAG** | Graph RAG | Simple, fast graph-based RAG with KG extraction | 28.7K stars, EMNLP 2025 |
| **Memory-R1** | RL memory | RL-trained ADD/UPDATE/DELETE/NOOP; 48% F1 improvement over Mem0 | arXiv:2508.19828 |
| **MemRL** | RL memory | Runtime RL on episodic memory; MDP formalization of memory use | arXiv:2601.03192 |
| **AgeMem** | RL memory | Unified LTM/STM via progressive RL; SOTA on 5 long-horizon benchmarks | arXiv:2601.01885 |
| **G-Memory** | Multi-agent | Three-tier graph hierarchy for multi-agent systems | NeurIPS 2025 Spotlight |
| **CAM** | Constructivist | Piaget-inspired assimilation/accommodation of memory schemas | NeurIPS 2025 |
| **SGMem** | Lightweight | Sentence-level graphs; no LLM extraction needed; strong LoCoMo/LongMemEval | arXiv:2509.21212 |
| **RGMem** | Physics-inspired | Renormalization group multi-scale memory with phase transitions | arXiv:2510.16392 |
| **Memoria** | Weighted KG | Exponential weighted average for conflict resolution; 87.1% LongMemEvals | arXiv:2512.12686 |
| **CortexGraph** | Forgetting | Ebbinghaus forgetting curves; Markdown-compatible storage | GitHub |
| **PowerMem** | Hybrid | Vector + FTS + graph with Ebbinghaus forgetting; backed by OceanBase | GitHub |
| **Papr Memory** | Multi-DB | MongoDB + Qdrant + Neo4j; GraphQL API; 91% STARK accuracy | GitHub |

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
| **Motorhead** | Yes (buffer) | Partial (summaries) | No | No | No |
| **Engram** | Agent-managed | Yes (agent-directed) | No | No | No |
| **OpenViking** | Yes (L0 context) | Yes (L1 sessions) | Yes (L2 persistent) | No | No |
| **Supermemory** | Agent-managed | Yes | Yes (graph) | No | Yes (extraction) |
| **Hindsight** | Agent-managed | Yes (experience) | Yes (world + opinion) | No | Yes (reflection) |
| **Memvid** | Agent-managed | Yes (Smart Frames) | No | No | No |
| **HippoRAG** | Agent-managed | No | Yes (KG as hippocampus) | No | No |
| **SYNAPSE** | Agent-managed | Yes | Yes (unified graph) | No | Yes (activation-based) |
| **Mem-alpha** | Yes (core) | Yes | Yes | No | Yes (RL-learned) |
| **LangMem** | Agent-managed | No | Yes (extracted facts) | Yes (prompt updates) | No |

**Key observation:** Cross-memory learning (episodic -> semantic) remains rare.
Alaya, Generative Agents, SYNAPSE, and Mem-alpha implement principled
consolidation. Hindsight adds a novel "opinion" memory type with belief
evolution. Most systems still store everything flat or rely on one-shot LLM
extraction.

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
| **Engram** | SQLite + FTS5 | 0 services | Single Go binary | Fully local |
| **OpenViking** | VikingDB (virtual filesystem) | 1 service | VikingDB setup | Configurable |
| **Supermemory** | KG + vector + graph DB | 2-3 services | Docker or cloud | Cloud-dependent |
| **Hindsight** | Configurable | 1-2 services | Docker | Configurable |
| **Cortex-Mem** | Configurable (Rust) | 0-1 services | `cargo install` or Docker | Configurable |
| **Memvid** | Single `.mv2` file (Rust) | 0 services | Single binary | Fully local |
| **Redis Memory** | Redis + optional backends | 1+ services | Docker + Redis | Redis-dependent |
| **LangMem** | LangGraph store | 0-1 services | pip install | Configurable |
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
| **Motorhead** | No | Yes (Redisearch VSS) | No | N/A | N/A | Naive |
| **Engram** | FTS5 | No | No | N/A | N/A | Naive |
| **OpenViking** | No | Yes (VikingDB) | No | Directory positioning | Hierarchical | Advanced |
| **Supermemory** | No | Yes | Yes (graph) | Not specified | Yes | Advanced |
| **Hindsight** | No | Yes | No | Tempr (temporal priming) | Cara (adaptive) | Advanced |
| **Memvid** | Tantivy FTS | HNSW | No | Not specified | Chronological | Advanced |
| **HippoRAG** | No | Yes | Personalized PageRank | PPR scores | None | Modular |
| **SYNAPSE** | No | Yes | Spreading activation + lateral inhibition | Triple hybrid | Activation-based | Modular |
| **Mem-alpha** | No | Yes | No | RL-learned | RL-learned | Advanced |
| **LangMem** | No | Yes | No | N/A | N/A | Naive |

**Key observation:** Full three-signal retrieval (sparse + dense + graph) with
principled fusion is implemented by Alaya (RRF), Zep/Graphiti (RRF + MMR), and
SYNAPSE (triple hybrid). HippoRAG achieves comparable associative retrieval via
PPR on a knowledge graph. Most systems rely on vector similarity alone.

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
| **Motorhead** | Direct buffer append | Incremental summarization | Window eviction | Not built-in | Not built-in |
| **Engram** | Agent-directed save | Session summaries on end | Not built-in | Not built-in | Not built-in |
| **OpenViking** | Virtual file write | Tiered L0 -> L1 -> L2 | Implicit (tiered deprioritization) | Not built-in | Not built-in |
| **Supermemory** | LLM extraction | Not specified | Decay curves + expiry | Not specified | LLM-extracted |
| **Hindsight** | LLM extraction | Tempr (temporal priming) | Belief confidence evolution | Not specified | Opinion memory (novel) |
| **Memvid** | Append-only frames | Not built-in | None (immutable) | Not built-in | Not built-in |
| **HippoRAG** | LLM KG extraction | Not built-in | Not built-in | Not built-in | Not built-in |
| **SYNAPSE** | Direct storage | Activation-based | Temporal decay + lateral inhibition | Not built-in | Not built-in |
| **Mem-alpha** | RL-learned | RL-learned | RL-learned | Not built-in | Not built-in |
| **LangMem** | LLM extraction | Background manager | Not built-in | Not built-in | Via extracted facts |
| **LightMem** | Sensory compression | Sleep-time offline consolidation | Not specified | Not specified | Not built-in |

**Key observation:** Alaya remains the only system combining CLS-inspired
consolidation, dual-strength forgetting with RIF suppression, explicit
contradiction resolution, and preference crystallization. SYNAPSE comes closest
with temporal decay + lateral inhibition (analogous to RIF). LightMem's
sleep-time consolidation parallels Alaya's offline lifecycle. Hindsight
introduces opinion memory with belief evolution — a capability Alaya's vasana
model partially addresses from a different angle.

---

## Landscape Analysis

### Dominant Paradigms

Five architectural paradigms have emerged:

1. **Vector store** (flat semantic retrieval): ChromaDB, Pinecone, LangChain
   memory, Memvid. Simple, fast, but no structural understanding of
   relationships.

2. **Knowledge graph** (structured relationships): Zep/Graphiti, Mem0g, Cognee,
   HippoRAG, Supermemory. Rich relational reasoning, but typically requires
   external graph DB and LLM for construction.

3. **OS-inspired tiering** (RAM/disk metaphor): Letta, MemoryOS, SCM, MemOS,
   EverMemOS. Hierarchical management with promotion/eviction.

4. **RL-trained memory policies** (learned management): Mem-alpha, Memory-R1,
   MemRL, AgeMem. Instead of hand-crafted heuristics, train the memory policy
   via reinforcement learning. This is the strongest emerging trend in 2025-2026
   academic research.

5. **Parametric memory** (embedded in model weights): Second Me's L2 layer,
   MemoryLLM/M+. Zero retrieval latency for deeply learned knowledge, but
   requires fine-tuning infrastructure.

### Underserved Areas

**Forgetting:** MemoryBank (Ebbinghaus), Mem0 (exponential decay), Generative
Agents (recency decay), Supermemory (decay curves), SYNAPSE (temporal decay +
lateral inhibition), CortexGraph/PowerMem (Ebbinghaus), and Alaya (Bjork
dual-strength + RIF) implement principled forgetting. The Bjork model remains
the most theoretically grounded (distinguishing storage vs. retrieval strength),
but the field is catching up — Ebbinghaus-based decay is becoming common.

**Preference emergence:** Most systems either don't model preferences or rely on
LLM extraction (Mem0, Supermemory). Hindsight's opinion memory with confidence-
scored belief evolution is a notable new entrant. Alaya's vasana/perfuming model
— where preferences crystallize from accumulated impressions without explicit
extraction — remains unique.

**Adaptive retrieval:** CoALA identifies this as a major underexplored
direction. SYNAPSE's spreading activation + lateral inhibition and HippoRAG's
Personalized PageRank are the closest to Alaya's approach. RL-trained systems
(Mem-alpha, MemRL) learn retrieval policies from reward signals. Alaya's
Hebbian graph provides organic adaptation (paths reshape through use) without
explicit training.

**Cross-memory consolidation:** Now better represented: Alaya (CLS-inspired),
Generative Agents (reflection), SYNAPSE (activation-based), LightMem (sleep-
time), and EverMemOS (encoding/consolidation/retrieval pipeline). Still rare
relative to the total number of systems.

### The Graph Question

Graph memory is table stakes in 2026. The field has split into three approaches:

1. **Static knowledge graphs** (Zep/Graphiti, HippoRAG): LLM extracts entities
   and relationships. Graph doesn't change unless the LLM updates it.

2. **Multi-graph decomposition** (MAGMA): Separate semantic, temporal, causal,
   and entity graphs with query-adaptive traversal policies.

3. **Dynamic/Hebbian graphs** (Alaya, SYNAPSE): Links reshape through use —
   co-retrieval strengthens (LTP), disuse weakens (LTD). SYNAPSE adds lateral
   inhibition. No LLM intervention required for graph evolution.

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
| **Three-store architecture** | Episodic + semantic + implicit | Mem-alpha (core + episodic + semantic) | Nearly identical decomposition; Mem-alpha learns management via RL, Alaya uses cognitive principles |
| **Hebbian graph** | Dynamic, reshapes through use | SYNAPSE (spreading activation + lateral inhibition) | Both bio-inspired; SYNAPSE adds lateral inhibition, Alaya adds Hebbian LTP/LTD weight evolution |
| **Bjork forgetting** | Dual-strength decay + RIF suppression | MemoryBank / CortexGraph (Ebbinghaus) | Alaya models storage and retrieval strength independently; Ebbinghaus uses single-curve decay |
| **Vasana preferences** | Impressions crystallize into preferences | Hindsight (opinion memory with belief evolution) | Alaya's preferences emerge without LLM; Hindsight tracks opinions explicitly with confidence scores |
| **CLS consolidation** | Episodic -> semantic pipeline | LightMem (sleep-time offline consolidation) | Both inspired by CLS theory; LightMem adds sensory compression stage |
| **Zero external dependencies** | Single SQLite file, Rust | Memvid (single .mv2 file, Rust) | Both Rust, single-file, zero-dep; Memvid is append-only (no lifecycle), Alaya has full cognitive lifecycle |
| **LLM-agnostic** | Agent provides via traits, works with NO LLM | Memvid (ONNX local embeddings) | Both work without cloud LLM; Alaya degrades to BM25-only, Memvid uses local ONNX models |
| **RRF multi-signal fusion** | BM25 + vector + graph | Zep/Graphiti (cosine + BM25 + graph BFS) | Both three-signal + RRF; Zep adds MMR, Alaya adds spreading activation |

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

- Gutierrez, B. J., Shu, Y., Gu, Y., Yasunaga, M., & Su, Y. (2024).
  HippoRAG: Neurobiologically Inspired Long-Term Memory for Large Language
  Models. *NeurIPS 2024*. arXiv:2405.14831.

- Gutierrez, B. J., Shu, Y., Qi, W., Zhou, S., & Su, Y. (2025). From RAG to
  Memory: Non-Parametric Continual Learning for Large Language Models. *ICML
  2025*. arXiv:2502.14802.

- Jiang, H., et al. (2026). SYNAPSE: Empowering LLM Agents with Episodic-
  Semantic Memory via Spreading Activation. arXiv:2601.02744.

- Wang, Y., et al. (2025). Mem-alpha: Learning Memory Construction via
  Reinforcement Learning. arXiv:2509.25911.

- Jiang, D., et al. (2026). MAGMA: A Multi-Graph based Agentic Memory
  Architecture for AI Agents. arXiv:2601.03236.

- Fang, J., et al. (2025). LightMem: Lightweight and Efficient Memory-Augmented
  Generation. *ICLR 2026*. arXiv:2510.18866.

- Rezazadeh, et al. (2025). MemTree: Dynamic Tree Memory. *ICLR 2025*.
  arXiv:2410.14052.

- Tan, Z., et al. (2025). In Prospect and Retrospect: Reflective Memory
  Management for Long-term Personalized Dialogue Agents. *ACL 2025*.
  arXiv:2503.08026.

- Latimer, C., et al. (2025). Hindsight is 20/20: Building Agent Memory that
  Retains, Recalls, and Reflects. arXiv:2512.12818.

- Zhang, G., et al. (2025). G-Memory: Tracing Hierarchical Memory for Multi-
  Agent Systems. *NeurIPS 2025 Spotlight*. arXiv:2506.07398.

- Zhang, S., et al. (2026). MemRL: Self-Evolving Agents via Runtime
  Reinforcement Learning on Episodic Memory. arXiv:2601.03192.

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
