# F02.4.4.1.5.4.4.5.3.2 blocker record

Revision: `9532058fe683985a8e73e76911a0cbfc06776b59` upstream-selector probe
Validation: selector recheck 4/4; roadmap checker; external condition re-linked from the completed probe
Result: BLOCKED
Artifacts: self-hashed selector record `0157a3c6e5015c68307f224aa236d64107d021613b93bb3d8f70c7934fb971ee`
Profile: external VCTS authority only; no source admission, guest API, renderer, CTS, or performance claim

Task ID and date: F02.4.4.1.5.4.4.5.3.2, 2026-09-11 Europe/Bucharest.

Observed condition: the completed [upstream-selector recheck](../01-upstream-selector-recheck/evidence.md)
pins the latest observed Khronos 1.4.6 family tag, its immutable peeled commit, and the three published
mustpass roots. None is an explicit, immutable, provenance-bearing selector for complete Vulkan 1.4 core.

First failing subcheck: the required upstream artifact does not exist in the observed release. A new tag,
`vk-default`, `vk-fraction-mandatory-tests`, `vksc-default`, local filtering, taxonomy, root-only scope,
or a Docs artifact cannot substitute for it.

Decision and limits: leave every checkbox open. This status records an external prerequisite; it does not
validate membership, capture bytes, alter F02/F03, execute VCTS, or prove guest/browser compatibility,
Khronos certification, or near-native performance. Recheck only after Khronos publishes the required
artifact; then remove this status and perform the still-open validation work.

Commit/push verification: source probe is local on `codex/graphics-f01-baseline`; remote publication and
CI remain unrun because push requires fresh explicit authorization.

Next ready task: none; the external artifact is required before this leaf becomes executable.
