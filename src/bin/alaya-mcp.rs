//! Alaya MCP Server — expose memory operations over Model Context Protocol.
//!
//! Usage:
//!   cargo build --release --features mcp
//!   ./target/release/alaya-mcp
//!
//! Environment:
//!   ALAYA_DB — path to SQLite database (default: ~/.alaya/memory.db)

use std::path::PathBuf;

use alaya::mcp::AlayaMcp;
use alaya::AlayaStore;
use rmcp::ServiceExt;
use tokio::io::{stdin, stdout};

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
    let db_path = resolve_db_path();
    eprintln!("alaya-mcp: opening database at {}", db_path.display());

    let store = AlayaStore::open(&db_path)?;
    let server = AlayaMcp::new(store);

    let transport = (stdin(), stdout());
    let service = server.serve(transport).await?;
    service.waiting().await?;

    Ok(())
}
