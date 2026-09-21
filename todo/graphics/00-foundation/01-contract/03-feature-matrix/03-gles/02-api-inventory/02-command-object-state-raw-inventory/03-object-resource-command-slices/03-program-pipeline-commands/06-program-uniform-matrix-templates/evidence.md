# F03.3.2.2.3.3.6 evidence

Revision: 7ff2687c85a2ad594a01503253b9be7044482acf
Validation: local focused and repository gates; no remote CI run was listed
Result: PASS
Artifacts: `gles_program_uniform_matrix_template_inventory.json`
(`d0eaea31cb70bc8968c2d3cb33fd4daff43b91d70560f6edba66e05cb47a92a9`)
Profile: GLES 3.2 raw template forms only; promotion disabled

Task ID and date: F03.3.2.2.3.3.6, 2026-09-21.

Tested code commit: `7ff2687c85a2ad594a01503253b9be7044482acf`, parent
`c2783cc96bf07e7200eb0722ea0449ad06d6c633`; patch SHA-256:
`9b2471601a360ebc7a3d54f4f6401dcb1ea3cf93fb4f9cefa273a242a23fd557`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The raw set has nine expanded ProgramUniform matrix forms from physical p.129
(printed p.111), §7.6.1. Its raw-entry hash is
`b07ebc0ca4036421e29217afdc6335d2e224a7b0068537292e95a27fa59af416`.
The validator seals both p.129 forms and binds grammar orders 10/11 with
expansion counts 3/6. The section heading is witnessed on p.126, while the
source locator and raw facts remain p.129 only. The p.128 scalar/vector forms,
behavior prose, errors, and §7.6.2 content remain out.

Reproduce from repository root:

```sh
make graphics-gles-program-uniform-matrix-template-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/03-program-pipeline-commands/06-program-uniform-matrix-templates/gles_program_uniform_matrix_template_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 6/6; the CLI passed `9 bounded GLES
ProgramUniform matrix forms; source-only`. Full `make test` passed, including
1,127 Rust unit tests (3 ignored) and 338 web tests. Source limits passed 6/6;
diff and roadmap checks passed.

Negative checks reject omitted, reordered, rerouted, or resealed forms; p.128
scalar/vector intrusion; a wrong section witness; grammar order/count or
expansion drift; reordered rectangular forms; stale domain order/route;
fabricated semantic fields; stale, duplicate, nonfinite, or symlinked
artifacts; unavailable ESSL substitutes; and cache/private-loader substitution.
The artifact is self-hashed and raw-only.

Decision and limits: this records only expanded spelling, formal template, and
source location. It creates no program state, matrix value, transpose behavior,
type, precision, storage, layout, linking, ESSL, shader, program behavior,
guest/browser, support, conformance, certification, or performance claim. It
has no runtime Matrix or CTS execution.

Commit/push verification: `7ff2687c` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list
--commit 7ff2687c --limit 10` returned no runs.

Next ready task: F03.3.2.2.3.4.
