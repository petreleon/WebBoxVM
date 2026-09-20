# F05 aggregate evidence

Revision: `17da1b1987239ad6655d66171cab2913038e2399`
Validation: F05.1 generic runner tests, F05.2 registration tests, live auxiliary registry receipt, full local `make test`, source limits, diff and roadmap checks
Result: PASS
Scope: reproducible observation and fail-closed registration; not graphics compatibility
Artifacts: [F05.1 receipt](01-generic-runner/evidence.md); [F05.2 receipt](02-profile-bound-registration/evidence.md)
Profile: all API, guest, browser, conformance, certification, and performance states remain false or blocked

F05.1 supplies the profile-neutral executor: it preserves command, output, artifacts,
tool versions, nonzero observed counts, and missing prerequisites. F05.2 supplies the
sealed profile-registration layer without changing that generic executor. It requires
the exact F02 contract/lock, rejects cross-profile pairs and auxiliary promotion, and
keeps `profile_implementation_count: 0` until an independently reviewed semantic matrix
leaf exists.

The current registry registration is an auxiliary 1,458-row structural diagnostic; it
does not make Vulkan supported or CTS-covered. Build and transport lanes are separately
registered as non-profile observations. A missing cache, browser, hardware, guest image,
or CTS suite is recorded as `BLOCKED`, never as PASS. A later F03 semantic leaf must bind
its own normative/full-suite pair before F05 can register a profile implementation check.

No guest API compatibility, browser rendering, Khronos conformance/certification, or
near-native performance result follows from F05. Commit/push verification is recorded
after the validated feature commit.
