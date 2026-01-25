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

