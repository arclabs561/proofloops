use proofpatch_core::config;

#[test]
fn research_preset_accepts_must_include_all() {
    let txt = r#"
[research.presets.demo]
query = "LLL swap count"
must_include_any = ["lll"]
must_include_all = ["swap", "count"]
"#;
    let cfg: config::ProofpatchConfig = toml::from_str(txt).expect("toml parse");
    let p = cfg.research.resolve_preset("demo").expect("preset");
    assert_eq!(p.query, "LLL swap count");
    assert_eq!(p.must_include_any, vec!["lll"]);
    assert_eq!(p.must_include_all, vec!["swap", "count"]);
}

#[test]
fn research_preset_merges_tree_search_defaults() {
    let txt = r#"
[research.defaults.tree_search]
goal_first_k = 5
smt_depth = 2
smt_solver = "z3"
smt_timeout_ms = 2500
smt_explain = true
smt_explain_max_hyps = 9
hint_packs = ["base"]

[research.presets.demo]
query = "LLL swap count"

[research.presets.demo.tree_search]
goal_first_k = 7
smt_solver = "cvc5"
smt_timeout_ms = 4000
hint_packs = ["demo"]
"#;
    let cfg: config::ProofpatchConfig = toml::from_str(txt).expect("toml parse");
    let p = cfg.research.resolve_preset("demo").expect("preset");
    let ts = p.tree_search.expect("tree_search");
    assert_eq!(ts.goal_first_k, Some(7));
    assert_eq!(ts.smt_depth, Some(2));
    assert_eq!(ts.smt_solver.as_deref(), Some("cvc5"));
    assert_eq!(ts.smt_timeout_ms, Some(4000));
    assert_eq!(ts.smt_explain, Some(true));
    assert_eq!(ts.smt_explain_max_hyps, Some(9));
    let expected = vec!["demo".to_string()];
    assert_eq!(ts.hint_packs.as_deref(), Some(expected.as_slice()));
}
