# F02.4.4.1.5.3 blocker record

Revision: 1c57599cca7fb3f68d4d49f91bd362316d450fc3
Validation: pinned VCTS tag/root/tree inspection + policy/audit/include/boundary 15/10/6/9
Result: BLOCKED
Artifacts: root sha256=b689703bdc65a04764db3b9a8f6fe872b3fe94d0df68d78f6da6e5a06cfa9ed4;
98-member total=434669348 B; lower-bound scoped total=415109500 B; 14 members exceed 8388608 B
Profile: source-provenance discovery only; no inventory admission, guest, API, browser, CTS,
conformance, or performance behavior

Task ID and date: F02.4.4.1.5.3, 2026-09-09 Europe/Bucharest.
Pinned source: `vulkan-cts-1.4.6.2` annotated tag
`42c723aa10d2652590f02741827aef43b0421d23` peels to commit
`f6a29701220f34dd1407513bfe80d74ca7b392ce`. Its required root
[`vk-default.txt`](https://raw.githubusercontent.com/KhronosGroup/VK-GL-CTS/f6a29701220f34dd1407513bfe80d74ca7b392ce/external/vulkancts/mustpass/main/vk-default.txt)
is 3,347 B with SHA-256 `b689703bdc65a04764db3b9a8f6fe872b3fe94d0df68d78f6da6e5a06cfa9ed4`.
The exact upstream Vulkan generator has one `main/default` configuration: `main.txt` is `dEQP-VK.*` and
offers no canonical Vulkan-1.4-core selector. The root is therefore not an eligible core closure.

Reproduce tag/root provenance with GitHub's git-ref and annotated-tag endpoints, then inspect the commit tree
at the peeled commit. The 98 direct selected blobs total 434,669,348 B; 14 individually exceed F02.2's
8,388,608-B cap:

| Pinned `vk-default` relative path | Bytes |
| --- | ---: |
| `api.txt` | 40,296,059 |
| `binding-model.txt` | 21,368,255 |
| `fragment-shading-rate.txt` | 15,710,739 |
| `image/host-image-copy.txt` | 10,269,828 |
| `pipeline/fast-linked-library.txt` | 18,943,120 |
| `pipeline/monolithic/monolithic.txt` | 61,932,251 |
| `pipeline/pipeline-library.txt` | 16,990,842 |
| `pipeline/shader-object-unlinked-spirv/shader-object-unlinked-spirv.txt` | 55,935,076 |
| `renderpasses.txt` | 10,273,402 |
| `robustness.txt` | 13,045,748 |
| `shader-object/rendering.txt` | 36,351,931 |
| `synchronization.txt` | 8,738,894 |
| `synchronization2.txt` | 11,515,449 |
| `transform-feedback.txt` | 20,427,868 |

Removing all six already excluded WSI/video/extension paths is deliberately only an overpermissive lower
bound, never a core classifier. It still has 92 members, 415,109,500 B, and the same 14 over-limit blobs;
none of the 14 is among those exclusions. Thus scope removal cannot cure the policy failure, and splitting
opaque upstream text files would silently invent a source boundary.

First failing subcheck: no upstream-published immutable Vulkan-1.4-core must-pass selector exists at the
pinned release. Even the full canonical root cannot be admitted as member inputs under F02.2 because 14
members violate the hard per-input limit. Existing boundary facts remain accurate: 98 observations are not
members, the root fallback is forbidden, and all scope/identity/recursive-selection requirements remain open.

Decision: keep every checklist item, this child, F02.4.4.1.5, F02.4.4.1, F02.4.4.2, and all Vulkan support
claims open. A legitimate future path requires a Khronos-published immutable Vulkan-1.4-core must-pass artifact
whose recursively selected core members have explicit scope and each fit policy. A locally derived selector or
opaque-file splitting would require an explicit source-contract redesign before it could be considered; neither
is a substitute for the canonical current root.

## 2026-09-10 upstream recheck

The untagged `main` commit `659bbe6987197b4ff7ac20011261b92009286100` still exposes only `vk-default.txt`,
`vk-fraction-mandatory-tests.txt`, and Vulkan-SC's `vksc-default.txt`; the
[official README](https://github.com/KhronosGroup/VK-GL-CTS/blob/659bbe6987197b4ff7ac20011261b92009286100/external/vulkancts/README.md#L249-L270)
names `vk-default` as the Vulkan mustpass. The fraction list is only mandatory `dEQP-VK.info.*` metadata for
parallel fractions, not a core selector. The generator still has only `default` and `fraction-mandatory-tests`
Vulkan configurations. Current `api.txt` remains 32,673,653 B, so even a newer untagged tree cannot cure the cap
or scope failure. This read-only recheck changes no pin, policy, or blocker status.

Commands and limits: Python 3.14.6 ran the existing F02.2 policy, Vulkan audit, include, and boundary suites
(15/10/6/9). The existing source audit provides the reproducible 98-selector transcript; this record adds exact
pinned tree sizes. Remote CI, CTS, guest execution, browser execution, conformance, and performance measurement
were not run. This is a concrete mandatory blocker, not a PASS receipt.
