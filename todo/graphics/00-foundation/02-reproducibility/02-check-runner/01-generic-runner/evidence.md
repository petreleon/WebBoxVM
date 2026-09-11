# F05.1 evidence

Revision: `3fc946d497713901bbf1ab63e4d3e4ac931e270b` (clean feature commit)
Validation: focused runner, source limits, roadmap fixtures, full local `make test`, and diff/roadmap checks
Result: PASS
Artifacts: runner/test source SHA-256 values and hermetic JSON-receipt reproduction below
Profile: profile-independent local observation substrate; no graphics API or hardware support claim

## Snapshot

Task ID and date: F05.1, 2026-09-11.

Tested commit and dirty diff hash: `3fc946d497713901bbf1ab63e4d3e4ac931e270b`; the
working tree was clean before the receipt edit, so the tested dirty-diff SHA-256 is
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

Upstream manifest revision: N/A. This runner has no source inventory or profile binding.
Guest image/build, browser/OS/adapter/driver: N/A; no guest, browser, GPU, or performance lane ran.

## Commands and observations

Working directory: `/Users/petreleon/code/WebBoxVM`. Tool versions: Git 2.52.0,
Python 3.14.6, GNU Make 3.81. The runner stores Git HEAD, SHA-256 of
`git diff --no-ext-diff --binary HEAD`, declared tool output, command, duration,
exit status, counts, output digests, and artifact hashes in its JSON result.

```sh
make graphics-runner-test
cargo test -p emulator --test source_file_limits --quiet
python3 scripts/test_check_graphics_roadmap.py
python3 scripts/test_check_graphics_roadmap_blocked.py
make test
git diff --check
python3 scripts/check_graphics_roadmap.py
```

Expected result: the focused suite has five nonzero test cases; every normal runner
probe passes, and deliberately invalid or unavailable probes return their documented
nonzero result without becoming PASS. All repository checks exit zero.

Actual result: focused runner suite 5/0/0; source-file limits 6/0/0; roadmap fixtures
10/0/0 and blocked fixtures 4/0/0. `make test` passed its 5 runner cases, 1,127 Rust
tests with 0 failures and 3 ignored, and 338 Node tests with 0 failures/skips. The
roadmap checker passed before this status edit with F05.1 as its sole ready leaf; after
the receipt/status update it passed 356 documents, 211 tasks, 74 PASS-complete, and no ready leaf.

The positive probe writes `artifact.bin`, records its SHA-256, and requires observed
count 2. The negative probe preserves child exit 17 and both streams. Empty/unknown
selection and malformed catalog return 2; a zero count returns 1; missing executable,
asset, browser, hardware, and permission probes each produce JSON `BLOCKED`, observed
count 0, no child exit, and process exit 3. These are passing assertions about blocked
or failed observations, not claims that the represented prerequisites are available.

## Artifacts, decision, and limits

Tracked implementation hashes: `catalog.py`
`85b4de38fa682370670acbcd20137f76b5466c6cd18225ec355e921e791c3e0b`;
`prerequisites.py` `73e26e18947aee5a69a62b04bc2f7740adf9be006e9f7b3730af16028360cb99`;
`result.py` `bba7d2f19061a5b30d1665761496e5e89a948df40da9615a92240b35a070fe39`;
`run.py` `b98fefed1f6ef86d236ba15306302d54f8608b9cdbea892281ee1b12a55229f4`;
`runner.py` `be3b283d9d20b1b2a6ca38086e08e6176ea1745f4b52fc47d532b993a5c6742e`; and
`test_graphics_runner.py` `e323e3dcad216c540181350202abe95d521cf3f77d9a2b6ace1ca53042609d3f`.

The focused hermetic test creates and inspects the machine-readable JSON result, then
removes its temporary directory. Reproduce it with `make graphics-runner-test`; no
large capture is retained. No software fallback, native reference, conformance result,
browser route, GPU completion route, or performance measurement is implied.

First failing subcheck or blocker: none for F05.1. Synthetic missing-prerequisite
observations are expected test cases. The decision is to leave profile-bound catalog
registration to F05.2 after F02; this substrate must not advertise an API profile.

Commit/push verification: feature commit `3fc946d4` is local on
`codex/graphics-f01-baseline`. No push was attempted because no fresh push authorization
was given. Local validation is distinct from remote CI.

Next ready task: none; F05.2 awaits the still-incomplete F02 source-contract closure.
