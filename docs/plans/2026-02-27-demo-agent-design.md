# Alaya Scripted Walkthrough Demo — Design

**Date:** 2026-02-27
**Status:** Approved

## Goal

Build a dual-purpose demo: an interactive showcase that demonstrates Alaya's neuroscience-inspired features, and a developer reference showing the integration pattern.

## Decisions

- **Format:** Sequential "chapter" walkthrough (Approach A)
- **Provider:** Switchable -- rule-based `KeywordProvider` by default, `--llm` flag for LLM-backed (future extension)
- **Scenario:** Scripted walkthrough with pre-written conversations
- **Location:** `examples/demo.rs`

## Architecture

A single `examples/demo.rs` (~300-400 lines) runs 6 sequential chapters. Each chapter prints a header, runs Alaya operations, and prints annotated results. No external dependencies beyond what's already in Cargo.toml.

The rule-based `KeywordProvider` implements `ConsolidationProvider` using simple string matching -- no HTTP client, no API keys needed for the default mode.

## Chapter Outline

### Chapter 1: Episodic Memory (store + query)
- Store 8-10 episodes simulating a multi-session coding conversation (Rust, async, databases)
- Query "Rust async" -- show BM25 retrieval with scored results
- Print status report showing episode counts

### Chapter 2: Hebbian Graph (temporal links + co-retrieval)
- Show temporal links created during episode storage
- Run two overlapping queries -- trigger co-retrieval link strengthening
- Use `neighbors()` to show spreading activation from a seed node
- Print link counts and how association formed

### Chapter 3: Consolidation (episodic -> semantic)
- Run `consolidate()` with `KeywordProvider`
- Show ConsolidationReport: episodes processed, nodes created, links created
- Query `knowledge()` to display extracted semantic nodes with confidence scores

### Chapter 4: Perfuming (vasana -> preferences)
- Feed 6+ interactions through `perfume()` with consistent behavioral signals
- Show impressions accumulating per domain
- Watch preference crystallize at threshold (5+ impressions)
- Query `preferences()` to display crystallized behavioral patterns

### Chapter 5: Transformation (refinement)
- Run `transform()` to deduplicate, prune weak links, decay old preferences
- Show TransformationReport
- Compare status before/after

### Chapter 6: Forgetting (Bjork dual-strength)
- Run `forget()` multiple cycles
- Show retrieval strength decaying while storage strength remains
- Demonstrate memory revival by querying a "forgotten" memory
- Print final status showing archived vs retained

## KeywordProvider (Rule-Based)

Implements `ConsolidationProvider`:

- **extract_knowledge:** Scans episode text for "I use X", "X is Y", entity co-occurrence. Returns `NewSemanticNode` with type Fact/Relationship.
- **extract_impressions:** Detects "I prefer", "I like", "can you be more". Returns `NewImpression` with domain/observation/valence.
- **detect_contradiction:** Conservative -- returns false for most pairs.

## Output Format

```
═══════════════════════════════════════════════════
  Chapter 3: Consolidation — Episodic → Semantic
═══════════════════════════════════════════════════

  Running CLS replay on 8 unconsolidated episodes...

  ConsolidationReport:
    episodes_processed: 8
    nodes_created:      3
    links_created:      9

  Extracted Knowledge:
    [Fact]         "User works with Rust and SQLite" (confidence: 0.75)
    [Relationship] "Rust → async/await patterns"     (confidence: 0.60)

  ★ Insight: Like hippocampal replay during sleep, consolidation
    extracts stable patterns from raw episodes without overwriting
    the originals.
```

## Constraints

- No new dependencies (uses only alaya crate)
- All `.unwrap()` with descriptive context (demo, not production)
- Runs via `cargo run --example demo`
- Deterministic scripted input for reproducible output
