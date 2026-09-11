# F02.5.3.1 evidence

Revision: `16dd32f0`
Validation: exact-root tests, fresh immutable-source receipt, release-tag proof, full test suite,
source-file limits, roadmap and diff checks
Result: PASS
Artifacts: three unmodified Khronos selector roots and two commit-bound Apache-2.0 license payloads
Profile: source identity only; no CTS execution, guest API, compatibility, conformance, certification,
profile-support, or performance result

## Observed receipt

- Task/date: F02.5.3.1, 2026-09-11; implementation commit `16dd32f0`.
- Empty external cache: `/private/tmp/webboxvm-f02531-roots.bVOC5g`.
- Root IDs: `opengl-cts-gl46-main`, `gles-cts-main`, and `vulkan-cts-default`.
- OpenGL and GLES bind to `067e8832315e79817ede1c4863804e440f5d1c80` via annotated tag
  `opengl-cts-4.6.8.1` (`f7eefdfcae4a19fa69ad7df0c00da8c1a65723a3`).
- Vulkan binds to `f6a29701220f34dd1407513bfe80d74ca7b392ce` via annotated tag
  `vulkan-cts-1.4.6.2` (`42c723aa10d2652590f02741827aef43b0421d23`).
- Each release `LICENSE` is 11,358 bytes, SHA-256
  `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`, and contains
  the Apache License Version 2.0 notice.
- The local receipt reports `cts_executions: 0` and every local claim false. It says exactly that
  `vk-default` is a Khronos default root broader than a Vulkan-1.4-core-only selector.

## Commands and results

From `/Users/petreleon/code/WebBoxVM`:

    make graphics-full-suite-root-test
    # 5 passed, 0 failed, 0 skipped

    python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/03-full-suite-record/01-canonical-full-suite-roots/full_suite_roots.py \
      --cache-root /private/tmp/webboxvm-f02531-roots.bVOC5g --timeout 30
    # fresh 3-root and 2-license digest/byte receipt: exit 0

    git ls-remote --tags https://github.com/KhronosGroup/VK-GL-CTS.git \
      'refs/tags/opengl-cts-4.6.8.1*' 'refs/tags/vulkan-cts-1.4.6.2*'
    # tag-object and peeled-commit values above: exit 0

    make test
    # graphics 10+4+2+5+9+3+6+3+3+3+4+5; Rust 1,130; Node 338: all pass

    cargo test -p emulator --test source_file_limits --quiet
    # 6 passed

    git diff --check
    python3 scripts/check_graphics_roadmap.py
    # PASS: 375 documents, 223 tasks, 45 PASS-complete, 81 superseded
    # Ready: F02.5.3.2, F02.5.3.3; Blocked: none

The hostile tests reject a missing/reordered root, mismatched suite ID, mutable or foreign URL,
filtered root, substituted Vulkan fraction path, positive conformance claim, altered license payload,
and a nonempty cache before network access.
All final commands exited zero; no focused or full-suite-root test was skipped.

## Decision and limits

These records are kept outside the F02.5.2 immutable normative-source catalog. They preserve the
unfiltered upstream roots; the 8 MiB limit applies only to later WebBoxVM transforms, never to an
upstream root or member. The `vulkan-1.4-core` profile label does not make `vk-default` a Khronos
core-only selector. No CTS case ran, so this evidence does not establish guest compatibility,
conformance, certification, or performance.

First failing subcheck: the initial relative import of `source_cache` was wrong and was corrected before
the recorded tests and receipt. The later sandboxed refresh could not resolve the source host; the
authorized fresh external-network run passed. Neither was a source-identity mismatch.

Commit/push verification: implementation `16dd32f0` is committed locally; no remote push or CI claim.
Next ready tasks: F02.5.3.2 and F02.5.3.3 after this checked completion.
