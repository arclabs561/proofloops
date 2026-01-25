use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProofpatchConfig {
    #[serde(default)]
    pub research: ResearchConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ResearchConfig {
    #[serde(default)]
    pub presets: HashMap<String, ResearchPreset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchPreset {
    pub query: String,
    #[serde(default)]
    pub must_include_any: Vec<String>,
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub llm_summary: bool,
    #[serde(default = "default_llm_timeout_s")]
    pub llm_timeout_s: u64,
    /// Which structured summary schema to request from the LLM.
    /// Examples: "formalization_v1", "formalization_v2".
    #[serde(default)]
    pub llm_summary_kind: Option<String>,
    /// Cap: number of items in `top`.
    #[serde(default = "default_llm_max_top")]
    pub llm_max_top: usize,
    /// Cap: number of items in each string list (keywords/idents/queries/etc).
    #[serde(default = "default_llm_max_list_items")]
    pub llm_max_list_items: usize,
    /// Cap: max characters per emitted string item (Unicode scalar values).
    #[serde(default = "default_llm_max_str_chars")]
    pub llm_max_str_chars: usize,
}

fn default_max_results() -> usize {
    8
}

fn default_timeout_ms() -> u64 {
    20_000
}

fn default_llm_timeout_s() -> u64 {
    20
}

fn default_llm_max_top() -> usize {
    3
}

fn default_llm_max_list_items() -> usize {
    12
}

fn default_llm_max_str_chars() -> usize {
    160
}

pub fn config_path(repo_root: &Path) -> PathBuf {
    repo_root.join("proofpatch.toml")
}

pub fn load_from_repo_root(repo_root: &Path) -> Result<Option<ProofpatchConfig>, String> {
    let p = config_path(repo_root);
    if !p.exists() {
        return Ok(None);
    }
    let txt = std::fs::read_to_string(&p).map_err(|e| format!("read {}: {e}", p.display()))?;
    let cfg: ProofpatchConfig =
        toml::from_str(&txt).map_err(|e| format!("parse {}: {e}", p.display()))?;
    Ok(Some(cfg))
}

