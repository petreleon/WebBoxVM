# F02.4.4.1.2 — Map all required-source shapes

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.4.4.1.2
Depends: F02.1, F02.2, F02.4.1, F02.4.2, F02.4.3, F02.4.4.1.1
Evidence: [receipt](evidence.md)

Prerequisite lists: [F03 requirements](../../../../../03-feature-matrix/01-profile-scope/source_requirements.json)
and the source audits.

## Outcome

Every one of the six F03 required IDs receives one auditable shape record: a complete single source,
a fully bounded unadmitted closure, or an explicit unresolved root. No record treats a root as its
unproven members.

## Starting points

- [candidate catalog](../../../candidate_catalog.py)
- [OpenGL audit](../../../01-opengl-input-audit/README.md)
- [GLES closure probe](../01-gles-closure-probe/README.md)
- [six-source map](source_map.json)
- [map contract](source_map_contract.py)
- [map tests](source_map_test.py)

## Checklist

- [x] Enumerate exactly the six canonical required IDs and their audited root identities.
- [x] Bind accepted single-source records separately from compound logical closures.
- [x] Bind every known physical member, configuration, and core-scope limit to its logical ID.
- [x] Reject omissions, role swaps, stale roots, and an unresolved root posing as a member set.
- [x] Record each unresolved generated or transitive member as a blocker rather than inventing it.

## Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 source_map_test.py` must report eight passing tests.
- The map has no implicit identity, mutable upstream reference, or inventory-entry result.
- A mapping proves only source-shape provenance; it does not satisfy F03 or run any graphics workload.
