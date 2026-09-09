# F06.2 — Enforce maintained-file line limits

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F06.2
Depends: F06.1
Evidence: pending

Prerequisite lists: [F06.1](../01-module-boundaries/README.md).

## Outcome

The 180-line rule covers maintained graphics roadmap, guest, generator, and source files consistently.

## Starting points

- [module boundaries](../01-module-boundaries/README.md)
- [source-file limit test](../../../../../../emulator/tests/source_file_limits.rs)
- [roadmap checker](../../../../../../scripts/check_graphics_roadmap.py)

## Checklist

- [ ] Define tracked-root coverage and a narrow documented exemption policy for legal text and
  immutable third-party patches with recorded provenance; do not blanket-exempt generated source.
- [ ] Extend line-limit checks to maintained `todo`, guest, research, generator, and graphics paths;
  split `research/virgl-resource-residency.md` and any touched oversized maintained file by role.
- [ ] Add an isolated 181-line fixture plus boundary cases that prove rejection, allowed exemptions,
  and correct physical-line counting.
- [ ] Run the full source-limit and roadmap checks, recording exact counts and every exempt file.

## Verification

- The 181-line fixture fails with its relative path; a 180-line counterpart passes.
- The expanded check catches every maintained source root and leaves only reviewed exemptions.
