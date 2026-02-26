# Alaya Brand Guidelines

## 1. Brand Essence

### Etymology

**Alaya** (Sanskrit: *alaya-vijnana*, "storehouse consciousness") comes from Yogacara Buddhist psychology. The alaya-vijnana is the eighth consciousness in the Yogacara model -- the persistent substrate that holds all *bija* (seeds) of experience. These seeds are not inert data; they are living potentials that ripen when conditions align and reshape through each moment of contact.

This is not decorative naming. The etymology encodes the library's core technical thesis: memory is not retrieval from a static store, it is a living process where every access transforms what is stored, and patterns of behavior emerge from accumulated impressions rather than explicit declaration.

### Positioning Statement

For agent developers building privacy-first, relationship-heavy AI agents, **Alaya** is an embeddable Rust memory library that provides a complete cognitive lifecycle -- consolidation, forgetting, perfuming, transformation -- in a single SQLite file with zero external dependencies. Unlike cloud-dependent systems that require infrastructure stacks and mandatory LLM calls, Alaya delivers neuroscience-grounded memory processes that run entirely on the developer's machine, with implicit preference emergence that no other system provides.

### Tagline

**Memory is a process, not a database.**

Secondary variants for specific contexts:

- **crates.io / Cargo.toml:** "A memory engine for conversational AI agents, inspired by neuroscience and Buddhist psychology"
- **Conference talks:** "What if your agent's memory worked like actual memory?"
- **Technical discussions:** "Three stores, Hebbian graph, Bjork forgetting, zero dependencies."

---

## 2. Brand Identity

### Name

The name is always written as **Alaya** -- capitalized, no all-caps, no camelCase. In code contexts, it follows Rust conventions:

| Context | Format | Example |
|---------|--------|---------|
| Prose, documentation, talks | Alaya | "Alaya provides three-tier memory" |
| Crate name | `alaya` | `cargo add alaya` |
| Struct names | `AlayaStore`, `AlayaConfig` | `let store = AlayaStore::open("memory.db")?;` |
| Environment variables | `ALAYA_` prefix | `ALAYA_LOG_LEVEL=debug` |
| File paths | `alaya` | `~/.alaya/`, `memory.db` |

Never: ALAYA, aLaYa, Alaya.ai, Alaya Memory, Alaya AI.

The name stands alone. It does not need qualifiers. If disambiguation is necessary, use "Alaya memory library" or "the Alaya crate."

### Logo Concept

Alaya is a terminal-native library. It does not have a graphical logo in the traditional sense. Its visual identity lives in:

- **ASCII art** for the README header -- a stylized seed glyph or storehouse motif rendered in monospace characters
- **Terminal color** -- when Alaya produces diagnostic output, it uses amber/gold (`#D4A017`) for key metrics and muted gray for secondary information
- **Favicon / crates.io icon** -- a minimal geometric seed form, monochrome, recognizable at 32x32 pixels

The visual identity should evoke: seeds germinating, neural networks forming, processes in motion. Not: databases, clouds, brain clipart, lotus flowers.

### Color Philosophy

Alaya's "colors" appear in terminal output, documentation syntax highlighting, and diagrams. The palette is functional, not decorative:

| Role | Color | Hex | Usage |
|------|-------|-----|-------|
| Primary | Amber/Gold | `#D4A017` | Key metrics, success states, emphasis |
| Secondary | Slate | `#64748B` | Secondary text, metadata, timestamps |
| Accent | Teal | `#0D9488` | Links, interactive elements, graph edges |
| Warning | Copper | `#B87333` | Decay notifications, threshold alerts |
| Background | Terminal default | -- | Never override the user's terminal theme |

Rule: Never set background colors. Alaya's output adapts to the developer's terminal, whether dark or light. Foreground colors only, and always with a no-color fallback (`NO_COLOR` environment variable support).

### Typography

As a Rust library, Alaya's typography is whatever the developer's editor and terminal use. Documentation typography follows:

- **README and docs:** Standard GitHub-flavored Markdown. No custom fonts. Monospace for all code, including inline references to types (`AlayaStore`, `Query`, `ScoredMemory`).
- **Diagrams:** Mermaid for architecture diagrams in documentation. ASCII art for terminal-rendered output.
- **Comments in code:** Standard Rust doc comments (`///` and `//!`). Prose in doc comments is written in complete sentences.
- **API documentation:** Generated via `cargo doc`. Follows the Rust API Guidelines for documentation structure: summary line, extended description, examples, panics, errors.

---

## 3. Voice & Tone

### Core Voice Attributes

Alaya speaks like a well-read systems programmer who has done the neuroscience reading and respects the developer's intelligence. The voice is:

**Technical but accessible.** Use precise terminology (Hebbian LTP, Bjork dual-strength, CLS consolidation) but always provide enough context for a developer who hasn't read the papers. Never assume the reader has a neuroscience background. Never dumb it down either.

**Research-grounded, not research-heavy.** Cite the science when it informs a design decision. Don't cite it to impress. The README links to papers; the API docs explain what the function does.

**Honest about tradeoffs.** If brute-force vector search degrades past 10K vectors, say so. If consolidation quality depends on the LLM provider the developer plugs in, say that too. Alaya does not hide limitations behind marketing language.

**Quiet confidence.** Alaya's comparison table in the README speaks for itself. The project does not need to attack competitors or inflate claims. "No other surveyed system models preference tradeoffs as emergent behavioral patterns" is a factual statement backed by a 50-system survey. State it plainly.

### Tone by Context

| Context | Tone | Example |
|---------|------|---------|
| README | Direct, inviting, technically precise | "One SQLite database, no external services." |
| API docs | Concise, example-driven, warnings where needed | "Panics if `depth` exceeds 10. Use `neighbors_bounded()` for untrusted input." |
| Error messages | Actionable, never blame the user | "Episode content is empty. Provide non-empty content or use `NewEpisode::quick()` for a placeholder." |
| Changelog | Factual, links to relevant discussion | "Add LTD (long-term depression) to Hebbian graph updates. Closes #42." |
| Conference talks | Narrative, research-contextual, demo-driven | "Bjork and Bjork showed in 1992 that forgetting is not failure -- it's a retrieval optimization. Alaya implements this." |
| GitHub issues | Collaborative, grateful, direct | "Thanks for the detailed reproduction. This is the deferred transaction upgrade trap -- we need BEGIN IMMEDIATE here." |

### Language Rules

1. **Active voice.** "Alaya decays retrieval strength" not "Retrieval strength is decayed by Alaya."
2. **Present tense for features.** "Alaya stores episodes" not "Alaya will store episodes."
3. **Second person for guides.** "You can query with embeddings" not "One can query with embeddings" or "The user can query."
4. **No superlatives without evidence.** Never say "best" or "fastest" or "most advanced." Say what it does and let the reader decide.
5. **No filler words.** Cut "very," "really," "actually," "basically," "simply," "just." If something is simple, the code example proves it.
6. **Concrete over abstract.** "Single SQLite file, no network calls" not "lightweight and privacy-focused."
7. **Abbreviations on second use.** Spell out on first reference: "Complementary Learning Systems (CLS) theory." Then use "CLS" thereafter.

---

## 4. Core Beliefs

These beliefs are not slogans. They are architectural commitments that constrain implementation decisions. Every feature, API design choice, and documentation section should be traceable to one or more of these beliefs.

### Belief 1: Memory is a process, not a database

Every retrieval changes what is remembered. The Hebbian graph reshapes through use -- co-retrieved nodes strengthen their links (LTP), unused links weaken (LTD). Retrieval-induced forgetting suppresses competitors of accessed memories. Consolidation distills episodes into semantic knowledge. The database is an implementation detail; the process is the product.

**Implication for design:** Alaya never provides a raw "get all memories" dump without side effects. Querying is an act that transforms the memory landscape. This is by design, not a bug. Document it clearly.

**Implication for documentation:** Never describe Alaya as a "memory store" or "memory database." It is a "memory engine" or "memory system." The word "store" appears only in `AlayaStore` (the struct name, chosen for Rust API conventions) and the three internal stores (episodic, semantic, implicit), which are components of the larger process.

### Belief 2: Forgetting is a feature

Strategic decay improves retrieval quality over time. Bjork's dual-strength model distinguishes storage strength (how deeply encoded a memory is) from retrieval strength (how accessible it is right now). Retrieval strength decays, and that decay rate depends inversely on storage strength. Well-encoded memories resurface when needed; poorly-encoded ones fade. This is not data loss -- it is signal refinement.

**Implication for design:** The `forget()` lifecycle method is not a cleanup routine. It is a core cognitive process. Alaya archives low-retrieval-strength nodes rather than deleting them, preserving the possibility of resurrection if storage strength remains high. The default configuration enables forgetting. Turning it off is an opt-out, not an opt-in.

**Implication for documentation:** Never apologize for forgetting. Frame it as retrieval optimization. "Memories that haven't been accessed fade from immediate recall, but deeply-encoded experiences persist in cold storage and can be reactivated."

### Belief 3: Preferences emerge, they are not declared

The vasana (perfuming) model lets behavioral patterns crystallize from accumulated observations. Alaya does not ask the agent to report "user prefers simplicity." Instead, it observes episodes where simplicity competes with other values, tracks resolution patterns, and crystallizes a preference with context-dependent win ratios and confidence scores. The preference "simplicity > performance in API design contexts (85% win rate, high confidence, stable trend)" emerges from data, not declaration.

**Implication for design:** Alaya never exposes a `set_preference()` method. Preferences are write-only through the perfuming lifecycle. The API provides `preferences()` and `resolve_tradeoff()` for reading, and `perfume()` for processing new observations. The developer cannot manually insert a preference -- only feed the system interactions from which preferences emerge.

**Implication for documentation:** Explain the two-phase model (impression accumulation then crystallization) early in the guide. Developers coming from key-value "user profile" systems will expect `set_preference("theme", "dark")`. Alaya does something fundamentally different and this must be clear before they hit the API.

### Belief 4: The agent owns identity

Alaya stores seeds. The agent decides which seeds matter and how to present them. Alaya does not assemble prompts, does not call LLMs, does not decide what to say. It provides structured memory that the agent incorporates into its own identity and context assembly. The `SOUL.md` or system prompt or persona definition lives in the agent, not in Alaya.

**Implication for design:** Alaya's query results are `ScoredMemory` structs with content, scores, and metadata. It does not return formatted prompts or context windows. The agent developer decides how to integrate memory into their prompt pipeline. Alaya is maximally useful to agents with diverse architectures precisely because it does not assume any particular prompt format.

**Implication for documentation:** Always show Alaya as one component in a larger agent architecture. Diagrams should depict the agent calling Alaya, not Alaya wrapping the agent. The agent is the subject; Alaya is the tool.

### Belief 5: Graceful degradation

No embeddings? BM25-only retrieval still works. No LLM for consolidation? Episodes accumulate until one is available. No graph data yet? Retrieval proceeds on text and vector signals alone. Every feature works independently. The system improves as more capabilities are provided, but it never fails because an optional dependency is absent.

**Implication for design:** Every retrieval pathway must have a fallback. The RRF fusion pipeline handles missing signals gracefully -- if only BM25 scores exist, RRF uses only BM25 rankings. The `NoOpProvider` is a first-class citizen, not a testing stub. A developer who calls `cargo add alaya` and writes three lines of code gets a working memory system. Everything above that baseline is incremental improvement.

**Implication for documentation:** The Quick Start section must work without embeddings and without an LLM provider. Embeddings and custom providers appear in subsequent examples, framed as "enhancing" the baseline, not "enabling" it.

---

## 5. What We Are Not (Kill List)

These are anti-identities -- things Alaya deliberately refuses to become. They constrain scope and prevent drift. Each item has a specific reason.

### Not cloud-dependent

Alaya makes zero network calls. Memory never leaves the machine unless the developer explicitly exports it. There is no Alaya cloud, no telemetry, no analytics, no "sync" feature. Privacy is architectural, not policy-based -- there is no server to breach because there is no server.

**Why:** The target developers are building agents that handle intimate personal data (coaches, companions, personal assistants). "We promise not to look" is not sufficient. "There is nothing to look at" is.

### Not enterprise

Alaya does not target multi-tenant SaaS deployments, horizontal scaling, or team collaboration features. It is embedded in a single agent process. If an organization needs shared memory across agents, they build that coordination layer themselves, using Alaya as the per-agent memory substrate.

**Why:** Enterprise requirements (RBAC, audit logging, multi-region replication, SLAs) would bloat the library and dilute focus. The cognitive memory lifecycle is hard enough to get right for one agent. Solve that first.

### Not LLM-coupled

Alaya works without any LLM. Consolidation, impression extraction, and contradiction detection benefit from an LLM through the `ConsolidationProvider` trait, but they are not required. The library compiles, runs, stores episodes, retrieves memories, and runs forgetting cycles with zero LLM calls.

**Why:** LLM coupling creates vendor lock-in, cost dependencies, latency requirements, and privacy concerns. Agent developers should choose their LLM (or choose none) independently of their memory system.

### Not hype-driven

Alaya does not use trending AI marketing language. No "AGI-ready." No "autonomous memory." No "self-evolving intelligence." The README uses words like "Hebbian," "Bjork," "RRF," and "SQLite" because those describe what the system actually does.

**Why:** The target developers are systems programmers and agent builders who evaluate libraries by reading source code and running benchmarks. Hype language signals that the project has more marketing than substance. Alaya's substance speaks through its architecture and its comparison table.

### Not a standalone service

Alaya is a library. `cargo add alaya`. It links into the agent's process. There is no Alaya daemon, no Alaya API server, no Alaya Docker image (the `alaya-mcp` server is a separate crate that wraps the library for MCP integration, not a service in its own right).

**Why:** Services introduce operational burden (deployment, monitoring, networking, auth). The target developers want memory as a dependency, not as infrastructure. A library call is a function call -- no network hop, no serialization, no auth token.

### Not procedural memory

Alaya does not store executable skills, tool-use patterns, or action sequences. It stores what the agent has observed and learned about the user, not what the agent knows how to do. Procedural memory ("how to call the GitHub API") belongs in the agent's code, not in its memory system.

**Why:** Procedural and declarative memory have fundamentally different access patterns, update semantics, and failure modes. Mixing them in one system produces a system that does neither well.

### Not parametric memory

Alaya does not fine-tune models. It does not modify weights. It operates entirely in the non-parametric domain -- storing, retrieving, and processing explicit memory representations. Model adaptation is the agent's concern.

**Why:** Parametric and non-parametric memory serve different temporal horizons and have different cost profiles. Alaya focuses on non-parametric memory because it is the layer most agent frameworks neglect.

---

## 6. Design Principles

These principles govern how Alaya presents itself through its primary surfaces: documentation, API design, and open-source project management.

### Documentation Principles

**Show, don't tell.** Every concept gets a code example. The Quick Start works in under 5 minutes with copy-paste. Architecture descriptions include Mermaid diagrams. Lifecycle processes include before/after state illustrations.

**Progressive disclosure.** The README covers: what it is (30 seconds), why it matters (2 minutes), quick start (5 minutes), architecture (10 minutes), research foundations (as-needed reference). Detailed API documentation lives in `cargo doc`. Deep dives live in `docs/`.

**Honest comparison tables.** The competitor comparison in the README includes columns where competitors outperform Alaya (adoption, ecosystem, cloud features). The table is comprehensive and fair, not cherry-picked.

**Research citations as footnotes, not features.** Reference papers in parenthetical citations for developers who want to go deeper. Never require the reader to understand the paper to use the library.

### API Design Principles

**Three-line quick start.** `AlayaStore::open()`, `store_episode()`, `query()`. A developer evaluating the library should have working code in under a minute.

**Defaults that work.** Default configuration produces good results without tuning. Decay rates, retrieval weights, RRF parameters, and graph traversal depths all ship with researched defaults. Builder configuration exists for developers who need to tune.

**Types over strings.** Roles are enums (`Role::User`, `Role::Assistant`), not strings. Node types are enums. Link types are enums. The compiler catches mistakes that string-typed APIs defer to runtime.

**Errors are actionable.** Every error variant in `AlayaError` tells the developer what happened and what to do about it. No generic "operation failed" messages. Errors carry context: which episode, which query, which constraint was violated.

**Lifecycle is explicit.** The developer calls `consolidate()`, `forget()`, `transform()`, `perfume()` -- or the convenience `dream()` method that chains them. Alaya does not run background processes. The agent controls when cognition happens.

### Open Source Principles

**CHANGELOG is a contract.** Every release includes a human-readable changelog entry. Breaking changes are called out explicitly with migration guidance.

**Issues are conversations.** Bug reports receive acknowledgment within 48 hours. Feature requests receive honest assessment of alignment with project scope. "This is a great idea but it's outside Alaya's scope" is a valid and respectful response.

**PRs include context.** Why the change was made, not just what changed. Link to the issue. Explain the design choice. Future maintainers (including the original author six months later) need the "why."

**Benchmarks are reproducible.** Published benchmarks include the exact hardware, dataset, configuration, and commands to reproduce. No "up to X% improvement" without methodology.

---

## 7. Anti-Patterns (What Alaya Never Does in Communication)

These are specific patterns to actively avoid in all Alaya-related communication: documentation, talks, social media, issue responses, and README content.

### Never use AI hype language

| Avoid | Use Instead |
|-------|-------------|
| "AI-powered memory" | "Memory engine for AI agents" |
| "Intelligent retrieval" | "Hybrid retrieval with BM25, vector similarity, and graph activation" |
| "Self-evolving" | "Graph reshapes through use via Hebbian LTP/LTD" |
| "Autonomous memory management" | "Developer-controlled lifecycle processes" |
| "Next-generation" | Describe the specific capability |
| "Revolutionary" | Describe what is new and cite the comparison |
| "Powered by AI" | Alaya powers the AI, not the other way around |

### Never claim universality

Alaya is purpose-built for a specific use case: long-term conversational memory for privacy-first agents. It is not "memory for every AI application." It is not suitable for multi-user analytics platforms, real-time recommendation engines, or enterprise knowledge bases. Saying what Alaya is for is inseparable from saying what it is not for.

### Never hide behind complexity

If a developer asks "why should I use Alaya instead of just storing JSON in SQLite?" the answer is concrete: "Because Alaya's Hebbian graph learns which memories relate to each other through use, Bjork forgetting keeps retrieval quality high as memory grows, and vasana perfuming extracts preference tradeoffs you'd have to build from scratch. Here's a benchmark showing retrieval quality at 10K memories vs. naive SQLite storage."

### Never overpromise the Buddhist connection

The Yogacara psychology concepts (alaya-vijnana, bija, vasana, asraya-paravrtti) are genuine architectural inspirations, not metaphors. But Alaya is a software library, not a philosophical system. Use the Sanskrit terms when they precisely name a technical concept (vasana names the perfuming process more precisely than any English equivalent). Do not use them to add mystical cachet. Never imply that using Alaya is a spiritual practice or that the library embodies Buddhist teachings in any religious sense.

### Never gatekeep with jargon

Terms like "Hebbian LTP" and "Bjork dual-strength" appear in documentation because they are precise. But every technical term must be accompanied by enough context for a developer who has never encountered it. "Hebbian LTP (long-term potentiation) -- links strengthen when nodes are retrieved together, like neurons that fire together wiring together" is accessible. "Hebbian LTP is applied to co-activated nodes in the graph overlay" is not, without prior context.

### Never disparage competitors

The comparison table is factual and comprehensive. Mem0 requires external infrastructure -- this is a fact, not a criticism. Letta's LLM-as-memory-manager design is a legitimate architectural choice that serves different requirements. Alaya occupies a different point in the design space. State the differences. Let the developer decide which tradeoffs serve their project.

---

## 8. Social Positioning

### How Developers Describe Alaya

The goal is that developers who have used Alaya describe it in their own words roughly as follows:

- "It's like if your agent's memory actually worked like human memory -- things that matter stick around, things that don't fade, and it figures out what you care about without you telling it."
- "Single SQLite file, no infra, cargo add and go. But under the hood it's doing real neuroscience-based memory management."
- "The preference emergence thing is wild. After a few hundred conversations, it knew that I prioritize readability over performance in library code but flip that in hot paths. I never told it that."

### Addressing the Name

Some developers will question the Buddhist-inspired name. Anticipate and address this directly:

**"Why the Sanskrit name?"** Because alaya-vijnana precisely names what the library does -- it is a persistent substrate that holds seeds of experience which transform through interaction. No English term captures this as concisely. The Yogacara model also provided the architectural blueprint for the perfuming system (vasana), which is Alaya's most novel feature. The name is technical etymology, not branding ornamentation.

**"Is this cultural appropriation?"** The Yogacara concepts are used here in the same spirit that neural networks borrow "neuron," "synapse," and "plasticity" from neuroscience -- as precise technical vocabulary that names real computational processes. The README and documentation cite the philosophical sources, explain the mappings accurately, and do not trivialize the tradition. The author has studied the source material and presents it with care.

**"I can't pronounce it."** Ah-LAH-yah. Three syllables. Rhymes with "papaya." Easier than "Kubernetes."

### Reframing Academic vs. Practical

Alaya draws heavily from academic research (CLS theory, Bjork forgetting, spreading activation, Yogacara psychology). Some developers will perceive this as "academic" in the pejorative sense -- theoretically interesting but impractical. The reframe:

**The research is the engineering.** Bjork forgetting is not a citation for credibility; it is the algorithm that keeps retrieval quality high as memory grows. CLS consolidation is not a reference for the bibliography; it is the process that converts raw conversation logs into structured knowledge without losing the originals. Spreading activation is not a neuroscience footnote; it is the retrieval pathway that finds relevant memories that keyword search and vector similarity both miss.

**Benchmarks settle the argument.** When LoCoMo and LongMemEval results are published, they provide concrete evidence that the research-informed architecture produces better retrieval quality than ad-hoc alternatives. The academic foundations are means, not ends.

### Conference and Presentation Framing

Alaya's creator has access to DEF CON presentation channels. The library's security-conscious, privacy-by-architecture design aligns naturally with the security community. Presentation framing:

- **Security angle:** "Your agent's memory is a persistence layer that never phones home. Here's what that means for threat modeling conversational AI."
- **Research angle:** "We surveyed 50+ memory systems and found that none model preference tradeoffs as emergent behavioral patterns. Here's why that matters and how we built it."
- **Practical angle:** "Three lines of Rust, one SQLite file, zero ops. Here's what your agent remembers after 10,000 conversations."

---

## 9. Licensing & Ethics

### License: MIT

Alaya is released under the MIT license. The full text is in the repository root at `LICENSE`.

### Why MIT

- **Maximum adoption:** MIT is the most permissive widely-recognized license. Agent developers can embed Alaya in proprietary products, open-source projects, research prototypes, and commercial agents without legal friction.
- **Ecosystem compatibility:** MIT is compatible with Apache 2.0, GPL, and virtually every other open-source license. Alaya can be a dependency anywhere.
- **Simplicity:** MIT fits in a single page. Developers don't need a lawyer to evaluate it.
- **Aligned with goals:** Alaya's success metric is adoption -- agent developers shipping products on Alaya where users notice the difference. Restrictive licensing would be a barrier to the primary goal.

### Privacy Commitments

These are architectural commitments, not policy promises:

1. **No telemetry.** Alaya does not collect, transmit, or phone home any data. There is no opt-in, no opt-out, because there is no mechanism.
2. **No network calls.** The library makes zero network connections. DNS resolution, HTTP requests, WebSocket connections -- none. The developer's agent may make network calls; Alaya does not.
3. **Single-file storage.** All memory lives in one SQLite file at a path the developer chooses. Moving the file moves all memory. Deleting the file deletes all memory. There is no hidden state, no temp files, no registry entries.
4. **GDPR-compatible deletion.** `purge()` performs hard deletes followed by `VACUUM`, ensuring data is physically removed from the SQLite file. Crypto-shredding support is planned for field-level encryption scenarios.
5. **No model calls.** Alaya never calls an LLM or embedding model. The developer provides these capabilities through traits if they choose. Alaya cannot leak data to an AI provider because it has no connection to one.

### Ethical Boundaries

Alaya is a general-purpose memory library. It does not make ethical decisions about what memories to store or retrieve. However, the project maintains these positions:

- **Alaya provides the tools for responsible memory management.** Forgetting, purging, and scoping are first-class features, not afterthoughts. A developer who needs GDPR compliance, PII scrubbing, or session isolation has the primitives to build it.
- **Alaya does not surveil.** The library stores what the agent feeds it. It does not intercept, monitor, or capture data outside the explicit API calls. The developer controls the data pipeline entirely.
- **Documentation includes security guidance.** Memory poisoning (OWASP ASI06), FTS5 injection, embedding poisoning, and data leakage are documented threats with recommended mitigations. The project treats security documentation as a core deliverable, not an appendix.

---

## 10. Brand Governance

### Who Decides

As a solo-maintained open-source project, brand decisions are made by the maintainer. Community feedback via GitHub issues and discussions is welcomed and seriously considered, but final decisions on naming, positioning, and scope rest with the project author.

### When to Reference These Guidelines

- Writing or editing README, docs, or doc comments
- Preparing conference talks or blog posts about Alaya
- Responding to "what is Alaya?" in GitHub issues, forums, or social media
- Reviewing PRs for documentation or error message changes
- Deciding whether a proposed feature aligns with project scope

### Review Cadence

These guidelines are reviewed and updated:

- After each minor version release (0.x.0)
- When the competitive landscape shifts materially
- When a new core belief or anti-goal is identified through development experience
- When community feedback reveals a positioning gap

### Consistency Checks

Before any public-facing text is published (README update, blog post, conference abstract), verify:

1. Does it use "memory engine" or "memory system," not "memory database" or "memory store"?
2. Does it frame forgetting positively?
3. Does it show Alaya as a component the agent uses, not a wrapper around the agent?
4. Does it avoid every term in the anti-pattern table?
5. Does it work without assuming the reader has neuroscience knowledge?
6. Are Sanskrit terms used precisely and accompanied by sufficient context?
7. Is every claim backed by a specific capability, benchmark, or architectural fact?
