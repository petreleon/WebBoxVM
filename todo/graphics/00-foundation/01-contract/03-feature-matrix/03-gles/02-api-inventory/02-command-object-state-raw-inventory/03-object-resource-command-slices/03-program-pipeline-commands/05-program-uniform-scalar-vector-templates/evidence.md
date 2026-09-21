# F03.3.2.2.3.3.5 evidence

Revision: 8b75eb6952733ad73ca76f622e34f8b62a542fd1
Validation: local focused and repository gates; no remote CI run was listed
Result: PASS
Artifacts: `gles_program_uniform_scalar_vector_template_inventory.json`
(`a5eef3c396838262859d03776fc83d21d6d140ea0589a8ef53dc1c346b0303ef`)
Profile: GLES 3.2 raw template forms only; promotion disabled

Task ID and date: F03.3.2.2.3.3.5, 2026-09-21.

Tested code commit: `8b75eb6952733ad73ca76f622e34f8b62a542fd1`, parent
`8ed8cc3746bde971768cd121d9c25f5cb3339263`; patch SHA-256:
`54a10227b84afe4a6a8dd3a0e770817fb9f4f39c052cd40aed76add458f0ec20`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The raw set has 24 expanded ProgramUniform scalar/vector forms in grammar order
from p.128, §7.6.1. Its raw-entry hash is
`74f1ef8deb545c70f00d6e9f224208c0f78cbf80d67340ab2f7428699fa6da1d`.
The validator seals the four p.128 forms, admits all four scalar/vector forms,
and binds grammar orders 6/7/8/9 with expansion counts 8/8/4/4. The two
`ProgramUniformMatrix` forms on p.129 remain excluded for child `.6`.

Reproduce from repository root:

```sh
make graphics-gles-program-uniform-scalar-vector-template-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/03-program-pipeline-commands/05-program-uniform-scalar-vector-templates/gles_program_uniform_scalar_vector_template_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 6/6; the CLI passed `24 bounded GLES
ProgramUniform scalar/vector forms; source-only`. Full `make test` passed,
including 1,127 Rust unit tests (3 ignored) and 338 web tests. Source limits
passed 6/6; diff and roadmap checks passed.

Negative checks reject omitted, reordered, rerouted, or resealed forms; an
adjacent p.129 `ProgramUniformMatrix` intrusion; grammar order/expansion
drift; stale domain order/route; fabricated semantic fields; stale, duplicate,
nonfinite, or symlinked artifacts; unavailable ESSL substitutes; and
cache/private-loader substitution. The artifact is self-hashed and raw-only.

Decision and limits: this records only expanded spelling, formal template, and
source location. It creates no program state, uniform value, type, precision,
storage, layout, linking, ESSL, shader, program behavior, guest/browser,
support, conformance, certification, or performance claim. It has no Matrix or
CTS execution.

Commit/push verification: `8b75eb69` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list
--commit 8b75eb69 --limit 10` returned no runs.

Next ready task: F03.3.2.2.3.3.6.
