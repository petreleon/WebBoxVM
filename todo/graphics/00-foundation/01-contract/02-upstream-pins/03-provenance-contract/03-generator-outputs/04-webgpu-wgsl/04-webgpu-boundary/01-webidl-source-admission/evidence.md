# F02.3.3.4.4.1 evidence

Revision: `b3a3e76b2c71dcab1bdbbe06c9ca022e31b0f331` planning baseline plus the local WebIDL-admission diff
Validation: source audit, F02.1 structural 6/6, composite-layout 6/6, F02.2 contract 15/15, live fetch/reuse 17/17, final local gates below
Result: PASS
Artifacts: `webgpu-idl` metadata only; disposable 17-file, 18 MiB external cache removed after audit
Profile: immutable WebIDL binding/interop input accounting only; no semantic, browser, guest, renderer, compatibility, or performance behavior

Task ID and date: F02.3.3.4.4.1, 2026-09-09 Europe/Bucharest.

## Reviewed input

The new `webgpu-idl` entry is the only record in its new source family. It pins
`https://raw.githubusercontent.com/gpuweb/gpuweb/e95743d3940e0ff3c267ab55ced9ae6120c7d416/webgpu.idl`,
whose observed byte count is 38,618 and SHA-256 is
`bd35b2fc04f12f7ec22a9c1ae2826060778f980d8ce45dc055b8a5a8c644266a`. The file header says it is
upstream-generated and carries the W3C Software License signal. The reviewed role is `future
WebGPU WebIDL binding/interop generator input; no semantic or runtime implementation`.

The distinct `webgpu` entry remains `host API semantic reference; no generated code`. The source
audit records that Dawn's optional Node interop build consumes this exact WebIDL alongside additional
IDLs through its Go generator; that proves narrow binding/interop input suitability, not a complete
source, API implementation, guest interface, or graphics path.

## Inventory and fetch result

The new source is isolated in `inputs/part-0002.toml`; `part-0001.toml` remains byte-identical. The
schema-v2 raw inventory lock SHA-256 is
`08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6`.
The F02.1 structural suite passed 6/6, including the new hostile second-component lock case; F02.2's
contract suite passed 15/15 and verifies a stale second component fails before cache use.

The first F02.2 live command fetched all 17 pinned inputs into
`/private/tmp/webboxvm-f02-webidl-live.5QD1Hi`; the second reused and rehashed all 17 offline. A
separate `shasum -a 256` and `wc -c` check matched the WebIDL's declared digest and 38,618 bytes.
The cache was then removed. No upstream payload is tracked in Git.

## Commands and limits

From `/Users/petreleon/code/WebBoxVM`, the focused commands were
`validate_manifest.py --self-test` (6/6), `inventory_layout_test.py` (6/6), and
`source_fetch_test.py` (15/15), followed by the two `source_fetch.py` commands recorded in the
live-inventory receipt. The first new multi-component fixture failed because its fixture IDs were
numbered independently per part, producing duplicates; after indexing by the original interleaved
position, the focused suite passed. This was fixture-only and did not accept a malformed inventory.

The final local source-limit suite passed 6/6. `make test` passed 1,151 Rust tests with 0 failed and
3 ignored plus 337 Node tests with 0 failed; macOS `xcrun` cache-path warnings were environmental
warnings only. The roadmap checker and whitespace check pass after this receipt. No remote CI or
push is claimed by this receipt. The next task, F02.3.3.4.4.2, must renew every lock-bound
provenance and reproducibility consumer before any current generator-record claim can be made.
