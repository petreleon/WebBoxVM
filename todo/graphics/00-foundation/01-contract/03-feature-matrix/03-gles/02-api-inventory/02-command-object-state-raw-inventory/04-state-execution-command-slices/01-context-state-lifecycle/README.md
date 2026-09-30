# F03.3.2.2.4.1 — Extract context state and lifecycle facts

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F03.3.2.2.4.1
Depends: F03.3.2.2.1, F03.3.2.2.2
Evidence: [scoped receipt](evidence.md)

## Outcome

A bounded raw inventory records explicit context-state, object-lifecycle, and transition rules only when a
triggering command and exact source section/table anchor are both present.

## Starting points

- [state/execution slice list](../README.md)
- [domain classification](../../01-command-domain-classification/README.md)
- [verified normative-PDF cache](../../../01-normative-pdf-cache/README.md)

## Checklist

- [x] Bind exact authority, cache, domain map, and declaration grammar identities.
- [ ] Extract only explicit triggered rules with physical page, section/table, and source-order anchors.
- [ ] Route limits, formats, ESSL semantics, extensions, and unspecified behavior away from raw state facts.
- [x] Reject inferred transitions, renderer behavior, registry entries, desktop/lower profiles, and promotions.
- [x] Self-hash raw facts with zero Matrix/CTS/owner/claim fields and attach a receipt.

## Initial group and remaining coverage

The [finite contract](contract.md) extracts twelve exact literal-trigger quotations
from the object model and shared-object lifecycle prose. The shared engine binds
all eight assigned domain families and generates a self-hashed index plus
source-family fragments. The focused suite has twelve tests and the exact-cache
validator passes. These are source quotations, not executable transition rules.

This leaf remains open. Five assigned families still require their state-command
or triggered-table extraction: vertex-remaining-state, programmable-vertex-stage,
post-vertex-state, programmable-fragment-stage, and tables 21.1–21.39. The ledger
reports `complete: false`; no empty fragment is evidence that its family is done.
Completing and routing the entire assigned scope is still required by the two
unchecked actions above. The scoped receipt therefore records `Result: FAIL` for
the full leaf while preserving the passing initial-group evidence.

Run `gles_context_state_lifecycle_inventory_test.py` with Python from the repository
root. `WEBBOXVM_GRAPHICS_CACHE_ROOT` selects another verified external cache.
The validator's `--cache-root` argument is mandatory; exact commands and hashes
are recorded in the receipt.

## Verification

Raw source facts do not prove lifecycle correctness, memory ordering, guest/browser behavior, support,
conformance, certification, or performance.
