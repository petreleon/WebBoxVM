# F02.3.3.4.1.1 evidence

Revision: ed78340d558f691ec4c525641e808654fa874616
Validation: five child receipts + focused 86/86 + source limit 6/6 + roadmap + diff + full suite
Result: PASS
Artifacts: schema-v2 root sha256=5d57d9d4f9c1016a4fd50bb11b8fb859b2dcbbc8e3cdf7a9414e7b0a7ee52f86; component sha256=c78503afeebdb0fbdda321f190d8ef5268948c38490fc72d1be6deaba9ab6b4a; lock sha256=cb85958df7f8e2a9b6b749f6218621d6b24b1f6de5dc9f5be3468ab5fb7ddd5b
Profile: offline inventory/provenance/reproducibility migration only; no upstream payload, guest, API, browser, or graphics-runtime behavior

Task ID and date: F02.3.3.4.1.1, 2026-09-09 Europe/Bucharest.
Tested commit and dirty diff hash: ed78340d558f691ec4c525641e808654fa874616; clean tree before receipts.
Upstream manifest revision: schema-v1 SHA-256
8f81ece8dc895698f2715493c927b3f5ad10f24a9f20f1b48d82405a4a9a288e before the cutover; the
canonical active provenance revision at this task's tested revision was the raw `inventory.lock`
SHA-256 above. A later grammar-lock task may renew that current identity.
Guest image and build hashes: not applicable; no guest was built or run.
Browser, OS, adapter and driver: not applicable; no browser or GPU execution path was exercised.
Exact command(s), working directory and tool versions: Python 3.14 stdlib focused suites recorded in
the five child receipts from `/Users/petreleon/code/WebBoxVM`; the atomic cutover reran layout,
F02.1/F02.2, provenance/ABI/Venus/GL/Vulkan/WebGPU-boundary, chunker, and reproducibility suites,
then `make test`, `cargo test -p emulator --test source_file_limits --quiet`,
`python3 scripts/check_graphics_roadmap.py`, and `git diff --check`.
Expected result and minimum nonzero case count: all 15 entries retain their exact bytes in a bounded
component; every active consumer shares one lock revision; lock/component and stale consumer states
fail closed; each focused suite has at least one passing case.
Actual passed/failed/skipped counts and exit codes: 86/0/0 focused tests; direct ABI/GL validators
passed; source limits 6/0/0; full Rust 1,151/0/3 ignored; Node 337/0/0; all exit 0.
Negative/reference checks and observed output: layout tests reject reordered, omitted, renamed, and
byte-mutated components; F02.1/F02.2 reject legacy/stale source identities; F02.3 rejects stale
sidecars; F06 rejects stale generated metadata. The old manifest body and current component have the
same SHA-256 c78503afeebdb0fbdda321f190d8ef5268948c38490fc72d1be6deaba9ab6b4a.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no external payload,
image, or runtime sample was retained. Rerun the listed offline commands from the repository root.
Software fallback detection and actual execution route: Python stdlib/TOML/JSON and deterministic
local generator validation only; no network, renderer, or GPU route.
Performance conditions and frozen protocol version, when applicable: not applicable; none measured.
First failing subcheck or blocker, when applicable: none; the first focused layout suite passed 5/5.
Decision and limits of the evidence: accept the composite-lock migration and its record/fixture renewal
only. It adds no source, changes none of the 15 accepted identities, exposes no graphics API, and does
not demonstrate guest-visible rendering or near-native performance.
Commit/push verification: local feature commit ed78340d558f691ec4c525641e808654fa874616; not pushed
because explicit authorization for `https://github.com/petreleon/WebBoxVM.git` has not been granted.
Next ready task: F02.3.3.4.1.2 — Lock and fetch the WGSL grammar.
