# Discovery Session

## Date
2026-02-26T12:40:00+08:00

## Exploration Summary

### What
Alaya is an embeddable Rust library that provides long-term cognitive memory for
AI agents. Three stores (episodic, semantic, implicit), a Hebbian graph overlay,
hybrid retrieval with spreading activation, and adaptive lifecycle processes
(consolidation, forgetting, perfuming, transformation). Single SQLite file, zero
external dependencies, LLM-agnostic.

### Why
Existing memory solutions fall into two camps, neither satisfying:
1. Cloud-dependent systems (Mem0, Supermemory) that require external infrastructure
   and an LLM for every operation — violating privacy requirements
2. Zero-dependency alternatives (Memvid, Engram) that are too shallow — they store
   memories but don't understand the user

No system provides the complete cognitive memory lifecycle with zero dependencies.
No system learns implicit preference tradeoffs from observed behavior.

### Who
Agent developers building:
- Privacy-first personal agents (memory never leaves the machine)
- Relationship-heavy agents (companions, coaches, coding agents with
  conversational depth like OpenClaw)

The developer is the customer (they call `cargo add alaya`). The end user is the
beneficiary (they notice their agent "knows" them after months of use).

### Differentiator
Implicit preference emergence with contextual tradeoff resolution. Alaya doesn't
just remember "user likes simplicity" — it learns that when simplicity conflicts
with performance, the user picks simplicity 85% of the time in API design contexts
but flips in production contexts. This emerges from observed behavior without LLM
extraction or explicit declaration.

No other system in the field — production or academic (surveyed 50+ systems) —
models preference tradeoffs as emergent behavioral patterns with contextual
ranking and trend detection.

### Key Quotes
> "I'm passionate about psychology and as a DEF CON Group president I can present
> research at DEF CON conferences easily"

> "Alaya understands the ranking of my memory / preference, when multiple ideas /
> beliefs / preferences come in conflict, Alaya knows my resolution"

> "I think Alaya should infer tradeoffs [from episode content, not agent-reported]"

## Design Decisions Made During Discovery

### Preference Tradeoff Resolution (New Feature)
- **PreferenceTension**: New primitive capturing observed conflicts between values
- **TradeoffPattern**: Emergent patterns with context, win ratio, confidence, trend
- **Query interface**: `resolve_tradeoff()`, `preference_ranking()`
- **Tension detection**: Alaya infers from episodes via ConsolidationProvider trait
  (same pattern as consolidation — agent sends raw conversations, lifecycle handles
  inference)
- **Value granularity**: Free-form strings with vector embeddings for clustering
  ("simplicity" and "keep it simple" group naturally via cosine similarity)

### API Surface Alignment
- Follows Mem0/Supermemory pattern: agent sends raw conversations, memory system
  does the intelligence. Agent developer's API surface stays simple.

## Open Questions
- Exact implementation of heuristic tension detection (for LLM-free fallback)
- Whether preference ranking should use pairwise comparison (Bradley-Terry) or
  simpler ratio-based approach
- Sample use case: build with ~/src/ccchat, build smaller demo, or convince an
  existing agent project to adopt
- Distribution strategy beyond DEF CON presentations
