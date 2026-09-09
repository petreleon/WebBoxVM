# F02.3.3.4.1.1.2 evidence

Revision: f6ce5e788fc3cfafa1015707e4b5b69226327d01
Validation: F02.1 5/5 + F02.2 14/14 + transport 7/7 + limits + roadmap + full suite
Result: PASS
Artifacts: F02.1/F02.2 shared-loader adoption; SHA-256s below.
Profile: offline provenance-layout only; no fetch, cache payload, guest, browser, or graphics-runtime behavior

Task ID and date: F02.3.3.4.1.1.2, 2026-09-09 Europe/Bucharest.
Tested commit and dirty diff hash: f6ce5e788fc3cfafa1015707e4b5b69226327d01; clean tree before receipt.
Input identity: active schema-v1 inventory remains the existing 15 inputs; no source URL, revision,
digest, byte count, cache payload, or accepted identity changed.
Artifacts and SHA-256: `validate_manifest.py`
d3fc5e7faed4623a99385556898f6fc53b06cd0cd8a3dbccfe3d45133188867a;
`source_model.py` f78d1d8c33e5940e9181d41a1692882ccde15b121e661ceeb30902c0ac762e65;
`source_fetch_test.py` 7a796874f18868be52d617f382b801a1e53aee66967b26ebc73797509061c9f5.
Guest image and build hashes: not applicable; this task neither builds nor runs a guest.
Browser, OS, adapter and driver: not applicable; this task has no browser or GPU execution path.
Exact commands, working directory and tool versions: Python 3.14 stdlib tests from
`/Users/petreleon/code/WebBoxVM`: `PYTHONDONTWRITEBYTECODE=1 python3 .../inventory_layout_test.py`,
`.../validate_manifest.py --self-test`, `.../source_fetch_test.py`, and
`.../fixture_transport_test.py`; then `cargo test -p emulator --test source_file_limits --quiet`,
`python3 scripts/check_graphics_roadmap.py`, `git diff --check`, and `make test`.
Expected result and minimum nonzero case count: v1 retains 15 inputs; v2 root, component, and lock
fixtures load only through the shared loader, and any stale byte fails before an input/cache action.
Actual passed/failed/skipped counts and exit codes: layout 5/0/0; F02.1 5/0/0; F02.2 14/0/0;
transport 7/0/0; source limit 6/0/0; full Rust 1,151/0/3 ignored; Node 337/0/0; all exit 0.
Negative/reference checks and observed output: F02.1 mutates root, fragment, and lock bytes and
raises `inventory.lock`; F02.2 repeats those mutations and rejects before its cache root exists.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no external sample,
image, or payload was retained; rerun the listed hermetic Python commands with bytecode disabled.
Software fallback detection and actual execution route: Python stdlib/TOML only; no network route.
Performance conditions and frozen protocol version, when applicable: not applicable; no performance
or protocol behavior is measured.
First failing subcheck or blocker, when applicable: an initial aggregate command used obsolete
`02-fixture-transport`; correcting it to `02-hermetic-fixtures` gave 7/7. No implementation test failed.
Decision and limits of the evidence: accept only loader adoption and hostile-fixture coverage; the
cutover, new WGSL input, and all graphics-feature claims remain outside this child.
Commit/push verification: local feature commit f6ce5e788fc3cfafa1015707e4b5b69226327d01; not pushed
because explicit authorization for the remote destination has not been granted.
Next ready task: F02.3.3.4.1.1.5 after the three preparation receipts are recorded.
