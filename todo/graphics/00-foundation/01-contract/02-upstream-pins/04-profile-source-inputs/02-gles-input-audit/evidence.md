# F02.4.2 evidence

Revision: `37e2d28834beaafb1e2f5e6894330a7a91cf9c2d`
Validation: direct pinned-source re-fetch, local candidate/closure contracts, hostile tests, limits,
roadmap, whitespace, and `make test`
Result: PASS
Artifacts: [candidates](candidates.json), [closure](cts_closure.json),
[configurations](cts_configurations.json), [contract](../compound_selector_contract.py), and
[tests](candidate_audit_test.py)
Profile: GLES 3.2 source provenance and descriptor analysis only; no guest, browser, CTS-run, or
performance claim.

Task ID and date: F02.4.2, 2026-09-09.

Tested commit: `37e2d28834beaafb1e2f5e6894330a7a91cf9c2d`. The status/receipt edits were not in
that commit; they are deliberately kept separate from tested audit code. Upstream manifest revision:
`08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6`.

The accepted candidate is [the GLES 3.2 PDF](https://raw.githubusercontent.com/KhronosGroup/OpenGL-Registry/1cdd228e34966dd6b95bd203e9f84faba0f371a1/specs/es/3.2/es_spec_3.2.pdf), commit `1cdd228e34966dd6b95bd203e9f84faba0f371a1`,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`,
2,198,754 bytes, with the Khronos conditional reproduction terms recorded in the candidate.

The [ES CTS descriptor](https://raw.githubusercontent.com/KhronosGroup/VK-GL-CTS/067e8832315e79817ede1c4863804e440f5d1c80/external/openglcts/data/gl_cts/data/mustpass/gles/khronos_mustpass/main/mustpass.xml)
is pinned to commit `067e8832315e79817ede1c4863804e440f5d1c80`, SHA-256
`9f466f19a26120149bab06e70596c9fff57b61d767cfdbadc7ba13934ce2ea96`, 4,284 bytes, and
Apache-2.0 repository terms. It is rejected for admission now: the descriptor has a required
multi-file closure, so it cannot masquerade as a complete single source.

Direct `curl --fail --silent --show-error --location` probes downloaded the two candidates plus each
component into a disposable `/private/tmp/webboxvm-gles-audit.*` directory. `shasum -a 256` and
`wc -c` matched every record below; the directory was deleted after verification and no upstream
payload is tracked in Git.

| Selector material | Bytes | SHA-256 |
| --- | ---: | --- |
| `gles2-khr-main.txt` | 38,127 | `aed17d34d64047973c439835ee7cfa8a109c2159512fe1fe59fb07fc05d395f2` |
| `gles3-khr-main.txt` | 443,126 | `c79097f9f9c69a4556e235b3301c0dc9a5d73375c05f193a6f5e127d853ace72` |
| `gles31-khr-main.txt` | 288,777 | `9fd8e4ff51616262c567f7a9eec83f69d2d311a4293a45d2d377306d426d520e` |
| `gles32-khr-main.txt` | 105,294 | `426fa31d557e08e214b75fda5d9efcaec54a42cdc14d4915939a7d02ccb6f249` |
| excluded `gles32-khr-glesext.txt` | 81,374 | `789b0474c61693baaa7cbc941575f23b383e2ba3c4991b1a910e046a17a6e0e6` |

The descriptor names four unique core lists with 12,477 unique cases and twelve required core
configuration variants, yielding 30,574 case-configuration runs. Its one optional-extension
configuration and 1,097-case `gles32-khr-glesext.txt` list are recorded only in the excluded boundary.
This count describes the pinned descriptor; it is not a CTS execution result.

Exact local commands, from `/Users/petreleon/code/WebBoxVM`:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/02-fetch-verifier/01-fetch-contract/source_fetch_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/candidate_contract.py todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/02-gles-input-audit/candidates.json gles-3.2
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/compound_selector_contract.py todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/02-gles-input-audit/cts_closure.json todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/02-gles-input-audit/cts_configurations.json todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/02-gles-input-audit/candidates.json gles-3.2
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/02-gles-input-audit/candidate_audit_test.py
cargo test -p emulator --test source_file_limits --quiet
PYTHONDONTWRITEBYTECODE=1 make test
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
git diff --check
```

Expected: two decision records, four core selector materials, an explicit optional boundary, and
nonzero hostile checks. Actual: `AUDIT: gles-3.2 accepted, rejected`; `COMPOUND: gles-3.2 4 core
selectors, 12477 unique cases, 12 configurations, 30574 case-configuration runs, 1 excluded selector,
1 excluded configuration`; fetch contract 15/15; GLES audit 9/9; source-file limits 6/6; roadmap
PASS before status propagation; and local `make test` exited zero. Cargo reported no failures and Node
reported 337 passing tests. The known local `xcrun` FSEvents/cache warnings were non-fatal.

Negative/reference checks reject a mismatched raw revision, arbitrary source family, compound-to-single
decision swap, deleted core list, changed digest, float case count, omitted optional list, collapsed
configuration, boolean-for-integer schema/count aliases, altered descriptor digest, and stale candidate
binding. The source policy now checks only the host-specific raw ref slot: it rejects a mutable ref but
allows the legitimate content directory named `main` after the pinned 40-hex commit.

Software fallback detection, actual execution route, and performance conditions: not applicable; no
graphics workload ran. F02.4.4 still must atomically admit inputs, refresh a cache, parse the descriptor,
and verify the revised provenance closure before F03.1 can change its source blocker.

First failing subcheck: the initial CTS candidate was falsely rejected because the old policy treated the
content directory `.../khronos_mustpass/main/...` as a mutable branch. The fixed policy preserves the
exact raw-GitHub revision slot and the regression suite now passes 15/15. The roadmap checker then
required a canonical `Result: PASS`; publication remains a separate unchecked action.

Commit/push verification: code commit `37e2d28834beaafb1e2f5e6894330a7a91cf9c2d` exists locally.
Its push was blocked before transport by the host's external-remote authorization policy, so no new
remote SHA or remote-CI result is claimed. The last pre-commit verified remote SHA was
`8878c33797dc0d93ba08eef459979a61b11939a4`.

Decision and limits: the PDF is a reviewed candidate; the CTS descriptor remains concretely rejected
until F02.4.4 handles its full closure. This receipt does not claim GLES support, Mesa/VirGL behavior,
conformance, browser execution, or near-native performance. Completion propagation is held until the
verified code can be published. Next independent ready task: F02.4.3.
