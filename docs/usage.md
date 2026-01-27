# proofpatch usage

`proofpatch` is designed to run against any Lean 4 repo by pointing at a repo root and a target file/decl/region.

## CLI conventions

- **Repo root**: pass an absolute path with `--repo /abs/path/to/lean-repo`.
- **Outputs**: commands print JSON to stdout; many commands also support writing artifacts under `<repo_root>/.generated/…`.
- **Early exits**: many commands return quickly when work is unnecessary (e.g. “no `sorry` found”).

## Common commands

### Verify + scan for `sorry`

```bash
proofpatch triage-file --repo /abs/path/to/lean-repo --file Some/File.lean
```

### Extract a bounded context pack

```bash
proofpatch context-pack --repo /abs/path/to/lean-repo --file Some/File.lean --decl some_theorem
```

### Patch a lemma (in-memory) and verify

```bash
proofpatch patch --repo /abs/path/to/lean-repo --file Some/File.lean --lemma some_theorem --replacement-file /tmp/replacement.lean
```

## Focus controls

When a file has multiple `sorry`s, you can pin the search to one declaration:

```bash
proofpatch tree-search-nearest --repo /abs/path/to/lean-repo --file Some/File.lean --focus-decl MyNamespace.my_lemma
```

More strict variants:

- `--focus-decl-hard`: avoid drifting to other decls.
- `--focus-decl-strict`: fail fast if the decl does not match any `sorry` location.

## Output stability

Many commands include a stable `result_kind` string (e.g. `early_no_sorries`, `search`, `solved`) so downstream tooling can branch without brittle text matching.

