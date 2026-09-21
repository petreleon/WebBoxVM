# F03.3.2.2.3.3 evidence

Revision: 7ff2687c85a2ad594a01503253b9be7044482acf
Validation: six child receipts plus final local repository gates; no remote CI run was listed
Result: PASS
Artifacts: six self-hashed child raw inventories, 118 entries in total
Profile: GLES 3.2 formal program/shader/pipeline declarations only; promotion disabled

Task ID and date: F03.3.2.2.3.3, 2026-09-21.

Aggregate tested revision: `7ff2687c85a2ad594a01503253b9be7044482acf`.
The six verified child code revisions are `9aaae658`, `f1ec38a1`,
`2d397eec`, `7cec745e`, `8b75eb69`, and `7ff2687c`; each receipt
records its own patch hash and source artifact hash.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The aggregate is six disjoint raw-only inventories: 29 program/pipeline literals,
23 reflection/object queries, 24 Uniform scalar/vector forms, 9 Uniform matrix
forms, 24 ProgramUniform scalar/vector forms, and 9 ProgramUniform matrix forms.
Every child binds the same authority/cache/domain/grammar/ledger identities and
keeps only expanded spelling, formal declaration, physical page, section, and
source order.

Child receipts and artifacts:

- [literals](01-shader-program-pipeline-binaries/evidence.md) — 29 entries
- [queries](02-uniform-reflection-and-object-queries/evidence.md) — 23 entries
- [Uniform scalar/vector](03-uniform-scalar-vector-templates/evidence.md) — 24 entries
- [Uniform matrix](04-uniform-matrix-templates/evidence.md) — 9 entries
- [ProgramUniform scalar/vector](05-program-uniform-scalar-vector-templates/evidence.md) — 24 entries
- [ProgramUniform matrix](06-program-uniform-matrix-templates/evidence.md) — 9 entries

Reproduce from repository root:

```sh
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: the final full `make test` passed, including 1,127 Rust unit
tests (3 ignored) and 338 web tests. The new ProgramUniform children passed 6/6
focused tests; their direct CLIs passed 24 scalar/vector and 9 matrix forms.
Source limits passed 6/6; diff and roadmap checks passed after this receipt.

Negative/reference checks in the child receipts reject source-window or grammar
drift, cross-family forms, resealed omissions/reordering, semantic promotion,
unavailable substitutes, stale/duplicate/nonfinite/symlinked artifacts, and
cache/private-loader substitution.

Decision and limits: this aggregation does not add API behavior, shader semantics,
precision, linking, program state, uniform values, matrix/transpose behavior,
guest/browser execution, support, conformance, certification, or performance
claims. It has no runtime Matrix or CTS execution.

Commit/push verification: every listed feature revision was pushed to
`origin/codex/graphics-f01-baseline` with a matching remote SHA; each queried
`gh run list --commit` had no run. This receipt is committed separately.

Next ready task: F03.3.2.2.3.4.
