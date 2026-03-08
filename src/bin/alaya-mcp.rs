//! Alaya MCP Server — expose memory operations over Model Context Protocol.
//!
//! Usage:
//!   cargo build --release --features mcp
//!   ./target/release/alaya-mcp
//!
//! Environment:
//!   ALAYA_DB — path to SQLite database (default: ~/.alaya/memory.db)

use std::path::PathBuf;
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

// ---------------------------------------------------------------------------
// MCP Server
// ---------------------------------------------------------------------------

pub struct AlayaMcp {
    store: Mutex<AlayaStore>,
}

impl Clone for AlayaMcp {
    fn clone(&self) -> Self {
        // MCP servers are single-instance; clone should not be called in practice.
        // This satisfies the derive requirement from rmcp.
        panic!("AlayaMcp should not be cloned — single-instance server")
    }
}

impl AlayaMcp {
    pub fn new(store: AlayaStore) -> Self {
        Self {
            store: Mutex::new(store),
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
            Ok(id) => format!("Stored episode {} in session '{}'", id.0, params.session_id),
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
        description = "Get Alaya memory statistics: counts of episodes, semantic nodes, preferences, impressions, links, and embeddings."
    )]
    fn status(&self) -> String {
        match self.with_store(|s| s.status()) {
            Ok(st) => format!(
                "Memory Status:\n  Episodes: {}\n  Semantic nodes: {}\n  Preferences: {}\n  Impressions: {}\n  Links: {}\n  Embeddings: {}\n  Categories: {}",
                st.episode_count,
                st.semantic_node_count,
                st.preference_count,
                st.impression_count,
                st.link_count,
                st.embedding_count,
                st.category_count,
            ),
            Err(e) => format!("Error: {e}"),
        }
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
                let confidence = fact.confidence.unwrap_or(0.8);
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
            Ok(report) => format!(
                "Learned {} facts: {} nodes created, {} links created, {} categories assigned",
                count, report.nodes_created, report.links_created, report.categories_assigned
            ),
            Err(e) => format!("Error: {e}"),
        }
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
