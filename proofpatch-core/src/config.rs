use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProofpatchConfig {
    #[serde(default)]
    pub research: ResearchConfig,
    #[serde(default)]
    pub hints: HintsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct HintsConfig {
    #[serde(default)]
    pub defaults: HintsDefaults,
    #[serde(default)]
    pub packs: HashMap<String, HintPack>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct HintsDefaults {
    /// Hint packs enabled by default for this repo.
    ///
    /// This is the main knob for "repo-shaped wisdom" while keeping proofpatch itself repo-agnostic.
    #[serde(default)]
    pub enabled_packs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct HintPack {
    #[serde(default)]
    pub rules: Vec<HintRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct HintRule {
    /// If non-empty, all of these substrings must appear in the goal/context surface.
    #[serde(default)]
    pub when_contains_all: Vec<String>,
    /// If non-empty, at least one of these substrings must appear in the goal/context surface.
    #[serde(default)]
    pub when_contains_any: Vec<String>,
    /// Candidate tactic scripts to try when this rule matches.
    ///
    /// These can be single-line tactics or multi-line blocks; they should be *deterministic* unless
    /// you explicitly opt into sorry-bearing candidates elsewhere.
    #[serde(default)]
    pub candidates: Vec<String>,
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
    /// Optional defaults for proof search behavior (consumed by `proofpatch-cli tree-search-nearest`).
    #[serde(default)]
    pub tree_search: Option<TreeSearchPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct TreeSearchPolicy {
    #[serde(default)]
    pub goal_first_k: Option<usize>,
    #[serde(default)]
    pub smt_depth: Option<usize>,
    /// Optional hint packs to enable for goal-derived candidates.
    ///
    /// Packs are defined in `proofpatch.toml` under `[hints.packs.<name>]`.
    #[serde(default)]
    pub hint_packs: Option<Vec<String>>,
    /// Which SMT solver to use for LIA checks.
    ///
    /// Values:
    /// - "auto" (default): let `smtkit` pick (`SMTKIT_SOLVER` env var or a small built-in list)
    /// - "z3": force `z3 -in -smt2`
    /// - "cvc5": force `cvc5 --lang smt2 --incremental`
    /// - otherwise: treated as an SMTKIT_SOLVER-style command line.
    #[serde(default)]
    pub smt_solver: Option<String>,
    #[serde(default)]
    pub smt_timeout_ms: Option<u64>,
    #[serde(default)]
    pub smt_explain: Option<bool>,
    #[serde(default)]
    pub smt_explain_max_hyps: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchPreset {
    pub query: String,
    #[serde(default)]
    pub must_include_any: Vec<String>,
    /// Post-filter: all of these tokens must appear in (title + abstract), lowercased substring match.
    #[serde(default)]
    pub must_include_all: Vec<String>,
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
    /// Optional per-preset proof search policy.
    #[serde(default)]
    pub tree_search: Option<TreeSearchPolicy>,
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
    pub must_include_all: Vec<String>,
    pub max_results: usize,
    pub timeout_ms: u64,
    pub llm_summary: bool,
    pub llm_timeout_s: u64,
    pub llm_summary_kind: Option<String>,
    pub llm_max_top: usize,
    pub llm_max_list_items: usize,
    pub llm_max_str_chars: usize,
    pub tree_search: Option<TreeSearchPolicy>,
}

impl ResearchConfig {
    pub fn resolve_preset(&self, name: &str) -> Option<ResearchPresetResolved> {
        let p = self.presets.get(name)?.clone();
        let d = &self.defaults;
        let tree_search = {
            let mut out: TreeSearchPolicy = TreeSearchPolicy::default();
            let mut any = false;
            if let Some(td) = d.tree_search.clone() {
                out = td;
                any = true;
            }
            if let Some(tp) = p.tree_search.clone() {
                if tp.goal_first_k.is_some() {
                    out.goal_first_k = tp.goal_first_k;
                }
                if tp.smt_depth.is_some() {
                    out.smt_depth = tp.smt_depth;
                }
                if tp.hint_packs.is_some() {
                    out.hint_packs = tp.hint_packs;
                }
                if tp.smt_solver.is_some() {
                    out.smt_solver = tp.smt_solver;
                }
                if tp.smt_timeout_ms.is_some() {
                    out.smt_timeout_ms = tp.smt_timeout_ms;
                }
                if tp.smt_explain.is_some() {
                    out.smt_explain = tp.smt_explain;
                }
                if tp.smt_explain_max_hyps.is_some() {
                    out.smt_explain_max_hyps = tp.smt_explain_max_hyps;
                }
                any = true;
            }
            if any {
                Some(out)
            } else {
                None
            }
        };
        Some(ResearchPresetResolved {
            query: p.query,
            must_include_any: p.must_include_any,
            must_include_all: p.must_include_all,
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
            tree_search,
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
