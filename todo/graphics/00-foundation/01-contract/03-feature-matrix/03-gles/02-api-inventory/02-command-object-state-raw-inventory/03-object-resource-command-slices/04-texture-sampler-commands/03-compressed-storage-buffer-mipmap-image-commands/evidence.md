# F03.3.2.2.3.4.3 evidence

Revision: 6bcb92fa9c5ddb37c366cbe0fb7043b72451eef6
Validation: local focused and repository gates; no remote CI run was listed
Result: PASS
Artifacts: `gles_texture_extended_command_inventory.json`
(`396af96ec12d71d32421889a8984f0ca1a700ae89d89eb2a812c83f00734cc0d`)
Profile: GLES 3.2 raw extended texture literals only; promotion disabled

Task ID and date: F03.3.2.2.3.4.3, 2026-09-21.

Tested code commit: `6bcb92fa9c5ddb37c366cbe0fb7043b72451eef6`, parent
`69cfa9ca4be1d8f553652639028a81425bdcb110`; patch SHA-256:
`189c18768af981dc6a7106c6cd42665b912c4bbf0b3d06e88391f6a8df99e912`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`, authority hash
`0e7ee3270ca2c527ffafb366f216839a8d180dfc3777d2ef2b2a2a532023c7ae`.
The retained cache boundary is
`c32574099c24c6d6b78b43ef82cf88869853353cdac619ca9f0b8b58100b5f79`.
Guest image/build, browser/adapter/driver: not applicable to this source-only task.

The raw set has 12 ordered texture literals: `CompressedTexImage2D`,
`CompressedTexImage3D`, `CompressedTexSubImage2D`, `CompressedTexSubImage3D`,
`TexStorage2DMultisample`, `TexStorage3DMultisample`, `TexBufferRange`,
`TexBuffer`, `GenerateMipmap`, `TexStorage2D`, `TexStorage3D`, and
`BindImageTexture`. Its raw-entry hash is
`633cb81648103af03496c3ed718e46ab23ff373e988ca5aabec0bed6ecb53bdb`.
It binds texture family order 17 and seals exact source fragments on pp.195,
199, 201, 203–204, 222, 226–227, and 233, under sections 8.7, 8.8, 8.9,
8.14.4, 8.18, and 8.23.

Reproduce from repository root:

```sh
make graphics-gles-texture-extended-command-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/04-texture-sampler-commands/03-compressed-storage-buffer-mipmap-image-commands/gles_texture_extended_command_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 6/6; the CLI passed
`12 bounded GLES texture extended forms; source-only`. Full `make test` passed,
including 1,127 Rust unit tests (3 ignored) and 338 web tests. Source limits
passed 6/6; diff and roadmap checks passed.

Negative checks reject missing/reordered forms; altered source pages, headings,
sections, declarations, or normalizer output; cross-route/domain drift;
image/copy/parameter-family intrusion; semantic promotion; duplicate, nonfinite,
or symlinked artifacts; unavailable ESSL/registry substitutes; and private-loader
or cache substitution. The artifact is self-hashed and raw-only.

Decision and limits: this records only expanded spelling, formal declaration,
physical page, section, and source order. It creates no compression, allocation,
storage, buffer, mipmap, image access, synchronization, format, pixel-layout,
guest/browser execution, support, conformance, certification, or performance claim.
It has no runtime Matrix or CTS execution.

Commit/push verification: `6bcb92fa` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list
--commit 6bcb92fa --limit 10` returned no runs.

Next ready task: F03.3.2.2.3.4.4.
