# F02.4.4.1.5.2.1 receipt — successor raw/generated identity

Revision: `afc814cb11687795ef94dc5d06d8f9dd86edc5e9` implementation commit
Validation: 14 focused fixture tests, 15 source-fetch, 10 Vulkan-audit, 9 boundary, 8 transition tests,
source-limit suite, full `make test`, roadmap, and whitespace checks
Result: PASS
Artifacts: synthetic identity fixture, immutable successor cache plan, exact predecessor adapter, and hostile suites
Profile: isolated successor-fixture contract only; no Docs admission, guest API, browser, CTS, or performance claim

Scope result: **PASS** for an isolated, synthetic successor-fixture contract. This is not a Docs admission,
cache-reuse, generator-execution, F03, guest API, browser, CTS, conformance, or performance result.

## Bound contract

- `successor_identity.fixture.json` is locked to `fixture-only-unadmitted`, a synthetic profile/role/logical
  identity, and a root-first ordered three-member union. It cannot be relabelled as `vulkan-1.4-core`.
- Every raw member retains F02.2-compatible immutable HTTPS/revision/selector binding, a successor-only cache
  namespace, and the per-member 8 MiB cap. A generated member has no raw URL or revision; it requires earlier
  producers, a generation ID, selector, digest, byte count, license, role, provenance, and derived cache path.
- Recipe/configuration/toolchain/output-tree identities and two equal clean-run tree hashes are bound. Output
  tree identity includes ordered output ID, selector, digest, and byte count.
- Fresh validation returns a frozen `ClosureCachePlan` containing all raw/generated identities, generation
  identities, scope and closure digests, and the rejected predecessor decision. It is not admitted or ready.
- The adapter pins predecessor bytes and also validates the exact V1 transition grammar, Vulkan F02.4 audit,
  and V1 boundary. Synthetic relabelling, altered audit roots/inventory, or erased Docs exclusions fail.

The fixture URLs, digests, and byte counts are deliberately synthetic identity tokens, not fetched source or
cache evidence. The next child must build byte-derived temporary fixtures, validate them afresh, and then use
the returned plan; it must not execute a recipe or fetch from the network.

## Focused verification

```text
PYTHONDONTWRITEBYTECODE=1 python3 successor_identity_test.py
Ran 8 tests ... OK

PYTHONDONTWRITEBYTECODE=1 python3 successor_identity_hostile_test.py
Ran 3 tests ... OK

PYTHONDONTWRITEBYTECODE=1 python3 successor_identity_anchor_test.py
Ran 3 tests ... OK

PYTHONDONTWRITEBYTECODE=1 python3 successor_identity_contract.py \
  successor_identity.fixture.json
FIXTURE: fixture-validated, 3 members, 1 generated, 0 cutover-ready
```

Relevant predecessor regressions also pass: source fetch 15/15, Vulkan audit 10/10, boundaries 9/9, and
post-cutover rules 8/8. The source-file-limit suite passes 6/6; after this receipt update the roadmap checker
reports 241 documents, 147 tasks, 47 complete. `git diff --check` is clean.

## Scope boundary

No active manifest, inventory layout, candidate JSON, active cache loader, F03 requirement, or V1 grammar was
modified. Actual Docs closure reproduction remains in F02.4.4.1.5.2.3 after the isolated cache-verifier child;
the parent Docs blocker remains open.
