# F02.3.1 evidence

Revision: `16cd943eb67fa3249850870f2abc079dbe27b971` feature implementation
Validation: hermetic provenance and inventory parsers, full `make test`, and final structural checks
Result: PASS
Artifacts: no upstream bytes, ABI fixture sidecars, or generator outputs are added
Profile: record contract only; F02.3.2/.3.3 bindings and F02.3.4 fresh-cache closure remain open

## Active v2 authority

The active F02.3 contract now accepts only schema-v2 sidecars bound to the exact raw
[`inventory.lock`](../../01-input-inventory/inventory.lock) SHA-256
`cb85958df7f8e2a9b6b749f6218621d6b24b1f6de5dc9f5be3468ab5fb7ddd5b`. A legacy one-file
inventory or schema-v1 sidecar is rejected before it can become an active binding.

## Historical v1 contract result

The validator computes the F02.1 manifest revision as SHA-256 of its exact raw bytes. It then accepts
only an exact JSON schema whose manifest digest matches that revision and whose sorted, unique input
references exactly match declared F02.1 IDs, SHA-256 values, and license strings. It rejects stale
manifest/input identities, unknown or duplicate references, malformed JSON, missing fields, and
wrong scalar types before a record is accepted.

At the recorded v1 implementation, the canonical authority was [F02.1's manifest](../../01-input-inventory/manifest.toml),
not an upstream commit, fetched cache, or a real artifact record. This leaf validates generic record
schema only; F02.3.2 and F02.3.3 own actual ABI and generator-output bindings.

Origin rules make the record honest: generated output needs a non-`none` generator identity/version;
copied upstream output must equal the one declared input digest; handwritten output cannot reuse a
declared input digest and must name generator `none`. The generic test record is not a claim about a
real ABI adapter or generated protocol artifact.

## Commands and actual results

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/01-provenance-record/provenance_record_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory/validate_manifest.py --self-test
PYTHONDONTWRITEBYTECODE=1 make test
cargo test -p emulator --test source_file_limits --quiet
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
git diff --check
```

Actual before completion markers: provenance tests ran 9/9 and inventory tests 4/4. The test used a
hard-coded then-current manifest SHA-256 and Linux UAPI ID/digest/license to prove a valid handwritten
sample binds the inventory; it rejects missing fields, bool schema, unknown/stale references, stale
license, dishonest origin, malformed JSON, duplicate, and unsorted references. `make test` passed
1,151 Rust tests with 0 failures and 3 ignored, plus 337 Node tests with 0 failures, cancellations,
skips, or todos. Source limits passed 6/6 and `git diff --check` exited zero.

No network, external cache, fresh fetch, ABI fixture binding, generator output binding, or graphics
runtime behavior is claimed. After the completion markers were applied, source limits passed 6/6, the
roadmap checker printed `PASS: 160 documents, 102 tasks, 14 complete; links/dependencies/limits
valid` and `Ready: F02.3.2, F02.3.3`, and `git diff --check` exited zero.

The feature implementation was committed as `16cd943eb67fa3249850870f2abc079dbe27b971` and pushed
to `origin/codex/graphics-f01-baseline`; immediately after the push, `git ls-remote origin
refs/heads/codex/graphics-f01-baseline` returned that exact SHA. No remote CI result is asserted.
