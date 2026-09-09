# F06.1 evidence

Revision: `6e6afc4caf947c765a9214d9eae87f7c17bb2182` plus this focused worktree change
Validation: focused browser modules, full browser suite, Wasm package build, source/roadmap checks
Result: PASS
Artifacts: source-only refactor; exact commands and counts below
Profile: architecture boundary only; no GL/GLES/Vulkan profile or performance result

Task ID and date: F06.1, 2026-09-09.

Tested commit and dirty diff hash: baseline branch commit `6e6afc4` before this F06.1 change;
the tested worktree contained only the files named below. Upstream manifest revision: pending F02.

Boundary map: [boundaries](boundaries.md) assigns protocol, shared-runtime, shader,
browser-platform, guest, and fixture owners; WBG3 and bounded VirGL remain distinct.

Changed browser boundary: `webgpu-3d.js` is now the stable public facade;
`webgpu-3d-dispatch.js` routes bounded VirGL packets; `webgpu-3d-legacy.js` owns WBG3
pipeline/device lifecycle. All changed/new maintained files are at most 125 lines.

Commands, working directory, and expected result:

```sh
node --test web/js/gpu-display-3d.test.mjs web/js/webgpu-3d-boundaries.test.mjs
node --test web/js/*.test.mjs
node scripts/stamp_web_asset_version.mjs --check
make web-pkg
cargo test -p emulator --test source_file_limits --quiet
make test
python3 scripts/check_graphics_roadmap.py
git diff --check
```

Actual result: focused facade/display tests passed 10/10; the top-level Node command passed 213/213;
the recursive `make test` browser lane passed 337/337. Asset stamping passed. `make web-pkg` built
fresh serial and threaded Wasm packages successfully (the threaded compiler emitted its existing
unstable-atomics warning). The source-file limit passed 1/1; `make test` passed 1,146 Rust cases
with 0 failures and 3 ignored. The final checker passed 136 documents, 92 tasks, 2 complete and
reported `Ready: F02.1, F06.2`. The focused facade test proves the three public module boundaries
load; display tests retain WBG3 negative acknowledgments, async completion, reset,
device-generation rebuild, and ownership behavior. The full browser suite covers bounded VirGL
routes independently. No real stock-Mesa guest, browser adapter, or native GPU comparison is implied.

Negative/reference checks: malformed WBG3 input stays negatively acknowledged; browser test doubles
exercise errors and device loss. There is no native-reference rendering claim. Software fallback:
the existing display policy is unchanged; this work neither enables nor disguises a fallback.

Performance conditions: no benchmark changed and P01 is incomplete. First failing subcheck/blocker:
none for F06.1 after the recorded checks; later compatibility/performance gates stay open.

Decision and limits: the split makes ownership reviewable without changing the bounded protocol
surface. It does not establish general VirGL, Mesa, Venus, OpenGL/GLES, Vulkan, or near-native work.

Commit/push verification: F06.1 completion commit
`464b34e3d6b121a01c16305a937a995f4ff7071b` was pushed to
`origin/codex/graphics-f01-baseline`; `git ls-remote` and the tracking ref resolved to that SHA.
No remote CI result is claimed locally.
Next ready task: F02.1 continues independently; F06.2 requires this leaf's completion.
