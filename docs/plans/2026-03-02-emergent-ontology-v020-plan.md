# Emergent Ontology v0.2.0 Plan — Yogacara Grounding

**Date:** 2026-03-02
**Scope:** v0.2.0 — hierarchical categories, MCP integration, Yogacara concepts
**Depends on:** v0.1.0 flat categories MVP

## Features

### Category Hierarchy (vikalpa)
- Parent-child trees via `LinkType::IsA` edges in the Hebbian graph
- `Category.parent_id: Option<CategoryId>` field
- `AlayaStore::category_tree()` — returns hierarchical category structure
- Spreading activation flows through hierarchy (parent activates children)
- Transform discovers hierarchy: frequent co-occurrence of flat categories suggests parent

### Prototype Theory (nama-rupa)
- Categories defined by prototypical exemplars, not rigid boundaries
- Graded membership: `CategoryMembership { node_id, category_id, typicality: f32 }`
- Typicality score based on distance from centroid + corroboration
- Atypical members can belong to multiple categories with different typicality

### Category Seeds (bija)
- Dormant categories that were dissolved but retain seed state
- `CategorySeed { label, last_centroid, dissolved_at, revival_threshold }`
- When new uncategorized nodes cluster near a seed's centroid, the category re-emerges
- Models how concepts can be "forgotten" but revive with new evidence

### Conceptual Transformation (asraya-paravrtti)
- Categories themselves evolve through use
- Track category centroid drift over time
- Category label can be re-generated when centroid shifts significantly
- Category splits: detect bimodal member distribution, spawn child categories

### Cross-Domain Bridging
- Spreading activation through category nodes enables analogical reasoning
- "Cooking techniques" category node connects to "chemistry" through shared members
- Query for "cooking" can surface chemistry-related memories through category bridges

### MCP Tool Extensions
- `categories` tool: list emergent categories with stability filtering
- `knowledge` tool: extended with optional `category` filter parameter
- `recall` tool: extended with optional `boost_categories` parameter

## Dependencies
- v0.1.0 flat categories (data model, CRUD, consolidation/transform integration)
- Potential schema migration from flat to hierarchical
- EmbeddingProvider trait (also planned for v0.2.0)

## Research Questions
- What hierarchy depth is useful in practice? (hypothesis: 2-3 levels max)
- How should typicality interact with Bjork retrieval strength?
- Should category seeds decay like memories (Bjork) or persist indefinitely?
- What threshold triggers category splits vs. just adding members?
