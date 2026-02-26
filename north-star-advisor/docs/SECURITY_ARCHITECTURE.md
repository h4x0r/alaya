# Alaya Security Architecture

> Security Architecture for Alaya v0.1 -- Embeddable Rust Memory Engine

**Document type:** Library Security Architecture
**Version:** 0.1.0
**Last updated:** 2026-02-26
**Status:** Living document, tracks implementation
**OWASP mapping:** Top 10 for Agentic Applications (2025 draft)

---

## 1. Executive Summary

Alaya is a Rust library crate. It is not a web service, not a cloud platform, not a running process. There are no API keys, no user sessions, no network endpoints, no authentication flows. The security architecture is fundamentally different from service-oriented security: the entire attack surface is the library API boundary and the single SQLite file on disk.

This document defines Alaya's threat model, mitigation strategies, and security guidance for consuming agents. It addresses seven categories of risk specific to an embedded memory library:

1. **Memory poisoning** -- adversarial content that corrupts the memory graph
2. **FTS5 injection** -- special characters that subvert search queries
3. **Memory resurrection** -- deleted data re-emerging through consolidation
4. **Cross-user data leakage** -- inadequate scoping between users
5. **PII persistence** -- personal information persisting in memories and embeddings
6. **Embedding poisoning** -- manipulated vectors that distort retrieval
7. **SQLite integrity** -- WAL corruption, transaction deadlocks, unbounded growth

### Security Model: Architectural, Not Operational

Alaya's security model is architectural. The guarantees come from what the library structurally cannot do, not from what it promises not to do:

| Property | Guarantee | Mechanism |
|----------|-----------|-----------|
| No network calls | The core crate contains zero HTTP, DNS, or socket code | Compile-time: no networking dependencies in core |
| No telemetry | No usage data leaves the process | No networking code exists to transmit data |
| No API keys | No credentials stored or required | No remote services to authenticate against |
| No cloud dependency | All state in a single local SQLite file | Single-File Invariant: if it cannot go in the file, it does not persist |
| No background threads | Lifecycle runs only when the consumer calls it | Explicit lifecycle API: `consolidate()`, `forget()`, `transform()`, `perfume()` |

What Alaya does **not** guarantee:

- **Application-level access control.** Alaya is a library. The consuming agent is responsible for authentication, authorization, and user isolation.
- **Encryption at rest.** Standard SQLite does not encrypt. Consumers requiring encryption should use SQLCipher or filesystem-level encryption.
- **Content filtering.** Alaya stores what it is given. The consuming agent must validate content before calling `store_episode()`.
- **PII detection.** Alaya provides deletion tools (`purge()`, `forget()`). The consumer identifies what constitutes PII.

### Axiom Hierarchy Applied to Security

The conflict resolution hierarchy -- **Safety > Privacy > Correctness > Simplicity > Performance > Features** -- governs all security decisions:

- A memory integrity check that slows writes by 5% ships. (Safety > Performance)
- A feature requiring optional network access lives behind a feature flag in a separate crate, never in core. (Privacy > Features)
- FTS5 sanitization rejects queries that could produce incorrect results rather than attempting partial execution. (Correctness > Performance)
- `BEGIN IMMEDIATE` adds transaction overhead but prevents WAL deadlocks. (Safety > Performance)

---

## 2. Threat Model

### 2.1 Attack Surface Map

Alaya's attack surface is narrower than a service because there is no network layer. All threats arrive through two channels: the library API (controlled by the consuming agent) and the SQLite file on disk (controlled by the operating system).

```
+--------------------------------------------------------------------+
|                    ALAYA ATTACK SURFACE                             |
+--------------------------------------------------------------------+
|                                                                    |
|  CHANNEL 1: Library API (consuming agent is the caller)            |
|  +------------------+  +------------------+  +------------------+  |
|  | Memory Poisoning |  | FTS5 MATCH       |  | PII Persistence  |  |
|  | (ASI06)          |  | Injection        |  |                  |  |
|  | Malicious content|  | Special chars in |  | PII stored in    |  |
|  | via store_*()    |  | query text       |  | episodes, nodes, |  |
|  |                  |  |                  |  | embeddings       |  |
|  +------------------+  +------------------+  +------------------+  |
|  +------------------+  +------------------+  +------------------+  |
|  | Cross-User       |  | Provider Output  |  | Embedding        |  |
|  | Leakage          |  | Injection        |  | Poisoning        |  |
|  | Shared store     |  | Malicious LLM    |  | Manipulated      |  |
|  | without scoping  |  | responses via    |  | vectors via      |  |
|  |                  |  | ConsolidationPr. |  | embedding field  |  |
|  +------------------+  +------------------+  +------------------+  |
|                                                                    |
|  CHANNEL 2: SQLite File (filesystem access)                        |
|  +------------------+  +------------------+  +------------------+  |
|  | File Theft       |  | WAL Corruption   |  | Memory           |  |
|  | Unencrypted DB   |  | Crash during     |  | Resurrection     |  |
|  | readable by any  |  | write; unbounded |  | Consolidation    |  |
|  | process with     |  | WAL growth       |  | re-derives data  |  |
|  | file access      |  |                  |  | from remnants    |  |
|  +------------------+  +------------------+  +------------------+  |
|  +------------------+  +------------------+                        |
|  | Transaction      |  | File Permission  |                        |
|  | Deadlock         |  | Misconfiguration |                        |
|  | Concurrent WAL   |  | World-readable   |                        |
|  | writers without  |  | memory.db        |                        |
|  | BEGIN IMMEDIATE  |  |                  |                        |
|  +------------------+  +------------------+                        |
|                                                                    |
|  SENSITIVE ASSETS                                                  |
|  +--------------------------------------------------------------+  |
|  | Conversation history (episodes table)                        |  |
|  |   Raw user/assistant messages; highest PII density           |  |
|  | Extracted knowledge (semantic_nodes table)                   |  |
|  |   Consolidated facts, relationships, events                  |  |
|  | Behavioral patterns (impressions + preferences tables)       |  |
|  |   Emerged preferences reveal personality and habits          |  |
|  | Embeddings (embeddings table)                                |  |
|  |   Encode semantic content; can leak PII via inversion        |  |
|  | Graph relationships (links table)                            |  |
|  |   Temporal, topical, causal, co-retrieval connections        |  |
|  | Node strengths (node_strengths table)                        |  |
|  |   Access frequency reveals behavioral patterns               |  |
|  +--------------------------------------------------------------+  |
+--------------------------------------------------------------------+
```

### 2.2 OWASP Top 10 for Agentic Applications Mapping

The OWASP Top 10 for Agentic Applications targets agent systems and their components. Alaya is a component within such systems. This mapping identifies which threats apply to the library itself versus the consuming agent.

| # | OWASP Agentic Threat | Alaya Exposure | Mitigation | Owner |
|---|----------------------|----------------|------------|-------|
| ASI01 | Agentic Identity Spoofing | None in library. Alaya has no authentication. | Consumer must authenticate users before calling Alaya. | Consumer |
| ASI02 | Prompt/Instruction Manipulation | Indirect. Poisoned memories retrieved by `query()` could manipulate the agent's LLM context. | Content validation hooks (pre-storage). Content-hash integrity (post-storage tamper detection). Consumer validates before injecting into prompts. | Shared |
| ASI03 | Agentic Resource Overuse | Low. SQLite is bounded by disk. No network calls means no runaway API costs. | Consumer controls lifecycle call frequency. WAL checkpoint management prevents unbounded WAL growth. `max_results` parameter on `query()`. | Shared |
| ASI04 | Tool Argument Injection | Moderate. FTS5 MATCH syntax accepts operators (`AND`, `OR`, `NOT`, `NEAR`, `^`, `*`, `"`) that can alter query semantics. | `search_bm25()` strips all non-alphanumeric, non-whitespace characters before MATCH. Empty sanitized queries return empty results. No raw SQL exposure in public API. | Alaya |
| ASI05 | Agentic Privilege Escalation | None in library. Alaya has no privilege model. All methods are equal. | Consumer implements authorization around Alaya calls. Separate SQLite files per trust boundary. | Consumer |
| ASI06 | Agent Memory Poisoning | **Primary threat.** Adversary injects false memories via `store_episode()` that persist and influence future `query()` results. Consolidation can amplify poisoned episodes into semantic nodes. | Content validation hooks. Provider output validation. Tombstone mechanism for cascading deletion. Quarantine API (planned). Content-hash integrity for tamper detection. | Shared |
| ASI07 | Insufficient Agentic Feedback | Not applicable. Alaya is a library, not an agent. | N/A | N/A |
| ASI08 | Agentic Action Validation | Not applicable. Alaya does not take actions. | N/A | N/A |
| ASI09 | Agentic Trust Boundary Violations | Moderate. Without scoping, one user's memories could be retrieved for another user. | Mandatory scoping guidance: separate SQLite files per user, or enforce `session_id` scoping at the consumer level. Alaya does not enforce multi-user isolation within a single file. | Consumer |
| ASI10 | Agentic Knowledge Conflict | Moderate. Contradictory memories from different sources can coexist without resolution. | `detect_contradiction()` in ConsolidationProvider. Corroboration tracking (confidence increases with corroboration). Consumer-supplied contradiction resolution logic. | Shared |

### 2.3 Threat Severity Matrix

| Threat | Likelihood | Impact | Severity | Status |
|--------|-----------|--------|----------|--------|
| Memory poisoning (ASI06) | High | High | **Critical** | Mitigated by design (validation hooks, typed reports); hardening needed (quarantine, content hash) |
| FTS5 MATCH injection (ASI04) | High | Medium | **High** | Mitigated: `search_bm25()` strips special characters |
| Cross-user data leakage (ASI09) | Medium | High | **High** | Mitigated by guidance (separate files); no library-level enforcement |
| PII persistence | High | High | **Critical** | Partially mitigated: `purge()` exists; no PII detection, no encryption at rest |
| Memory resurrection | Medium | Medium | **Medium** | Not mitigated: no tombstone table; planned for v0.1 |
| Embedding poisoning | Low | Medium | **Medium** | Partially mitigated: metadata tracking exists; no validation |
| WAL corruption | Low | High | **Medium** | Mitigated: `synchronous = NORMAL`; checkpoint management needed |
| Transaction deadlock | Medium | Medium | **Medium** | Not mitigated: `BEGIN IMMEDIATE` not yet implemented |
| Provider output injection | Medium | High | **High** | Not mitigated: provider output stored without validation |
| File theft (unencrypted DB) | Medium | High | **High** | Consumer responsibility; guidance provided |

---

## 3. Data Protection

### 3.1 The Single-File Invariant

All Alaya state lives in one SQLite file. This is a security simplification: protect one file, protect all data. There is no configuration scattered across directories, no external cache, no temporary files (SQLite WAL and SHM files are co-located and managed by SQLite itself).

```
memory.db          <- All 7 tables, FTS5 index, embeddings, graph, strengths
memory.db-wal      <- Write-Ahead Log (SQLite-managed, auto-checkpointed)
memory.db-shm      <- Shared memory map (SQLite-managed, ephemeral)
```

**File permission recommendation:** `0600` (owner read/write only). The consuming agent should set this on creation. Alaya does not set file permissions because the appropriate permissions depend on the deployment context.

### 3.2 Data at Rest

SQLite does not encrypt data by default. The contents of `memory.db` are readable by any process with filesystem access. This includes:

- All episode content (raw conversation text)
- All semantic node content (extracted knowledge)
- All impressions and preferences (behavioral patterns)
- All embeddings (which can be inverted to approximate original content)
- All graph relationships (social and behavioral connections)
- All node strength data (access frequency patterns)

**Options for encryption at rest:**

| Approach | Complexity | Performance Impact | Recommendation |
|----------|-----------|-------------------|----------------|
| SQLCipher (compile-time) | Medium | 5-15% overhead | Best option for application-level encryption. Requires building rusqlite with `bundled-sqlcipher` feature. |
| Filesystem encryption (LUKS, FileVault, BitLocker) | Low | Minimal | Good default for single-user devices. Transparent to Alaya. |
| Field-level encryption | High | Moderate | Only if specific fields require different key management. Not supported by Alaya directly. |
| Full-disk encryption | Low | Minimal | Baseline recommendation for all deployments. |

### 3.3 Data Classification

| Table | Sensitivity | PII Risk | Encryption Priority |
|-------|------------|----------|---------------------|
| `episodes` | **High** | **High** -- raw conversation text | 1 (highest) |
| `semantic_nodes` | **High** | **High** -- extracted facts about the user | 1 |
| `impressions` | **High** | **Medium** -- behavioral observations | 2 |
| `preferences` | **High** | **Medium** -- crystallized behavioral patterns | 2 |
| `embeddings` | **Medium** | **Medium** -- invertible to approximate content | 2 |
| `links` | **Low** | **Low** -- structural relationships only | 3 |
| `node_strengths` | **Low** | **Low** -- access frequency metadata | 3 |
| `episodes_fts` | **High** | **High** -- tokenized content index | 1 |

### 3.4 PII Handling

Alaya does not detect PII. It stores what the consuming agent provides. PII can appear in:

1. **Episode content** -- the user says "My name is Alice, I live at 123 Main St"
2. **Semantic nodes** -- consolidation extracts "User's name is Alice"
3. **Impressions** -- perfuming records "User mentioned home address"
4. **Preferences** -- crystallization produces "User prefers communicating from home office"
5. **Embeddings** -- vector representations of any of the above
6. **FTS5 index** -- tokenized episode content (deleted when episode is deleted, via trigger)

**Consumer responsibilities for PII:**

```
Pre-storage (consumer scrubs before calling Alaya):
  1. Run PII detection on episode content before store_episode()
  2. Redact or mask PII: "My name is [REDACTED], I live at [REDACTED]"
  3. Or use field-level encryption on sensitive fields before storage

Post-storage (consumer uses Alaya's deletion API):
  1. purge(PurgeFilter::Session(session_id)) -- delete all episodes in a session
  2. purge(PurgeFilter::All) -- nuclear option, delete everything
  3. forget() -- decay strengths, archive weak nodes (gradual, not immediate)
  4. VACUUM after purge -- reclaim disk space and overwrite deleted pages
```

### 3.5 Content-Hash Integrity

To detect tampering with stored memories (e.g., an adversary modifying the SQLite file directly), Alaya should compute and store a content hash for each episode and semantic node.

**Current state:** Not implemented.

**Planned implementation (v0.1):**

```rust
// On store_episode():
let hash = blake3::hash(episode.content.as_bytes());
// Store hash in a new `content_hash` column

// On query() retrieval:
let stored_hash = row.get::<_, Vec<u8>>("content_hash");
let computed_hash = blake3::hash(content.as_bytes());
if stored_hash != computed_hash.as_bytes() {
    // Flag as tampered, exclude from results or warn
}
```

This protects against offline modification of the SQLite file but not against an adversary who also updates the hash column. For stronger guarantees, the consumer should use filesystem-level integrity (e.g., dm-verity, fs-verity, or application-level signing with a key stored outside the SQLite file).

### 3.6 Surrogate Key Architecture

Alaya uses integer surrogate keys (`EpisodeId`, `NodeId`, `PreferenceId`, `ImpressionId`, `LinkId`) for all internal references. No PII appears in primary keys, foreign keys, or index keys. This means:

- Deleting content from a table removes all PII from that table
- No PII leaks through graph structure (links reference integer IDs, not names)
- No PII in node_strengths (references by type + integer ID)
- FTS5 index contains tokenized content but is kept in sync by triggers (delete episode -> delete FTS5 entry)

---

## 4. Input Validation

### 4.1 FTS5 MATCH Sanitization

FTS5 supports a query language with operators that can alter search semantics or cause errors:

| Operator | Example | Risk |
|----------|---------|------|
| `"..."` | `"exact phrase"` | Unbalanced quotes cause parse error |
| `*` | `prog*` | Prefix search, can return unexpected results |
| `^` | `^first` | Initial token filter |
| `AND` / `OR` / `NOT` | `cat AND dog` | Boolean operators change semantics |
| `NEAR` | `NEAR(a b)` | Proximity search |
| `{column}:` | `content:word` | Column filter |

**Current implementation** in `src/retrieval/bm25.rs`:

```rust
let sanitized: String = query
    .chars()
    .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
    .collect();
```

This replaces all non-alphanumeric, non-whitespace characters with spaces. The result is that:

- FTS5 operators are stripped (quotes, asterisks, carets become spaces)
- Boolean keywords (`AND`, `OR`, `NOT`) become regular search terms (FTS5 treats them as implicit OR when not in uppercase operator context after sanitization)
- Empty queries after sanitization return empty results (no error)
- Unicode alphanumeric characters are preserved (international text works)

**Validation:** If the sanitized query is empty after trimming, `search_bm25()` returns `Ok(vec![])` without executing any SQL.

### 4.2 Episode Content Validation

**Current state:** No validation. Empty strings, excessively long content, and content with control characters are accepted.

**Planned validation (v0.1):**

| Check | Rule | Error |
|-------|------|-------|
| Non-empty | `content.trim().is_empty()` -> reject | `AlayaError::InvalidInput("episode content is empty")` |
| Length limit | `content.len() > MAX_EPISODE_BYTES` -> reject | `AlayaError::InvalidInput("episode content exceeds maximum length")` |
| Session ID non-empty | `session_id.trim().is_empty()` -> reject | `AlayaError::InvalidInput("session_id is empty")` |
| Timestamp positive | `timestamp <= 0` -> reject | `AlayaError::InvalidInput("timestamp must be positive")` |

The `MAX_EPISODE_BYTES` constant should default to 1 MiB (1,048,576 bytes). This prevents a consumer from accidentally storing multi-megabyte content that degrades FTS5 indexing and retrieval performance.

### 4.3 Embedding Validation

**Current state:** No validation. Zero-length embeddings and NaN/infinity values are accepted.

**Planned validation (v0.1):**

| Check | Rule | Error |
|-------|------|-------|
| Non-empty | `embedding.is_empty()` -> reject | `AlayaError::InvalidInput("embedding is empty")` |
| Dimension consistency | If store has existing embeddings, new embedding must match dimension | `AlayaError::InvalidInput("embedding dimension mismatch: expected {n}, got {m}")` |
| Finite values | Any NaN or infinity in embedding -> reject | `AlayaError::InvalidInput("embedding contains non-finite values")` |
| Non-zero norm | All-zero embedding -> reject | `AlayaError::InvalidInput("embedding has zero norm")` |

### 4.4 Query Parameter Validation

**Current state:** `max_results` is unchecked. A `max_results` of 0 is meaningless. A `max_results` of `usize::MAX` could cause memory pressure.

**Planned validation (v0.1):**

| Check | Rule | Behavior |
|-------|------|----------|
| `max_results` bounds | Clamp to `1..=1000` | Silent clamp, no error |
| `text` non-empty | `text.trim().is_empty()` -> empty result | Return `Ok(vec![])` |
| `embedding` validation | Same rules as embedding storage | `AlayaError::InvalidInput` |

### 4.5 Provider Output Validation

The `ConsolidationProvider` trait is implemented by the consumer. Its output is stored directly into Alaya's tables. A malicious or buggy provider could:

1. Return semantic nodes with empty content
2. Return nodes referencing non-existent source episodes
3. Return impressions with extreme valence values
4. Return excessively large numbers of results (memory exhaustion)

**Planned validation (v0.1):**

```rust
// In consolidation.rs, after provider.extract_knowledge():
for node in &new_nodes {
    if node.content.trim().is_empty() {
        continue; // Skip, log warning
    }
    if node.content.len() > MAX_SEMANTIC_NODE_BYTES {
        continue; // Skip, log warning
    }
    // Verify source episodes exist
    for ep_id in &node.source_episodes {
        if episodic::get_episode(conn, *ep_id).is_err() {
            continue; // Skip this source reference
        }
    }
    // Store the validated node
}
```

---

## 5. SQLite Security

### 5.1 Transaction Safety

**Threat:** SQLite in WAL mode allows one writer and multiple readers. If two threads attempt to write simultaneously without `BEGIN IMMEDIATE`, the second writer gets `SQLITE_BUSY` after the default timeout. With `BEGIN DEFERRED` (the default), a read transaction that later attempts to write can deadlock against another writer.

**Current state:** Alaya does not use `BEGIN IMMEDIATE`. All transactions use the default `BEGIN DEFERRED`.

**Required fix (v0.1):** Wrap all write operations in `BEGIN IMMEDIATE` transactions.

```rust
// Current (vulnerable to WAL deadlock):
conn.execute("INSERT INTO episodes ...", params![...])?;

// Required:
conn.execute_batch("BEGIN IMMEDIATE")?;
match conn.execute("INSERT INTO episodes ...", params![...]) {
    Ok(_) => conn.execute_batch("COMMIT")?,
    Err(e) => {
        conn.execute_batch("ROLLBACK")?;
        return Err(e.into());
    }
}
```

This applies to every write path:

- `store_episode()` -- inserts into episodes, embeddings, node_strengths, links
- `store_semantic_node()` -- inserts into semantic_nodes, embeddings, node_strengths
- `store_impression()` -- inserts into impressions
- `crystallize_preferences()` -- inserts/updates preferences
- `create_link()` -- inserts into links
- `on_co_retrieval()` -- updates links
- `consolidate()` -- multi-table inserts
- `transform()` -- multi-table updates and deletes
- `forget()` -- multi-table updates and deletes
- `purge()` -- multi-table deletes

### 5.2 WAL Management

**Threat:** WAL files can grow unbounded if checkpointing is delayed. A large WAL file increases recovery time after a crash and can exhaust disk space.

**Current state:** Alaya sets `PRAGMA journal_mode = WAL` but does not configure WAL size limits or manual checkpointing.

**Planned mitigation (v0.1):**

```sql
-- Set in init_db():
PRAGMA wal_autocheckpoint = 1000;  -- Checkpoint every 1000 pages (~4 MiB)
PRAGMA journal_size_limit = 67108864;  -- Limit WAL to 64 MiB
```

**Consumer guidance:** After `purge(PurgeFilter::All)`, consumers should call:

```rust
// Via raw SQL if needed, or via a planned compact() method:
conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")?;
conn.execute_batch("VACUUM")?;
```

`VACUUM` rewrites the entire database, reclaiming space from deleted pages and overwriting the original file. This is important after large deletions because SQLite's default behavior leaves deleted content in free pages that are reusable but still readable on disk.

### 5.3 PRAGMA Configuration

The current PRAGMA configuration in `schema.rs`:

```sql
PRAGMA journal_mode = WAL;       -- Write-Ahead Logging for concurrent reads
PRAGMA foreign_keys = ON;        -- Enforce referential integrity
PRAGMA synchronous = NORMAL;     -- Balanced durability vs. performance
```

**Security implications:**

| PRAGMA | Setting | Security Effect |
|--------|---------|-----------------|
| `journal_mode = WAL` | Enabled | Allows concurrent reads during writes; WAL file contains recent writes in cleartext |
| `foreign_keys = ON` | Enabled | Prevents orphaned references; not currently leveraged (no FK constraints in schema) |
| `synchronous = NORMAL` | Enabled | Possible data loss of last transaction on power failure; acceptable for memory library |

**Note on foreign_keys:** The schema does not define `FOREIGN KEY` constraints between tables (e.g., links do not reference episodes with a foreign key). The `foreign_keys = ON` pragma has no practical effect until FK constraints are added. This is a known gap -- referential integrity is currently maintained by application logic, not database constraints.

### 5.4 No Raw SQL Exposure

Alaya's public API exposes only typed Rust methods. There is no `execute_raw_sql()` method, no `Connection` accessor, no SQL string parameters. All SQL is internal to the crate, written as string literals with parameterized queries (`?1`, `?2`, etc.).

This eliminates classical SQL injection. The only injection vector is FTS5 MATCH syntax, which is handled by the sanitizer in `search_bm25()`.

### 5.5 FTS5 Automerge

**Threat:** FTS5 accumulates segment files during writes. Without periodic merging, query performance degrades and disk usage increases.

**Current state:** FTS5 automerge is at the SQLite default (auto-merge at 4 segments).

**Planned configuration (v0.1):**

```sql
-- After table creation, or in a periodic maintenance call:
INSERT INTO episodes_fts(episodes_fts, rank) VALUES('automerge', 8);
```

Setting automerge to 8 reduces merge frequency at the cost of slightly more segments, which is appropriate for Alaya's write-light, read-heavy pattern.

---

## 6. Memory Integrity

### 6.1 Memory Poisoning

**The primary threat.** An adversary (or a hallucinating LLM) can inject false information via `store_episode()`. Because Alaya trusts the consuming agent, this false information:

1. Appears in `query()` results with legitimate-looking BM25 and vector scores
2. Gets consolidated into semantic nodes (amplifying the false memory)
3. Gets linked into the Hebbian graph (spreading the false memory to related nodes)
4. Influences preference emergence (behavioral patterns shift toward the false narrative)

**Defense in depth:**

| Layer | Mechanism | Status |
|-------|-----------|--------|
| Pre-storage validation | Consumer validates content before `store_episode()` | Consumer responsibility |
| Content-hash integrity | Detect post-storage tampering via hash column | Planned (v0.1) |
| Provider output validation | Validate semantic nodes returned by `extract_knowledge()` | Planned (v0.1) |
| Quarantine API | Flag suspicious memories, exclude from retrieval | Planned (v0.2) |
| Corroboration tracking | Semantic nodes track how many source episodes support them; low-corroboration nodes are less trusted | Implemented (`corroboration_count` column) |
| Typed reports | Every lifecycle method returns a report, so the consumer can audit what changed | Implemented |

**Consumer guidance for memory poisoning prevention:**

1. Never store raw user input without validation. At minimum, check for prompt injection patterns.
2. Use the `source_episodes` field on semantic nodes to trace provenance.
3. Monitor `ConsolidationReport.nodes_created` -- unexpected spikes indicate unusual consolidation activity.
4. Implement `detect_contradiction()` in your `ConsolidationProvider` to catch conflicting information.
5. Prefer separate SQLite files per trust boundary (per user, per agent, per application).

### 6.2 Memory Resurrection

**Threat:** After deleting an episode with `purge()`, the episode's content may still exist in:

1. Semantic nodes derived from it during consolidation
2. The FTS5 index (cleared by trigger on episode deletion)
3. Embeddings (not automatically cascade-deleted)
4. Graph links (not automatically cascade-deleted)
5. Node strengths (not automatically cascade-deleted)

If a user requests deletion of episode E, but a semantic node S was derived from E, the knowledge persists in S. Worse, a subsequent consolidation cycle could re-derive similar knowledge from other episodes, effectively resurrecting the deleted information.

**Current state:** Episode deletion triggers FTS5 cleanup (via SQL trigger). No other cascading deletion occurs.

**Required fix (v0.1): Tombstone mechanism.**

```sql
-- New table:
CREATE TABLE IF NOT EXISTS tombstones (
    node_type TEXT NOT NULL,
    node_id   INTEGER NOT NULL,
    content_hash BLOB,
    deleted_at INTEGER NOT NULL,
    PRIMARY KEY (node_type, node_id)
);
```

**Cascade deletion workflow:**

```
purge(PurgeFilter::Session("s1")):
  1. Find all episodes in session s1
  2. For each episode E:
     a. Delete E from episodes (trigger cleans FTS5)
     b. Delete embedding for E from embeddings
     c. Delete all links where source or target is E from links
     d. Delete node_strength for E from node_strengths
     e. Insert tombstone (type="episode", id=E.id, hash=hash(E.content))
  3. Find all semantic nodes whose source_episodes only reference deleted episodes
     a. Delete those semantic nodes (cascade their embeddings, links, strengths)
     b. Insert tombstones for them
  4. VACUUM (consumer-triggered, recommended)
```

**Resurrection prevention during consolidation:**

```rust
// In consolidation.rs:
// Before storing a new semantic node, check if its content hash matches
// any tombstone. If so, skip it -- this knowledge was explicitly deleted.
if tombstones::is_tombstoned(conn, &content_hash)? {
    continue; // Do not resurrect deleted knowledge
}
```

### 6.3 Graph Consistency

**Threat:** After deletion, orphaned links can point to non-existent nodes. Spreading activation can follow these links and produce nonsensical results.

**Current state:** No orphan cleanup.

**Planned fix (v0.1):**

```sql
-- Run after any deletion:
DELETE FROM links
WHERE NOT EXISTS (
    SELECT 1 FROM episodes WHERE id = links.source_id AND links.source_type = 'episode'
    UNION ALL
    SELECT 1 FROM semantic_nodes WHERE id = links.source_id AND links.source_type = 'semantic'
    UNION ALL
    SELECT 1 FROM preferences WHERE id = links.source_id AND links.source_type = 'preference'
)
OR NOT EXISTS (
    SELECT 1 FROM episodes WHERE id = links.target_id AND links.target_type = 'episode'
    UNION ALL
    SELECT 1 FROM semantic_nodes WHERE id = links.target_id AND links.target_type = 'semantic'
    UNION ALL
    SELECT 1 FROM preferences WHERE id = links.target_id AND links.target_type = 'preference'
);
```

This should be part of `transform()` or a dedicated `cleanup_orphans()` method.

### 6.4 Embedding Integrity

**Threat:** An adversary who can modify the SQLite file can replace embeddings with vectors designed to manipulate retrieval results. For example, replacing an embedding with a vector close to common queries ensures that content always appears in results.

**Mitigation layers:**

1. **Metadata tracking:** The `embeddings` table stores `model` (which model generated the embedding) and `created_at`. Changed embeddings would have mismatched timestamps.
2. **Re-generation capability:** If embeddings are suspected of tampering, the consumer can delete all embeddings and re-generate them from the source content.
3. **Content-hash cross-reference:** Compare the embedding's `node_id` against the source content's hash to verify alignment.
4. **Filesystem protection:** The primary defense is filesystem permissions and encryption.

### 6.5 Version Tracking

Semantic nodes track `corroboration_count` and `last_corroborated`. This provides a provenance signal: nodes that were corroborated by multiple independent episodes are more trustworthy than single-source nodes.

**Consumer guidance:** When surfacing semantic knowledge to the user or LLM, weight by `corroboration_count` and `confidence`. Nodes with `corroboration_count == 1` should be treated as tentative.

---

## 7. Privacy by Architecture

### 7.1 Zero Network Calls

The core `alaya` crate has four dependencies: `rusqlite`, `serde`, `serde_json`, `thiserror`. None of these make network calls. There is no `reqwest`, no `hyper`, no `tokio::net`, no DNS resolution, no socket creation anywhere in the dependency tree of the core crate.

This is not a policy -- it is a structural property. Adding a networking dependency to the core crate would be a visible change in `Cargo.toml` that violates the first axiom ("Privacy > Features") and the kill list ("Not cloud-dependent").

**Feature-flag crates** (planned for v0.2) may introduce networking:

| Crate | Feature Flag | Network? | Justification |
|-------|-------------|----------|---------------|
| `alaya` (core) | none | **Never** | Core crate axiom |
| `alaya-mcp` | n/a (separate crate) | Local IPC | MCP server uses stdio/HTTP for local agent communication |
| `alaya` | `embed-ort` | No | ONNX Runtime runs locally |
| `alaya` | `embed-fastembed` | Model download on first use | Consumer controls when/if model is downloaded |

### 7.2 No Telemetry

Alaya does not:

- Count API calls
- Measure usage patterns
- Report errors to a remote service
- Send crash dumps
- Phone home for license validation
- Check for updates

The library uses `tracing` (planned) for structured logging that the consumer controls. By default, no logs are emitted. The consumer chooses the log level and subscriber.

### 7.3 No Content Logging

Alaya does not log memory content (episode text, semantic node content, impressions, preferences) at any log level. Error messages reference IDs and types, never content.

```rust
// Correct (implemented):
AlayaError::NotFound(format!("episode {}", id.0))

// Never (would leak content):
AlayaError::NotFound(format!("episode with content '{}'", content))
```

### 7.4 Consumer Controls Storage Location

The consumer decides where the SQLite file lives by passing a path to `AlayaStore::open(path)`. Alaya does not:

- Create files in default locations
- Use `~/.alaya/` or `$XDG_DATA_HOME` without the consumer specifying it
- Write to temporary directories
- Create backup files automatically

---

## 8. Consumer Security Guidance

This section is for developers integrating Alaya into their agents. Alaya provides tools and safe defaults. The consumer is responsible for application-level security.

### 8.1 File Permissions

```bash
# Recommended: owner read/write only
chmod 0600 memory.db
chmod 0600 memory.db-wal
chmod 0600 memory.db-shm

# Or set umask before Alaya creates the file:
umask 0077
```

On macOS and Linux, the SQLite file inherits the process's umask. The consumer should set restrictive permissions before calling `AlayaStore::open()`.

### 8.2 Multi-User Isolation

**The strongest isolation pattern is separate SQLite files per user.**

```rust
// Each user gets their own database file:
let store = AlayaStore::open(format!("/data/users/{}/memory.db", user_id))?;
```

This provides:

- Complete data isolation (different files, different file permissions)
- Independent lifecycle (each user's memories consolidate independently)
- Simple deletion (delete the file to delete all user data)
- No cross-user query leakage (impossible -- separate connections to separate files)

**If separate files are impractical** (e.g., MCP server serving multiple users from one process), the consumer must enforce scoping:

1. Always include `session_id` as a user-scoped value
2. Never share `AlayaStore` instances across users
3. Filter query results by the current user's session IDs
4. Audit that `purge()` operations are scoped to the correct user

### 8.3 Backup Strategy

```rust
// SQLite backup API (via rusqlite):
let src = AlayaStore::open("memory.db")?;
let backup = rusqlite::Connection::open("memory_backup.db")?;
src.conn.backup(rusqlite::DatabaseName::Main, &backup, rusqlite::DatabaseName::Main)?;
```

**Backup recommendations:**

- Use SQLite's online backup API, not filesystem `cp` (WAL consistency)
- Backup frequency matches data value (daily for personal agents, hourly for high-volume)
- Store backups with the same permissions as the source (0600)
- Encrypt backups if the source is encrypted
- Test restore from backup regularly

### 8.4 Provider Security

The `ConsolidationProvider` trait is the only extension point where external code (typically an LLM) influences what Alaya stores. Treat provider output as untrusted input:

```rust
impl ConsolidationProvider for MyProvider {
    fn extract_knowledge(&self, episodes: &[Episode]) -> Result<Vec<NewSemanticNode>> {
        let raw_nodes = self.llm.extract(episodes)?;

        // Validate before returning to Alaya:
        let validated: Vec<NewSemanticNode> = raw_nodes
            .into_iter()
            .filter(|n| !n.content.trim().is_empty())
            .filter(|n| n.content.len() < 10_000)
            .filter(|n| n.confidence >= 0.0 && n.confidence <= 1.0)
            .filter(|n| n.source_episodes.iter().all(|id| {
                episodes.iter().any(|e| e.id == *id)
            }))
            .collect();

        Ok(validated)
    }
}
```

**Provider security checklist:**

- [ ] Validate that returned content is non-empty and within size limits
- [ ] Verify that `source_episodes` references only exist in the input batch
- [ ] Clamp `confidence` to `[0.0, 1.0]`
- [ ] Limit the number of returned nodes per batch (prevent memory exhaustion)
- [ ] Handle LLM refusal/error gracefully (return empty vec, not garbage)
- [ ] Log provider call duration and output size for anomaly detection

### 8.5 PII Handling Recommendations

| Scenario | Recommendation |
|----------|---------------|
| Personal companion agent | Pre-scrub names and addresses if not needed for personalization. Use `purge()` + `VACUUM` on user request. |
| Customer support agent | Never store credit card numbers, SSNs, or passwords. Redact before `store_episode()`. |
| Healthcare agent | Do not use Alaya for PHI without SQLCipher encryption. See HIPAA section below. |
| Multi-user deployment | Separate files per user. Implement right-to-erasure via file deletion. |
| Development/testing | Use `open_in_memory()` for tests. Never use production data in development. |

### 8.6 Recommended Security Configuration

```rust
use alaya::{AlayaStore, NewEpisode, Query};

// 1. Open with explicit path (consumer controls location)
let store = AlayaStore::open("/secure/path/memory.db")?;

// 2. Set file permissions (platform-specific)
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(
        "/secure/path/memory.db",
        std::fs::Permissions::from_mode(0o600),
    )?;
}

// 3. Validate content before storage
fn validate_episode(content: &str) -> Result<(), &'static str> {
    if content.trim().is_empty() {
        return Err("content is empty");
    }
    if content.len() > 1_048_576 {
        return Err("content exceeds 1 MiB limit");
    }
    // Add PII scrubbing here if needed
    Ok(())
}

// 4. Monitor lifecycle operations
let report = store.consolidate(&my_provider)?;
if report.nodes_created > 50 {
    // Unusual: investigate provider output
    log::warn!("consolidation created {} nodes in one cycle", report.nodes_created);
}

// 5. Periodic maintenance
store.transform()?;
store.forget()?;
```

---

## 9. Compliance

### 9.1 GDPR (General Data Protection Regulation)

Alaya is a library, not a data controller or processor. The consumer's application is the controller. However, Alaya must provide the technical capabilities for GDPR compliance.

| GDPR Right | Alaya Capability | Implementation |
|------------|-----------------|----------------|
| Right to Access (Art. 15) | `query()`, `knowledge()`, `preferences()`, `status()` | Consumer calls these methods to export user data |
| Right to Rectification (Art. 16) | No direct update API for episodes | Consumer must delete and re-store corrected episode |
| Right to Erasure (Art. 17) | `purge()` with session, time, or all filters | Consumer calls purge, then `VACUUM` |
| Right to Data Portability (Art. 20) | All types implement `Serialize` | Consumer exports to JSON |
| Data Minimization (Art. 5) | `forget()` and `transform()` | Regular lifecycle reduces stored data |

**Crypto-shredding strategy:**

For consumers who need guaranteed erasure without `VACUUM` (which rewrites the entire database):

1. Encrypt episode content before storage using a per-user key
2. Store the key outside the SQLite file (e.g., OS keychain)
3. To erase: delete the key. All encrypted content becomes unrecoverable.
4. The surrogate key architecture ensures no PII exists in keys or indexes

```rust
// Pseudo-code for crypto-shredding:
let user_key = keychain::get_or_create(user_id);
let encrypted_content = encrypt(episode_content, &user_key);
store.store_episode(&NewEpisode {
    content: encrypted_content,
    // ... other fields
})?;

// To erase all user data:
keychain::delete(user_id);
// Content is now unrecoverable without the key
```

### 9.2 CCPA (California Consumer Privacy Act)

| CCPA Right | Alaya Capability |
|------------|-----------------|
| Right to Know | `query()`, `knowledge()`, `preferences()` -- consumer exports data |
| Right to Delete | `purge()` + `VACUUM` |
| Right to Opt-Out | N/A -- Alaya does not sell data or share with third parties |
| Non-Discrimination | N/A -- Alaya is a library, not a service |

### 9.3 SOC 2

Alaya simplifies SOC 2 compliance for consumers because:

- **No cloud dependencies:** No third-party sub-processors to audit
- **No network calls:** No data transmission to assess
- **No credentials:** No key management for the library itself
- **Single file:** Data inventory is trivial (one file per user)
- **Open source (MIT):** Full code audit is possible

The consumer's SOC 2 scope includes Alaya as an embedded dependency, not as an external service.

### 9.4 HIPAA

Alaya is **not** designed for HIPAA compliance and should not be used to store Protected Health Information (PHI) without additional safeguards:

| HIPAA Requirement | Alaya Status |
|-------------------|-------------|
| Encryption at rest | Not provided (consumer must use SQLCipher or filesystem encryption) |
| Access controls | Not provided (consumer must implement) |
| Audit logging | Not provided (consumer must implement around Alaya calls) |
| Breach notification | Not applicable (library, not service) |
| Business Associate Agreement | Not applicable (MIT library, no business relationship) |

**If a healthcare agent uses Alaya:**

1. Build with `rusqlite` `bundled-sqlcipher` feature for encryption at rest
2. Wrap every Alaya call with audit logging
3. Implement role-based access control around Alaya methods
4. Use separate encrypted files per patient
5. Implement automated PHI detection before `store_episode()`
6. Consult a HIPAA compliance specialist

---

## 10. Security Hardening Roadmap

### v0.1 (Current Priority)

| Item | Status | Severity |
|------|--------|----------|
| `BEGIN IMMEDIATE` for all write transactions | Not implemented | Critical |
| Input validation at API boundary (episodes, embeddings, queries) | Not implemented | High |
| Tombstone table for cascade deletion | Not implemented | High |
| Orphan link cleanup in `transform()` | Not implemented | Medium |
| WAL autocheckpoint and size limit configuration | Not implemented | Medium |
| Content-hash integrity column | Not implemented | Medium |
| Provider output validation in consolidation | Not implemented | Medium |
| `#[non_exhaustive]` on all public enums | Not implemented | Low |

### v0.2 (Ecosystem)

| Item | Status | Severity |
|------|--------|----------|
| Quarantine API (flag memories, exclude from retrieval) | Planned | High |
| `compact()` method (VACUUM + WAL checkpoint) | Planned | Medium |
| SQLCipher feature flag for encryption at rest | Planned | High |
| Audit trait (consumer-provided logging hook) | Planned | Medium |
| FTS5 automerge configuration | Planned | Low |
| Foreign key constraints in schema | Planned | Medium |

### v0.3 (Growth)

| Item | Status | Severity |
|------|--------|----------|
| Formal security audit | Planned | Critical |
| Fuzz testing (cargo-fuzz) for FTS5 sanitizer, embedding deserializer | Planned | High |
| Memory-safe API audit (unsafe usage review) | Planned | Medium |
| DEF CON AI Village presentation | Planned | N/A |

---

## 11. Security Checklist for Library Consumers

Use this checklist when integrating Alaya into a production agent.

### Pre-Deployment

- [ ] **File permissions:** SQLite file is `0600` (owner read/write only)
- [ ] **Storage location:** Database path is in a directory not accessible to unprivileged users
- [ ] **Encryption:** Either filesystem encryption or SQLCipher is configured if storing sensitive data
- [ ] **User isolation:** Each user has a separate SQLite file, or session_id scoping is strictly enforced
- [ ] **PII scrubbing:** Content is scrubbed before `store_episode()` if PII must not persist
- [ ] **Provider validation:** `ConsolidationProvider` output is validated before returning to Alaya
- [ ] **Content validation:** Episode content is checked for emptiness, length, and injection patterns before storage

### Runtime

- [ ] **Lifecycle monitoring:** `ConsolidationReport`, `ForgettingReport`, and `TransformationReport` are logged and monitored for anomalies
- [ ] **Error handling:** `AlayaError::Db` errors are handled (not silently ignored), especially `SQLITE_BUSY`
- [ ] **Periodic maintenance:** `transform()` and `forget()` are called on a schedule appropriate to the agent type
- [ ] **Backup strategy:** SQLite backup API is used on a schedule, not filesystem copy

### Data Deletion

- [ ] **User erasure:** A user deletion request triggers `purge(PurgeFilter::Session(user_sessions))` for all user sessions
- [ ] **Cascade verification:** After purge, verify that semantic nodes derived solely from deleted episodes are also deleted
- [ ] **VACUUM:** After large deletions, `VACUUM` is called to overwrite freed pages
- [ ] **Backup cleanup:** Backups containing deleted user data are also deleted
- [ ] **Tombstone check:** Tombstone table is consulted during consolidation to prevent resurrection

### Incident Response

- [ ] **Compromise detection:** If the SQLite file is suspected of tampering, re-verify content hashes
- [ ] **Provider compromise:** If the LLM provider is compromised, audit recent `ConsolidationReport` outputs and quarantine suspicious semantic nodes
- [ ] **File theft:** If the SQLite file is stolen, treat all content as compromised (it is unencrypted by default). Rotate any keys or credentials that appeared in conversations.

---

## Appendix A: Dependency Security

The core Alaya crate has a minimal dependency tree:

| Crate | Purpose | Network? | Unsafe? | Security Notes |
|-------|---------|----------|---------|----------------|
| `rusqlite` 0.32 | SQLite bindings | No | Yes (FFI to C) | Bundles SQLite C library; well-audited upstream |
| `serde` 1.x | Serialization framework | No | Yes (proc macro) | Most-downloaded crate on crates.io; extensively audited |
| `serde_json` 1.x | JSON serialization | No | Minimal | Standard JSON parser |
| `thiserror` 2.x | Error derive macro | No | No | Compile-time only |

**Supply chain mitigation:**

- Pin exact dependency versions in `Cargo.lock`
- `cargo audit` in CI (checks for known vulnerabilities)
- `cargo deny` for license and duplicate checking
- No transitive networking dependencies in core

## Appendix B: SQLite Security Configuration Reference

```sql
-- Full security-hardened PRAGMA configuration for init_db():

-- WAL mode: concurrent reads, single writer
PRAGMA journal_mode = WAL;

-- Enforce foreign key constraints (when added)
PRAGMA foreign_keys = ON;

-- Balanced durability: possible loss of last transaction on power failure
-- Use FULL for maximum durability at ~2x write cost
PRAGMA synchronous = NORMAL;

-- WAL checkpoint every 1000 pages (~4 MiB)
PRAGMA wal_autocheckpoint = 1000;

-- Limit WAL file to 64 MiB
PRAGMA journal_size_limit = 67108864;

-- Secure deletion: overwrite deleted content with zeros
-- Enable only if performance allows (significant write overhead)
-- PRAGMA secure_delete = ON;

-- Memory-mapped I/O limit (0 = disabled, prevents mmap-based attacks)
-- Trade-off: mmap improves read performance for large DBs
PRAGMA mmap_size = 0;
```

## Appendix C: Glossary

| Term | Definition |
|------|-----------|
| **Memory poisoning** | Injecting false information into the memory store that persists and influences future retrieval and consolidation |
| **Memory resurrection** | Deleted information re-appearing through consolidation of related memories that were not also deleted |
| **FTS5 injection** | Using FTS5 query syntax operators to alter the semantics of a full-text search |
| **Tombstone** | A record of a deleted entity, used to prevent resurrection during consolidation |
| **Crypto-shredding** | Rendering encrypted data unrecoverable by destroying the encryption key rather than the ciphertext |
| **Surrogate key** | An integer identifier with no inherent meaning, used to avoid PII in database keys |
| **WAL** | Write-Ahead Logging, SQLite's mode for concurrent read/write access |
| **Hebbian LTP** | Long-Term Potentiation: strengthening graph links when nodes are co-retrieved |
| **Bjork dual-strength** | Memory model where storage strength (learning) and retrieval strength (accessibility) are independent |
| **Vasana** | Yogacara term for subliminal impressions that accumulate and crystallize into preferences |
| **Provider** | Consumer-supplied implementation of the ConsolidationProvider trait, typically wrapping an LLM |

---

*Generated by North Star Advisor | Phase 8 | 2026-02-26*
