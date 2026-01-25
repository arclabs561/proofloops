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
    pub defaults: ResearchDefaults,
    #[serde(default)]
    pub presets: HashMap<String, ResearchPreset>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ResearchDefaults {
    #[serde(default)]
    pub max_results: Option<usize>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub llm_summary: Option<bool>,
    #[serde(default)]
    pub llm_timeout_s: Option<u64>,
    #[serde(default)]
    pub llm_summary_kind: Option<String>,
    #[serde(default)]
    pub llm_max_top: Option<usize>,
    #[serde(default)]
    pub llm_max_list_items: Option<usize>,
    #[serde(default)]
    pub llm_max_str_chars: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchPreset {
    pub query: String,
    #[serde(default)]
    pub must_include_any: Vec<String>,
    #[serde(default)]
    pub max_results: Option<usize>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub llm_summary: Option<bool>,
    #[serde(default)]
    pub llm_timeout_s: Option<u64>,
    /// Which structured summary schema to request from the LLM.
    /// Examples: "formalization_v1", "formalization_v2".
    #[serde(default)]
    pub llm_summary_kind: Option<String>,
    /// Cap: number of items in `top`.
    #[serde(default)]
    pub llm_max_top: Option<usize>,
    /// Cap: number of items in each string list (keywords/idents/queries/etc).
    #[serde(default)]
    pub llm_max_list_items: Option<usize>,
    /// Cap: max characters per emitted string item (Unicode scalar values).
    #[serde(default)]
    pub llm_max_str_chars: Option<usize>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchPresetResolved {
    pub query: String,
    pub must_include_any: Vec<String>,
    pub max_results: usize,
    pub timeout_ms: u64,
    pub llm_summary: bool,
    pub llm_timeout_s: u64,
    pub llm_summary_kind: Option<String>,
    pub llm_max_top: usize,
    pub llm_max_list_items: usize,
    pub llm_max_str_chars: usize,
}

impl ResearchConfig {
    pub fn resolve_preset(&self, name: &str) -> Option<ResearchPresetResolved> {
        let p = self.presets.get(name)?.clone();
        let d = &self.defaults;
        Some(ResearchPresetResolved {
            query: p.query,
            must_include_any: p.must_include_any,
            max_results: p
                .max_results
                .or(d.max_results)
                .unwrap_or_else(default_max_results),
            timeout_ms: p
                .timeout_ms
                .or(d.timeout_ms)
                .unwrap_or_else(default_timeout_ms),
            llm_summary: p.llm_summary.or(d.llm_summary).unwrap_or(false),
            llm_timeout_s: p
                .llm_timeout_s
                .or(d.llm_timeout_s)
                .unwrap_or_else(default_llm_timeout_s),
            llm_summary_kind: p.llm_summary_kind.or_else(|| d.llm_summary_kind.clone()),
            llm_max_top: p
                .llm_max_top
                .or(d.llm_max_top)
                .unwrap_or_else(default_llm_max_top),
            llm_max_list_items: p
                .llm_max_list_items
                .or(d.llm_max_list_items)
                .unwrap_or_else(default_llm_max_list_items),
            llm_max_str_chars: p
                .llm_max_str_chars
                .or(d.llm_max_str_chars)
                .unwrap_or_else(default_llm_max_str_chars),
        })
    }
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

