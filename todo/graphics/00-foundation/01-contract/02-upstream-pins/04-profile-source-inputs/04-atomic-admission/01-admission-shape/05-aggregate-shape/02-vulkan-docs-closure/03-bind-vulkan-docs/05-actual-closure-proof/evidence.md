# F02.4.4.1.5.2.3.5 blocker record

Baseline revision: `40dcf2f2521fcdb698316d47b58bb0352ae1429d`; child integration atop
`af430ce647621edc9eb8113a4bc6a49875b005b6`
Validation: baseline live witness/scope/comparison contracts passed; field/cache probes; child blockers below
Result: BLOCKED
Artifacts: plan `5dcc55e3cd979010d73142e7dde13361e0b36f64ff42aaa1e436a8a5eab5de77`;
observation and live artifact root named below; no actual-grammar closure manifest exists
Profile: Vulkan 1.4-core Docs source-closure proof only; staging and all observed facts remain unadmitted

Task ID and date: F02.4.4.1.5.2.3.5, 2026-09-10 Europe/Bucharest.
The original receipt was tested at `40dcf2f2521fcdb698316d47b58bb0352ae1429d` with an empty worktree.
The child records below were added atop `af430ce647621edc9eb8113a4bc6a49875b005b6`; their validation
is separately recorded in their evidence files.

The live observation was
`../03-capture-core-closure/01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json`.
The ignored live artifact root was
`/Users/petreleon/code/WebBoxVM/.artifacts/graphics/f02.4.4.1.5.2.3.3.1`.
These commands revalidated its reviewed, unadmitted prerequisites:

```sh
OBSERVATION=/Users/petreleon/code/WebBoxVM/todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/02-vulkan-docs-closure/03-bind-vulkan-docs/03-capture-core-closure/01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json
ARTIFACT_ROOT=/Users/petreleon/code/WebBoxVM/.artifacts/graphics/f02.4.4.1.5.2.3.3.1

PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_identity_contract.py vulkan_docs_build_witness.json
# WITNESS: actual-docs-build-witness, 1 rendered output, 0 cutover-ready

PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_scope_contract.py \
  vulkan_docs_core_input_scope.json "$OBSERVATION" "$ARTIFACT_ROOT" observer-a
# SCOPE: input-scope-only-unadmitted, 0 cutover-ready

PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_compare_contract.py \
  vulkan_docs_core_input_comparison.json "$OBSERVATION" "$ARTIFACT_ROOT"
# COMPARISON: input-comparison-only-unadmitted, 2 captures, 0 cutover-ready
```

The first command ran in `../02-actual-closure-identity`; the latter two ran in their
respective `../03-capture-core-closure/02-bind-core-input-scope` and
`../03-capture-core-closure/03-compare-fresh-captures` directories. `OBSERVATION` and
`ARTIFACT_ROOT` above are the explicit paths recorded above. All three exited zero.

From `../04-stage-build-witnesses/01-stage-contract`, a read-only `build_plan()` probe
compared its real `raw_records` and `derived_records` with `RAW_FIELDS` and `DERIVED_FIELDS`
from `../02-actual-closure-identity/vulkan_docs_identity_model.py`. It rebuilt the same plan
against the observation, artifact root, and a nonexistent external-cache pathname; the pathname
was still absent afterward. Reproduce that exact probe from that directory with:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -c '
from pathlib import Path
from vulkan_docs_stage_bind import build_plan
from vulkan_docs_identity_model import RAW_FIELDS, DERIVED_FIELDS
plan = build_plan(Path("../../03-capture-core-closure/01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json"), Path("/Users/petreleon/code/WebBoxVM/.artifacts/graphics/f02.4.4.1.5.2.3.3.1"), Path("/private/tmp/webboxvm-docs-proof-field-audit"))
raw = plan.value["inputs"]["raw_records"]
derived = plan.value["inputs"]["derived_records"]
assert all(set(row) == set(raw[0]) for row in raw)
assert all(set(row) == set(derived[0]) for row in derived)
print("COUNTS", len(raw), len(derived))
print("RAW-MISSING", ",".join(sorted(RAW_FIELDS - set(raw[0]))))
print("RAW-EXTRA", ",".join(sorted(set(raw[0]) - RAW_FIELDS)))
print("DERIVED-MISSING", ",".join(sorted(DERIVED_FIELDS - set(derived[0]))))
print("DERIVED-EXTRA", ",".join(sorted(set(derived[0]) - DERIVED_FIELDS)))
'
```

Its exact result was:

```text
COUNTS 298 1462
RAW-MISSING generated_code_role,id,license,local_cache,provenance,source_family
RAW-EXTRA phase_roles
DERIVED-MISSING generated_code_role,id,license,local_cache,producer_input_ids,provenance
DERIVED-EXTRA phase_roles
```

The actual closure grammar requires exact member schemas and a self-hashed
`vulkan-docs-actual-closure-v1`. It requires every derived member's
`producer_input_ids` to name earlier closure members. The current trace
`../03-capture-core-closure/01-observe-pinned-build-inputs/observer/input_trace.c` records
read/pread/readv/`mmap(PROT_READ)`/fread/fgets/copy/sendfile/splice events and process starts. It
does not capture generated-file writes, renames, or a producer-input-to-output edge. Its direct
`SYS_write` use emits the trace itself, not a build-output provenance event.

The observed records cannot themselves supply an exact actual-grammar manifest. Raw `source_family`, revision,
and URL are mechanically fixed by the pinned source contract; 171 selected raw files have direct SPDX headers and
the other 127 have pinned `REUSE.toml` annotations. Those raw facts still need an explicit, full-tree reifier; they
must not be treated as runtime-observation fields. No corresponding authority currently supplies derived licenses,
roles, cache/provenance policy, or an output-to-producer mapping. In particular, all 1,462 derived producer edges
cannot be reconstructed honestly from the present observations. Mapping derived facts from selectors or hashes
would invent semantic claims and violate this task's no-reclassification condition.

An auxiliary retained-cache check did not change that first blocker. At validation time,
`/private/tmp/webboxvm-input-stage.V2seBM` verified its 298 raw plus 1,462 derived input snapshot:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_input_contract.py verify \
  "$OBSERVATION" "$ARTIFACT_ROOT" /private/tmp/webboxvm-input-stage.V2seBM
# INPUT-STAGE: staging-only-unadmitted, 298 raw, 1462 derived, 0 markers

PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_output_contract.py verify \
  "$OBSERVATION" "$ARTIFACT_ROOT" /private/tmp/webboxvm-input-stage.V2seBM
# FAIL: output-stage receipt is missing
```

The input command exited zero; the output command exited two at its first missing receipt. This
ephemeral cache is not a retained full-stage/marker proof and cannot replace a fresh staging run.

First failing subcheck: actual-grammar member provenance cannot be produced from the captured
read-only observations. The prerequisite bytes, scope, witness, and two-capture comparison remain
consistent but are not a complete actual Docs closure.

Decision and limits: leave every checkbox in this leaf and its parents open. Do not label the
staging marker, the source/input snapshot, or the generated-output observation as an actual closure;
all remain `unadmitted` and `cutover_ready=False`. No inventory, candidate decision, F03 state,
guest API, browser, CTS, conformance, or performance claim changes here. This is a mandatory
provenance-model blocker, not a PASS or an artifact-availability exemption.

The split children must first reify authoritative member metadata, then add a separate proof-only tracer with
process ancestry plus generated-file write/rename events. They must run two fresh pinned builds and justify every
derived producer edge before materializing a full actual-grammar manifest. Only after hostile tests reject omitted,
ambiguous, reordered, scope-expanded, or invented provenance may they restage all 1,760 inputs, both output trees,
and a strict marker before proving the closure remains unadmitted.

Original-receipt repository gates: `make test` exited zero with 1,127 Rust tests passed, 3 ignored,
source-file limits 6/6 passed, and 337 Node tests passed. `git diff --check` exited zero.
`PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py` exited zero with
`PASS: 267 documents, 159 tasks, 59 complete; links/dependencies/limits valid`. The child integration has
its own current gate record rather than relabeling these historic results. Remote CI was not run.

The roadmap checker may label F02.4.4.1.5.2.3.5.1 and F02.4.4.1.5.2.3.5.2 `Ready` because their declared
dependencies are complete. That is bookkeeping/dependency-ready only, not an executable closure successor:
their respective blocker receipts retain absent derived-license authority and an unavailable trusted tracer. No
closure successor is executable under the current pinned evidence and no-privilege container route.
