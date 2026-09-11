# F02.5.3.2.2 evidence

Revision: `8489fd27`
Validation: focused hostile ledger test, fresh immutable GLES refresh, full test suite,
source-file limits, roadmap and diff checks
Result: PASS
Artifacts: 4,284-byte upstream XML root; five root-referenced case lists; 13 ordered configurations
Profile: GLES full-suite root observation only; no CTS execution or qualification result

Task ID and date: F02.5.3.2.2, 2026-09-11.
Tested commit and dirty diff hash: `8489fd27`; the code worktree was clean before this receipt update.
Upstream manifest revision: `067e8832315e79817ede1c4863804e440f5d1c80`, rooted at
`external/openglcts/data/gl_cts/data/mustpass/gles/khronos_mustpass/main/mustpass.xml`.
Guest image and build hashes: not applicable; no guest application or CTS binary ran.
Browser, OS, adapter and driver: not applicable.

## Commands and observations

From `/Users/petreleon/code/WebBoxVM`:

    make graphics-gles-full-closure-ledger-test
    # 6 passed, 0 failed, 0 skipped

    python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/03-full-suite-record/02-gl-gles-full-ledgers/02-gles-full-closure-ledger/gles_full_ledger.py \
      --cache-root /private/tmp/webboxvm-f025322-gles-final.H2Brvg --timeout 30
    # fresh root and five-list receipt; ledger SHA-256 3d4d8b...4b3655

    make test
    # graphics 10+4+2+5+9+3+6+3+3+3+4+5+5+6; Rust 1,130; Node 338: all pass

    cargo test -p emulator --test source_file_limits --quiet
    # 6 passed

    git diff --check
    python3 scripts/check_graphics_roadmap.py
    # PASS: 383 documents, 229 tasks, 47 PASS-complete, 81 superseded
    # Ready: F02.5.3.2.3, F02.5.3.3.1; Blocked: none

Expected result and minimum nonzero case count: the immutable root must re-hash to 4,284 bytes
and `9f466f19a26120149bab06e70596c9fff57b61d767cfdbadc7ba13934ce2ea96`; its exact closure is
five lists with counts 473, 6,498, 4,101, 1,405, and 1,097. The 13 XML configurations preserve
order and repeated main-list configurations. Actual result: all final commands exited zero; no
focused or source-ledger test was skipped. The local receipt SHA-256 is
`3d4d8b10d3a46a51f0c30606430131dce8207253423e0a33c996573ffb4b3655` and all claims are false.

Negative/reference checks and observed output: DTD/entity declarations, malformed XML, omitted,
duplicated, reordered, substituted, or unpinned lists fail. The receipt validator regenerates the
boundary observation, so a tampered `glesext_boundary.semantics` field also fails. A nonempty cache
fails before network access.

Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: raw sources remain
outside Git at `/private/tmp/webboxvm-f025322-gles-final.H2Brvg`; rerun the listed command with a
new empty absolute cache. List SHA-256 values are `aed17d...d395f2`, `c79097...3ace72`,
`9fd8e4...d520e`, `426fa31...b6f249`, and `789b04...a6e0e6`; no image exists.
Software fallback detection and actual execution route: Python HTTPS fetch, content-addressed
cache, re-hash, strict XML parser, and strict UTF-8/LF list parser; no renderer, guest, or
software graphics route ran. Performance conditions and frozen protocol version: not applicable.

First failing subcheck or blocker: the initial draft receipt accepted a serialized `glesext`
classification without regenerating it from the exact source closure. The validator now rejects
that tampering. The isolated sandbox could not resolve the source host; the authorized fresh
external-network refresh passed. Neither was an upstream identity mismatch.

Decision and limits of the evidence: the raw Khronos root and five members remain immutable
upstream sources. `gles32-khr-glesext.txt` is retained because the root references it, but its
semantics are deliberately `not-classified-by-this-ledger`. The local WebBoxVM map reports
`cts_executions: 0` and no API-support, conformance, certification, profile-support, or performance
claim. Commit/push verification: `8489fd27` committed locally; no remote push or CI claim.
Next ready task: F02.5.3.2.3 GL/GLES no-claim receipt and F02.5.3.3.1 Vulkan ledger taxonomy.
