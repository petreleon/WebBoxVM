# F02.4.1 evidence

Revision: 1362c3b21bdd0f7d3775bb7975f1bd7fd598700b
Validation: local candidate contract, hostile tests, limits, roadmap, whitespace, and `make test`
Result: PASS
Artifacts: [candidates](candidates.json), [contract](../candidate_contract.py), and [tests](candidate_audit_test.py)
Profile: OpenGL 4.6 core source provenance only; no guest, browser, CTS-run, or performance claim.

Task ID and date: F02.4.1, 2026-09-09.

Tested commit and dirty diff hash: `1362c3b21bdd0f7d3775bb7975f1bd7fd598700b`; only this
receipt and its parent checkbox were uncommitted during final validation.

Upstream manifest revision: `08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6`.

Candidate decision: accepted `opengl-46-core-spec`, [OpenGL 4.6 Core PDF](https://raw.githubusercontent.com/KhronosGroup/OpenGL-Registry/1cdd228e34966dd6b95bd203e9f84faba0f371a1/specs/gl/glspec46.core.pdf), commit `1cdd228e34966dd6b95bd203e9f84faba0f371a1`, SHA-256 `a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee`, 3,003,752 bytes, with the conditional reproduction terms on PDF p. iv recorded verbatim in the candidate.

Candidate decision: accepted `opengl-cts-manifest`, released [flat GL 4.6 CTS selector](https://raw.githubusercontent.com/KhronosGroup/VK-GL-CTS/067e8832315e79817ede1c4863804e440f5d1c80/external/openglcts/data/gl_cts/data/mustpass/gl/khronos_mustpass/4.6.1.x/gl46-main.txt), release `opengl-cts-4.6.8.1` peeled to commit `067e8832315e79817ede1c4863804e440f5d1c80`, SHA-256 `e28bbbbfd0f6c8d711554a01aa45819bdc7ca963c9426e997dfd1daaeb1d7b17`, 1,353,085 bytes, and stated Apache-2.0 repository terms. `4.6.1.x` is the upstream mustpass-layout label, not the release pin. The file has 19,714 nonempty unique `KHR-GL46.*` entries and no include or nested-list directive.

Exact commands, from `/Users/petreleon/code/WebBoxVM`: `git ls-remote --tags https://github.com/KhronosGroup/VK-GL-CTS.git 'opengl-cts-4.6.8.1'`; pinned raw-source `curl | shasum -a 256` and `curl | wc -c` probes; `PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/candidate_contract.py todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/01-opengl-input-audit/candidates.json opengl-4.6-core`; `PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/01-opengl-input-audit/candidate_audit_test.py`; `cargo test -p emulator --test source_file_limits --quiet`; `PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py`; `git diff --check`; and `PYTHONDONTWRITEBYTECODE=1 make test`.

Expected result and minimum nonzero case count: two exact candidate records, 10 focused checks, and
a selector with at least one case. Actual: `AUDIT: opengl-4.6-core accepted, accepted`; focused
10/10; selector 19,714 entries; source-file limits 6/6; roadmap PASS after parent propagation;
and local `make test` exited zero, with Cargo reporting no failures and Node reporting 337 pass.

Negative/reference checks: mutable/mismatched revision, existing or arbitrary family reuse, a coherent
alternate raw source, selector suffix substitution, zero-case manifest, compound acceptance, stale
inventory lock, stale canonical F03 requirement, and changed license/generator/provenance/cache all
fail closed. A cached bare `inventory_layout` decoy is ignored during exact loading and restored after.
The standalone `mustpass.xml` was rejected: it references 12 lists, including `gl42-compat-main.txt`.

Software fallback detection and actual execution route: not applicable; this is an offline source audit.

Performance conditions and frozen protocol version: not applicable; no graphics workload ran.

First failing subcheck: before parent propagation, the roadmap checker reported that F02.4's checkbox
did not match completed F02.4.1. It was corrected; the final checker pass has no failing subcheck.

Decision and limits: F02.4.4 may consider the two exact identities. This does not pin the CTS build
or dependency closure, execute CTS, establish conformance, expose a guest API, or measure performance.
It must not mark OpenGL support, VirGL behavior, browser execution, or near-native performance.

Commit/push verification: source-audit code through `1362c3b21bdd0f7d3775bb7975f1bd7fd598700b` was
pushed; `git ls-remote origin refs/heads/codex/graphics-f01-baseline` returned that exact SHA.

Next ready task: F02.4.2 and F02.4.3 remain ready; F02.4 stays open pending their audits and atomic admission.
