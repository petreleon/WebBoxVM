# F02.4.4.1.5.2.3.5.2.3 receipt — synthetic syscall collector

Revision: `166023cd3b561ab8b75a4a608a6fddd9bafb7ef4`
Validation: 22 focused collector/normalizer/Docker-policy/hosted-contract tests; 55 related focused tests; `make test`; source limit; roadmap; hosted run 34464021075
Result: PASS
Artifacts: public [GitHub Actions run 34464021075](https://github.com/petreleon/WebBoxVM/actions/runs/34464021075); ephemeral container work root removed
Profile: three-byte synthetic fixture only; `observed-unadmitted`, not Docs lineage or closure evidence

Task ID and date: F02.4.4.1.5.2.3.5.2.3, 2026-09-10 Europe/Bucharest.

The focused command below passed 22/22, including positive normalization/binding and negative malformed, escaped,
duplicated, lifecycle-invalid, mutation, and noncanonical-wire cases:

```sh
cd todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/02-vulkan-docs-closure/03-bind-vulkan-docs/05-actual-closure-proof/02-proof-lineage-trace
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  lineage_synthetic_normalize_test.py lineage_synthetic_collect_test.py \
  lineage_synthetic_docker_test.py lineage_ptrace_fixture_test.py \
  lineage_github_synthetic_witness_test.py -v
```

The complete focused set passed 55/55. `make test` passed 1,130 Rust and 337 Node tests; the source-limit suite
passed 6/6; `git diff --check` and the roadmap checker passed with this receipt update.

The hosted C run compiled from five immutable source blobs and emitted exactly 16 lifecycle rows, two matching
`726177` snapshots, and the one terminal row `{"kind":"terminal","status":"observed-unadmitted"}`: 19 rows total.
Its collector-output SHA-256 was `5d89e3799e4bfefeac901c5a39b0d0d5b98d23b4d5d670605d7e12856f3f086a`.
The raw and output snapshots both identify three bytes with SHA-256
`d7439bee24773bcbfa2d0a97947ee36227b10d1022b1a55847e928965bb6bfde`.

The hosted receipt verifies the C wire only. `lineage_synthetic_collect.py` then normalizes and binds that bounded
wire in the local focused tests above; its `argv_sha256` is a fixed fixture sentinel, not an observed Docs argv hash.
The hosted helper deliberately does not claim to run or publish a Docs binder receipt.

Decision and limits: this completes the fixed synthetic collector mechanism. It does not authorize a Docs mount,
Docs build, Docs argv, source provenance, Docs derived producer mapping, or actual-closure manifest. The next
Docs-binding child remains dependent on authoritative metadata and is unchecked.
