# F02.4.4.1.5.2 blocker record

Revision: 1c57599cca7fb3f68d4d49f91bd362316d450fc3
Validation: pinned Docs core-closure discovery twice + policy/audit/include/boundary 15/10/6/9
Result: BLOCKED
Artifacts: generated-tree manifest sha256=50dfa2132681546c8cae7ff5e616ebc3e2d6203aa4585ebe7f9b18af2b7ad757;
semantic closure sha256=57906676ea6dde2ef9add572ff2630e8a87c0f6a234dc04274da29794b51ac3b;
full closure sha256=0e15c0f42e53e4a189648d97676cf4560ace9a28daaf07bc8551b77c480b9130
Profile: source-provenance discovery only; no inventory admission, guest, API, browser, CTS,
conformance, or performance behavior

Task ID and date: F02.4.4.1.5.2, 2026-09-09 Europe/Bucharest.
Pinned source: Vulkan-Docs annotated tag `v1.4.362` is
`cafe0e3f42089d3543219dbd92b15bf9e0af72fe`, peeled commit
`f84d432d5b8912362f96f581f29bbc4f3c8c7843`. The current rejected root remains
[`vkspec.adoc`](https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/f84d432d5b8912362f96f581f29bbc4f3c8c7843/vkspec.adoc),
8,685 B, SHA-256 `069b7e6d6326969df7b4a86f189f7ba22359e93ca7aa76c23301504667d3c4b0`.
Reproduction route: check out the peeled commit and run
`./makeSpec -clean -spec core -version 1.4 -genpath "$generated" html`. The generator-only probe ran
the pinned `make generated` route twice with all base, compute, graphics, and generic version attributes
from 1.0 through 1.4 (20 attributes total), `EXTENSIONS=[]`, and fixed revision/date/remark values.
Both clean runs produced 2,448 generated files, 3,730,148 B, and the same generated-tree manifest digest
`50dfa2132681546c8cae7ff5e616ebc3e2d6203aa4585ebe7f9b18af2b7ad757`.

The semantic discovery retained 1,591 active text/generated members (9,079,965 B,
`57906676ea6dde2ef9add572ff2630e8a87c0f6a234dc04274da29794b51ac3b`) and 42 referenced SVGs
(1,182,290 B, `b22050b02abfec95429a64c6f690c9d46caba202eea770dad1fe8c85c4457775`). Combined,
the candidate closure has 1,633 members, 10,262,255 B, and digest
`0e15c0f42e53e4a189648d97676cf4560ace9a28daaf07bc8551b77c480b9130`. The largest member is
`chapters/resources.adoc` at 680,455 B; no discovered member exceeds F02.2's 8 MiB per-input cap.

The core configuration excluded 22 optional top-level selectors (including WSI, video, ray-tracing,
device-generated commands, Vulkan SC, and vendor paths); its canonical transcript digest is
`deb7c2d096413b453ef221130f8aaff95120a5a3d98d9ba7be8e211b41eae573`. Generic extension/promotion
metadata remains active where core text requires it. The probe does not claim the official HTML build:
the local environment lacked Asciidoctor, so it establishes generator determinism and closure discovery,
not a browser-rendered specification result.

First failing subcheck: the F02.2 manifest/source model represents each `SourceInput` as one immutable raw URL,
revision, digest, bytes, and cache object; `inventory_layout.py` additionally requires exactly one entry for each
of 17 unique source families. Together they have no ordered closure-member or generator-recipe representation.
Generated fragments have no immutable raw upstream URL: their identities derive from `vk.xml`, generator modules,
Makefile/config/macro inputs, attributes, toolchain, and output. Pretending that a generated fragment is a raw
source—or packing the 10,262,255-B closure into one input—would make URL/identity false; the latter also exceeds
the per-input limit by 1,873,647 B. The frozen V1 future closure grammar separately requires `immutable_url` and
`revision` for every member and rejects a quiet schema-version change, so it also cannot truthfully encode these
generated members.

Decision: keep every checklist item and parent checkbox open. The smallest legitimate next step is a dedicated
successor source-model and inventory-layout migration with ordered fetched/generated members, per-member F02.2
URL/revision/hash/cache rules, generator recipe and toolchain identities, output digest/bytes, and two-clean-run
determinism evidence. It must introduce a versioned raw-versus-generated successor transition grammar and adapter
that preserves V1 as predecessor evidence, rather than editing V1. Then a fresh environment must run the pinned
official `makeSpec … html` route and bind the declared include/image closure before any successor inventory,
admission, F03, guest, conformance, or performance statement.

Commands and limits: Python 3.14.6 ran the existing F02.2 policy, Vulkan audit, include, and boundary suites
(15/10/6/9). The discovered inputs were disposable `/private/tmp` probe material and are not repository payloads.
The prior 73-observation boundary is retained unchanged. Remote CI, CTS, guest execution, browser execution, and
performance measurement were not run. This is a concrete mandatory blocker, not a PASS receipt.
