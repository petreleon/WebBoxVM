# F06.3.1 evidence

Revision: `6c69d1b12ed80ec36d40988c201712cd1de63896` baseline; scoped worktree change
Validation: six hermetic generator tests, fixture write/check, source-limit suite, `make test`, roadmap, diff
Result: PASS
Artifacts: committed fixture input, chunk, and metadata hashes listed below
Profile: deterministic maintenance fixture only; no protocol artifact or graphics API behavior is generated

Task ID and date: F06.3.1, 2026-09-09.

The generator requires an ordered JSON record stream and the exact SHA-256 of a caller-supplied
source manifest. It emits `chunk-NNNN.md` names, a fixed provenance header, and canonical metadata.
Writes build a staging directory then switch it into place; check mode compares the complete file set
and bytes, so stale metadata, a stale header, or a changed ordering is rejected. Each candidate is
line-counted before output; an individual record that cannot fit a 180-line chunk fails closed.

The checked fixture binds to F02.1 manifest revision
`8f81ece8dc895698f2715493c927b3f5ad10f24a9f20f1b48d82405a4a9a288e`.
Its input SHA-256 is `45a1056255833eebf7a174c8a842d4e0d23e8aaa2b4698af687301ec922f7457`;
`chunk-0001.md` is `9eb18a787a5a3e877317a43dd4839ac41d10d44e7598773ab625722a1a03e78b`;
and `metadata.json` is `641f57ff68a2740d68911d34a500b113bfab8899ede250ea60be8e70e9045526`.

Commands, working directory, and actual result:

```sh
python3 scripts/test_graphics_chunker.py
python3 scripts/graphics_chunker.py --write --spec todo/graphics/00-foundation/02-reproducibility/03-file-layout/03-generation-and-checker/01-chunk-generator/fixture/input.json --source-manifest todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory/manifest.toml --output todo/graphics/00-foundation/02-reproducibility/03-file-layout/03-generation-and-checker/01-chunk-generator/fixture/expected
python3 scripts/graphics_chunker.py --check --spec todo/graphics/00-foundation/02-reproducibility/03-file-layout/03-generation-and-checker/01-chunk-generator/fixture/input.json --source-manifest todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory/manifest.toml --output todo/graphics/00-foundation/02-reproducibility/03-file-layout/03-generation-and-checker/01-chunk-generator/fixture/expected
cargo test -p emulator --test source_file_limits --quiet
make test
python3 scripts/check_graphics_roadmap.py
git diff --check
```

The focused suite passed 6/6: repeat write/check byte identity, unordered records, a manifest
mismatch, an over-180-line record, stale metadata, and a stale chunk header. Fixture generation
wrote one 13-line chunk, and check mode passed with the recorded metadata hash. Supplying a wrong
source manifest failed nonzero with `source manifest revision does not match chunk specification`.
The source-limit suite passed 6/6. The final `make test` exit was zero: 1,151 Rust tests passed,
none failed, three were ignored, and 337 Node tests passed with no failures/cancellations/skips/todos.
The roadmap checker passed 147 documents, 98 tasks, and 5 complete; `git diff --check` exited zero.

The first full-suite attempt stopped at the roadmap checker because concurrent F02.2.1 work had
briefly marked a missing receipt complete. That exact structural failure was repaired before this
final rerun; it is not counted as a passing result. No browser adapter, guest application, native
reference, fallback, performance, Mesa, OpenGL/GLES, or Vulkan claim follows from this fixture.

All new maintained files are at most 180 lines: generator 180, test 85, input 20, and outputs 13.
Commit/push verification: pending a scoped review and commit. Next ready tasks: F02.2.2, F06.3.2.
