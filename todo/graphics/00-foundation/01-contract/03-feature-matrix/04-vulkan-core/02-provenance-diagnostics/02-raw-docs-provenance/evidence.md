# F03.4.2.2 evidence

Revision: `84c66f083dd6d37a3513d2199fbbe07ab77d4658`
Validation: focused hostile suite, fresh immutable raw-fragment replay, and final local project gate
Result: PASS
Artifacts: `vulkan_raw_docs_citations.json` SHA-256 `ee3fe3a2e624eb2ac07eb8042992a5d332b9ce0f95ab51ab98f741790243ff02` (5,438 B)
Profile: Vulkan 1.4 core citation-only planning; `matrix-incomplete`

Task ID and date: F03.4.2.2, 2026-09-21 Europe/Bucharest.

The source set became `84c66f08`. Fresh cache `/private/tmp/webboxvm-f03422-final.FYptCW` is external to
the repository and contains exactly the cache manifest, F02 root `vkspec.adoc`, and two reviewed raw
fragments at immutable revision `f84d432d5b8912362f96f581f29bbc4f3c8c7843`:

- `vkspec.adoc`: 8,685 B, SHA-256 `069b7e6d6326969df7b4a86f189f7ba22359e93ca7aa76c23301504667d3c4b0`.
- `chapters/introduction.adoc`: 15,959 B, SHA-256 `a8cff7be42b43f9cab6c150af845587d4d4f3758a4266972290735ed55ac9767`, anchor `[[introduction]]`.
- `chapters/fundamentals.adoc`: 91,224 B, SHA-256 `70dde6bad83bd48c3438eb96a9cfc3387f53dfbdd8a1cae33a87dab1310cc0d3`, anchor `[[fundamentals-execmodel]]`.

The map is self-hashed as `2d6b3270cd4af8d14d752d593cd226c54d953aa034ce3f23cb221eecc0b22e44` and
contains two `citation_candidates`. Their relation is `same-revision-raw-fragment; inclusion-unproven`:
this is not an include closure, rendered Docs tree, source-coverage result, or normative-root promotion.

Commands from the repository root:

- `make graphics-vulkan-raw-docs-citations-test` — 6/6 passed.
- `PYTHONDONTWRITEBYTECODE=1 python3 vulkan_raw_docs_citations.py --cache-root /private/tmp/webboxvm-f03422-final.FYptCW` — exit 0; two candidates revalidated.
- `make test` — exit 0; all focused new suites passed 30/30, Rust reported 1,130 cases (3 ignored), and Node reported 338 pass / 0 fail.
- `cargo test -p emulator --test source_file_limits --quiet` — 6/6 passed.
- `git diff --check` and `PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py` — exit 0; pre-receipt structure was 441 documents / 265 tasks.

Hostile checks reject mixed revisions/manifests, bad licenses, absent/ambiguous anchors, generated paths,
role promotion, registry locators, symlink/FIFO/cache escapes, and ambient loader decoys. No Matrix row,
CTS run, guest/browser/GPU behavior, conformance, certification, or performance result exists; all claims
remain false. First failing subcheck: none.

Commit/push verification: `84c66f08` was pushed to `codex/graphics-f01-baseline` and `git ls-remote`
matched it. `gh run list --commit` returned no run. Next ready task: F03.4.2.4.
