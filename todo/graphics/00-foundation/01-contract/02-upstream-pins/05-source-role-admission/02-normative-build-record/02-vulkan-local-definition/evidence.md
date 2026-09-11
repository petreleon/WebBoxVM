# F02.5.2.2 evidence

Revision: e4bbc4d1
Validation: targeted contract test, live offline rebuild, make test, source-file limits, roadmap check
Result: PASS
Artifacts: reproducible 7,506-byte JSON, SHA-256 29b9b64f67eca6c8ddf3edd40bddcb0843c059403dd757ec3479c2939dfd7556
Profile: WebBoxVM engineering facts for XML VK_VERSION_1_4; not guest API support or qualification

## Identity and inputs

- Tested commit: e4bbc4d1; starting worktree was clean.
- Builder: webboxvm_source_builder.py, 5,615 bytes,
  SHA-256 5783e76ee1293887f3678153837d4887368ede3376970ce048a9c109aca5e5b6,
  introduced by de5d7d05.
- Runtime: CPython 3.14.6.
- Inputs: vulkan-14-spec revision
  f84d432d5b8912362f96f581f29bbc4f3c8c7843,
  SHA-256 069b7e6d6326969df7b4a86f189f7ba22359e93ca7aa76c23301504667d3c4b0;
  vulkan-registry at the same revision,
  SHA-256 cf31c965cf6e788697139601da0c7e02a75a9b6c7ac764e7641f5521ffd9da06.
- Canonical logical argv and network: forbidden are fixed in
  vulkan_definition_contract.py; the builder reads only supplied regular files.

## Commands and observations

From /Users/petreleon/code/WebBoxVM:

    make graphics-vulkan-definition-test
    # 3 builder tests + 3 contract tests: PASS

    python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/02-normative-build-record/02-vulkan-local-definition/vulkan_definition_contract.py \
      --vkspec /private/tmp/webboxvm-f0252-normative-roots-final/webboxvm-graphics/f02/vulkan-14-spec/069b7e6d6326969df7b4a86f189f7ba22359e93ca7aa76c23301504667d3c4b0.source \
      --vkxml /private/tmp/webboxvm-f0252-normative-roots-final/webboxvm-graphics/f02/vulkan-registry/cf31c965cf6e788697139601da0c7e02a75a9b6c7ac764e7641f5521ffd9da06.source \
      --output /private/tmp/webboxvm-f0252-contract.h7r22a/vulkan-14-core-definition.json
    # PASS: bytes=7506 sha256=29b9b64f...2939dfd7556

    make test
    # source-role, normative-root, local-definition, 1,130 Rust, and 338 Node tests: PASS

    cargo test -p emulator --test source_file_limits --quiet
    # 6 passed

    git diff --check
    python3 scripts/check_graphics_roadmap.py
    # PASS: 372 documents, 223 tasks, 42 PASS-complete, 81 superseded

The contract rejects a changed builder or runtime, altered input or output bytes,
noncanonical argv, Khronos authority, and a positive selector claim. The output stays
below 8 MiB; the limit applies to this local transform, not upstream suite content.

## Decision and limits

The artifact contains only VK_VERSION_1_4 XML facts. Promoted extension names remain
separate boundary metadata; WSI/video are neither extracted as core facts nor support
claims. All claims for selector, API support, conformance, certification, profile
support, and performance are false. No CTS case ran, no guest was exercised, and this
does not establish compatibility or Khronos conformance.

The generated JSON is intentionally not checked into Git: reproduce it in a fresh
temporary directory with the two verified cached inputs and the command above. The
digest, size, input identities, builder identity, runtime, and serialization are
validated before writing it.

First failing subcheck: none on the recorded revision.
Commit/push verification: e4bbc4d1 committed locally; no remote push performed.
Next ready task: F02.5.2.3 admission receipt, alongside the newly split F02.5.3 suite work.
