//! Minimal SMT-based entailment checks for linear integer arithmetic (LIA).
//!
//! This is intentionally conservative and best-effort:
//! - If we cannot confidently parse/sort variables, return `Ok(None)`.
//! - Uses an external SMT solver via `smtkit` if available; if none available, return `Ok(None)`.
//!
//! Soundness posture: this is a *heuristic signal* for ranking / candidate selection.
//! It must never be used as a proof of a Lean goal without verification.

use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VarKind {
    Int,
    Nat,
}

fn sanitize_name(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out = "x".to_string();
    }
    if out
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        out.insert(0, '_');
    }
    out
}

fn extract_decl_kind(hyp_text: &str) -> Option<(String, VarKind)> {
    // Recognize tiny declaration shapes like:
    // - `n : ℕ` / `n : Nat`
    // - `m : ℤ` / `m : Int`
    let (name, ty) = hyp_text.split_once(':')?;
    let name = name.trim();
    let ty = ty.trim();
    if name.is_empty() || ty.is_empty() {
        return None;
    }
    let kind = if ty.contains('ℕ') || ty.contains("Nat") {
        VarKind::Nat
    } else if ty.contains('ℤ') || ty.contains("Int") {
        VarKind::Int
    } else {
        return None;
    };
    Some((sanitize_name(name), kind))
}

#[derive(Debug, Clone)]
struct LinearExpr {
    // var -> coefficient
    coeffs: std::collections::BTreeMap<String, i64>,
    c0: i64,
}

fn parse_linear_expr_int(s: &str) -> Option<LinearExpr> {
    // Small parser: sums/differences of identifiers and integer literals.
    // Rejects obvious non-LIA operators.
    let bad = ['*', '/', '^', '·', '↑', '∑', '∏'];
    if s.chars().any(|c| bad.contains(&c)) {
        return None;
    }
    let mut coeffs: std::collections::BTreeMap<String, i64> = std::collections::BTreeMap::new();
    let mut c0: i64 = 0;
    let mut i = 0usize;
    let chars: Vec<char> = s.chars().collect();
    let mut sign: i64 = 1;
    while i < chars.len() {
        let ch = chars[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }
        if ch == '+' {
            sign = 1;
            i += 1;
            continue;
        }
        if ch == '-' {
            sign = -1;
            i += 1;
            continue;
        }
        if ch.is_ascii_digit() {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_ascii_digit() {
                j += 1;
            }
            let lit: String = chars[i..j].iter().collect();
            let v: i64 = lit.parse().ok()?;
            c0 = c0.saturating_add(sign.saturating_mul(v));
            i = j;
            continue;
        }
        if ch.is_alphanumeric() || ch == '_' || ch == '.' {
            let mut j = i + 1;
            while j < chars.len()
                && (chars[j].is_alphanumeric() || chars[j] == '_' || chars[j] == '.')
            {
                j += 1;
            }
            let raw: String = chars[i..j].iter().collect();
            let name = sanitize_name(&raw);
            *coeffs.entry(name.clone()).or_insert(0) =
                coeffs.get(&name).copied().unwrap_or(0).saturating_add(sign);
            i = j;
            continue;
        }
        return None;
    }
    Some(LinearExpr { coeffs, c0 })
}

#[derive(Debug, Clone)]
struct ParsedRelConstraint {
    sexp: smtkit::sexp::Sexp,
    vars: std::collections::BTreeSet<String>,
}

fn linear_expr_to_smt_sexp(e: &LinearExpr) -> smtkit::sexp::Sexp {
    use smtkit::smt2::t;
    let mut terms: Vec<smtkit::sexp::Sexp> = Vec::new();
    if e.c0 != 0 {
        terms.push(t::int_lit(e.c0));
    }
    for (v, c) in e.coeffs.iter() {
        if *c == 0 {
            continue;
        }
        let sym = t::sym(v.clone());
        if *c == 1 {
            terms.push(sym);
        } else if *c == -1 {
            terms.push(t::app("-", vec![sym]));
        } else {
            terms.push(t::app("*", vec![t::int_lit(*c), sym]));
        }
    }
    if terms.is_empty() {
        t::int_lit(0)
    } else if terms.len() == 1 {
        terms[0].clone()
    } else {
        t::add(terms)
    }
}

fn parse_rel_constraint_int(s: &str) -> Option<ParsedRelConstraint> {
    let s = s.trim();
    let ops = ["<=", "≤", ">=", "≥", "<", ">", "="];
    let (op, idx) = ops.iter().find_map(|op| s.find(op).map(|i| (*op, i)))?;
    let (lhs, rhs0) = s.split_at(idx);
    let rhs = rhs0.get(op.len()..)?;
    let lhs_e = parse_linear_expr_int(lhs.trim())?;
    let rhs_e = parse_linear_expr_int(rhs.trim())?;
    use smtkit::smt2::t;
    let a = linear_expr_to_smt_sexp(&lhs_e);
    let b = linear_expr_to_smt_sexp(&rhs_e);
    let sexp = match op {
        "<=" | "≤" => t::le(a, b),
        ">=" | "≥" => t::ge(a, b),
        "<" => t::lt(a, b),
        ">" => t::app(">", vec![a, b]),
        "=" => t::eq(a, b),
        _ => return None,
    };
    let mut vars: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    vars.extend(lhs_e.coeffs.keys().cloned());
    vars.extend(rhs_e.coeffs.keys().cloned());
    Some(ParsedRelConstraint { sexp, vars })
}

/// Entailment check on a `pp_dump`-shaped JSON payload:
/// UNSAT(hyps ∧ ¬target) => `Some(true)`
/// SAT(hyps ∧ ¬target)   => `Some(false)`
/// UNKNOWN / not-parsable => `None`
pub fn entails_from_pp_dump(pp_dump: &Value, timeout_ms: u64, seed: u64) -> Result<Option<bool>, String> {
    use smtkit::smt2::t;

    let goal = pp_dump
        .get("goals")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .ok_or_else(|| "pp_dump missing goals[0]".to_string())?;

    let pretty = goal.get("pretty").and_then(|v| v.as_str()).unwrap_or("");
    let target = pretty
        .lines()
        .find_map(|ln| ln.trim_start().strip_prefix("⊢").map(|r| r.trim().to_string()))
        .unwrap_or_default();
    if target.is_empty() {
        return Ok(None);
    }

    let mut var_kinds: std::collections::BTreeMap<String, VarKind> = std::collections::BTreeMap::new();
    if let Some(hyps) = goal.get("hyps").and_then(|v| v.as_array()) {
        for h in hyps {
            if let Some(txt) = h.get("text").and_then(|v| v.as_str()) {
                if let Some((name, kind)) = extract_decl_kind(txt) {
                    var_kinds.insert(name, kind);
                }
            }
        }
    }

    let target_rel = match parse_rel_constraint_int(&target) {
        Some(r) => r,
        None => return Ok(None),
    };

    let mut hyp_rels: Vec<ParsedRelConstraint> = Vec::new();
    if let Some(hyps) = goal.get("hyps").and_then(|v| v.as_array()) {
        for h in hyps {
            if let Some(txt) = h.get("text").and_then(|v| v.as_str()) {
                let rhs = txt.split_once(':').map(|(_, r)| r.trim()).unwrap_or("");
                if rhs.is_empty() {
                    continue;
                }
                if let Some(r) = parse_rel_constraint_int(rhs) {
                    hyp_rels.push(r);
                }
            }
        }
    }

    for m in target_rel
        .vars
        .iter()
        .chain(hyp_rels.iter().flat_map(|r| r.vars.iter()))
    {
        if !var_kinds.contains_key(m) {
            return Ok(None);
        }
    }

    let (mut sess, _used) = match smtkit::session::spawn_auto() {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    sess.set_logic("QF_LIA").map_err(|e| e.to_string())?;
    sess.set_print_success(false).map_err(|e| e.to_string())?;
    sess.set_produce_models(false).map_err(|e| e.to_string())?;
    sess.set_timeout_ms(timeout_ms).map_err(|e| e.to_string())?;
    sess.set_random_seed(seed).map_err(|e| e.to_string())?;

    for (name, kind) in var_kinds.iter() {
        sess.declare_const(name, &smtkit::smt2::Sort::Int.to_smt2())
            .map_err(|e| e.to_string())?;
        if *kind == VarKind::Nat {
            sess.assert_sexp(&t::ge(t::sym(name.clone()), t::int_lit(0)))
                .map_err(|e| e.to_string())?;
        }
    }
    for r in hyp_rels {
        sess.assert_sexp(&r.sexp).map_err(|e| e.to_string())?;
    }
    sess.assert_sexp(&t::not(target_rel.sexp))
        .map_err(|e| e.to_string())?;
    let st = sess.check_sat().map_err(|e| e.to_string())?;
    match st {
        smtkit::session::Status::Unsat => Ok(Some(true)),
        smtkit::session::Status::Sat => Ok(Some(false)),
        smtkit::session::Status::Unknown => Ok(None),
    }
}

