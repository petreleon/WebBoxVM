# F03.3.2.2.3.2 evidence

Revision: c83c95f0500fc0ecd95a5cd6308e7057a0b3847c
Validation: local focused and repository gates; no remote CI run was created
Result: PASS
Artifacts: `gles_buffer_command_raw_inventory.json`
(`7f68a5caeb69d64d0fbf503c2deb5f0390091b8673aac591c844905fa34119c1`)
Profile: GLES 3.2 raw declarations only; promotion disabled

Task ID and date: F03.3.2.2.3.2, 2026-09-21.

Tested code commit: `c83c95f0500fc0ecd95a5cd6308e7057a0b3847c`, parent
`0910cdb62aef53c279f059d7854e8c9d39776759`; patch SHA-256:
`b194d561176ea0ca4b1ca05fdc02e49382ec049bea0fcb5dfdeef07460cbbb2f`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The raw set has 15 Chapter 6 formal declarations in PDF source order across pp.
68–80, sections 6, 6.1, 6.1.1, 6.2, 6.3, 6.3.1, 6.5, and 6.6. Its raw-entry
hash is `a7dfa0193404298ca1b6ff23079dca8f3f917283da079565de8b05db239dd4af`.
The validator re-extracts the entire formal declaration window, seals its exact
page/declaration sequence, and separates the shared p.69 §6/§6.1 boundary.

Reproduce from repository root:

```sh
make graphics-gles-buffer-command-inventory-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/02-buffer-commands/gles_buffer_command_raw_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 7/7; the CLI passed `15 bounded GLES
buffer declarations; source-only`. Full `make test` passed, including 1,127 Rust
unit tests and 338 web tests. Source limits passed 6/6; diff and roadmap checks
passed.

Regression-test supplement: `53f92c1d193f8d93608e2aea78b7ca67eecdd5de`,
parent `74fda400ecf27884e6acf2309df1a546e2df8916`; patch SHA-256
`eef0d24b4f291dcf3b785e896de632c703321c0c0622ea42290bd0f7d77c9588`.
It makes the omitted-valid-list case independent of the preceding bogus-name
case. The focused test remained 7/7; the full local suite again passed 1,127
Rust tests (3 ignored), 338 web tests, source limits 6/6, diff, and roadmap.

Negative checks reject altered/omitted/reordered literals even when resealed, a
real cross-family `GenQueries` declaration, both p.69 section relabels, domain
anchor drift, cache or private-loader substitution, stale/duplicate/symlinked
artifacts, and promoted fields. The artifact is self-hashed and raw-only.

Decision and limits: this creates no API support, buffer behavior, binding,
mapping, lifetime, data-store, visibility, guest/browser, conformance,
certification, or performance claim. It has no Matrix or CTS execution.

Commit/push verification: `c83c95f0` and `53f92c1d` were pushed to
`origin/codex/graphics-f01-baseline`; each remote SHA matched. `gh run list
--commit` for each revision returned no runs.

Next ready task: F03.3.2.2.3.3.
