# SMT integration (optional oracle)

`proofpatch` can optionally use an external SMT solver via `smtkit` as a **heuristic signal** for linear integer arithmetic (LIA) entailment checks.

## Posture

- **Purpose**: rank/prune candidates in `tree-search-nearest` using cheap entailment checks.
- **Soundness**: Lean verification is the only “real” check; SMT is advisory.

## Repro artifacts

When enabled, `proofpatch` can record enough evidence to debug and reproduce solver behavior:

- solver probe (capability matrix; best-effort)
- dumped `.smt2` scripts (when `--smt-dump` is set)
- bounded inline previews (for chat/log inspection)

## Proof objects and UNSAT cores

Some solvers support `(get-proof)` and/or UNSAT cores (typically behind `:produce-proofs` / `:produce-unsat-cores`).

`proofpatch` can capture proof objects for **debugging/provenance**. This is not proof checking.

## `smt-repro`

Re-run a goal dump outside of `tree-search-nearest`:

```bash
proofpatch smt repro --input-json run.json --emit-smt2 repro.smt2 --emit-proof repro.sexp
```

`--input-json` can be either:
- a raw `pp_dump` JSON object, or
- a full `tree-search-nearest` JSON output (it will read `goal_dump.pp_dump`).

## `smt-probe`

Probe solver availability and capabilities:

```bash
proofpatch smt probe
```

## MCP

Two equivalent ways to probe solver capabilities:

- **Minimal toolset** (`PROOFPATCH_MCP_TOOLSET=minimal`): call the `proofpatch` tool with `action: "smt_probe"`.
- **Full toolset** (`PROOFPATCH_MCP_TOOLSET=full`): call the `proofpatch_smt_probe` tool.

Two equivalent ways to run repro:

- **Minimal toolset** (`PROOFPATCH_MCP_TOOLSET=minimal`): call the `proofpatch` tool with `action: "smt_repro"` (aliases like `smt.repro` also work).
- **Full toolset** (`PROOFPATCH_MCP_TOOLSET=full`): call the `proofpatch_smt_repro` tool.

For the **full toolset**, short aliases also exist:

- `smt_probe`
- `smt_repro`

