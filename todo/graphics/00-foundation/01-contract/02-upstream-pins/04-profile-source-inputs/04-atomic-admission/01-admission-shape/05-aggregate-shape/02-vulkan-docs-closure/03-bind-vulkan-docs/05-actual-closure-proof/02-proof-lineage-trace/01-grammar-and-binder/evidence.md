# F02.4.4.1.5.2.3.5.2.1 receipt — grammar and binder

Revision: `e5106dae12f1988339cadcb59402c453cd5e1888`
Validation: 13 focused positive/hostile grammar, binder, scope, and receipt tests; repository `make test`; source-file limit
Result: PASS
Artifacts: no source, generated output, cache, or lineage-capture artifact was created
Profile: proof-only parser/binder; no Docs provenance, closure, or collector claim

Task ID and date: F02.4.4.1.5.2.3.5.2.1, 2026-09-10 Europe/Bucharest.

From `02-proof-lineage-trace`, the focused command passed 13/13 tests:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  lineage_test.py lineage_hostile_test.py -v
```

It covers the successful writer/rename/final/bind relation and rejects duplicate or missing inputs, lifecycle errors,
unsafe paths, mutation, scope forgery, stale receipts, and cycles. `make test` also passed at this revision (1,151
Rust tests passed, 3 ignored; 337 Node tests passed), and the source-file-limit suite passed 6/6. This result validates
only proof-model mechanics; a real trace collector remains a separate unchecked child.
