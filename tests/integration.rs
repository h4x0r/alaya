use alaya::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a `NewEpisode` with sensible defaults.
fn episode(content: &str, session: &str, ts: i64) -> NewEpisode {
    NewEpisode {
        content: content.to_string(),
        role: Role::User,
        session_id: session.to_string(),
        timestamp: ts,
        context: EpisodeContext::default(),
        embedding: None,
    }
}

/// Build a `NewEpisode` that is chained to a preceding episode (creates a temporal link).
fn chained_episode(
    content: &str,
    session: &str,
    ts: i64,
    preceding: EpisodeId,
) -> NewEpisode {
    NewEpisode {
        content: content.to_string(),
        role: Role::User,
        session_id: session.to_string(),
        timestamp: ts,
        context: EpisodeContext {
            preceding_episode: Some(preceding),
            ..EpisodeContext::default()
        },
        embedding: None,
    }
}

/// Store `count` episodes in a session, returning all created IDs.
fn store_n_episodes(store: &AlayaStore, session: &str, count: usize, base_ts: i64) -> Vec<EpisodeId> {
    (0..count)
        .map(|i| {
            store
                .store_episode(&episode(
                    &format!("Episode {i} in session {session} about Rust programming"),
                    session,
                    base_ts + (i as i64) * 100,
                ))
                .unwrap()
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Test 1: Multi-session lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_multi_session_lifecycle() {
    let store = AlayaStore::open_in_memory().unwrap();

    // Store episodes across 3 sessions (4 each = 12 total)
    let _s1_ids = store_n_episodes(&store, "session-1", 4, 1_000);
    let _s2_ids = store_n_episodes(&store, "session-2", 4, 2_000);
    let _s3_ids = store_n_episodes(&store, "session-3", 4, 3_000);

    let status = store.status().unwrap();
    assert_eq!(status.episode_count, 12, "should have 12 episodes across 3 sessions");

    // Query -- BM25 should find episodes mentioning "Rust"
    let results = store.query(&Query::simple("Rust programming")).unwrap();
    assert!(!results.is_empty(), "query should return matching episodes");

    // Consolidate with NoOpProvider -- won't create semantic nodes but should
    // process episodes (>= 3 unconsolidated) and report them.
    let cr = store.consolidate(&NoOpProvider).unwrap();
    // NoOp returns empty knowledge, so no nodes created, but episodes_processed
    // should reflect the batch that was read.
    assert_eq!(cr.nodes_created, 0, "NoOp provider creates no semantic nodes");
    // With NoOp, episodes_processed should be > 0 because we have >= 3
    // unconsolidated episodes and the batch is read before the provider returns
    // an empty vec.
    assert!(
        cr.episodes_processed > 0,
        "consolidation should process the unconsolidated batch (got {})",
        cr.episodes_processed
    );

    // Transform -- dedup/prune/decay pass (no duplicates expected)
    let tr = store.transform().unwrap();
    // With only episodes and no semantic nodes or preferences, everything
    // should be zero (no duplicates to merge, no links to prune, etc.)
    assert_eq!(tr.duplicates_merged, 0);

    // After storing 12 episodes, each has an initialized strength record.
    // Forget decays retrieval strength on all strength records.
    let fr = store.forget().unwrap();
    assert!(
        fr.nodes_decayed > 0,
        "forget should decay the 12 episode strength records (got {})",
        fr.nodes_decayed
    );

    // Status should still reflect 12 episodes (forget only archives nodes
    // with very low strength, and a single decay pass won't drop them that far).
    let final_status = store.status().unwrap();
    assert_eq!(final_status.episode_count, 12, "episodes should survive a single forget pass");
}

// ---------------------------------------------------------------------------
// Test 2: Persistence across open/close
// ---------------------------------------------------------------------------

#[test]
fn test_persistence_across_open_close() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("persistence_test.db");

    // First session: open, store episodes, drop
    {
        let store = AlayaStore::open(&db_path).unwrap();
        store.store_episode(&episode("Rust has zero-cost abstractions", "persist-s1", 1000)).unwrap();
        store.store_episode(&episode("Ownership prevents data races", "persist-s1", 2000)).unwrap();
        store.store_episode(&episode("The borrow checker catches bugs at compile time", "persist-s1", 3000)).unwrap();

        let status = store.status().unwrap();
        assert_eq!(status.episode_count, 3);
        // store is dropped here
    }

    // Second session: reopen the same file and verify data survived
    {
        let store = AlayaStore::open(&db_path).unwrap();

        let status = store.status().unwrap();
        assert_eq!(
            status.episode_count, 3,
            "episodes should persist across open/close"
        );

        // Query should still find results
        let results = store.query(&Query::simple("Rust")).unwrap();
        assert!(
            !results.is_empty(),
            "query should return persisted episodes after reopen"
        );

        // Verify content is intact by checking that "zero-cost" appears in results
        let has_zero_cost = results
            .iter()
            .any(|m| m.content.contains("zero-cost"));
        assert!(has_zero_cost, "persisted content should be retrievable");
    }
}

// ---------------------------------------------------------------------------
// Test 3: Full retrieval pipeline with temporal links
// ---------------------------------------------------------------------------

#[test]
fn test_full_retrieval_pipeline_with_temporal_links() {
    let store = AlayaStore::open_in_memory().unwrap();

    // Store 5 episodes chained via preceding_episode to create temporal links.
    let id1 = store
        .store_episode(&episode("Rust memory safety guarantees", "chain-s1", 1000))
        .unwrap();
    let id2 = store
        .store_episode(&chained_episode(
            "The borrow checker enforces ownership rules",
            "chain-s1",
            2000,
            id1,
        ))
        .unwrap();
    let id3 = store
        .store_episode(&chained_episode(
            "Lifetimes prevent dangling references",
            "chain-s1",
            3000,
            id2,
        ))
        .unwrap();
    let id4 = store
        .store_episode(&chained_episode(
            "Smart pointers like Box and Rc manage heap allocation",
            "chain-s1",
            4000,
            id3,
        ))
        .unwrap();
    let _id5 = store
        .store_episode(&chained_episode(
            "Unsafe blocks opt out of the borrow checker",
            "chain-s1",
            5000,
            id4,
        ))
        .unwrap();

    // Verify links were created (4 temporal links for the chain of 5)
    let status = store.status().unwrap();
    assert_eq!(status.episode_count, 5);
    assert_eq!(
        status.link_count, 4,
        "should have 4 temporal links for a chain of 5 episodes"
    );

    // Query should find episodes matching "borrow checker"
    let results = store.query(&Query::simple("borrow checker")).unwrap();
    assert!(
        !results.is_empty(),
        "query should return episodes about 'borrow checker'"
    );

    // Spreading activation from the first episode should find temporal neighbors
    let neighbors = store.neighbors(NodeRef::Episode(id1), 2).unwrap();
    assert!(
        !neighbors.is_empty(),
        "spreading activation from episode 1 should find temporal neighbors"
    );

    // The direct neighbor of id1 should be id2 (the immediately chained episode)
    let neighbor_refs: Vec<NodeRef> = neighbors.iter().map(|(nr, _)| *nr).collect();
    assert!(
        neighbor_refs.contains(&NodeRef::Episode(id2)),
        "episode 2 should be a neighbor of episode 1 via temporal link"
    );
}

// ---------------------------------------------------------------------------
// Test 4: Multi-session purge isolation
// ---------------------------------------------------------------------------

#[test]
fn test_multi_session_purge_isolation() {
    let store = AlayaStore::open_in_memory().unwrap();

    // Store episodes in three sessions
    let _alpha_ids = store_n_episodes(&store, "alpha", 3, 1_000);
    let _beta_ids = store_n_episodes(&store, "beta", 3, 2_000);
    let _gamma_ids = store_n_episodes(&store, "gamma", 3, 3_000);

    assert_eq!(store.status().unwrap().episode_count, 9);

    // Purge session "beta"
    let purge_report = store.purge(PurgeFilter::Session("beta".to_string())).unwrap();
    assert_eq!(
        purge_report.episodes_deleted, 3,
        "purging 'beta' should delete its 3 episodes"
    );

    let status = store.status().unwrap();
    assert_eq!(
        status.episode_count, 6,
        "6 episodes should remain after purging 'beta'"
    );

    // Verify "alpha" episodes survive by querying
    let alpha_results = store.query(&Query::simple("session alpha")).unwrap();
    assert!(
        !alpha_results.is_empty(),
        "'alpha' episodes should survive the purge of 'beta'"
    );

    // Purge with OlderThan: alpha timestamps are 1000-1200, gamma are 3000-3200.
    // Use a cutoff of 2500 to remove alpha but keep gamma.
    let purge_report2 = store.purge(PurgeFilter::OlderThan(2500)).unwrap();
    assert_eq!(
        purge_report2.episodes_deleted, 3,
        "OlderThan(2500) should remove the 3 'alpha' episodes"
    );

    let final_status = store.status().unwrap();
    assert_eq!(
        final_status.episode_count, 3,
        "only 'gamma' episodes should remain"
    );

    // Verify the remaining episodes are from gamma
    let gamma_results = store.query(&Query::simple("session gamma")).unwrap();
    assert!(
        !gamma_results.is_empty(),
        "'gamma' episodes should still be queryable"
    );
}

// ---------------------------------------------------------------------------
// Test 5: Lifecycle idempotence
// ---------------------------------------------------------------------------

#[test]
fn test_lifecycle_idempotence() {
    // Part A: Run lifecycle operations twice on an empty DB -- should produce
    // consistent zero reports and no errors.
    let store = AlayaStore::open_in_memory().unwrap();

    for pass in 0..2 {
        let cr = store.consolidate(&NoOpProvider).unwrap();
        assert_eq!(cr.episodes_processed, 0, "empty consolidate pass {pass}");
        assert_eq!(cr.nodes_created, 0, "empty consolidate pass {pass}");

        let tr = store.transform().unwrap();
        assert_eq!(tr.duplicates_merged, 0, "empty transform pass {pass}");
        assert_eq!(tr.links_pruned, 0, "empty transform pass {pass}");

        let fr = store.forget().unwrap();
        assert_eq!(fr.nodes_decayed, 0, "empty forget pass {pass}");
        assert_eq!(fr.nodes_archived, 0, "empty forget pass {pass}");
    }

    // Part B: Store episodes and run the lifecycle twice. The second pass
    // should not panic and should produce a consistent state.
    let _ids = store_n_episodes(&store, "idempotence", 6, 1_000);

    let status_before = store.status().unwrap();
    assert_eq!(status_before.episode_count, 6);

    // First lifecycle pass
    let cr1 = store.consolidate(&NoOpProvider).unwrap();
    let _tr1 = store.transform().unwrap();
    let fr1 = store.forget().unwrap();

    // Consolidation should process the batch (>= 3 episodes)
    assert!(cr1.episodes_processed > 0, "first consolidate should process episodes");
    // Forget should decay the 6 strength records
    assert!(fr1.nodes_decayed > 0, "first forget should decay nodes");

    // Second lifecycle pass -- should not panic
    let cr2 = store.consolidate(&NoOpProvider).unwrap();
    let _tr2 = store.transform().unwrap();
    let fr2 = store.forget().unwrap();

    // NoOp never creates semantic links, so episodes remain "unconsolidated"
    // and the second consolidation pass should still process them.
    assert!(cr2.episodes_processed > 0, "second consolidate should re-process (NoOp leaves no links)");

    // Forget should still decay whatever strength records exist
    assert!(fr2.nodes_decayed > 0, "second forget should still decay nodes");

    // Status should be consistent -- episodes should still exist (strength
    // hasn't dropped below archive thresholds after only 2 decay passes).
    let status_after = store.status().unwrap();
    assert_eq!(
        status_after.episode_count, 6,
        "episodes should survive two lifecycle passes"
    );

    // Sanity: transform twice in a row is also fine
    let tr_a = store.transform().unwrap();
    let tr_b = store.transform().unwrap();
    assert_eq!(tr_a.duplicates_merged, tr_b.duplicates_merged);
}
