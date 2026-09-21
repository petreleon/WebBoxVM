# F03.3.2.2.3.3.4 evidence

Revision: 7cec745eea0cd34b68ae6c0eff3423909b7a531e
Validation: local focused and repository gates; no remote CI run was listed
Result: PASS
Artifacts: `gles_uniform_matrix_template_inventory.json`
(`99fce5aeaac1cdc69245fa6cbcd6de4de912ac4958796d7a8a3bc498419eedd0`)
Profile: GLES 3.2 raw template forms only; promotion disabled

Task ID and date: F03.3.2.2.3.3.4, 2026-09-21.

Tested code commit: `7cec745eea0cd34b68ae6c0eff3423909b7a531e`, parent
`4fa2c08a63205fcb1cba8b6d1a001e7d0682e9c4`; patch SHA-256:
`3fc835806862916f99f79f6460089ce164075d091c10ffb980fc2b7505a648d0`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The raw set has nine expanded matrix forms in grammar order from p.126, §7.6.1.
Its raw-entry hash is
`2b99ca996491adb34703eb52ec4ba06c7043343a251006e257949a197486d867`.
The validator seals all four p.126 formal templates, admits only the two
`UniformMatrix` forms, and binds their grammar orders 4/5. The two same-page
scalar/vector `Uniform` templates remain excluded for child `.3`.

Reproduce from repository root:

```sh
make graphics-gles-uniform-matrix-template-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/03-program-pipeline-commands/04-uniform-matrix-templates/gles_uniform_matrix_template_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 6/6; the CLI passed `9 bounded GLES
Uniform matrix forms; source-only`. Full `make test` passed, including 1,127
Rust unit tests and 338 web tests. Source limits passed 6/6; diff and roadmap
checks passed.

Negative checks reject omitted, reordered, rerouted, or resealed forms; a
same-page scalar/vector intrusion; grammar order/expansion drift; stale domain
order/route; fabricated semantic fields; stale, duplicate, nonfinite, or
symlinked artifacts; unavailable ESSL substitutes; and cache/private-loader
substitution. The artifact is self-hashed and raw-only.

Decision and limits: this records only expanded spelling, formal template, and
source location. It creates no matrix value, type, transpose, storage, layout,
precision, ESSL, shader, program, guest/browser, support, conformance,
certification, or performance claim. It has no Matrix or CTS execution.

Commit/push verification: `7cec745e` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list --commit
7cec745e --limit 10` returned no runs.

Next ready task: F03.3.2.2.3.3.5.
