//! Integration tests for MCP tool functions.
//!
//! These test the AlayaMcp tool methods directly (without MCP transport)
//! using an in-memory store.

#![cfg(feature = "mcp")]

// We can't directly import AlayaMcp from the binary,
// so we test the underlying AlayaStore operations that the MCP tools wrap.
// This validates the data flow that the MCP tools rely on.

use alaya::{
    AlayaStore, EpisodeContext, KnowledgeFilter, NewEpisode, NewSemanticNode, PurgeFilter, Query,
    Role, SemanticType,
};

fn make_episode(content: &str, role: Role, session: &str, ts: i64) -> NewEpisode {
    NewEpisode {
        content: content.to_string(),
        role,
        session_id: session.to_string(),
        timestamp: ts,
        context: EpisodeContext::default(),
        embedding: None,
    }
}

#[test]
fn test_mcp_remember_and_recall_flow() {
    let store = AlayaStore::open_in_memory().unwrap();

    // Simulate MCP "remember" tool
    let id = store
        .store_episode(&make_episode(
            "I love hiking in the mountains",
            Role::User,
            "session-1",
            1700000000,
        ))
        .unwrap();
    assert!(id.0 > 0);

    store
        .store_episode(&make_episode(
            "That sounds fun! Do you have a favorite trail?",
            Role::Assistant,
            "session-1",
            1700000001,
        ))
        .unwrap();

    store
        .store_episode(&make_episode(
            "Yes, I love the Appalachian Trail",
            Role::User,
            "session-1",
            1700000002,
        ))
        .unwrap();

    // Simulate MCP "recall" tool
    let results = store.query(&Query::simple("hiking")).unwrap();
    assert!(!results.is_empty(), "recall should find hiking memories");

    // Also search for "Appalachian"
    let results2 = store.query(&Query::simple("Appalachian")).unwrap();
    assert!(
        !results2.is_empty(),
        "recall should find Appalachian Trail memory"
    );
}

#[test]
fn test_mcp_status_flow() {
    let store = AlayaStore::open_in_memory().unwrap();

    // Empty status
    let status = store.status().unwrap();
    assert_eq!(status.episode_count, 0);
    assert_eq!(status.semantic_node_count, 0);
    assert_eq!(status.preference_count, 0);

    // Store episodes
    for i in 0..3 {
        store
            .store_episode(&make_episode(
                &format!("message {i}"),
                Role::User,
                "s1",
                1000 + i,
            ))
            .unwrap();
    }

    let status = store.status().unwrap();
    assert_eq!(status.episode_count, 3);
}

#[test]
fn test_mcp_preferences_flow() {
    let store = AlayaStore::open_in_memory().unwrap();

    // No preferences initially
    let prefs = store.preferences(None).unwrap();
    assert!(prefs.is_empty());

    // With domain filter
    let prefs = store.preferences(Some("style")).unwrap();
    assert!(prefs.is_empty());
}

#[test]
fn test_mcp_knowledge_flow() {
    let store = AlayaStore::open_in_memory().unwrap();

    // No knowledge initially
    let nodes = store.knowledge(None).unwrap();
    assert!(nodes.is_empty());

    // With type filter
    let nodes = store
        .knowledge(Some(KnowledgeFilter {
            node_type: Some(SemanticType::Fact),
            ..Default::default()
        }))
        .unwrap();
    assert!(nodes.is_empty());
}

#[test]
fn test_mcp_purge_session_flow() {
    let store = AlayaStore::open_in_memory().unwrap();

    // Store in two sessions
    store
        .store_episode(&make_episode("msg in s1", Role::User, "s1", 1000))
        .unwrap();
    store
        .store_episode(&make_episode("msg in s2", Role::User, "s2", 2000))
        .unwrap();

    assert_eq!(store.status().unwrap().episode_count, 2);

    // Purge session s1
    let report = store.purge(PurgeFilter::Session("s1".to_string())).unwrap();
    assert_eq!(report.episodes_deleted, 1);
    assert_eq!(store.status().unwrap().episode_count, 1);
}

#[test]
fn test_mcp_purge_all_flow() {
    let store = AlayaStore::open_in_memory().unwrap();

    store
        .store_episode(&make_episode("msg1", Role::User, "s1", 1000))
        .unwrap();
    store
        .store_episode(&make_episode("msg2", Role::User, "s1", 2000))
        .unwrap();

    store.purge(PurgeFilter::All).unwrap();
    assert_eq!(store.status().unwrap().episode_count, 0);
}

#[test]
fn test_mcp_maintain_flow() {
    let store = AlayaStore::open_in_memory().unwrap();

    // transform + forget on empty store should succeed
    let tr = store.transform().unwrap();
    assert_eq!(tr.duplicates_merged, 0);

    let fr = store.forget().unwrap();
    assert_eq!(fr.nodes_decayed, 0);
}

#[test]
fn test_mcp_recall_max_results() {
    let store = AlayaStore::open_in_memory().unwrap();

    for i in 0..10 {
        store
            .store_episode(&make_episode(
                &format!("Rust programming tip number {i}"),
                Role::User,
                "s1",
                1000 + i,
            ))
            .unwrap();
    }

    // Limit to 3 results
    let query = Query {
        text: "Rust programming".to_string(),
        embedding: None,
        context: alaya::QueryContext::default(),
        max_results: 3,
        boost_categories: None,
    };
    let results = store.query(&query).unwrap();
    assert!(results.len() <= 3, "should respect max_results limit");
}

#[test]
fn test_mcp_role_parsing() {
    let store = AlayaStore::open_in_memory().unwrap();

    // All three roles should work
    for (role, role_str) in [
        (Role::User, "user"),
        (Role::Assistant, "assistant"),
        (Role::System, "system"),
    ] {
        let id = store
            .store_episode(&make_episode("test", role, "s1", 1000))
            .unwrap();
        assert!(id.0 > 0, "role '{}' should be accepted", role_str);
    }
}

#[test]
fn test_mcp_learn_creates_knowledge() {
    let store = AlayaStore::open_in_memory().unwrap();

    // Store 3 facts via learn()
    let nodes = vec![
        NewSemanticNode {
            content: "User prefers Rust".to_string(),
            node_type: SemanticType::Fact,
            confidence: 0.9,
            source_episodes: vec![],
            embedding: None,
        },
        NewSemanticNode {
            content: "User knows async programming".to_string(),
            node_type: SemanticType::Fact,
            confidence: 0.8,
            source_episodes: vec![],
            embedding: None,
        },
        NewSemanticNode {
            content: "Rust and async are related".to_string(),
            node_type: SemanticType::Relationship,
            confidence: 0.7,
            source_episodes: vec![],
            embedding: None,
        },
    ];
    let report = store.learn(nodes).unwrap();
    assert_eq!(report.nodes_created, 3);

    let knowledge = store.knowledge(None).unwrap();
    assert_eq!(knowledge.len(), 3);
}

#[test]
fn test_mcp_learn_with_session_links() {
    let store = AlayaStore::open_in_memory().unwrap();

    // Store episodes first
    let ep1 = store
        .store_episode(&make_episode("msg1", Role::User, "s1", 1000))
        .unwrap();
    let ep2 = store
        .store_episode(&make_episode("msg2", Role::User, "s1", 2000))
        .unwrap();

    // Learn with those episodes as sources
    let nodes = vec![NewSemanticNode {
        content: "User discussed topic X".to_string(),
        node_type: SemanticType::Fact,
        confidence: 0.9,
        source_episodes: vec![ep1, ep2],
        embedding: None,
    }];
    let report = store.learn(nodes).unwrap();
    assert_eq!(report.nodes_created, 1);
    assert_eq!(report.links_created, 2); // 2 Causal links

    // Verify episodes are now consolidated
    let unconsolidated = store.unconsolidated_episodes(100).unwrap();
    assert!(unconsolidated.is_empty());
}

#[test]
fn test_mcp_episodes_by_session() {
    let store = AlayaStore::open_in_memory().unwrap();

    // Store episodes in two sessions
    store
        .store_episode(&make_episode("msg1", Role::User, "s1", 1000))
        .unwrap();
    store
        .store_episode(&make_episode("msg2", Role::Assistant, "s1", 2000))
        .unwrap();
    store
        .store_episode(&make_episode("msg3", Role::User, "s2", 3000))
        .unwrap();

    // Query session s1
    let eps = store.episodes_by_session("s1").unwrap();
    assert_eq!(eps.len(), 2);
    assert_eq!(eps[0].content, "msg1");
    assert_eq!(eps[1].content, "msg2");

    // Query session s2
    let eps = store.episodes_by_session("s2").unwrap();
    assert_eq!(eps.len(), 1);

    // Query non-existent session
    let eps = store.episodes_by_session("s999").unwrap();
    assert!(eps.is_empty());
}
