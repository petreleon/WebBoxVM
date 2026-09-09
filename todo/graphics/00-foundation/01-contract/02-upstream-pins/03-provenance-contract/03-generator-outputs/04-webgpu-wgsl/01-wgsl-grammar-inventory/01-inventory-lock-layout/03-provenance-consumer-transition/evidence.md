# F02.3.3.4.1.1.3 evidence

Revision: b957a798985a5587601adb3442ea7e5428d84d96
Validation: provenance 10/10 + ABI 7/7 + Venus 6/6 + GL 6/6 + Vulkan 5/5 + WebGPU 7/7
Result: PASS
Artifacts: versioned provenance consumers and fixtures; SHA-256s below.
Profile: offline provenance-layout only; no upstream fetch, guest, WebGPU runtime, or graphics feature

Task ID and date: F02.3.3.4.1.1.3, 2026-09-09 Europe/Bucharest.
Tested commit and dirty diff hash: b957a798985a5587601adb3442ea7e5428d84d96; clean tree before receipt.
Historical input identity at this receipt: active schema-v1 sidecars and generated artifact bytes
remained unchanged; v2 coverage was hermetic and the active 15 F02 inputs kept their identities.
Artifacts and SHA-256: `provenance_record.py`
56e7973ba2154fe8149cd807cead684ab0e39615a8de75037601a59a785d6cf3;
`provenance_record_test.py` b154d27ae852c8178da3166415f6166c28bb9f8648108ca56b01c1af1574dde5;
GL generator/test 0b95fb35be7bc627da6f1da4b4c8717814483e3ab08e606cd8e81c7555bec1c7 /
dfc7fbf5e7b0286d8aa00b87893a998254b86d894374289a62bb13e5555cb04d;
WebGPU boundary/test 6cae7f89f753e8b02228675b0c689fa87d6b320e8b68f3a121224bc930121e07 /
09f1e6f1c80e6fb03ea494c38420fc57c69dde23bb62e22fc9fe6928fec0ad02.
Guest image and build hashes: not applicable; this task neither builds nor runs a guest.
Browser, OS, adapter and driver: not applicable; no browser or GPU execution path was exercised.
Exact commands, working directory and tool versions: Python 3.14 stdlib suites from
`/Users/petreleon/code/WebBoxVM`: `PYTHONDONTWRITEBYTECODE=1 python3` each of
`provenance_record_test.py`, `validate_abi_records_test.py`, `venus_record_test.py`,
`validate_fixture_test.py`, `validate_vulkan_spirv.py`, and `webgpu_generator_boundary_test.py`;
then source limits, roadmap checker, diff check, and `make test` as recorded in the sibling receipt.
Expected result and minimum nonzero case count: v1 records preserve validation; v2 records require
`inventory_sha256`, reject legacy/stale/schema-mismatched/undeclared states, and never change artifacts.
Actual passed/failed/skipped counts and exit codes: provenance 10/0/0; ABI 7/0/0; Venus 6/0/0;
GL/GLES 6/0/0; Vulkan/SPIR-V 5/0/0; WebGPU boundary 7/0/0; limits 6/0/0; full Rust
1,151/0/3 ignored; Node 337/0/0; all exit 0.
Negative/reference checks and observed output: v2 rejects stale `inventory.lock`, a v1
`manifest_sha256`, schema mismatch, and undeclared inputs; GL v2 validation uses the lock identity.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no external payload,
image, or runtime sample was retained; rerun the six listed hermetic suites with bytecode disabled.
Software fallback detection and actual execution route: Python stdlib record/layout validation only.
Performance conditions and frozen protocol version, when applicable: not applicable; no performance
or protocol behavior is measured.
First failing subcheck or blocker, when applicable: none; the first focused subcheck, provenance,
passed 10/10. The existing WebGPU runtime-generator blocker remains unresolved by design.
Decision and limits of the evidence: accept only versioned offline provenance validation. It does not
alter checked-in sidecars or artifacts, add a source, expose WebGPU, or claim graphics acceleration.
Commit/push verification: local feature commit b957a798985a5587601adb3442ea7e5428d84d96; not pushed
because explicit authorization for the remote destination has not been granted.
Next ready task: F02.3.3.4.1.1.5 after the three preparation receipts are recorded.
