# F03.2.2.5.4.2 evidence

Revision: 56fbccfbb1047e13ed0fde5b7e54b127beee143b
Validation: local focused and repository gates; no remote CI run was created
Result: PASS
Artifacts: `opengl_draw_compute_submission.json` (`6bd39d2a53d60d314c7d9bc83c3e266cf0b1996db676a932d54a399fdbef75ee`) and five bound fragments
Profile: OpenGL 4.6 core draw/compute declaration inventory only; matrix-incomplete

Task ID and date: F03.2.2.5.4.2, 2026-09-21.

Tested commit and patch: parent `f1f25bd5d592c685e45e6e5f9063b7403585aa2d`;
the tested tree became `56fbccfb`; its patch SHA-256 is
`0e37755732eefc0c9755ec1c298861ae2d20a3b9691a8df7de9e0e8aedacac19`.

Source identity: sealed OpenGL 4.6 core PDF, 851 physical pages, SHA-256
`a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The inventory binds §10.4 (pages 386–398), §13.3.3 (page 472), and chapter 19
(pages 568–569). It preserves 28 formal source declarations: 26 actual GL commands
in five self-hashed fragments, plus `DrawArraysOneInstance` and
`DrawElementsOneInstance` as explicit non-GL source exclusions. Conditional rendering
is bound as a same-route but out-of-scope family, rather than silently omitted.

Reproduce from repository root:

```sh
make graphics-opengl-draw-compute-submission-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/02-command-object-state-inventory/05-remaining-command-object-vocabulary/04-non-object-command-slices/02-draw-compute-submission/opengl_draw_compute_submission.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 4/4; the CLI passed `26 source-only draw/compute
submission declarations; matrix-incomplete`; `make test` passed, including 1,127 Rust
unit tests and 338 web tests. Source limits passed 6/6; diff and roadmap checks passed.

Negative checks reject altered PDF anchors, classifier/grammar swaps, rehashed claim
promotion, duplicate JSON fields, modified fragments, symlinked dependencies/artifacts,
and preloaded dependency aliases. The root receipt binds the exact serialized fragment
bytes as well as their self-hashed contents.

Raw captures: no external capture was retained; the sealed cache, exact PDF locators,
root receipt, and five fragment receipts reproduce the bounded declaration result. No
native/browser run, software fallback, rendering capture, or performance protocol applies.

Decision and limits: this proves neither draw/dispatch execution nor rendering output.
It asserts no state semantics, API support, guest/browser behavior, conformance,
certification, or performance result.

Commit/push verification: `56fbccfb` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list --commit
56fbccfb --limit 10` returned no runs.

Next ready tasks: F03.2.2.5.4.3, F03.2.2.5.4.4, and F03.2.2.5.4.5.
