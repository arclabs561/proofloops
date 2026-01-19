//! Smoke test for `proofloops-mcp mcp-stdio`.
//!
//! This starts a child process running `proofloops-mcp mcp-stdio` and calls a couple tools.
//! It validates the stdio MCP surface without relying on Cursor as the client.

#[cfg(not(feature = "stdio"))]
fn main() {
    eprintln!("stdio_smoke requires `--features stdio` (or default features enabled)");
}

#[cfg(feature = "stdio")]
use rmcp::{
    model::CallToolRequestParam,
    service::ServiceExt,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
#[cfg(feature = "stdio")]
// keep serde_json in scope for json! macro usage
use serde_json as _;
#[cfg(feature = "stdio")]
use std::path::PathBuf;
#[cfg(feature = "stdio")]
use tokio::process::Command;

#[cfg(feature = "stdio")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // `CARGO_MANIFEST_DIR` here is `.../proofloops/mcp-server`.
    // The binary is built into the *workspace* `target/` by default.
    let workspace_root = root
        .parent()
        .expect("mcp-server should be nested under proofloops/")
        .to_path_buf();
    let bin = workspace_root.join("target/debug/proofloops-mcp");
    if !bin.exists() {
        anyhow::bail!(
            "missing server binary at {}\n\nBuild it with:\n  cargo build -p proofloops-mcp --bin proofloops-mcp --features stdio",
            bin.display()
        );
    }
    eprintln!("spawning: {} mcp-stdio", bin.display());

    // Make the smoke test independent of any particular repo layout.
    //
    // Default to a known Lean repo inside this workspace if the env var isn't set.
    // (This path is local developer convenience, not a public contract.)
    let repo_root = std::env::var("PROOFLOOPS_SMOKE_REPO_ROOT")
        .unwrap_or_else(|_| "/Users/arc/Documents/dev/geometry-of-numbers".to_string());
    let file = std::env::var("PROOFLOOPS_SMOKE_FILE")
        .unwrap_or_else(|_| "Covolume/Cauchy/Main.lean".to_string());

    let service = ()
        .serve(TokioChildProcess::new(Command::new(&bin).configure(
            |cmd| {
                cmd.arg("mcp-stdio");
            },
        ))?)
        .await?;

    let info = service.peer_info();
    println!("peer_info: {:#?}", info);

    let tools = service.list_tools(Default::default()).await?;
    println!("tools: {:#?}", tools);

    // Keep this cheap: triage one file with a small timeout.
    let triage = service
        .call_tool(CallToolRequestParam {
            name: "proofloops_triage_file".into(),
            arguments: Some(
                serde_json::json!({
                    "repo_root": repo_root.clone(),
                    "file": file.clone(),
                    "timeout_s": 120,
                    "max_sorries": 3,
                    "context_lines": 1
                })
                .as_object()
                .cloned()
                .unwrap_or_default(),
            ),
        })
        .await?;
    println!("proofloops_triage_file: {:#?}", triage);

    let pack = service
        .call_tool(CallToolRequestParam {
            name: "proofloops_context_pack".into(),
            arguments: Some(
                serde_json::json!({
                    "repo_root": repo_root.clone(),
                    "file": file.clone(),
                    "decl": "cauchy_decomposition",
                    "context_lines": 20,
                    "nearby_lines": 60,
                    "max_nearby_decls": 20,
                    "max_imports": 20
                })
                .as_object()
                .cloned()
                .unwrap_or_default(),
            ),
        })
        .await?;
    println!("proofloops_context_pack: {:#?}", pack);

    let step = service
        .call_tool(CallToolRequestParam {
            name: "proofloops_agent_step".into(),
            arguments: Some(
                serde_json::json!({
                    "repo_root": repo_root.clone(),
                    "file": file.clone(),
                    "timeout_s": 120,
                    "write": false
                })
                .as_object()
                .cloned()
                .unwrap_or_default(),
            ),
        })
        .await?;
    println!("proofloops_agent_step: {:#?}", step);

    service.cancel().await?;
    Ok(())
}
