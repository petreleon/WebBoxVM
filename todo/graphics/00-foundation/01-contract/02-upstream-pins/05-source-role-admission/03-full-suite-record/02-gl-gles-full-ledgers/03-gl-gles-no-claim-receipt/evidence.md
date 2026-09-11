# F02.5.3.2.3 evidence

Revision: `632a5dfa`
Validation: focused hostile receipt test, replay of independently refreshed immutable inputs,
full test suite, source-file limits, roadmap and diff checks
Result: PASS
Artifacts: one local receipt over the immutable GL root and complete GLES root closure
Profile: GL/GLES full-suite observations only; no CTS execution or qualification result

Task ID and date: F02.5.3.2.3, 2026-09-11.
Tested commit and dirty diff hash: `632a5dfa`; the code worktree was clean before this receipt update.
Upstream source revision: `067e8832315e79817ede1c4863804e440f5d1c80` for both Khronos roots.
Guest image and build hashes: not applicable; no guest application or CTS binary ran.
Browser, OS, adapter and driver: not applicable.

## Commands and observations

From `/Users/petreleon/code/WebBoxVM`:

    make graphics-gl-gles-ledger-receipt-test
    # 6 passed, 0 failed, 0 skipped

    python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/03-full-suite-record/02-gl-gles-full-ledgers/03-gl-gles-no-claim-receipt/gl_gles_receipt.py \
      --gl-cache-root /private/tmp/webboxvm-f025321-gl-final.P4HZdk \
      --gles-cache-root /private/tmp/webboxvm-f025322-gles-final.H2Brvg
    # GL 19,714 cases; GLES five lists and 13 configurations; receipt SHA-256 778c4e...069e95

    make test
    # graphics 10+4+2+5+9+3+6+3+3+3+4+5+5+6+6; Rust 1,130; Node 338: all pass

    cargo test -p emulator --test source_file_limits --quiet
    # 6 passed

    git diff --check
    python3 scripts/check_graphics_roadmap.py
    # PASS: 384 documents, 229 tasks, 49 PASS-complete, 81 superseded
    # Ready: F02.5.3.3.1; Blocked: none

Expected result: the combined record must preserve the separate Khronos roots and exact local
ledger identities `a88cd6...5a7517` (GL) and `3d4d8b...4b3655` (GLES), while its own authority,
producer, and claims remain WebBoxVM/false. Actual result: the content-addressed local receipt is
`778c4eeb479fa4a6169385ef0a8e75ed54bf128b2ee0d23e29ee1afdff069e95`; it reports zero CTS
executions, 19,714 GL cases, five GLES lists, 13 configurations, and the unclassified glesext boundary.

Negative/reference checks and observed output: every top-level claim, `cts_executions`, local or
source authority, root revision, ledger hash, omitted/reordered GLES member, configuration, glesext
semantics, and unknown receipt field is rejected. The receipt also rejects repository-root caches,
cache roots symlinked into the repository, and symlinked source components before any source read.

Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: sources remain outside
Git in the two absolute cache paths above, refreshed independently by F02.5.3.2.1 and F02.5.3.2.2.
Re-run the listed command against those retained external caches or equivalent fresh caches. No image
exists. The command performs no fetch and no CTS execution.

Software fallback detection and actual execution route: strict replay from external regular files,
content hashes and exact parsers, then canonical JSON hashing; no renderer, guest, or software graphics
route ran. Performance conditions and frozen protocol version: not applicable.

First failing subcheck or blocker: the first draft accepted a cache root inside the repository. An
independent audit caught that provenance-boundary gap; the final code uses the F02 external-cache
contract and rejects root or source symlinks. No raw source identity mismatch occurred.

Decision and limits of the evidence: Khronos authority belongs only to the two raw roots and GLES
members. This receipt is a WebBoxVM engineering map, not a Khronos selector, conformance result,
certification, profile-support, API-support, or performance claim. Commit/push verification:
`632a5dfa` committed locally; no remote push or CI claim. Next ready task: F02.5.3.3.1 Vulkan
ledger taxonomy.
