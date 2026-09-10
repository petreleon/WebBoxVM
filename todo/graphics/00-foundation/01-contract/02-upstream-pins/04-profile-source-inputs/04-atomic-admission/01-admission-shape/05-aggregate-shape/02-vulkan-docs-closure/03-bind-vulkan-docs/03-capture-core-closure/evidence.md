# F02.4.4.1.5.2.3.3 aggregate receipt — captured Docs core closure

Revision: `a71f33b5` tested comparison revision (implementation `e75fa430`)
Validation: child `.3.1` observation receipt, `.3.2` scope receipt, and `.3.3` comparison receipt
Result: PASS
Artifacts: [comparison receipt](03-compare-fresh-captures/vulkan_docs_core_input_comparison.json) plus ignored raw
captures under `.artifacts/graphics/f02.4.4.1.5.2.3.3.1/`
Profile: unadmitted Vulkan 1.4 core input/scope closure; not cutover-ready

The aggregate has three completed, linked stages. `.3.1` recorded two separately stored official-build observations;
`.3.2` bound the cap-valid raw/generated input and conditional-scope grammar; `.3.3` rebound both selected pinned
captures and compared their semantic identities. The final compact receipt records 298 raw, 1,462 derived, 1,973
include, 7 condition, 4 promotion, 9 extension-control, and 42 image counts without staging a payload tree.

The comparison requires distinct canonical source/output locations and regular primary source/HTML members, then
inherits only the reviewed output witness identity. It does not re-run the producer, inspect Git, re-hash output
payloads, establish a new time/host provenance claim, admit a source, or change any consumer state. The next child
`.4` owns output witness staging and verification.

The child receipt records the exact focused commands and green local checks: 16 comparison methods, 6 source-limit
tests, 1,127 emulator tests with 3 ignored, and 337 Node tests. Remote CI was not run.
