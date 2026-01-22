import Lake

open Lake DSL

package "lean-fixture" where
  -- keep it tiny; this is a CI/e2e fixture for proofpatch only

lean_lib LeanFixture where
  roots := #[`LeanFixture]

