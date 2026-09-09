# F02.3.3.4.1.1.5 evidence

Revision: ed78340d558f691ec4c525641e808654fa874616
Validation: focused 86/86 + direct validators + source limit 6/6 + roadmap + diff + full suite
Result: PASS
Artifacts: root sha256=5d57d9d4f9c1016a4fd50bb11b8fb859b2dcbbc8e3cdf7a9414e7b0a7ee52f86; component sha256=c78503afeebdb0fbdda321f190d8ef5268948c38490fc72d1be6deaba9ab6b4a; lock sha256=cb85958df7f8e2a9b6b749f6218621d6b24b1f6de5dc9f5be3468ab5fb7ddd5b
Profile: offline provenance/reproducibility migration only; no payload fetch, guest, API, browser, or graphics-runtime behavior

Task ID and date: F02.3.3.4.1.1.5, 2026-09-09 Europe/Bucharest.
Tested commit and dirty diff hash: ed78340d558f691ec4c525641e808654fa874616; clean tree before this receipt.
Upstream manifest revision: pre-cutover schema-v1 raw SHA-256
8f81ece8dc895698f2715493c927b3f5ad10f24a9f20f1b48d82405a4a9a288e; it is no longer an active
consumer identity.
Guest image and build hashes: not applicable; this task neither builds nor runs a guest.
Browser, OS, adapter and driver: not applicable; no browser or GPU execution path was exercised.
Exact command(s), working directory and tool versions: Python 3.14 stdlib tests from
`/Users/petreleon/code/WebBoxVM`: `inventory_layout_test.py`; `validate_manifest.py --self-test`;
`source_fetch_test.py`; `fixture_transport_test.py`; `provenance_record_test.py`;
`validate_abi_records_test.py`; `venus_record_test.py`; `validate_fixture_test.py`;
`validate_vulkan_spirv.py`; `webgpu_generator_boundary_test.py`; `scripts/test_graphics_chunker.py`;
and `scripts/test_graphics_reproducibility.py`, all with `PYTHONDONTWRITEBYTECODE=1`; then
`validate_abi_records.py`, `validate_fixture.py`, `make test`,
`cargo test -p emulator --test source_file_limits --quiet`,
`python3 scripts/check_graphics_roadmap.py`, and `git diff --check`.
Expected result and minimum nonzero case count: one schema-v2 root plus one sorted component preserves
all 15 raw input entries; every active provenance/F06 consumer binds the raw lock SHA-256; each focused
suite has a nonzero passing count; hostile component and stale-identity cases fail before acceptance.
Actual passed/failed/skipped counts and exit codes: layout 5/0/0; F02.1 5/0/0; F02.2 15/0/0;
transport 7/0/0; provenance 10/0/0; ABI 7/0/0; Venus 6/0/0; GL/GLES 6/0/0; Vulkan/SPIR-V 5/0/0;
WebGPU boundary 7/0/0; chunker 9/0/0; reproducibility 4/0/0 (86/0/0 total); direct ABI and GL
validators passed; source limit 6/0/0; `make test` Rust 1,151/0/3 ignored and Node 337/0/0; all exit 0.
Negative/reference checks and observed output: the layout suite rejects reordered, omitted, renamed, and
byte-mutated components; active F02.1/F02.2 reject schema-v1 and stale-lock inputs; provenance rejects
legacy-manifest and stale-sidecar identities; F06 rejects stale generated metadata and the wrong source
flag. The pre-cutover payload body hashes identically to `inputs/part-0001.toml`.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no external payload, image,
or runtime sample was retained. Re-run the listed hermetic commands from the repository root. F06 fixture
input sha256=def70dcf0d65f400740d0e15777ade809864535c6e16111330640bddfe4fa1e4; generated chunk
sha256=3570bfb1fa063f65875671df8d6f8f8f5ca850b8114f0f9394dcb511a576f47b; metadata
sha256=f4ce8ccaa00f261bbdc7e0bcafb9c6c743c21d91984e3022e48839b8b771840f.
Software fallback detection and actual execution route: Python stdlib/TOML/JSON validation and a local
deterministic chunk generator only; no network, parser fallback, renderer, or GPU path.
Performance conditions and frozen protocol version, when applicable: not applicable; no performance or
protocol behavior was measured.
First failing subcheck or blocker, when applicable: none; the first focused subcheck, layout, passed 5/5.
Decision and limits of the evidence: accept the atomic provenance/reproducibility cutover only. It adds no
source, preserves the 15 accepted identities, exposes no WebGPU/VirGL/Vulkan API, and demonstrates no
guest-visible rendering or near-native performance.
Commit/push verification: local feature commit ed78340d558f691ec4c525641e808654fa874616; not pushed
because explicit authorization for `https://github.com/petreleon/WebBoxVM.git` has not been granted.
Next ready task: F02.3.3.4.1.2 — Lock and fetch the WGSL grammar; F02.3.3.4.4 remains independently
blocked pending an explicit WebGPU generator source.
