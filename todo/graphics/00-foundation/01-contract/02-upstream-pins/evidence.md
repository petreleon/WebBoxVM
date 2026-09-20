# F02 aggregate evidence

Revision: `f5337681`
Validation: F02.1–F02.5 child receipts; historic 17-input closure plus role-aware source cutover; local gates
Result: PASS
Artifacts: immutable F02 inventory lock, historic external cache `/private/tmp/webboxvm-f0234-live.OwEDjs`,
and [F02.5 aggregate receipt](05-source-role-admission/evidence.md)
Profile: reproducible source/provenance foundation only; no graphics runtime, browser, or performance claim

## Historic closure and completed role-aware scope

F03.1 subsequently identified six unpinned target-profile normative/conformance inputs. F02.5 closed
that gap with the sealed eight-record role-aware contract, six mandatory role bindings, and three
unfiltered full-suite closures. The active F03 gate now reaches only `matrix-incomplete`; it is not
an API-support, CTS, conformance, certification, profile, browser, or performance result.

Task ID and date: F02, 2026-09-09 Europe/Bucharest.

F02 completes its five checked children:

- [immutable 17-input inventory](01-input-inventory/evidence.md)
- [fail-closed external fetch/cache contract](02-fetch-verifier/evidence.md)
- [ABI and generator provenance bindings](03-provenance-contract/evidence.md)
- [historical profile-source plan](04-profile-source-inputs/README.md)
- [role-aware source admission and consumer cutover](05-source-role-admission/evidence.md)

The final F02.3.4 acceptance used a fresh external cache: all 17 immutable inputs fetched, all 17
were reread and rehashed offline, and the closure validator accepted exactly 17 cache sources and 12
reviewed provenance records. The empty-cache negative run exited 2 on its first unavailable input.
The cache total was 18,953,659 bytes and no payload was added to Git.

The final local gates were the documented F02 focused suites, source limits 6/6, `make test`, roadmap
verification, and `git diff --check`. The F02.5 capture refreshed the separate external nine-file
selector/license cache `/private/tmp/webboxvm-f02543.bO41B6`; its self-hashed receipt is
`85980a92e5223261e7fce6fc162e10171bd671800a5fe9d4c2460e01b68d5242`. This proves only reproducible,
fail-closed source and local provenance metadata. It does not prove a guest driver, guest-visible GPU
API, VirGL/OpenGL/GLES, Venus/Vulkan, browser rendering, compatibility, native comparison, remote CI,
or near-native performance. `f5337681` is pushed; no Actions run is claimed. Next ready task: F03.2,
F03.3, or F03.4.1.
