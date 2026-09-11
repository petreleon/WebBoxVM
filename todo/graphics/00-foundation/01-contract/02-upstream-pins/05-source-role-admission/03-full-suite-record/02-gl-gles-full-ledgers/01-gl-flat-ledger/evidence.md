# F02.5.3.2.1 evidence

Revision: `b1f42173`
Validation: focused hostile ledger test, fresh immutable GL refresh, full test suite, source-file limits,
roadmap and diff checks
Result: PASS
Artifacts: 1,353,085-byte external cache root; 19,714-case sequence receipt
Profile: OpenGL 4.6 full-suite root observation only; no CTS execution or qualification result

Task ID and date: F02.5.3.2.1, 2026-09-11.
Tested commit and dirty diff hash: `b1f42173`; code worktree was clean before this receipt update.
Upstream manifest revision: `067e8832315e79817ede1c4863804e440f5d1c80`, rooted at
`external/openglcts/data/gl_cts/data/mustpass/gl/khronos_mustpass/4.6.1.x/gl46-main.txt`.
Guest image and build hashes: not applicable; no guest application or CTS binary ran.
Browser, OS, adapter and driver: not applicable.

## Commands and observations

From `/Users/petreleon/code/WebBoxVM`:

    make graphics-gl-flat-ledger-test
    # 5 passed, 0 failed, 0 skipped

    python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/03-full-suite-record/02-gl-gles-full-ledgers/01-gl-flat-ledger/gl_flat_ledger.py \
      --cache-root /private/tmp/webboxvm-f025321-gl-final.P4HZdk --timeout 30
    # fresh root receipt: 19,714 cases; raw and sequence SHA-256 e28bbbb...1d7b17

    make test
    # graphics 10+4+2+5+9+3+6+3+3+3+4+5+5; Rust 1,130; Node 338: all pass

    cargo test -p emulator --test source_file_limits --quiet
    # 6 passed

    git diff --check
    python3 scripts/check_graphics_roadmap.py
    # PASS: 382 documents, 229 tasks, 46 PASS-complete, 81 superseded
    # Ready: F02.5.3.2.2, F02.5.3.3.1; Blocked: none

Expected result and minimum nonzero case count: one exact immutable root must re-hash to
1,353,085 bytes and `e28bbbbfd0f6c8d711554a01aa45819bdc7ca963c9426e997dfd1daaeb1d7b17`,
then produce 19,714 unique ordered nonblank LF case lines. Actual result: all final commands exited
zero; no focused or root-ledger test was skipped. The local receipt has ledger SHA-256
`a88cd6b9fad2268f9ef355806cdfea1cac54994ff512b14f8292be89bd5a7517` and all claims false.

Negative/reference checks and observed output: blank, padded, NUL, duplicate, non-LF, reordered,
substituted, wrong-count, and wrong-digest sequences fail. A preexisting cache fails before network
access. The public ledger API no longer accepts caller-supplied root metadata, so authentic bytes
cannot be misattributed to a forged root record.

Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: the raw source is held
outside Git at `/private/tmp/webboxvm-f025321-gl-final.P4HZdk`; rerun the listed command with a new
empty absolute cache. Its source and sequence SHA-256 are the pinned value above; no image exists.
Software fallback detection and actual execution route: Python HTTPS fetch, content-addressed cache,
re-hash, and strict UTF-8/LF parser; no renderer, guest, or software graphics route ran.
Performance conditions and frozen protocol version, when applicable: not applicable.

First failing subcheck or blocker, when applicable: the initial freshness test surfaced a foreign
`FullSuiteError` instead of this leaf's `LedgerError`; the boundary now translates it. A later sandboxed
network refresh could not resolve the source host; the authorized fresh external-network run passed.
Neither was a source identity or sequence mismatch.

Decision and limits of the evidence: the Khronos raw root remains the full-suite selector. The local
WebBoxVM engineering map only commits its observed sequence identity; it reports `cts_executions: 0`
and no API support, conformance, certification, profile-support, or performance claim.
Commit/push verification: `b1f42173` committed locally; no remote push or CI claim.
Next ready task: F02.5.3.2.2 GLES full-closure ledger and F02.5.3.3.1 Vulkan ledger taxonomy.
