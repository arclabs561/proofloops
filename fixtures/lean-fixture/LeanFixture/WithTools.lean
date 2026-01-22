import ProofpatchTools

namespace LeanFixture

-- Ensure proofpatch can extend LEAN_PATH to import the helper tools package.
theorem proofpatch_tools_smoke : True := by
  pp_dump
  trivial

end LeanFixture

