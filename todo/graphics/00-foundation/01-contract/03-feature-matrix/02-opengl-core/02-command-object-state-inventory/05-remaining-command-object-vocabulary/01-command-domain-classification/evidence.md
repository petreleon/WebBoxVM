# F03.2.2.5.1 evidence

Revision: e50861976fb011e78a3a384a4ba73f8f8bb1c409
Validation: local focused and repository gates; no remote CI run was created
Result: PASS
Artifacts: `opengl_command_domain_classification.json` (`aa011d762f0765d373df5db364fa66da82dc87da075a6f257a5a3af423d77842`)
Profile: OpenGL 4.6 core source-routing only; no implementation or support claim

Task ID and date: F03.2.2.5.1, 2026-09-21.

Tested commit and dirty diff hash: the staged tree tested before the code commit
became `e5086197`; parent `0702f339`; resulting patch SHA-256
`1b18f370c02274821f18c0674c0b566c55e46d3adb821ec12d11131fbc91df90`.

Upstream manifest revision: `1cdd228e34966dd6b95bd203e9f84faba0f371a1`.
Guest image/build, browser/adapter/driver: not applicable to this source-only task.

Source and reproduction: the sealed 851-page OpenGL 4.6 PDF is 3,003,752 bytes,
SHA-256 `a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee`;
reproduce from the F02 cache with:

```sh
make graphics-opengl-command-domain-classification-test
python3 opengl_command_domain_classification.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Run the Python command from this task directory; the others run from repository root.
Python 3.14.6, GNU Make 3.81, and cargo 1.93.0 were used.

Expected/actual: 28 closed source families, six non-command routes, zero Matrix rows
and zero CTS executions. Focused tests passed 7/7; `make test` passed; source limits
passed 6/6; diff check passed; final roadmap validation reported 498 documents,
312 tasks, and 82 PASS-complete leaves.

Negative checks: test cases reject tampered hashes, missing/reordered/rerouted
families, catch-all scopes, wrong owners, and malformed index witnesses. The physical
PDF index pp. 801–851 supplies 12 exact `Create*` omission witnesses only; it creates
no declaration fact.

Raw captures: no external capture was retained. The committed JSON and fragment
receipts are self-hashed; the commands above recreate the result from the sealed cache.
Software fallback/performance protocol: not applicable; no execution was attempted.

Decision and limits: this is only a bounded source-family routing map. It proves no
OpenGL behavior, support, conformance, certification, guest/browser execution, or
performance. The map explicitly remains blocked by the incomplete Matrix.

Commit/push verification: `e5086197` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list --commit
e5086197 --limit 10` returned no runs.

Next ready task: F03.2.2.5.2.
