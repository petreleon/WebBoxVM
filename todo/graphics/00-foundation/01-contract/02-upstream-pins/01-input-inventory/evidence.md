# F02.1 evidence

Revision: `ca6900dd4e8d121f402c2031cff03491a0ad6c19` baseline; this leaf is intentionally uncommitted
Validation: offline TOML schema/parser tests, full `make test`, roadmap check and source-file-limit check
Result: PASS
Artifacts: no vendor payload is tracked; 15 source bytes were read from public pinned URLs in `/private/tmp`
Profile: inventory only; no source was accepted into a cache and no graphics API behavior is claimed

## Inventory capture

Date: 2026-09-09. The manifest lists exactly 15 source families: Linux UAPI; Mesa VirGL and Venus;
virglrenderer; Venus protocol; GL/GLES; GLSL; ESSL; Vulkan; SPIR-V; WebGPU; WGSL; VK-GL-CTS;
WebGPU CTS; and Piglit. Each record names a 40-hex commit in its URL, source family, per-file
license/provenance, byte count, SHA-256, external cache path and generator/reference role.

The source payloads were fetched only to `/private/tmp/webboxvm-f02-inputs` while creating this
inventory, then hashed with `shasum -a 256` and counted with `wc -c`. They were not added to the
working tree. The authoritative source identity is the manifest's exact URL, commit, byte count
and SHA-256; F02.2 must independently re-fetch and verify those values before any cache entry can
be accepted.

The Linux entry deliberately records `BSD-3-Clause`: the pinned `virtio_gpu.h` payload itself says
“This header is BSD licensed” and contains the three-clause BSD text. That file-level notice wins
over any broad Linux-repository licensing shorthand.

The Piglit payload is fetched from the public `intel-lgci-fdo-gitlab-mirror/mesa.piglit` mirror at
the exact canonical Mesa Piglit commit. Its `provenance` field names the canonical
`gitlab.freedesktop.org/mesa/piglit` project; the mirror is used only because its commit-addressed
raw endpoint was available during this capture.

## Commands and results

```sh
python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory/validate_manifest.py
python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory/validate_manifest.py --self-test
cargo test -p emulator --test source_file_limits --quiet
python3 scripts/check_graphics_roadmap.py
git diff --check
```

Expected: the parser reports 15 entries, four hermetic tests pass, source-file limits pass, the
roadmap is structurally valid and the diff has no whitespace errors. The focused tests prove
manifest shape and complete family coverage independently of WebBoxVM's graphics implementation:
they accept the reviewed file and reject a missing family, a zero/placeholder SHA-256 and a mutable
`main` source URL. They perform no network access.

Actual: the parser printed `PASS: 15 immutable inputs` with manifest SHA-256
`8f81ece8dc895698f2715493c927b3f5ad10f24a9f20f1b48d82405a4a9a288e`; the hermetic suite ran
four tests and passed all four. The source-file-limit target passed 6/6. The roadmap checker passed
with 141 documents, 95 tasks and three completed tasks, reporting `Ready: F02.2.1, F06.2`. `git
diff --check` exited zero. `make test` passed 1,151 Rust tests (zero failed, three ignored) and 337
Node tests (zero failed/cancelled/skipped/todo). No network call occurs in either parser command.

No `make test`, browser run, conformance suite or live source-fetch verification is claimed for this
documentation/parser leaf; F02.2 owns live fetching and later leaves own those compatibility checks.

## Limits and handoff

The manifest is not a vendored checkout or proof that the selected APIs work. It pins the minimal
reviewed source files used to define later protocol, generator and reference-runner inputs. A new
input must be added as a fresh committed URL/revision/hash/byte/license/provenance record, then
the required-family test must be deliberately updated. The next dependent leaf is F02.2.

Commit/push verification: F02.1 completion commit
`67a793ec3bbbf959e4fccab44144d3bd2eb0e8b3` was pushed to
`origin/codex/graphics-f01-baseline`; `git ls-remote` and the tracking ref resolved to that SHA.
No remote CI result is claimed locally. The next ready leaf is F02.2.1.
