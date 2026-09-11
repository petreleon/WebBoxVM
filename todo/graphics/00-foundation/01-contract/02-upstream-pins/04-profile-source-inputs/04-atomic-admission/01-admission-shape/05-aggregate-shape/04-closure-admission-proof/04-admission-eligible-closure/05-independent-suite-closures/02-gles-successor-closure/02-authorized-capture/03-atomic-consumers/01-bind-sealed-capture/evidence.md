# F02.4.4.1.5.4.4.5.2.2.3.1 evidence

Revision: ba796557ec6d5ae9d06eebb6deed9fd9fe0e45c2
Validation: binding 4/4; real offline replay; capture 6/6; successor integration 3/3
Result: PASS
Artifacts: self-checked `gles-sealed-capture-binding-v1`, SHA-256 `42a221ba44ad7f44aefc40fe876933000b452284d659db00bed5d34da6a525e7`
Profile: a sealed raw GLES capture only; captured-unadmitted, with every active, admission, support, conformance, certification, and performance effect false

Task ID and date: F02.4.4.1.5.4.4.5.2.2.3.1, 2026-09-11 Europe/Bucharest.

## Bound identity

The committed binding relates successor-integration SHA-256
`d22eeadf9da86ccd79493e348d7809d090d01dbf91c9a4e4316b6db92d2caf7c` to the exact six-member
GLES capture: contract `63e41c35e1f78f7eecee9241153f3cf2135cda48ca674d399a28b4310d501899`, closure
`fcca79aa08ca156cddca92b624e66513526109219b84b253bd513879f34419c4`, and configuration document
`ed27d6d675494a4773bb55533dc0d2916d1486d49450d3feb94743e38727a64e`.

It records the separate successor marker path
`webboxvm-graphics/f02-successor/gles-cts/fcca79aa08ca156cddca92b624e66513526109219b84b253bd513879f34419c4/capture.json`,
the marker's semantic self-hash
`3a19d46b46162f94a6d1bc96c300658d28eb42001931c63658d470c9e7214531`, and its exact physical-byte
SHA-256 `a2a9029c65eb121945a8c5d3f63418dafbbcda3fe9d6c1556a3d8d6e167edb3f`. The latter prevents a
semantically equivalent but physically substituted marker from satisfying the binding.

The ordered identities remain `gles-cts-manifest`, `gles-cts-gles2-khr-main`,
`gles-cts-gles3-khr-main`, `gles-cts-gles31-khr-main`, `gles-cts-gles32-khr-main`, and
`gles-cts-gles32-khr-glesext`. Payloads are not committed; the marker remains a successor namespace,
while every payload path retains the frozen F02 grammar.

## Commands and hostile checks

From `/Users/petreleon/code/WebBoxVM`:

```text
bind_dir=todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/04-closure-admission-proof/04-admission-eligible-closure/05-independent-suite-closures/02-gles-successor-closure/02-authorized-capture/03-atomic-consumers/01-bind-sealed-capture
python3 -B -m unittest -v "$bind_dir/sealed_capture_binding_test.py"
Ran 4 tests ... OK

python3 -B "$bind_dir/sealed_capture_binding.py" --verify
BINDING: captured-unadmitted 42a221ba44ad7f44aefc40fe876933000b452284d659db00bed5d34da6a525e7

python3 -B "$bind_dir/sealed_capture_binding.py" --replay --cache-root /private/tmp/webboxvm-f02-gles-capture-20260911-4dc168a4
BINDING: captured-unadmitted 42a221ba44ad7f44aefc40fe876933000b452284d659db00bed5d34da6a525e7
```

The real replay reuses the external sealed capture and has no fetch/capture transport path. The four
focused tests cover deterministic read-only reconstruction; duplicate, oversized, FIFO, and symlink
records; mandatory replay without fetching; partial replay; contract/configuration/marker/member
substitution; claimed readiness; type tricks; and semantic or raw-byte marker forgery. A focused
security review found the raw-marker-digest gap during development; replay now reopens the marker via
descriptor-pinned no-follow reads and verifies its recorded physical digest. Re-review found no P1/P2.

The predecessor capture suite passed 6/6 and successor integration passed 3/3 in this checkpoint.
No active F02 inventory, F03 input, cache freshness, admission, CTS execution, guest API, browser
renderer, conformance/certification, or performance state changed. The next required child is
[F02/F03 consumer revalidation](../02-revalidate-consumers/README.md); its result must still preserve
the independent Docs and VCTS blockers.
