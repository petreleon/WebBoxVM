# F02 historical F02.1–F02.3 evidence

Revision: `b50dcbedd79e719ace50628daa773b8eac3cb81a` final provenance-closure implementation commit
Validation: child receipts; fresh 17-input fetch and offline rehash; 17-cache/12-record closure; focused suites; source limits; local `make test`; roadmap; whitespace
Result: PASS
Artifacts: immutable F02 inventory lock and external-only fresh cache `/private/tmp/webboxvm-f0234-live.OwEDjs`
Profile: reproducible input/provenance foundation only; no graphics runtime, browser, compatibility, or performance claim

## Reopened scope

F03.1 subsequently identified six unpinned target-profile normative/conformance inputs. F02 is
therefore open through F02.4; this receipt records only the prior 17-input F02.1–F02.3 closure and
must not be used as proof that OpenGL 4.6, GLES 3.2, or Vulkan 1.4 source coverage is complete.

Task ID and date: F02, 2026-09-09 Europe/Bucharest.

F02 completes its three checked children:

- [immutable 17-input inventory](01-input-inventory/evidence.md)
- [fail-closed external fetch/cache contract](02-fetch-verifier/evidence.md)
- [ABI and generator provenance bindings](03-provenance-contract/evidence.md)

The final F02.3.4 acceptance used a fresh external cache: all 17 immutable inputs fetched, all 17
were reread and rehashed offline, and the closure validator accepted exactly 17 cache sources and 12
reviewed provenance records. The empty-cache negative run exited 2 on its first unavailable input.
The cache total was 18,953,659 bytes and no payload was added to Git.

The final local gates were the documented F02 focused suites, closure tests 7/7, source limits 6/6,
`make test`, roadmap verification, and `git diff --check`. This means only that inputs and local
provenance metadata are reproducible and fail closed. It does not prove a guest driver, guest-visible
GPU API, VirGL/OpenGL/GLES, Venus/Vulkan, browser rendering, compatibility, native comparison, remote
CI, or near-native performance.
