# Emergent Ontology MVP Implementation Plan

**Date:** 2026-03-02
**Scope:** v0.1.0 — flat categories with dual-signal clustering
**Design doc:** `docs/plans/2026-03-02-emergent-ontology-design.md`
**Approach:** TDD — tests first, implementation second

## What Ships in v0.1.0

### Data Model
- `CategoryId` newtype (i64)
- `Category` struct (id, label, prototype_node, member_count, centroid_embedding, created_at, last_updated, stability)
- `categories` SQL table with schema migration via PRAGMA user_version
- `category_id` column on `semantic_nodes` table
- `NodeRef::Category(CategoryId)` enum variant
- `LinkType::MemberOf` enum variant

### Consolidation Integration
- After creating semantic nodes, attempt category assignment:
  1. Embedding similarity to category centroids (threshold 0.6)
  2. Graph neighbor majority vote fallback
- Never creates new categories (zero LLM cost)
- Updates centroid (running average) and member_count on assignment

### Transform Integration
- After dedup/pruning, discover new categories from uncategorized nodes:
  1. Pairwise cosine similarity clustering (threshold 0.7, min cluster size 3)
  2. Graph support scoring (combined = 0.6*embedding + 0.4*graph)
  3. LLM naming (optional — placeholder labels from prototype content if unavailable)
- Maintenance: merge converging categories (>0.85), dissolve unstable (<0.2 after 3+ cycles), garbage-collect empty

### Public API
- `AlayaStore::categories(min_stability: Option<f32>) -> Result<Vec<Category>>`
- `AlayaStore::node_category(node_id: NodeId) -> Result<Option<Category>>`
- `KnowledgeFilter.category: Option<String>` field
- `Query.boost_categories: Option<Vec<String>>` field

### Graceful Degradation
- No embeddings: graph-only clustering (neighbor counting / Hebbian link overlap)
- No LLM: placeholder labels (first 3 words of prototype node content)
- No categories yet: consolidation skips assignment, transform discovers when >= 3 uncategorized nodes exist

### Tests (TDD)
**Unit:**
- Category CRUD (store, get, list, delete)
- Incremental assignment via embedding similarity
- Incremental assignment via graph neighbors
- No match leaves uncategorized
- Centroid recomputation after new member
- Stability increment/decrement logic

**Integration:**
- Full lifecycle: episodes -> consolidate -> transform -> categories emerge
- Graph-only fallback (no embeddings)
- Stability tracking across multiple transform cycles
- Category merging when clusters converge
- Category dissolution when members diverge
- Forgetting integration (empty category cleanup)
- NoOpProvider path (placeholder labels)

## Not in v0.1.0
- MCP tool extensions (categories, knowledge filter, recall boost) — v0.2.0
- Category hierarchy / parent-child trees — v0.2.0
- Prototype theory / exemplar-based boundaries — v0.2.0
- Category seeds (bija) / dormant re-emergence — v0.2.0
- Cross-domain bridging — v0.2.0
