# F03.3.2.2.3.3.1 evidence

Revision: 9aaae658be55aa36bd7e939c46058508c6c260f8
Validation: local focused and repository gates; no remote CI run was created
Result: PASS
Artifacts: `gles_program_pipeline_literal_raw_inventory.json`
(`72777e4a0924127abdf50868433eb250904e22305cb5e71f661373b371855125`)
Profile: GLES 3.2 raw declarations only; promotion disabled

Task ID and date: F03.3.2.2.3.3.1, 2026-09-21.

Tested code commit: `9aaae658be55aa36bd7e939c46058508c6c260f8`, parent
`22b7e86a7d910a170ffbfdf3cb45da037332c0c8`; patch SHA-256:
`d7238abd5a19e041dea899066620c4ae01e2174310965b8bfee37e42183b7ad3`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The raw set has 29 literal Chapter 7 declarations in PDF source order across pp.
85–118, sections 7.1, 7.2, 7.3, 7.3.1, 7.4, and 7.5. Its raw-entry hash is
`85d5d780660b93c4a5a75a4183dcdbcd4cccf3b81bea0cd2865276b444c69adc`.
The validator re-extracts the complete formal-declaration window and fences the
shared p.88 §7.1/§7.2 boundary by the actual section marker.

Reproduce from repository root:

```sh
make graphics-gles-program-pipeline-literal-inventory-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/03-program-pipeline-commands/01-shader-program-pipeline-binaries/gles_program_pipeline_literal_raw_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 6/6; the CLI passed `29 bounded GLES
program/pipeline literals; source-only`. Full `make test` passed, including
1,127 Rust unit tests and 338 web tests. Source limits passed 6/6; diff and
roadmap checks passed.

Negative checks reject altered, omitted, reordered, or cross-family literals
even when resealed; both p.88 section relabels; stale domain order/route;
unavailable ESSL substitution; malformed nonfinite, stale, duplicate, or
symlinked artifacts; and cache/private-loader substitution. The artifact is
self-hashed and raw-only.

Decision and limits: this records only declaration spellings and source
locations. It creates no compilation, link, binary, precision, validation,
lifecycle, guest/browser, support, conformance, certification, or performance
claim. It has no Matrix or CTS execution.

Commit/push verification: `9aaae658` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list --commit
9aaae658 --limit 10` returned no runs.

Next ready task: F03.3.2.2.3.3.2.
