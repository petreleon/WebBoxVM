# F02.5.2.3 evidence

Revision: 91d18e32
Validation: targeted admission test, fresh live source refresh and rebuild, make test, source-file limits, roadmap check
Result: PASS
Artifacts: catalog SHA-256 c89707a9ede388aefab1a21ff132328ba9fb56e6ef4fefad57a9c3ad6bef527e; output 7,506 bytes
Profile: source-consumer admission for local Vulkan XML facts; not a CTS execution or guest support result

## Observed admission

- Fresh external cache: /private/tmp/webboxvm-f02523-admission-live.ZigTr6.
- Source IDs: opengl-46-core-spec, gles-32-spec, vulkan-14-spec, and vulkan-registry.
- Local output: vulkan-14-core-definition,
  SHA-256 29b9b64f67eca6c8ddf3edd40bddcb0843c059403dd757ec3479c2939dfd7556,
  7,506 bytes.
- Receipt authority and producer are WebBoxVM. It reports zero CTS executions and false values for
  selector, API support, conformance, certification, profile support, and performance.

## Commands and checks

From /Users/petreleon/code/WebBoxVM:

    make graphics-source-admission-test
    # 4 passed

    python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/02-normative-build-record/03-admission-receipt/admission_receipt.py \
      --cache-root /private/tmp/webboxvm-f02523-admission-live.ZigTr6 --timeout 30
    # fresh four-source refresh, byte-verified rebuild, catalog SHA-256 c89707a9...6bef527e

    make test
    # 4 admission tests, 1,130 Rust tests, 338 Node tests, and all existing graphics tests: PASS

    cargo test -p emulator --test source_file_limits --quiet
    # 6 passed

    git diff --check
    python3 scripts/check_graphics_roadmap.py
    # PASS: 374 documents, 223 tasks, 44 PASS-complete, 81 superseded

Synthetic negative checks reject a tampered cached source, staged builder, or output;
source substitution or omission; each positive transform claim; a forged receipt; and
a nonempty cache before any network request. The live run stages the verified builder
and checks all artifact files through the shared streamed verifier.

## Decision and limits

The combined catalog does not merge with a full-suite-root catalog. The four entries
remain immutable Khronos source roots; the local output remains a WebBoxVM transform.
No full CTS root was admitted here, no CTS case was executed, and no compatibility,
conformance, certification, profile, or performance statement follows.

First failing subcheck: the initial sandboxed fetch of opengl-46-core-spec failed DNS
resolution; the approved fresh network run above passed. This was an environment access
failure, not a source or rebuild mismatch.
Commit/push verification: 91d18e32 committed locally; no remote push performed.
Next ready task: F02.5.3.1 canonical full-suite roots.
