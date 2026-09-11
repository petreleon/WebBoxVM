# F02.4.4.1.5.4.4.5.2.2.3.2 evidence

Revision: 0ae4dddf5f1de354819105074d073b1935809532
Validation: consumer revalidation 4/4; binding 4/4; capture 6/6; successor integration 3/3; F02 fetch policy 15/15; F03 expected-blocked result; reproducible seed builds
Result: PASS
Artifacts: self-checked `f02-f03-consumer-revalidation-v1`, SHA-256 `d858ec3957a989da9b471b925ba6f5d4445f910a810ab3c3d897945394aa80c6`
Profile: unchanged blocked consumers; no active inventory, F03, cache-freshness, admission, support, conformance, certification, or performance effect

Task ID and date: F02.4.4.1.5.4.4.5.2.2.3.2, 2026-09-11 Europe/Bucharest.

## Atomic unchanged boundary

The result binds the sealed GLES capture binding `42a221ba44ad7f44aefc40fe876933000b452284d659db00bed5d34da6a525e7`
and raw binding document `f765b917f8144288205fad737d68e5efd02e0665f0f8e140f0ca138ad32740aa` to
closure `fcca79aa08ca156cddca92b624e66513526109219b84b253bd513879f34419c4`. It calls only its
predecessor's `validate()`: no capture, replay, fetch, cache write, or active-inventory mutation occurs.

The active F02 v2 lock remains `08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6`,
with manifest `f6b4b8bc18751d4645cf638faa4958b9ddb01c5e12815ad599f7eb7ce26c4adb`, two exact part
digests, and 17 active inputs. The historical blocked receipt remains exactly
`9ff372bcfeb83538882da396b2ecbb0801703ca55a1b0dd38fa7ab51d4bf501a`; it is not promoted or
rewritten by this child. Every captured GLES identity is outside F02 by both ID and `(SHA-256, bytes)`.

F03 remains `inventory-sources-incomplete`, with its scope and requirements digests unchanged and this
ordered gap list: `opengl-46-core-spec`, `opengl-cts-manifest`, `gles-32-spec`, `gles-cts-manifest`,
`vulkan-14-spec`, `vulkan-cts-mustpass`. All three profile rows stay `blocked`. This result does not
resolve the independent Vulkan Docs or VCTS blockers.

## Commands and hostile checks

From `/Users/petreleon/code/WebBoxVM`:

```text
consumer_dir=todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/04-closure-admission-proof/04-admission-eligible-closure/05-independent-suite-closures/02-gles-successor-closure/02-authorized-capture/03-atomic-consumers/02-revalidate-consumers
env -u PYTHONPATH python3 -B -m unittest -v "$consumer_dir/consumer_revalidation_test.py"
Ran 4 tests ... OK

env -u PYTHONPATH PYTHONHASHSEED=1 python3 -B "$consumer_dir/consumer_revalidation.py" --build
env -u PYTHONPATH PYTHONHASHSEED=2 python3 -B "$consumer_dir/consumer_revalidation.py" --build
byte-identical output

env -u PYTHONPATH python3 -B todo/graphics/00-foundation/01-contract/03-feature-matrix/01-profile-scope/validate_profile_scope.py
BLOCKED: missing required inventory inputs: ...six ordered IDs above
exit 3 (expected)
```

The hostile cases reject a resealed promotion, lock or binding substitution, active ID or content alias,
F03 omission/reordering/substitution, partial historical readiness, duplicate keys, oversized, FIFO,
leaf and intermediate-directory symlinks, `NaN`, and an oversized JSON integer. Reads walk every path
component through no-follow directory descriptors and bind the leaf identity before and after its bounded
read. A review found and closed the raw-lock, alias, ancestor-link, and parser failure paths; no P1/P2
remained under the declared trusted-checkout model.

The verifier runs in a fresh controlled Python interpreter without untrusted `PYTHONPATH` or preloaded
modules. The checkout's reviewed code is trusted; JSON receipts and declared data paths are adversarial.
This is not a claim that an attacker may safely replace the verifier's own Python source.

Decision and limits: PASS proves the sealed capture has not silently changed active F02/F03 consumers.
It does not admit sources, prove cache freshness, run a CTS, expose a guest API, execute a browser
renderer, establish conformance/certification, or measure performance. The next child must reconcile
this same blocked result with the independently unadmitted Docs and VCTS boundaries.
