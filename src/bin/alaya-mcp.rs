//! Alaya MCP Server — expose memory operations over Model Context Protocol.
//!
//! Usage:
//!   cargo build --release --features mcp
//!   ./target/release/alaya-mcp
//!
//! Environment:
//!   ALAYA_DB — path to SQLite database (default: ~/.alaya/memory.db)

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use alaya::{
    AlayaStore, CategoryId, EpisodeContext, EpisodeId, KnowledgeFilter, NewEpisode, NewSemanticNode,
    NodeId, NodeRef, PreferenceId, PurgeFilter, Query, Role, SemanticType,
};
use rmcp::{model::ServerInfo, schemars, tool, ServerHandler, ServiceExt};
use tokio::io::{stdin, stdout};

// ---------------------------------------------------------------------------
// Parameter types (schemars::JsonSchema for MCP tool schemas)
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RememberParams {
    /// The message content to store
    #[schemars(description = "The message content to remember")]
    pub content: String,

    /// Role: "user", "assistant", or "system"
    #[schemars(description = "Who said it: user, assistant, or system")]
    pub role: String,

    /// Session identifier to group related messages
    #[schemars(description = "Session ID to group related messages")]
    pub session_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RecallParams {
    /// What to search for in memory
    #[schemars(description = "What to search for in memory")]
    pub query: String,

    /// Maximum number of results (default: 5)
    #[schemars(description = "Maximum results to return (default: 5)")]
    pub max_results: Option<usize>,

    /// Category ID to boost in results
    #[schemars(description = "Category ID to boost in ranking (memories in this category score higher)")]
    pub boost_category: Option<i64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PreferencesParams {
    /// Optional domain filter (e.g. "style", "tone", "format")
    #[schemars(description = "Optional domain filter (e.g. style, tone, format)")]
    pub domain: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct KnowledgeParams {
    /// Filter by type: "fact", "relationship", "event", "concept"
    #[schemars(description = "Filter by type: fact, relationship, event, concept")]
    pub node_type: Option<String>,

    /// Minimum confidence threshold (0.0 to 1.0)
    #[schemars(description = "Minimum confidence threshold (0.0 to 1.0)")]
    pub min_confidence: Option<f32>,

    /// Maximum number of results
    #[schemars(description = "Maximum results to return (default: 20)")]
    pub limit: Option<usize>,

    /// Filter by category label
    #[schemars(description = "Filter by category label (exact match)")]
    pub category: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PurgeParams {
    /// Purge scope: "session", "older_than", or "all"
    #[schemars(description = "Purge scope: session, older_than, or all")]
    pub scope: String,

    /// Session ID (required when scope is "session")
    #[schemars(description = "Session ID (required when scope is session)")]
    pub session_id: Option<String>,

    /// Unix timestamp (required when scope is "older_than")
    #[schemars(description = "Unix timestamp (required when scope is older_than)")]
    pub before_timestamp: Option<i64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CategoriesParams {
    /// Minimum stability threshold (0.0 to 1.0)
    #[schemars(description = "Minimum stability threshold (0.0 to 1.0). Categories below this are filtered out.")]
    pub min_stability: Option<f32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NeighborsParams {
    /// Node type: "episode", "semantic", "preference", "category"
    #[schemars(description = "Node type: episode, semantic, preference, or category")]
    pub node_type: String,

    /// Node ID
    #[schemars(description = "The numeric ID of the node")]
    pub node_id: i64,

    /// Traversal depth (default: 1)
    #[schemars(description = "How many hops to traverse (default: 1)")]
    pub depth: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NodeCategoryParams {
    /// Semantic node ID
    #[schemars(description = "The numeric ID of the semantic node")]
    pub node_id: i64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LearnFactEntry {
    /// The knowledge content
    #[schemars(description = "The knowledge content")]
    pub content: String,

    /// Type: fact, relationship, event, or concept
    #[schemars(description = "Type: fact, relationship, event, or concept")]
    pub node_type: String,

    /// Confidence level 0.0-1.0 (default: 0.8)
    #[schemars(description = "Confidence level 0.0-1.0 (default: 0.8)")]
    pub confidence: Option<f32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LearnParams {
    /// Facts to learn
    #[schemars(description = "Facts to learn: [{content, node_type, confidence?}]")]
    pub facts: Vec<LearnFactEntry>,

    /// Session ID to link facts to (episodes in this session become source episodes)
    #[schemars(description = "Session ID to link facts to (episodes in this session become source episodes)")]
    pub session_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ImportClaudeMemParams {
    /// Path to claude-mem.db (default: ~/.claude-mem/claude-mem.db)
    #[schemars(description = "Path to claude-mem.db (default: ~/.claude-mem/claude-mem.db)")]
    pub path: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ImportClaudeCodeParams {
    /// Path to Claude Code JSONL conversation file
    #[schemars(description = "Path to Claude Code JSONL conversation file (e.g., ~/.claude/projects/-Users-me-myproject/{uuid}.jsonl)")]
    pub path: String,
}

// ---------------------------------------------------------------------------
// Claude Code JSONL parsing helpers
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct ClaudeCodeEntry {
    #[serde(rename = "type")]
    entry_type: Option<String>,
    message: Option<ClaudeCodeMessage>,
    timestamp: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
}

#[derive(serde::Deserialize)]
struct ClaudeCodeMessage {
    #[allow(dead_code)]
    role: Option<String>,
    content: Option<serde_json::Value>,
}

fn extract_content(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            arr.iter()
                .filter_map(|item| {
                    item.get("text").and_then(|t| t.as_str()).map(String::from)
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        _ => String::new(),
    }
}

// ---------------------------------------------------------------------------
// MCP Server
// ---------------------------------------------------------------------------

pub struct AlayaMcp {
    store: Mutex<AlayaStore>,
    /// Total episodes stored this session.
    episode_count: AtomicU32,
    /// Episodes stored since last `learn` call.
    unconsolidated_count: AtomicU32,
}

impl Clone for AlayaMcp {
    fn clone(&self) -> Self {
        // MCP servers are single-instance; clone should not be called in practice.
        // This satisfies the derive requirement from rmcp.
        panic!("AlayaMcp should not be cloned \u{2014} single-instance server")
    }
}

impl AlayaMcp {
    pub fn new(store: AlayaStore) -> Self {
        Self {
            store: Mutex::new(store),
            episode_count: AtomicU32::new(0),
            unconsolidated_count: AtomicU32::new(0),
        }
    }

    fn with_store<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&AlayaStore) -> alaya::Result<T>,
    {
        let store = self.store.lock().map_err(|e| format!("lock error: {e}"))?;
        f(&store).map_err(|e| format!("{e}"))
    }
}

#[tool(tool_box)]
impl AlayaMcp {
    /// Store a conversation message in memory.
    #[tool(
        description = "Store a conversation message in Alaya's episodic memory. Call this for each message in the conversation that should be remembered."
    )]
    fn remember(&self, #[tool(aggr)] params: RememberParams) -> String {
        let role = match params.role.to_lowercase().as_str() {
            "user" => Role::User,
            "assistant" => Role::Assistant,
            "system" => Role::System,
            _ => {
                return format!(
                    "Error: invalid role '{}'. Use: user, assistant, system",
                    params.role
                )
            }
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let episode = NewEpisode {
            content: params.content.clone(),
            role,
            session_id: params.session_id.clone(),
            timestamp: now,
            context: EpisodeContext::default(),
            embedding: None,
        };

        match self.with_store(|s| s.store_episode(&episode)) {
            Ok(id) => {
                let ep_total = self.episode_count.fetch_add(1, Ordering::Relaxed) + 1;
                let uncons = self.unconsolidated_count.fetch_add(1, Ordering::Relaxed) + 1;

                let mut response =
                    format!("Stored episode {} in session '{}'.", id.0, params.session_id);

                // Consolidation prompt at 10 unconsolidated episodes
                if uncons >= 10 {
                    if let Ok(episodes) = self.with_store(|s| s.unconsolidated_episodes(20)) {
                        response.push_str(&format!(
                            "\n\n--- Consolidation suggested ---\n\
                             You have {} unconsolidated episodes. \
                             Please extract key facts and call the 'learn' tool.\n\
                             Recent unconsolidated episodes:",
                            episodes.len()
                        ));
                        for ep in &episodes {
                            response.push_str(&format!(
                                "\n[{}] {}: {}",
                                ep.id.0,
                                ep.role.as_str(),
                                ep.content
                            ));
                        }
                    }
                }

                // Auto-maintenance every 25 episodes
                if ep_total % 25 == 0 {
                    let tr = self.with_store(|s| s.transform());
                    let fr = self.with_store(|s| s.forget());
                    match (tr, fr) {
                        (Ok(tr), Ok(fr)) => {
                            response.push_str(&format!(
                                "\n\n--- Auto-maintenance ---\n\
                                 Transform: {} merged, {} links pruned, {} categories discovered\n\
                                 Forget: {} decayed, {} archived",
                                tr.duplicates_merged,
                                tr.links_pruned,
                                tr.categories_discovered,
                                fr.nodes_decayed,
                                fr.nodes_archived,
                            ));
                        }
                        (Err(e), _) | (_, Err(e)) => {
                            response
                                .push_str(&format!("\n\n--- Auto-maintenance error: {} ---", e));
                        }
                    }
                }

                response
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Search memory for relevant information.
    #[tool(
        description = "Search Alaya's memory using hybrid retrieval (BM25 + vector + graph + RRF fusion). Returns the most relevant memories matching the query."
    )]
    fn recall(&self, #[tool(aggr)] params: RecallParams) -> String {
        let query = Query {
            text: params.query,
            embedding: None,
            context: alaya::QueryContext::default(),
            max_results: params.max_results.unwrap_or(5),
            boost_categories: params.boost_category.map(|c| vec![c.to_string()]),
        };

        match self.with_store(|s| s.query(&query)) {
            Ok(results) if results.is_empty() => "No memories found.".to_string(),
            Ok(results) => {
                let mut out = format!("Found {} memories:\n\n", results.len());
                for (i, mem) in results.iter().enumerate() {
                    let role = mem.role.map(|r| r.as_str()).unwrap_or("unknown");
                    out.push_str(&format!(
                        "{}. [{}] (score: {:.3}) {}\n",
                        i + 1,
                        role,
                        mem.score,
                        mem.content
                    ));
                }
                out
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Get memory statistics.
    #[tool(
        description = "Get Alaya memory statistics: episode counts, knowledge breakdown by type, categories, preferences, graph links with strongest connection, and embedding coverage."
    )]
    fn status(&self) -> String {
        let st = match self.with_store(|s| s.status()) {
            Ok(st) => st,
            Err(e) => return format!("Error: {e}"),
        };

        let session_eps = self.episode_count.load(Ordering::Relaxed);
        let unconsolidated = self.unconsolidated_count.load(Ordering::Relaxed);

        // Episodes line
        let mut out = format!("Memory Status:\n  Episodes: {}", st.episode_count);
        if session_eps > 0 || unconsolidated > 0 {
            out.push_str(&format!(
                " ({session_eps} this session, {unconsolidated} unconsolidated)"
            ));
        }

        // Knowledge breakdown
        let knowledge_line = match self.with_store(|s| s.knowledge_breakdown()) {
            Ok(breakdown) if !breakdown.is_empty() => {
                let mut parts = Vec::new();
                for (st, label) in [
                    (SemanticType::Fact, "facts"),
                    (SemanticType::Relationship, "relationships"),
                    (SemanticType::Event, "events"),
                    (SemanticType::Concept, "concepts"),
                ] {
                    if let Some(&count) = breakdown.get(&st) {
                        parts.push(format!("{count} {label}"));
                    }
                }
                parts.join(", ")
            }
            Ok(_) => "none".to_string(),
            Err(_) => "error".to_string(),
        };
        out.push_str(&format!("\n  Knowledge: {knowledge_line}"));

        // Categories
        let cat_line = match self.with_store(|s| s.categories(None)) {
            Ok(cats) if !cats.is_empty() => {
                let labels: Vec<&str> = cats.iter().map(|c| c.label.as_str()).collect();
                format!("{} ({})", cats.len(), labels.join(", "))
            }
            Ok(_) => "0".to_string(),
            Err(_) => "error".to_string(),
        };
        out.push_str(&format!("\n  Categories: {cat_line}"));

        // Preferences
        out.push_str(&format!(
            "\n  Preferences: {} crystallized, {} impressions accumulating",
            st.preference_count, st.impression_count
        ));

        // Graph + strongest link
        let strongest_desc = match self.with_store(|s| {
            let link = s.strongest_link()?;
            match link {
                Some((src, tgt, w)) => {
                    let src_label = s.node_content(src)?
                        .unwrap_or_else(|| format!("{}#{}", src.type_str(), src.id()));
                    let tgt_label = s.node_content(tgt)?
                        .unwrap_or_else(|| format!("{}#{}", tgt.type_str(), tgt.id()));
                    Ok(Some(format!(
                        " (strongest: \"{src_label}\" <-> \"{tgt_label}\" weight {w:.2})"
                    )))
                }
                None => Ok(None),
            }
        }) {
            Ok(Some(desc)) => desc,
            _ => String::new(),
        };
        out.push_str(&format!(
            "\n  Graph: {} links{strongest_desc}",
            st.link_count
        ));

        // Embedding coverage
        let total_nodes = st.episode_count + st.semantic_node_count;
        let coverage = if total_nodes > 0 {
            format!(
                "{}/{} nodes ({}%)",
                st.embedding_count,
                total_nodes,
                st.embedding_count * 100 / total_nodes
            )
        } else {
            "0/0 nodes".to_string()
        };
        out.push_str(&format!("\n  Embedding coverage: {coverage}"));

        out
    }

    /// Get user preferences.
    #[tool(
        description = "Get crystallized user preferences learned from past interactions. Optionally filter by domain (e.g. 'style', 'tone', 'format')."
    )]
    fn preferences(&self, #[tool(aggr)] params: PreferencesParams) -> String {
        match self.with_store(|s| s.preferences(params.domain.as_deref())) {
            Ok(prefs) if prefs.is_empty() => "No preferences found.".to_string(),
            Ok(prefs) => {
                let mut out = format!("Found {} preferences:\n\n", prefs.len());
                for p in &prefs {
                    out.push_str(&format!(
                        "- [{}] {} (confidence: {:.2}, evidence: {})\n",
                        p.domain, p.preference, p.confidence, p.evidence_count
                    ));
                }
                out
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Get semantic knowledge.
    #[tool(
        description = "Get distilled semantic knowledge (facts, relationships, events, concepts) extracted from past conversations."
    )]
    fn knowledge(&self, #[tool(aggr)] params: KnowledgeParams) -> String {
        let filter = KnowledgeFilter {
            node_type: params.node_type.as_deref().and_then(SemanticType::from_str),
            min_confidence: params.min_confidence,
            limit: params.limit.or(Some(20)),
            category: params.category,
        };

        match self.with_store(|s| s.knowledge(Some(filter))) {
            Ok(nodes) if nodes.is_empty() => "No knowledge found.".to_string(),
            Ok(nodes) => {
                let mut out = format!("Found {} knowledge nodes:\n\n", nodes.len());
                for n in &nodes {
                    out.push_str(&format!(
                        "- [{}] {} (confidence: {:.2})\n",
                        n.node_type.as_str(),
                        n.content,
                        n.confidence
                    ));
                }
                out
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Run memory maintenance (dedup, prune weak links, decay preferences).
    #[tool(
        description = "Run memory maintenance: deduplicates nodes, prunes weak links, decays stale preferences. Call periodically to keep memory healthy."
    )]
    fn maintain(&self) -> String {
        let transform = self.with_store(|s| s.transform());
        let forget = self.with_store(|s| s.forget());

        match (transform, forget) {
            (Ok(tr), Ok(fr)) => format!(
                "Maintenance complete:\n  Duplicates merged: {}\n  Links pruned: {}\n  Preferences decayed: {}\n  Nodes decayed: {}\n  Nodes archived: {}",
                tr.duplicates_merged,
                tr.links_pruned,
                tr.preferences_decayed,
                fr.nodes_decayed,
                fr.nodes_archived,
            ),
            (Err(e), _) | (_, Err(e)) => format!("Error: {e}"),
        }
    }

    /// List emergent categories.
    #[tool(
        description = "List emergent categories discovered from semantic knowledge clusters. Categories form automatically and evolve through use."
    )]
    fn categories(&self, #[tool(aggr)] params: CategoriesParams) -> String {
        match self.with_store(|s| s.categories(params.min_stability)) {
            Ok(cats) if cats.is_empty() => "No categories found.".to_string(),
            Ok(cats) => {
                let mut out = format!("Found {} categories:\n\n", cats.len());
                for c in &cats {
                    let parent = c
                        .parent_id
                        .map(|p| format!(" (parent: {})", p.0))
                        .unwrap_or_default();
                    out.push_str(&format!(
                        "- [{}] {} — {} members, stability: {:.2}{}\n",
                        c.id.0, c.label, c.member_count, c.stability, parent
                    ));
                }
                out
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Get graph neighbors of a node.
    #[tool(
        description = "Get graph neighbors of a memory node via spreading activation. Shows connected memories with link weights."
    )]
    fn neighbors(&self, #[tool(aggr)] params: NeighborsParams) -> String {
        let node_ref = match params.node_type.to_lowercase().as_str() {
            "episode" => NodeRef::Episode(EpisodeId(params.node_id)),
            "semantic" => NodeRef::Semantic(NodeId(params.node_id)),
            "preference" => NodeRef::Preference(PreferenceId(params.node_id)),
            "category" => NodeRef::Category(CategoryId(params.node_id)),
            _ => {
                return format!(
                    "Error: invalid node_type '{}'. Use: episode, semantic, preference, category",
                    params.node_type
                )
            }
        };
        let depth = params.depth.unwrap_or(1);

        match self.with_store(|s| s.neighbors(node_ref, depth)) {
            Ok(neighbors) if neighbors.is_empty() => "No neighbors found.".to_string(),
            Ok(neighbors) => {
                let mut out = format!("Found {} neighbors:\n\n", neighbors.len());
                for (nr, weight) in &neighbors {
                    let (ntype, nid) = match nr {
                        NodeRef::Episode(id) => ("episode", id.0),
                        NodeRef::Semantic(id) => ("semantic", id.0),
                        NodeRef::Preference(id) => ("preference", id.0),
                        NodeRef::Category(id) => ("category", id.0),
                        _ => ("unknown", 0),
                    };
                    out.push_str(&format!("- {} #{} (weight: {:.3})\n", ntype, nid, weight));
                }
                out
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Get the category of a semantic node.
    #[tool(
        description = "Get which category a semantic knowledge node belongs to. Returns the category or 'uncategorized'."
    )]
    fn node_category(&self, #[tool(aggr)] params: NodeCategoryParams) -> String {
        match self.with_store(|s| s.node_category(NodeId(params.node_id))) {
            Ok(Some(cat)) => {
                let parent = cat
                    .parent_id
                    .map(|p| format!(" (parent: {})", p.0))
                    .unwrap_or_default();
                format!(
                    "Node {} belongs to category [{}] '{}' — {} members, stability: {:.2}{}",
                    params.node_id, cat.id.0, cat.label, cat.member_count, cat.stability, parent
                )
            }
            Ok(None) => format!("Node {} is uncategorized.", params.node_id),
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Teach Alaya extracted knowledge directly.
    #[tool(
        description = "Teach Alaya extracted knowledge directly. The agent extracts facts from conversation and calls this tool. Each fact becomes a semantic node with full lifecycle wiring (strength, categories, graph links)."
    )]
    fn learn(&self, #[tool(aggr)] params: LearnParams) -> String {
        // Resolve session_id → source episode IDs
        let source_episodes = match &params.session_id {
            Some(sid) => match self.with_store(|s| s.episodes_by_session(sid)) {
                Ok(eps) => eps.iter().map(|e| e.id).collect::<Vec<_>>(),
                Err(e) => return format!("Error resolving session '{}': {}", sid, e),
            },
            None => vec![],
        };

        // Convert LearnFactEntry → NewSemanticNode
        let nodes: Vec<NewSemanticNode> = params
            .facts
            .iter()
            .map(|fact| {
                let node_type = SemanticType::from_str(&fact.node_type).unwrap_or(SemanticType::Fact);
                let confidence = fact.confidence.unwrap_or(0.8).clamp(0.0, 1.0);
                NewSemanticNode {
                    content: fact.content.clone(),
                    node_type,
                    confidence,
                    source_episodes: source_episodes.clone(),
                    embedding: None,
                }
            })
            .collect();

        let count = nodes.len();
        match self.with_store(|s| s.learn(nodes)) {
            Ok(report) => {
                self.unconsolidated_count.store(0, Ordering::Relaxed);
                format!(
                    "Learned {} facts: {} nodes created, {} links created, {} categories assigned",
                    count, report.nodes_created, report.links_created, report.categories_assigned
                )
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Import memories from claude-mem SQLite database.
    #[tool(
        description = "Import memories from claude-mem (claude-mem.db SQLite database). Reads observations and converts facts/concepts into Alaya semantic nodes."
    )]
    fn import_claude_mem(&self, #[tool(aggr)] params: ImportClaudeMemParams) -> String {
        // 1. Resolve path (default to ~/.claude-mem/claude-mem.db)
        let path = params.path.unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            format!("{home}/.claude-mem/claude-mem.db")
        });

        // 2. Open SQLite read-only
        let source_conn = match rusqlite::Connection::open_with_flags(
            &path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ) {
            Ok(c) => c,
            Err(e) => return format!("Cannot open claude-mem.db at '{path}': {e}"),
        };

        // 3. Query observations
        let mut stmt = match source_conn.prepare(
            "SELECT title, facts, narrative, concepts, created_at FROM observations",
        ) {
            Ok(s) => s,
            Err(e) => return format!("Error reading observations: {e}"),
        };

        let mut nodes = Vec::new();
        let mut obs_count = 0u32;

        let rows = match stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0).unwrap_or_default(), // title
                row.get::<_, String>(1).unwrap_or_default(), // facts JSON
                row.get::<_, String>(2).unwrap_or_default(), // narrative
                row.get::<_, String>(3).unwrap_or_default(), // concepts JSON
                row.get::<_, String>(4).unwrap_or_default(), // created_at
            ))
        }) {
            Ok(r) => r,
            Err(e) => return format!("Error querying observations: {e}"),
        };

        for row_result in rows {
            let (title, facts_json, _narrative, concepts_json, _created_at) = match row_result {
                Ok(r) => r,
                Err(_) => continue,
            };
            obs_count += 1;
            let _ = title; // used for counting, title context is implicit in facts

            // Parse facts JSON array
            if let Ok(facts) = serde_json::from_str::<Vec<String>>(&facts_json) {
                for fact in facts {
                    if fact.trim().is_empty() {
                        continue;
                    }
                    nodes.push(NewSemanticNode {
                        content: fact,
                        node_type: SemanticType::Fact,
                        confidence: 0.8,
                        source_episodes: vec![],
                        embedding: None,
                    });
                }
            }

            // Parse concepts JSON array
            if let Ok(concepts) = serde_json::from_str::<Vec<String>>(&concepts_json) {
                for concept in concepts {
                    if concept.trim().is_empty() {
                        continue;
                    }
                    nodes.push(NewSemanticNode {
                        content: concept,
                        node_type: SemanticType::Concept,
                        confidence: 0.7,
                        source_episodes: vec![],
                        embedding: None,
                    });
                }
            }
        }

        if obs_count == 0 {
            return format!("No observations found in '{path}'.");
        }

        let node_count = nodes.len();
        match self.with_store(|s| s.learn(nodes)) {
            Ok(report) => format!(
                "Imported {obs_count} observations \u{2192} {node_count} semantic nodes. {} categories assigned.",
                report.categories_assigned
            ),
            Err(e) => format!("Error importing: {e}"),
        }
    }

    /// Import conversation history from Claude Code JSONL files.
    #[tool(
        description = "Import conversation history from Claude Code JSONL files. Reads messages and stores them as episodes."
    )]
    fn import_claude_code(&self, #[tool(aggr)] params: ImportClaudeCodeParams) -> String {
        let path = &params.path;

        // Read the JSONL file
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => return format!("Cannot read file '{path}': {e}"),
        };

        let mut imported = 0u32;
        let mut sessions = std::collections::HashSet::new();
        let mut errors = 0u32;

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let entry: ClaudeCodeEntry = match serde_json::from_str(line) {
                Ok(e) => e,
                Err(_) => {
                    errors += 1;
                    continue;
                }
            };

            // Only import human and assistant messages
            let entry_type = entry.entry_type.as_deref().unwrap_or("");
            if entry_type != "human" && entry_type != "assistant" {
                continue;
            }

            let message = match entry.message {
                Some(m) => m,
                None => continue,
            };

            let role = match entry_type {
                "human" => Role::User,
                "assistant" => Role::Assistant,
                _ => continue,
            };

            let content_value = match message.content {
                Some(c) => c,
                None => continue,
            };
            let content_text = extract_content(&content_value);
            if content_text.trim().is_empty() {
                continue;
            }

            // Truncate very long content (Claude Code messages can be huge)
            let content_text = if content_text.len() > 2000 {
                format!("{}...", &content_text[..2000])
            } else {
                content_text
            };

            let session_id = entry
                .session_id
                .unwrap_or_else(|| "imported".to_string());
            sessions.insert(session_id.clone());

            // Parse timestamp (string of unix seconds)
            let timestamp = entry
                .timestamp
                .as_deref()
                .and_then(|ts| ts.parse::<i64>().ok())
                .unwrap_or(0);

            let episode = NewEpisode {
                content: content_text,
                role,
                session_id,
                timestamp,
                context: EpisodeContext::default(),
                embedding: None,
            };

            match self.with_store(|s| s.store_episode(&episode)) {
                Ok(_) => {
                    imported += 1;
                    self.episode_count.fetch_add(1, Ordering::Relaxed);
                    self.unconsolidated_count.fetch_add(1, Ordering::Relaxed);
                }
                Err(_) => errors += 1,
            }
        }

        if imported == 0 {
            return format!(
                "No importable messages found in '{path}'.{}",
                if errors > 0 {
                    format!(" ({errors} lines failed to parse)")
                } else {
                    String::new()
                }
            );
        }

        format!(
            "Imported {imported} messages from {} sessions as episodes.{} Call 'learn' to consolidate.",
            sessions.len(),
            if errors > 0 {
                format!(" ({errors} parse errors skipped)")
            } else {
                String::new()
            }
        )
    }

    /// Purge memories by session, timestamp, or everything.
    #[tool(
        description = "Purge memories. Scope: 'session' (requires session_id), 'older_than' (requires before_timestamp), or 'all' (deletes everything)."
    )]
    fn purge(&self, #[tool(aggr)] params: PurgeParams) -> String {
        let filter = match params.scope.as_str() {
            "session" => match params.session_id {
                Some(sid) => PurgeFilter::Session(sid),
                None => return "Error: session_id required for scope 'session'".to_string(),
            },
            "older_than" => match params.before_timestamp {
                Some(ts) => PurgeFilter::OlderThan(ts),
                None => {
                    return "Error: before_timestamp required for scope 'older_than'".to_string()
                }
            },
            "all" => PurgeFilter::All,
            _ => {
                return format!(
                    "Error: invalid scope '{}'. Use: session, older_than, all",
                    params.scope
                )
            }
        };

        match self.with_store(|s| s.purge(filter)) {
            Ok(report) => format!(
                "Purge complete: {} episodes deleted",
                report.episodes_deleted
            ),
            Err(e) => format!("Error: {e}"),
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for AlayaMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Alaya is a memory engine for AI agents. Use 'remember' to store messages, \
                 'recall' to search memory, 'learn' to teach extracted knowledge directly, \
                 'status' to check stats, 'preferences' for user preferences, 'knowledge' for \
                 semantic facts, 'categories' for emergent clusters, 'neighbors' for graph \
                 traversal, 'node_category' to check a node's category, 'maintain' for cleanup, \
                 'import_claude_mem' to import from claude-mem.db, \
                 'import_claude_code' to import from Claude Code JSONL files, \
                 and 'purge' to delete data."
                    .into(),
            ),
            ..Default::default()
        }
    }
}

// ---------------------------------------------------------------------------
// Entrypoint
// ---------------------------------------------------------------------------

fn resolve_db_path() -> PathBuf {
    if let Ok(path) = std::env::var("ALAYA_DB") {
        return PathBuf::from(path);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(home).join(".alaya");
    std::fs::create_dir_all(&dir).ok();
    dir.join("memory.db")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Stderr for logging (stdout is reserved for MCP JSON-RPC)
    let db_path = resolve_db_path();
    eprintln!("alaya-mcp: opening database at {}", db_path.display());

    let store = AlayaStore::open(&db_path)?;
    let server = AlayaMcp::new(store);

    let transport = (stdin(), stdout());
    let service = server.serve(transport).await?;
    service.waiting().await?;

    Ok(())
}
