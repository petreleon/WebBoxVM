# F02.4.4.1.5.4.4.5.2.2.2 evidence

Revision: 4dc168a4a35d57a4669dcc6d12a2bf75f9251cd4
Validation: real six-member capture and offline replay; focused 6/6; contract 3/3; F02 fetch policy 15/15; successor integration 3/3; boundary 3/3; source limits 6/6
Result: PASS
Artifacts: external cache marker, 2,562 bytes; self-record hash 3a19d46b46162f94a6d1bc96c300658d28eb42001931c63658d470c9e7214531
Profile: captured-unadmitted raw GLES selector closure; no producer, CTS, guest, browser, F03, conformance, certification, or performance execution

Task ID and date: F02.4.4.1.5.4.4.5.2.2.2, 2026-09-11 Europe/Bucharest.

## Capture receipt

The fresh external root was `/private/tmp/webboxvm-f02-gles-capture-20260911-4dc168a4`.
It contains six immutable raw source payloads totaling 960,982 bytes plus a 2,562-byte marker at
`webboxvm-graphics/f02-successor/gles-cts/fcca79aa08ca156cddca92b624e66513526109219b84b253bd513879f34419c4/capture.json`.
The full marker-file SHA-256 is `a2a9029c65eb121945a8c5d3f63418dafbbcda3fe9d6c1556a3d8d6e167edb3f`.
Its `marker_sha256` is deliberately different: it hashes the canonical marker body excluding that
self-hash field. Raw payload bytes and durable network logs are intentionally not committed.

| role | identity | bytes | SHA-256 |
| --- | --- | ---: | --- |
| root | gles-cts-manifest | 4,284 | 9f466f19a26120149bab06e70596c9fff57b61d767cfdbadc7ba13934ce2ea96 |
| core | gles-cts-gles2-khr-main | 38,127 | aed17d34d64047973c439835ee7cfa8a109c2159512fe1fe59fb07fc05d395f2 |
| core | gles-cts-gles3-khr-main | 443,126 | c79097f9f9c69a4556e235b3301c0dc9a5d73375c05f193a6f5e127d853ace72 |
| core | gles-cts-gles31-khr-main | 288,777 | 9fd8e4ff51616262c567f7a9eec83f69d2d311a4293a45d2d377306d426d520e |
| core | gles-cts-gles32-khr-main | 105,294 | 426fa31d557e08e214b75fda5d9efcaec54a42cdc14d4915939a7d02ccb6f249 |
| excluded extension | gles-cts-gles32-khr-glesext | 81,374 | 789b0474c61693baaa7cbc941575f23b383e2ba3c4991b1a910e046a17a6e0e6 |

All six marker references use the F02 payload grammar
`webboxvm-graphics/f02/{id}/{sha256}.source`; the separate `f02-successor` path names only the
closure marker. The validated contract binds each raw GitHub URL to commit
`067e8832315e79817ede1c4863804e440f5d1c80`, Apache-2.0 repository terms, provenance, exact byte
count, digest, and the 8,388,608-byte member cap. The marker binds contract SHA-256
`63e41c35e1f78f7eecee9241153f3cf2135cda48ca674d399a28b4310d501899`, closure SHA-256
`fcca79aa08ca156cddca92b624e66513526109219b84b253bd513879f34419c4`, and configuration document
SHA-256 `ed27d6d675494a4773bb55533dc0d2916d1486d49450d3feb94743e38727a64e`.

The capture command first requires the external root not to exist. It writes only verified regular
files, reopens them through descriptor-bound no-follow paths, validates the complete root XML, then
publishes the self-hashed marker last. The parser accepts precisely 12 ordered core configurations
and one explicit excluded configuration. `producer_execution_proved`, `output_attestation_present`,
and every support effect remain false.

The initial sandboxed request could not resolve the hostname and created no cache entry. The authorized
retry completed the fresh capture; its replay command used no fetch call or transport path and returned
the same marker. The cache remains external and has not been made an active F02 cache or F03 input.

## Commands and hostile checks

From `/Users/petreleon/code/WebBoxVM`:

```text
capture_dir=todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/04-closure-admission-proof/04-admission-eligible-closure/05-independent-suite-closures/02-gles-successor-closure/02-authorized-capture/02-capture-and-replay
python3 -B "$capture_dir/gles_capture.py" --capture --cache-root /private/tmp/webboxvm-f02-gles-capture-20260911-4dc168a4 --timeout 30
PASS: capture captured-unadmitted .../capture.json

python3 -B "$capture_dir/gles_capture.py" --replay --cache-root /private/tmp/webboxvm-f02-gles-capture-20260911-4dc168a4
PASS: replay captured-unadmitted .../capture.json
```

`gles_capture_test.py` passed 6/6. Its positive case proves six fetches, marker-last publication,
and a replay whose fetch function is replaced by an immediate failure. Its hostile cases reject an
unsafe or reordered root, root-only closure, substituted payload, marker republish/mutation, and
source or marker symlink/FIFO. The contract test passed 3/3; predecessor integration and independent
suite-boundary tests each passed 3/3; the F02 fetch-policy suite passed 15/15; source-file limits
passed 6/6. Final `make test` passed 1,130 Rust tests with 0 failures and 3 ignored, then 337 Node
tests with 0 failures. `git diff --check` and the roadmap checker were clean; the latter reports 338
documents, 202 tasks, 66 PASS-complete, and 27 superseded.

Reproduction file SHA-256 values: `gles_capture.py`
`fb248c678398de5f4bb50c1b5a3d76d15f2f285f959c5db238090d19f853b370`; `gles_capture_marker.py`
`83f60cc24cf38aa8536965825ba7a696d772b4b2fb76d47d0238d03cedc23a32`; `gles_capture_plan.py`
`11765088f856a453597277175e30c17f9ae30ae4c360a9ced97ae4b4f7adae81`; `gles_capture_test.py`
`0dbf11173454f0d231b267ad6466c38689ad76adddf2fb3122eb1fb9ee757207`.

Decision and limits: PASS proves a bounded immutable raw-source capture and offline cache replay
only. It does not admit a source, prove producer output, change active inventory or F03, run a CTS,
expose a guest API, execute a browser renderer, establish conformance/certification, or measure
performance. The next required child is atomic consumer revalidation; Vulkan's independent
Khronos-manifest blocker remains unchanged.
