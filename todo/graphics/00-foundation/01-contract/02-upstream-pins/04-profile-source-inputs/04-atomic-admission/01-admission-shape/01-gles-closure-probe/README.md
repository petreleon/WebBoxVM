# F02.4.4.1.1 — Probe the GLES rejected closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.4.4.1.1
Depends: F02.1, F02.2, F02.4.1, F02.4.2, F02.4.3
Evidence: pending

Prerequisite lists: [GLES audit](../../../02-gles-input-audit/README.md) and
[F02.2](../../../../02-fetch-verifier/README.md).

## Outcome

The current rejected GLES CTS root is represented only as an exact four-member, configuration-bound,
pre-admission logical closure. It can never return an admitted inventory input or change the root's
rejected candidate decision.

## Starting points

- [compound selector audit](../../../compound_selector_contract.py)
- [GLES closure](../../../02-gles-input-audit/cts_closure.json)
- [F02 source policy](../../../../02-fetch-verifier/01-fetch-contract/source_model.py)
- [pre-admission adapter](../closure_shape_contract.py)
- [hermetic probe test](../closure_shape_test.py)

## Checklist

- [ ] Bind the audited rejected root, four core selector members, and excluded extension boundary.
- [ ] Preserve F02.2 immutable URL, byte, cache, and selector validation for each physical member.
- [ ] Reject root-only, stale, mutable, oversize, duplicate, configuration-drift, and scope-expanded input.
- [ ] Make the returned state unconditionally `unadmitted`; do not expose an inventory or cache target.
- [ ] Record the exact focused command and nonzero hostile-suite count in a receipt.

## Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 ../closure_shape_test.py` must report six passing tests.
- The current `gles-cts-manifest` audit remains `rejected`; this is provenance-only and not CTS,
  guest, browser, API, conformance, or performance evidence.
