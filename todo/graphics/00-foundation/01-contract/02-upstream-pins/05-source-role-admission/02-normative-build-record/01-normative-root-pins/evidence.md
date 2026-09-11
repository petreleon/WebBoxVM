# F02.5.2.1 evidence

Revision: `0e54eba1`
Validation: live immutable-source refresh; `make test`; source-file limits; roadmap and diff checks
Result: PASS
Artifacts: four external-cache payloads verified by SHA-256/bytes; `normative_roots.py`; nine focused tests
Profile: planning-only immutable source roots; no guest API, browser, CTS execution, conformance, certification, or performance claim

Task ID and date: F02.5.2.1, 2026-09-11
Tested commit and dirty diff hash: `0e54eba1`; code worktree was clean before this receipt/status update.
Upstream manifest revision: OpenGL-Registry `1cdd228e34966dd6b95bd203e9f84faba0f371a1` and
Vulkan-Docs `f84d432d5b8912362f96f581f29bbc4f3c8c7843`; `v1.4.362^{}` resolved to the latter.
Guest image and build hashes: not applicable; no guest or browser workload ran.
Browser, OS, adapter and driver: not applicable.
Exact command(s), working directory and tool versions:

- `/Users/petreleon/code/WebBoxVM`: `python3 normative_roots.py --cache-root /private/tmp/webboxvm-f0252-normative-roots-final --timeout 60` (Python 3.14.6; Poppler `pdftotext` 26.02.0).
- `/Users/petreleon/code/WebBoxVM`: `git ls-remote --tags https://github.com/KhronosGroup/Vulkan-Docs.git refs/tags/v1.4.362 refs/tags/v1.4.362^{}`.
- `/Users/petreleon/code/WebBoxVM`: `make test`, `cargo test -p emulator --test source_file_limits --quiet`,
  `python3 scripts/check_graphics_roadmap.py`, and `git diff --check`.

Expected result and minimum nonzero case count: all four immutable payloads must match their recorded
URL, commit, bytes, SHA-256, licence and attribution; all nine focused catalog/notice cases must pass.
Actual passed/failed/skipped counts and exit codes: final fresh cache fetched and validated 4/4 sources;
a second use of that cache fails before a network request. `make test` exit 0 (roadmap 10+4+2, runner 5,
source roles 9+3, roots 6, notices 3, emulator 1,130, Node 338 pass/0 fail); source-file limits 6/6;
no focused-test skips.
Negative/reference checks and observed output: incomplete catalogs, a mutable or foreign URL, changed
digest/bytes, absent licence, altered attribution, false local producer, registry/prose relabeling, any
preexisting cache file or target, missing source notice, and every positive API-support claim raise
`RootError`, `RoleError`, or `NoticeError`.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: external cache
`/private/tmp/webboxvm-f0252-normative-roots-final`; re-run the listed refresher with a new empty cache root to obtain the four content-addressed
paths. The records embed their SHA-256 and byte counts; payloads are intentionally not committed.
Software fallback detection and actual execution route: Python standard-library HTTPS fetch plus streamed
SHA-256 verification; no GPU or software rendering route ran.
Performance conditions and frozen protocol version, when applicable: not applicable.
First failing subcheck or blocker, when applicable: the first notice check looked for GLES terms on file
page 3; the verified PDF places them on file page 2. The locator and test were corrected, then the final
fresh-cache refresh passed. An initial sandbox DNS failure was environmental and the authorized live run succeeded.
Decision and limits of the evidence: OpenGL 4.6 and GLES 3.2 PDFs are distinct normative roots with
their conditional-reproduction notices at file pages 3 and 2 respectively. `vkspec.adoc` is only a pinned Vulkan prose root, not
a complete generated-docs closure; `vk.xml` is registry metadata, not normative prose or CTS. This
proves source identity only and makes no Khronos selector, API support, conformance, certification,
profile-support, or performance claim.
Commit/push verification: implementation is local commits `c03cdb69` and `0e54eba1`; no remote push or CI claim.
Next ready task: F02.5.2.2 (bounded local Vulkan definition) and, independently, F02.5.3.
