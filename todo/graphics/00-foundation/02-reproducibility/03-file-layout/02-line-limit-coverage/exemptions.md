# F06.2 line-limit exemption ledger

The source-limit test scans every maintained eligible file under `emulator`,
`models`, `scripts`, `web`, `todo`, `guest`, `research`, and `patches`, plus the
listed repository-root files. It recognizes C, headers, linker scripts,
Makefiles, Markdown, patches, and the existing source extensions.

Only these exact paths are exempt. A similarly named file elsewhere fails the
check; the test creates `guest/LICENSE.md` and `patches/other.patch` to prove it.

| Path | Why it is exempt | Reviewed provenance |
| --- | --- | --- |
| `LICENSE.md` | 640-line legal text, not implementation source. | Repository license, SHA-256 `e38cab12de8b5ed00e5f97235aa5e1b82ff0836fe01851962989d5395496a4e5`. |
| `patches/wasm-bindgen-memory64-threads.patch` | 518-line immutable patch applied to a separately pinned upstream checkout. | wasm-bindgen `ddd322514d87a4b21342b7ab9a9d70796fc60576`, SHA-256 `2e5e4d704f59a177039ee608cf2005a7c07d62ef3e1a5e2b9fe13bd5a10d2d21`; [build script](../../../../../../scripts/build_wasm_bindgen_memory64_threads.sh) derives `PATCH_ID` from its bytes. |

`web/pkg` and `web/pkg-threaded` are exact generated package-output directories,
not maintained-source exemptions. No generic generated-file or generated-directory
rule exists; any other eligible source file in a scanned root remains checked.

Changing either exempt file requires review of this ledger and its provenance
identifier. New legal text or a new third-party patch needs a separate explicit
entry; it is not covered by either path above.
