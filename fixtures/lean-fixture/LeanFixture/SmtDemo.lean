namespace LeanFixture

-- A tiny file that intentionally contains `sorry` so we can E2E-test:
-- - tree-search nearest-first patching
-- - SMT precheck (LIA) influencing candidate choice
--
-- Keep it fast to elaborate.

-- This goal is intentionally LIA-trivial so SMT entailment can succeed:
-- hypotheses contain the target exactly.
theorem smt_int_trivial (x y : Int) (h : x ≤ y) : x ≤ y := by
  sorry

theorem smt_nat_monotone (n m : Nat) (h : n ≤ m) : n + 1 ≤ m + 1 := by
  sorry

theorem smt_int_monotone (x y : Int) (h : x ≤ y) : x + 1 ≤ y + 1 := by
  sorry

end LeanFixture

