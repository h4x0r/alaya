# Alaya MCP Server — Quick Start Guide

This guide walks you through setting up the Alaya MCP server and using it
in your first session. By the end, you will have a working memory system
that stores conversations, extracts knowledge, and builds an associative
graph — all from a single SQLite file.

## 1. Setup

### Building the server

```bash
git clone https://github.com/SecurityRonin/alaya.git
cd alaya
cargo build --release --features mcp --bin alaya-mcp
```

The binary is at `target/release/alaya-mcp`.

### Configuring your agent

Add the server to your agent's MCP configuration.

**Claude Desktop** (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "alaya": {
      "command": "/path/to/alaya/target/release/alaya-mcp"
    }
  }
}
```

**Claude Code** (`.claude/settings.json` or project settings):

```json
{
  "mcpServers": {
    "alaya": {
      "command": "/path/to/alaya/target/release/alaya-mcp",
      "args": []
    }
  }
}
```

The server communicates over stdio (JSON-RPC). Data is stored in
`~/.alaya/memory.db` by default. Override with the `ALAYA_DB` environment
variable.

## 2. First Session Walkthrough

### Step 1: Import existing memories (optional)

If you already have memories in claude-mem or Claude Code conversation
files, import them first.

**From claude-mem:**

```
Tool: import_claude_mem
Response: "Imported 42 observations -> 127 semantic nodes. 5 categories assigned."
```

**From Claude Code JSONL files:**

```
Tool: import_claude_code(path: "~/.claude/projects/-Users-me-myproject/abc123.jsonl")
Response: "Imported 156 messages from 3 sessions as episodes. Call 'learn' to consolidate."
```

### Step 2: Check status

```
Tool: status
Response:
  Memory Status:
    Episodes: 156 (0 this session, 156 unconsolidated)
    Knowledge: 89 facts, 21 relationships, 8 events, 9 concepts
    Categories: 5 (programming, cooking, fitness, travel, music)
    Preferences: 0 crystallized, 0 impressions accumulating
    Graph: 254 links (strongest: "Rust" <-> "async" weight 0.82)
    Embedding coverage: 127/283 nodes (45%)
```

The status tool gives a full breakdown: episode counts (including how many
are unconsolidated), knowledge by type, emergent categories, preferences,
graph statistics with the strongest link, and embedding coverage.

### Step 3: Store conversation messages

Call `remember` for each message you want Alaya to retain:

```
Tool: remember(content: "I've been learning Rust for 3 months", role: "user", session_id: "session-1")
Response: "Stored episode 157 in session 'session-1'."

Tool: remember(content: "That's great! What aspects interest you most?", role: "assistant", session_id: "session-1")
Response: "Stored episode 158 in session 'session-1'."

Tool: remember(content: "Async programming and the ownership model", role: "user", session_id: "session-1")
Response: "Stored episode 159 in session 'session-1'."
```

Continue storing messages as the conversation progresses.

### Step 4: See the consolidation prompt

After 10 unconsolidated episodes, Alaya prompts you to extract knowledge:

```
Tool: remember(content: "...", role: "user", session_id: "session-1")
Response:
  "Stored episode 166 in session 'session-1'.

  --- Consolidation suggested ---
  You have 10 unconsolidated episodes. Please extract key facts and call the 'learn' tool.
  Recent unconsolidated episodes:
  [157] user: I've been learning Rust for 3 months
  [158] assistant: That's great! What aspects interest you most?
  [159] user: Async programming and the ownership model
  ..."
```

This is your cue to extract knowledge from the conversation.

### Step 5: Extract and learn

Read the unconsolidated episodes, extract facts, and call `learn`:

```
Tool: learn(facts: [
    {content: "User has been learning Rust for 3 months", node_type: "fact", confidence: 0.9},
    {content: "User is interested in async programming", node_type: "fact", confidence: 0.8},
    {content: "Rust and async programming are related for the user", node_type: "relationship", confidence: 0.7}
], session_id: "session-1")
Response: "Learned 3 facts: 3 nodes created, 10 links created, 1 category assigned"
```

The `learn` tool:
- Creates semantic nodes with full lifecycle wiring (strength, decay, graph links)
- Links facts to source episodes via the `session_id`
- Auto-assigns categories based on content clustering
- Resets the unconsolidated counter

### Step 6: Query knowledge

**Search memories:**

```
Tool: recall(query: "What does the user know about Rust?")
Response:
  Found 3 memories:

  1. [fact] (score: 0.912) User has been learning Rust for 3 months
  2. [user] (score: 0.847) I've been learning Rust for 3 months
  3. [fact] (score: 0.793) User is interested in async programming
```

**Browse all semantic knowledge:**

```
Tool: knowledge
Response:
  Found 92 knowledge nodes:

  - [fact] User has been learning Rust for 3 months (confidence: 0.90)
  - [fact] User is interested in async programming (confidence: 0.80)
  - [relationship] Rust and async programming are related for the user (confidence: 0.70)
  ...
```

**Filter by type or category:**

```
Tool: knowledge(node_type: "relationship", category: "programming")
Response:
  Found 5 knowledge nodes:

  - [relationship] Rust and async programming are related for the user (confidence: 0.70)
  ...
```

### Step 7: Check status again

```
Tool: status
Response:
  Memory Status:
    Episodes: 166 (10 this session, 0 unconsolidated)
    Knowledge: 92 facts, 22 relationships, 8 events, 9 concepts
    Categories: 5 (programming, cooking, fitness, travel, music)
    Preferences: 0 crystallized, 0 impressions accumulating
    Graph: 264 links (strongest: "Rust" <-> "async" weight 0.82)
    Embedding coverage: 130/288 nodes (45%)
```

Notice: unconsolidated is now 0. The knowledge counts increased. The graph
has more links connecting the new facts to existing knowledge.

## 3. Recommended System Prompt

Add this to your agent's system prompt to guide memory usage:

```
You have access to Alaya, a memory system. Use it as follows:

- Call 'remember' to store important conversation messages (both user and assistant).
- When you see a consolidation prompt (after 10 messages), extract key facts,
  relationships, and concepts from the listed episodes, then call 'learn' with them.
- Call 'recall' before responding to retrieve relevant context from past conversations.
- Call 'status' periodically to monitor memory health (unconsolidated count, categories).
- Use 'knowledge' and 'preferences' to access distilled information without searching.
- The system handles maintenance automatically every 25 episodes, but you can call
  'maintain' manually if memory feels stale.
- Use 'import_claude_mem' or 'import_claude_code' to bootstrap from existing memory sources.
```

## 4. Architecture Overview

Alaya organizes memory into three stores, inspired by cognitive science:

```
Episodic Store          Semantic Store          Implicit Store
(raw conversations)     (extracted knowledge)   (behavioral patterns)
       |                       |                       |
       +---- Hebbian Graph ----+---- Graph Links ------+
             (co-retrieval strengthens associations)
```

**Episodic Store** holds raw conversation messages. Every call to `remember`
creates an episode. Episodes are the raw material for knowledge extraction.

**Semantic Store** holds distilled knowledge — facts, relationships, events,
and concepts. The `learn` tool creates semantic nodes from episodes. These
nodes have confidence scores, decay over time, and cluster into emergent
categories.

**Implicit Store** holds user preferences that emerge from accumulated
behavioral impressions. Preferences crystallize when enough evidence
accumulates (handled via the Rust API's `perfume` method).

The **Hebbian graph** connects all three stores. Links strengthen through
co-retrieval (memories retrieved together become more strongly associated)
and weaken through Long-Term Depression during maintenance. Spreading
activation traverses the graph to find indirect connections.

### The learn tool bridges episodic and semantic

The key workflow is: `remember` (episodes) -> consolidation prompt ->
`learn` (semantic nodes). The agent acts as the consolidation provider,
reading episodes and extracting structured knowledge. This keeps the LLM
in the loop for knowledge extraction while Alaya handles storage, graph
dynamics, categorization, and lifecycle management.

## 5. Tool Reference

| Tool | Description |
|------|-------------|
| `remember` | Store a conversation message as an episode |
| `recall` | Search memory with hybrid retrieval (BM25 + vector + graph + RRF) |
| `learn` | Teach extracted knowledge directly (facts, relationships, events, concepts) |
| `status` | Get rich memory statistics (episodes, knowledge breakdown, categories, graph, embeddings) |
| `preferences` | Get crystallized user preferences, optionally filtered by domain |
| `knowledge` | Get semantic knowledge nodes, filterable by type, confidence, and category |
| `categories` | List emergent categories with stability filter |
| `neighbors` | Get graph neighbors of a node via spreading activation |
| `node_category` | Check which category a semantic node belongs to |
| `maintain` | Run memory maintenance (dedup, link pruning, decay) |
| `purge` | Delete memories by session, age, or all |
| `import_claude_mem` | Import observations from a claude-mem SQLite database |
| `import_claude_code` | Import conversation history from Claude Code JSONL files |
