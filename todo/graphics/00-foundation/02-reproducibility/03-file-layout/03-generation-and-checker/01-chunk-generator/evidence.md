# F06.3.1 evidence

Revision: `6c69d1b12ed80ec36d40988c201712cd1de63896` baseline; scoped worktree change
Validation: six hermetic generator tests, fixture write/check, source-limit suite, `make test`, roadmap, diff
Result: PASS
Artifacts: historical schema-v1 fixture hashes plus the active schema-v2 renewal below
Profile: deterministic maintenance fixture only; no protocol artifact or graphics API behavior is generated

Task ID and date: F06.3.1, 2026-09-09.

At the cited F06.3.1 completion revision, the generator used an ordered JSON record stream and the
exact SHA-256 of a caller-supplied source manifest. It emits `chunk-NNNN.md` names, a fixed
provenance header, and canonical metadata. Writes build a staging directory then switch it into
place; check mode compares the complete file set and bytes, so stale metadata, a stale header, or a
changed ordering is rejected. Each candidate is line-counted before output; an individual record
that cannot fit a 180-line chunk fails closed.

At that historical revision, the checked fixture bound to F02.1 manifest revision
`8f81ece8dc895698f2715493c927b3f5ad10f24a9f20f1b48d82405a4a9a288e`.
Its input SHA-256 is `45a1056255833eebf7a174c8a842d4e0d23e8aaa2b4698af687301ec922f7457`;
`chunk-0001.md` is `9eb18a787a5a3e877317a43dd4839ac41d10d44e7598773ab625722a1a03e78b`;
and `metadata.json` is `641f57ff68a2740d68911d34a500b113bfab8899ede250ea60be8e70e9045526`.

Active schema-v2 fixture renewal: the checked input binds the canonical raw `inventory.lock`
SHA-256 `cb85958df7f8e2a9b6b749f6218621d6b24b1f6de5dc9f5be3468ab5fb7ddd5b`, not the root
manifest bytes. Its input SHA-256 is `def70dcf0d65f400740d0e15777ade809864535c6e16111330640bddfe4fa1e4`;
`chunk-0001.md` is `3570bfb1fa063f65875671df8d6f8f8f5ca850b8114f0f9394dcb511a576f47b`; and
`metadata.json` is `f4ce8ccaa00f261bbdc7e0bcafb9c6c743c21d91984e3022e48839b8b771840f`.

Historical commands, working directory, and actual result:

```sh
python3 scripts/test_graphics_chunker.py
python3 scripts/graphics_chunker.py --write --spec todo/graphics/00-foundation/02-reproducibility/03-file-layout/03-generation-and-checker/01-chunk-generator/fixture/input.json --source-manifest todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory/manifest.toml --output todo/graphics/00-foundation/02-reproducibility/03-file-layout/03-generation-and-checker/01-chunk-generator/fixture/expected
python3 scripts/graphics_chunker.py --check --spec todo/graphics/00-foundation/02-reproducibility/03-file-layout/03-generation-and-checker/01-chunk-generator/fixture/input.json --source-manifest todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory/manifest.toml --output todo/graphics/00-foundation/02-reproducibility/03-file-layout/03-generation-and-checker/01-chunk-generator/fixture/expected
cargo test -p emulator --test source_file_limits --quiet
make test
python3 scripts/check_graphics_roadmap.py
git diff --check
```

Current schema-v2 fixture regeneration and check:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 scripts/graphics_chunker.py --write --spec todo/graphics/00-foundation/02-reproducibility/03-file-layout/03-generation-and-checker/01-chunk-generator/fixture/input.json --inventory-lock todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory/inventory.lock --output todo/graphics/00-foundation/02-reproducibility/03-file-layout/03-generation-and-checker/01-chunk-generator/fixture/expected
PYTHONDONTWRITEBYTECODE=1 python3 scripts/graphics_chunker.py --check --spec todo/graphics/00-foundation/02-reproducibility/03-file-layout/03-generation-and-checker/01-chunk-generator/fixture/input.json --inventory-lock todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory/inventory.lock --output todo/graphics/00-foundation/02-reproducibility/03-file-layout/03-generation-and-checker/01-chunk-generator/fixture/expected
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

At that completion, all maintained files were at most 180 lines: generator 180, test 85, input 20,
and outputs 13. The active versioned generator remains bounded: generator 165, schema helper 83,
output helper 49, generator test 123, reproducibility test 170, input 20, and outputs 13 lines.
Commit/push verification: F06.3.1 completion commit
`646907482bde635fbe841786d8dcc858d0d8aa75` was pushed to
`origin/codex/graphics-f01-baseline`; `git ls-remote` and the tracking ref resolved to that SHA.
No remote CI result is claimed locally. Next ready tasks: F02.2.2, F06.3.2.
