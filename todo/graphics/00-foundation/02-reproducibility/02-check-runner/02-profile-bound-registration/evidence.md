# F05.2 evidence

Revision: `17da1b1987239ad6655d66171cab2913038e2399`
Validation: `make graphics-profile-registration-test`; live registry registration; `make test`; source limits; diff and roadmap checks
Result: PASS
Scope: no-claim registration gate; all profile states remain `blocked` / `matrix-incomplete`
Artifacts: catalog self-hash `e390118a78145dac3c5c080413317f55659ba202fc536bae377b6c2b81556cf6`; live receipt `/private/tmp/webboxvm-f052-registry-result.json` SHA-256 `f3286e69d012b862d7598192b24357193c2cb179e9f409cc3ca76dd0ca19e5ab`
Profile: no semantic implementation is registered; `profile_implementation_count: 0`

Task ID and date: F05.2, 2026-09-21.

The catalog is self-hashed and binds the sealed F02 source-contract hash
`d2be08ced8a806f001543e9218b6758a0a4b89825b0c4c940b9ced7c119f1ac3`
and inventory lock `44a0f280e0ed854091a33e458122bd8cd9a0f9fc2e4c51a7a5be92bf48d8c6f4`.
It accepts only an exact same-profile normative/full-suite pair for a future
implementation check. Current registrations are the Vulkan registry as
`auxiliary-inventory` with `profile_effect: none`, plus independent `make test`,
serial Wasm, threaded Wasm, and transport-only guest lanes with `profile: null`.

Focused schema tests pass 6/6; execution-boundary tests pass 3/3. They reject stale
headers, bad self-hash, cross-profile source pairs, auxiliary-role promotion, qualifying
source evidence, empty command/count/artifact/evidence, and a profile-null baseline
promotion. Missing/relative selector cache, browser, hardware, guest image, and CTS
fixtures return `BLOCKED` before the child runs. A child failure preserves its streams
and exit code.

Live command, from the repository root:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 \
  todo/graphics/00-foundation/02-reproducibility/02-check-runner/02-profile-bound-registration/profile_registration_run.py \
  --select vulkan-registry-inventory-v2 \
  --selector-cache-root /private/tmp/webboxvm-f0341.cqT6ZX \
  --result /private/tmp/webboxvm-f052-registry-result.json
```

It exited 0 with `LIVE: 1458 blocked rows` and one observed-count marker of 1,458.
Its receipt is an expected PASS for the auxiliary diagnostic only: all qualification
claims are false, CTS executions are zero, and Vulkan profile status remains blocked.

No guest Mesa application, browser GPU path, device, CTS case, certification, native
comparison, or performance measurement ran. The generic `make test` lane is not invoked
by this F05 test target, avoiding recursive validation. Commit/push verification is
recorded after the validated feature commit.
