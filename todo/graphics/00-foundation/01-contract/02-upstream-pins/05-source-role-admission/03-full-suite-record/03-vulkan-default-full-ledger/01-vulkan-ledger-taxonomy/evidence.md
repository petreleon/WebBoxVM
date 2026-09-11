# F02.5.3.3.1 evidence

Revision: `03211806`
Validation: focused hostile ledger/taxonomy test, deterministic local record rebuild,
full test suite, source-file limits, roadmap and diff checks
Result: PASS
Artifacts: no downloaded payload; exact 98-member VCTS metadata observation and taxonomy receipt
Profile: Vulkan `vk-default.txt` full-suite root observation only; no CTS execution or qualification result

Task ID and date: F02.5.3.3.1, 2026-09-11.
Tested commit and dirty diff hash: `03211806`; the code worktree was clean before this receipt update.
Upstream release proof: `vulkan-cts-1.4.6.2`, tag object
`42c723aa10d2652590f02741827aef43b0421d23`, peeled commit
`f6a29701220f34dd1407513bfe80d74ca7b392ce`.
Guest image and build hashes: not applicable; no guest application or CTS binary ran.
Browser, OS, adapter and driver: not applicable.

## Commands and observations

From `/Users/petreleon/code/WebBoxVM`:

    make graphics-vulkan-ledger-taxonomy-test
    # 4 passed, 0 failed, 0 skipped

    python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/03-full-suite-record/03-vulkan-default-full-ledger/01-vulkan-ledger-taxonomy/vulkan_ledger_taxonomy.py
    # 98 members, 434,669,348 bytes; local record SHA-256 cea452...0ed6f3

    make test
    # graphics 10+4+2+5+9+3+6+3+3+3+4+5+5+6+6+4; Rust 1,130; Node 338: all pass

    cargo test -p emulator --test source_file_limits --quiet
    # 6 passed

    git diff --check
    python3 scripts/check_graphics_roadmap.py
    # PASS: 385 documents, 229 tasks, 50 PASS-complete, 81 superseded
    # Ready: F02.5.3.3.2; Blocked: none

Expected result: the F02.5.3 `vulkan-cts-default` source root must bridge to the distinct V2
suite identifier without conflating either identifier or selector scope. Its fixed root SHA-256 is
`b689703bdc65a04764db3b9a8f6fe872b3fe94d0df68d78f6da6e5a06cfa9ed4`; V2 identity, ledger and
taxonomy digests are `30b272...bd5218`, `608d52...e917f0`, and `752a3a...cb2f7`. Actual result:
the local record is `cea45295dab76b0adeed650f884cb14a12ff9e169bade31200c10649e50ed6f3` with
categories core=0, WSI=1, video=1, extension=4, unknown=92.

Negative/reference checks and observed output: missing, duplicate, reordered, mixed-revision,
changed-hash, re-sealed, and recursive-tail ledgers fail. A re-sealed taxonomy fails. Serialized
records with a positive claim, CTS execution, changed order, changed category, core-manifest state,
or a boolean substituted for an integer fail under type-aware comparison.

Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: the input metadata is
the reviewed V2 identity, closure ledger and taxonomy in the repository; run the listed command to
rebuild the record. This leaf deliberately does not download or read the 434 MiB suite payload.
Fourteen upstream members exceed 8 MiB (maximum 61,932,251 bytes); the local transform limit does
not filter them. No image exists.

Software fallback detection and actual execution route: deterministic JSON and metadata validation
only; no network, cache write, renderer, guest, CTS executable, or software graphics route ran.
Performance conditions and frozen protocol version: not applicable.

First failing subcheck or blocker: an audit found a time-of-check/time-of-use gap between validation
and later ledger reads. A changed second snapshot could have entered the local record. The final
adapter binds the raw snapshot and taxonomy view to the already fixed ledger digest; the hostile
race test now fails closed.

Decision and limits of the evidence: `vk-default.txt` remains a Khronos unfiltered default selector
broader than a Vulkan-1.4-core-only selector. The source root alone retains its Khronos selector
claim; this WebBoxVM record has all claims false and `cts_executions: 0`. It does not admit, certify,
or demonstrate Vulkan 1.4 core support. Commit/push verification: `03211806` committed locally; no
remote push or CI claim. Next ready task: F02.5.3.3.2 streaming cache replay.
