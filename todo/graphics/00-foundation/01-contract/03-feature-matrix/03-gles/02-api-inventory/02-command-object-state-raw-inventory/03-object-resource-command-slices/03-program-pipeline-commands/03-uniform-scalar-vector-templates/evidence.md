# F03.3.2.2.3.3.3 evidence

Revision: 2d397eec8fe8c5eb9f97bdf858ca7947321453e8
Validation: local focused and repository gates; no remote CI run was listed
Result: PASS
Artifacts: `gles_uniform_scalar_vector_template_inventory.json`
(`7cfcc022d63168601cad6f4111aadc0b02b01bbe8e611b108821cf311b0ee13e`)
Profile: GLES 3.2 raw template forms only; promotion disabled

Task ID and date: F03.3.2.2.3.3.3, 2026-09-21.

Tested code commit: `2d397eec8fe8c5eb9f97bdf858ca7947321453e8`, parent
`a01d2ed3be88e01d04c413264c8152f90aba76c8`; patch SHA-256:
`3af666e975d222fa0632e58db24440423220efb357ccebe4de732897736170a6`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The raw set has 24 expanded scalar/vector forms in grammar order from p.126,
§7.6.1. Its raw-entry hash is
`4ac7d8e5c53590de92be228a70bbf1b288da46f36c25ad0218d4d7885e6e130f`.
The validator seals all four p.126 formal templates, admits only the first two
(`Uniform{1234}{if ui}` and its `v` form), and binds their grammar orders 2/3.
The two same-page `UniformMatrix` templates remain excluded for child `.4`.

Reproduce from repository root:

```sh
make graphics-gles-uniform-scalar-vector-template-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/03-program-pipeline-commands/03-uniform-scalar-vector-templates/gles_uniform_scalar_vector_template_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 6/6; the CLI passed `24 bounded GLES
Uniform scalar/vector forms; source-only`. Full `make test` passed, including
1,127 Rust unit tests and 338 web tests. Source limits passed 6/6; diff and
roadmap checks passed.

Negative checks reject omitted, reordered, rerouted, or resealed forms; a
same-page `UniformMatrix` intrusion; grammar order/expansion drift; stale domain
order/route; fabricated semantic fields; stale, duplicate, nonfinite, or
symlinked artifacts; unavailable ESSL substitutes; and cache/private-loader
substitution. The artifact is self-hashed and raw-only.

Decision and limits: this records only expanded spelling, formal template, and
source location. It creates no uniform value, type, precision, storage, layout,
linking, ESSL, shader, program, guest/browser, support, conformance,
certification, or performance claim. It has no Matrix or CTS execution.

Commit/push verification: `2d397eec` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list --commit
2d397eec --limit 10` returned no runs.

Next ready task: F03.3.2.2.3.3.4.
