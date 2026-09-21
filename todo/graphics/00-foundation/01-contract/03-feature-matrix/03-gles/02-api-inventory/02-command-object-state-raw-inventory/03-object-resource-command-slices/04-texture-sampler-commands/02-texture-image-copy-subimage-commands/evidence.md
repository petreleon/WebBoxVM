# F03.3.2.2.3.4.2 evidence

Revision: 71cdaf8744f80d7dcf31206cc2d19f01d4873128
Validation: local focused and repository gates; no remote CI run was listed
Result: PASS
Artifacts: `gles_texture_image_copy_subimage_inventory.json`
(`a004f75129c8ee0c19399db50a588ca448904bb9a4f93eb1eb393e8cbb6bfe22`)
Profile: GLES 3.2 raw image/copy/subimage forms only; promotion disabled

Task ID and date: F03.3.2.2.3.4.2, 2026-09-21.

Tested code commit: `71cdaf8744f80d7dcf31206cc2d19f01d4873128`, parent
`627def853f3074ce6f15f7d2308dcd3d856bbc03`; patch SHA-256:
`bb70b9e3b82d53c91dfc74f08074fb624ed07ed8cb6b7ee66b53b3199cbcc20f`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The raw set has seven ordered texture literals: `TexImage3D`, `TexImage2D`,
`CopyTexImage2D`, `TexSubImage3D`, `TexSubImage2D`, `CopyTexSubImage3D`,
and `CopyTexSubImage2D`. Its raw-entry hash is
`87fc77cbc81204e62d0ea3ca81a693412b32201cef52a12c2d3e60f90e73df1b`.
It binds texture family order 17. Raw facts preserve the physical page spans
`[185, 187]` for `CopyTexImage2D` and `[191, 192]` for `TexSubImage3D`.
The intervening p.186 Figure 8.6 is sealed as a figure-only gap; the validator
requires the exact source fragments rather than a contiguous reconstruction.

Reproduce from repository root:

```sh
make graphics-gles-texture-image-copy-subimage-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/04-texture-sampler-commands/02-texture-image-copy-subimage-commands/gles_texture_image_copy_subimage_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 6/6; the CLI passed `7 bounded GLES
texture image/copy forms; source-only`. Full `make test` passed, including
1,127 Rust unit tests (3 ignored) and 338 web tests. Source limits passed 6/6;
diff and roadmap checks passed.

Negative checks reject missing/reordered forms; modified fragments, gap witness,
or source pages; fabricated continuous spans; compressed-family intrusion; stale
domain order/route; normalization drift; fabricated semantic fields; stale,
duplicate, nonfinite, or symlinked artifacts; unavailable ESSL substitutes; and
cache/private-loader substitution. The artifact is self-hashed and raw-only.

Decision and limits: this records only expanded spelling, formal declaration,
physical page/span, section, and source order. It creates no image allocation,
copy, pixel layout, framebuffer, format, dimensions, image content, guest/browser
execution, support, conformance, certification, or performance claim. It has no
runtime Matrix or CTS execution.

Commit/push verification: `71cdaf87` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list
--commit 71cdaf87 --limit 10` returned no runs.

Next ready task: F03.3.2.2.3.4.3.
