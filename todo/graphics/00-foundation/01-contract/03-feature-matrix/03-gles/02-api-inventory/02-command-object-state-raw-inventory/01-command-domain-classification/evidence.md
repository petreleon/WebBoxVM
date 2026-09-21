# F03.3.2.2.1 evidence

Revision: e50861976fb011e78a3a384a4ba73f8f8bb1c409
Validation: local focused and repository gates; no remote CI run was created
Result: PASS
Artifacts: classification SHA-256 `17c73d25c7b4eaa125bda373bd4603df6bdb7f61f3c8c07f09c2eb65038facab`
Profile: GLES 3.2 source-routing only; raw-only and promotion disabled

Task ID and date: F03.3.2.2.1, 2026-09-21.

Tested commit and dirty diff hash: the staged tree tested before the code commit
became `e5086197`; parent `0702f339`; resulting patch SHA-256
`1b18f370c02274821f18c0674c0b566c55e46d3adb821ec12d11131fbc91df90`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages and 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`.
Guest image/build, browser/adapter/driver: not applicable to source classification.

Reproduction from repository root:

```sh
make graphics-gles-command-domain-classification-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/01-command-domain-classification/gles_command_domain_classification.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: 44 closed families, split into six self-hashed chunks; the CLI
passed with the `source-routing-only` fence. Main tests passed 4/4 and independent
cross-check tests passed 4/4; `make test` passed; source limits passed 6/6; diff and
roadmap checks passed.

Negative/reference checks: tests reject a removed, re-scoped, rerouted, duplicate,
overlapping, wildcard, or catch-all family before rendering; empty/prefix/wrong-
section anchors; malformed cache/source identity; short/duplicate index pages; and a
rehash of a substituted index receipt. The pp. 571–601 index has 13 fixed witnesses
and is explicitly omission-crosscheck-only. Tables 21.43–21.57 remain visibly routed
to the separate unreviewed limit/format inventory.

Raw captures: no external capture was retained; the committed catalog, chunks and
separate index receipt recreate the checks from the sealed cache. No native/browser
run, software fallback, or performance protocol applies.

Decision and limits: this map does not extract GLES facts and proves no API behavior,
ownership, implementation/support, conformance, certification, guest/browser
execution, or performance. It has zero Matrix rows and CTS runs.

Commit/push verification: `e5086197` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list --commit
e5086197 --limit 10` returned no runs.

Next ready task: F03.3.2.2.2.
