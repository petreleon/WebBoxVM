# F06.2 evidence

Revision: `ca6900dd4e8d121f402c2031cff03491a0ad6c19` (dirty workspace)
Validation: focused source-limit suite, `make test`, roadmap checker, diff check
Result: PASS
Artifacts: reproducible commands; temporary fixtures are deleted by the test
Profile: maintainability policy only; no GL/GLES/Vulkan or performance claim

## Scope and policy

Task ID and date: F06.2, 2026-09-09.

Tested commit and dirty diff hash: `ca6900dd4e8d121f402c2031cff03491a0ad6c19`,
`4e95e589602ea9ba03be8a8786cdeb3beba7c6addb17f5d3ef3b0dd96dd1524b` before
this receipt. Concurrent F02 work was present and preserved; this task did not
edit its paths.

The expanded scan covers `emulator`, `models`, `scripts`, `web`, `todo`, `guest`,
`research`, and `patches`, plus root source files. It adds C, headers, linker
scripts, nested Makefiles, and `.patch` to the recognized source forms. Only two
exact paths are exempt: `LICENSE.md` and
`patches/wasm-bindgen-memory64-threads.patch`. Their line counts, SHA-256 values,
and patch provenance are recorded in [the exemption ledger](exemptions.md).

`research/virgl-resource-residency.md` was the only oversized scanned maintained
file at 188 lines. It is now a 123-line linked entry point; the moved protocol,
cost, and validation material is in the 75-line
[companion](../../../../../../research/virgl-resource-residency-validation.md).

## Commands and results

Working directory: `/Users/petreleon/code/WebBoxVM`.

```sh
cargo test -p emulator --test source_file_limits --quiet
cargo test -p emulator --quiet
make test
python3 scripts/check_graphics_roadmap.py
git diff --check
```

Expected result: a 180-line file passes, a 181-line fixture reports its relative
path, only the two reviewed exemptions are allowed, and every command exits zero.

Actual result: the focused suite passed 6/6. Its hermetic fixture reported
`todo/rejected.md: 181`; C, H, LD, and nested Makefile fixtures each reported
181 lines; `guest/LICENSE.md` and `patches/other.patch` were rejected while the
two exact exempt paths were accepted. The complete Rust run passed 1,151, failed
0, and ignored 3. `make test` exited 0 and its Node suite passed 337, failed 0,
cancelled 0, skipped 0, and todo 0. The final roadmap checker passed 142 documents
and 95 tasks with valid links, dependencies, and limits. `git diff --check` exited 0.

The fixture is created in an OS temporary directory named
`webboxvm-source-limit-<pid>-<sequence>` and removed on test completion. No
hardware/browser, guest image, software fallback, or performance evidence applies.
There was no failing subcheck or blocker for this leaf.

Commit/push verification: F06.2 completion commit
`56cda63dcc98b2738a00054c2bf561f281b74450` was pushed to
`origin/codex/graphics-f01-baseline`; `git ls-remote` and the tracking ref resolved to that SHA.
No remote CI result is claimed locally.
Next ready task: F06.3, subject to its dependencies and the roadmap checker.
