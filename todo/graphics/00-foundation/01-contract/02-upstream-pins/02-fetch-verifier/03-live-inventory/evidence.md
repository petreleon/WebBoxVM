# F02.2.3 evidence

Revision: `59d65bf56c88d548cea919d8e1eceab86a6e9f9a` implementation commit; scoped live-inventory run
Validation: historical 15/16-input runs plus current live 17-input fetch, offline 17-input re-hash, cache audit, focused suites, source limits, `make test`, roadmap, diff
Result: PASS
Artifacts: disposable external cache `/private/tmp/webboxvm-f02-live.9GeZap`; no payload is stored in Git
Profile: pinned-source availability and integrity only; no generator, guest, graphics API, browser, or performance claim

Task ID and date: F02.2.3, 2026-09-09.

The first command below contacted only the immutable HTTPS URLs in the maintained manifest and emitted
15 `PASS: <id> fetched <path>` lines. The second command ran without network elevation and emitted the
same 15 IDs as `reused`, so `fetch_to_cache` re-read and re-hashed the populated files without opening
transport. The final read-only audit independently compared every file's byte count and SHA-256 to the
manifest values shown below. No mismatch or unavailable source occurred.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/02-fetch-verifier/01-fetch-contract/source_fetch.py --cache-root /private/tmp/webboxvm-f02-live.9GeZap
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/02-fetch-verifier/01-fetch-contract/source_fetch.py --cache-root /private/tmp/webboxvm-f02-live.9GeZap
```

Each exact cache path is `/private/tmp/webboxvm-f02-live.9GeZap/webboxvm-graphics/f02/<id>/<sha>.source`.
The table records the observed fetch-to-reuse transition, actual byte count and digest, and maintained
license record for every manifest input.

| Input | State | Bytes | SHA-256 | License |
| --- | --- | ---: | --- | --- |
| linux-virtio-gpu-uapi | fetched -> reused | 11554 | 7c9e2f7d47fa0b1a2c737fc5a741f57c5cf25303dd5c68c2c9738e9bb761eee6 | BSD-3-Clause (pinned file's explicit BSD notice) |
| mesa-virgl-screen | fetched -> reused | 42906 | e558fa5550e572cffad113581b736696f33b3895e0516950181b8ff93eb38ff0 | MIT (file notice) |
| mesa-venus-device | fetched -> reused | 23220 | 68f06ca4ddc2d5a62bebaa469fc43dc5e66e640eec57d46f38172327d087a4c4 | MIT (SPDX file notice) |
| virglrenderer-protocol | fetched -> reused | 30358 | 094a6e2b210f1d3dd0f1ac254ed8820c1d889e57d2735c0c6194dbc94f3b1ac7 | MIT (file notice) |
| venus-protocol-registry | fetched -> reused | 20989 | d92839bc728fa9ad9a7decdc6b91df6fa1a0fb26cffae4009865f18a789e0535 | Apache-2.0 OR MIT (SPDX file notice) |
| opengl-gles-registry | fetched -> reused | 2774722 | b9ca2cfa5c676e901c20d34af3407f1687cde0f1336a5ff7a8974d04c7494ad3 | Apache-2.0 (SPDX file notice) |
| glsl-460-spec | fetched -> reused | 5520366 | b22b0c0967a1254c8a3487394d8a3bf72cd8a35a872a43c8a01e03bab3eeb468 | Khronos conditional spec reproduction license (PDF p. iv) |
| essl-320-spec | fetched -> reused | 4724914 | e9a04262013f6447844ed8724456bb63081cd61e249859a0abbd6ce05979be43 | Khronos conditional spec reproduction license (PDF p. iv) |
| vulkan-registry | fetched -> reused | 3309653 | cf31c965cf6e788697139601da0c7e02a75a9b6c7ac764e7641f5521ffd9da06 | Apache-2.0 OR MIT (SPDX file notice) |
| spirv-core-grammar | fetched -> reused | 612052 | db8581272b63d232268094a47b68d18a0464fc911e06004d57419924fe660ba4 | MIT (grammar notice) |
| webgpu-spec | fetched -> reused | 848425 | 85e732c1589c911ede74faccaefa439fb6222b96d86d352b9e44c7dc347d96a5 | W3C Software and Document License (repo LICENSE.md) |
| wgsl-spec | fetched -> reused | 924614 | d54430daf037051f2b7c46399441b9c01931f2374caf5ed8780686defacf44d4 | W3C Software and Document License (repo LICENSE.md) |
| vk-gl-cts-api-version | fetched -> reused | 36614 | 875ca8c0d65dd8a1024c8e504fe433305151f6afda9e95b6f58c967441a6fd07 | Apache-2.0 (file notice) |
| webgpu-cts-buffer-map | fetched -> reused | 17766 | e3d8dc9c4f5cd9cfc93a8bca688bf431c81ea1014d40d83b8b0a1139da604c19 | BSD-3-Clause (repo LICENSE.txt) |
| piglit-gl30-bindfragdata | fetched -> reused | 6307 | 11ef8899405be1dc4ebe4a6ebe351c838c9a1c581c8446fcd971f982c059e8da | MIT (file notice; canonical COPYING) |

## Grammar renewal

On 2026-09-09, F02.3.3.4.1.2 renewed this evidence against a new external cache,
`/private/tmp/webboxvm-f02-wgsl-live.E64MYf`. Its first `source_fetch.py` run fetched all 16
declared inputs; the second run reused and offline-rehashed all 16. An independent local audit read
each cache file and matched its declared byte count and SHA-256. No input was unavailable or accepted
with a mismatch. The previous 15 rows retain their recorded identities; the added result is:

| Input | State | Bytes | SHA-256 | License |
| --- | --- | ---: | --- | --- |
| wgsl-grammar-syntax | fetched -> reused | 10581 | 838b6fd1d01e4efd06e233200479d57667e8f8ba74783598e51f8f6195f762a1 | W3C Software and Document License (repo LICENSE.md; document) |

The grammar's file comment identifies a nonstandard BNF dialect, so this renewal records immutable
source integrity only; it does not establish a standard parser, compiler, WebGPU API, guest, or
browser feature. The disposable cache was removed after the offline audit; no payload is tracked.

## WebIDL admission renewal

On 2026-09-09, F02.3.3.4.4.1 used fresh external cache
`/private/tmp/webboxvm-f02-webidl-live.5QD1Hi`. The first `source_fetch.py` run fetched all 17
declared inputs; the second reused and offline-rehashed all 17. A 17-file, 18 MiB disposable cache
was independently inspected before removal. The new result is:

| Input | State | Bytes | SHA-256 | License |
| --- | --- | ---: | --- | --- |
| webgpu-idl | fetched -> reused | 38618 | bd35b2fc04f12f7ec22a9c1ae2826060778f980d8ce45dc055b8a5a8c644266a | W3C Software License (webgpu.idl file header) |

All prior 16 entries also fetched and rehashed under their unchanged identities. This checks pinned
source integrity only. It does not make the upstream-generated WebIDL a semantic specification,
browser implementation, guest API/device, renderer, compatibility result, or performance result.

Commands and final local results:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/02-fetch-verifier/01-fetch-contract/source_fetch_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/02-fetch-verifier/02-hermetic-fixtures/fixture_transport_test.py
cargo test -p emulator --test source_file_limits --quiet
make test
python3 scripts/check_graphics_roadmap.py
git diff --check
```

The inherited contract suite passed 13/13 and the hermetic transport suite passed 7/7. The source-limit
suite passed 6/6. `make test`, roadmap verification, and whitespace verification exited zero. The 18 MB
cache was an explicitly disposable temporary artifact and was removed after the audit; no payload remains
in the repository or is required for later work.
Commit/push verification: implementation commit `59d65bf56c88d548cea919d8e1eceab86a6e9f9a`
was pushed to `origin/codex/graphics-f01-baseline`; `git ls-remote` resolved that branch to the same
SHA. No remote CI result is claimed locally. Next ready task: F02.3.1.
