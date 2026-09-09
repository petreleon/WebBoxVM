# F06.3.3 evidence

Revision: `86148af2d1a1e56969886e94e206b5536dfe8b27` implementation commit
Validation: three CLI proof cases, generator and checker-fixture suites, source limits, `make test`, roadmap, diff
Result: PASS
Artifacts: `scripts/test_graphics_reproducibility.py`; temporary first/second output directories are deleted after each run
Profile: deterministic fixture and roadmap-maintenance proof only; no protocol artifact or graphics API behavior

Task ID and date: F06.3.3, 2026-09-09.

The proof invokes the tracked `graphics_chunker.py` CLI twice with the same ordered F06.3.1
fixture and the tracked F02.1 manifest. Fresh temporary output directories have identical file
sets and SHA-256 values: `chunk-0001.md`
`9eb18a787a5a3e877317a43dd4839ac41d10d44e7598773ab625722a1a03e78b`, and `metadata.json`
`641f57ff68a2740d68911d34a500b113bfab8899ede250ea60be8e70e9045526`. The input fixture is
`45a1056255833eebf7a174c8a842d4e0d23e8aaa2b4698af687301ec922f7457`; its manifest revision is
`8f81ece8dc895698f2715493c927b3f5ad10f24a9f20f1b48d82405a4a9a288e`. It also reads the actual
generated chunk and asserts its physical line count is at most 180.

The same proof first accepts an untouched generated directory through `--check`, then replaces
only its temporary metadata and observes nonzero failure containing `generated output is stale:
metadata.json`. It writes a temporary reordered copy of the input and observes nonzero `--check`
failure containing `records must be uniquely ordered by id`. The proof also invokes the F06.3.2
runner, which in turn uses the real roadmap-checker entrypoint for one valid and four invalid roots.

Commands, working directory, and actual result:

```sh
python3 scripts/test_graphics_reproducibility.py
python3 scripts/test_graphics_chunker.py
python3 scripts/test_check_graphics_roadmap.py
cargo test -p emulator --test source_file_limits --quiet
make test
python3 scripts/check_graphics_roadmap.py
git diff --check
```

Working directory: `/Users/petreleon/code/WebBoxVM`. The new proof passed 3/3, including two
negative CLI checks. The established chunker suite passed 6/6; the checker fixture suite passed
5/5; source limits passed 6/6. `make test`, the real roadmap checker, and `git diff --check`
exited zero. Temporary proof roots are removed automatically. No blocker occurred; concurrent
F02 worktree changes were present and intentionally not modified by this proof.

This evidence establishes stable bytes and stale/order rejection only. It does not exercise a
browser adapter, guest workload, native reference, fallback route, Mesa, OpenGL/GLES, Vulkan,
WebGPU, or performance behavior. It neither establishes graphics compatibility nor near-native
performance. Commit/push verification: implementation commit
`86148af2d1a1e56969886e94e206b5536dfe8b27` was pushed to
`origin/codex/graphics-f01-baseline`; `git ls-remote` resolved that branch to the same SHA. No remote
CI result is claimed locally. Next ready task: F02.3.1.
