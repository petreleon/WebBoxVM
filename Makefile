ARTIFACTS_DIR ?= .artifacts
DEBIAN_ARM64_ISO ?= $(ARTIFACTS_DIR)/debian-arm64-netinst.iso
DEBIAN_ARM64_ISO_BASE ?= https://cdimage.debian.org/debian-cd/current/arm64/iso-cd
DEBIAN_ARM64_ISO_FILE ?=
WEB_BENCHMARK_DISK ?= output/webboxvm-final-install-compact.wbdisk
WEB_BENCHMARK_SHA256 ?= 97d819803774d67c9aabaa19f336f066656cc5235b5e8276cb8dc14fdff6217d
DRY_RUN ?= 0
WEB_PORT ?= 8080
WEB_TARGET ?= wasm64-unknown-unknown
WEB_TOOLCHAIN ?= +nightly
WEB_CARGO_FLAGS ?= -Z build-std=std,panic_abort
WEB_THREADS_RUSTFLAGS ?= -C target-feature=+atomics,+bulk-memory -C link-arg=--shared-memory -C link-arg=--max-memory=4294967296 -C link-arg=--import-memory -C link-arg=--export=__wasm_init_tls -C link-arg=--export=__tls_size -C link-arg=--export=__tls_align -C link-arg=--export=__tls_base -C link-arg=--export=__heap_base
WASM_BINDGEN_THREADS_ROOT ?= $(ARTIFACTS_DIR)/tools/wasm-bindgen-memory64-threads
WASM_BINDGEN_THREADS ?= $(WASM_BINDGEN_THREADS_ROOT)/bin/wasm-bindgen

.PHONY: busybox iso-debian-arm64 iso-info terminal-image terminal-debian-arm64 terminal-iso wasm-bindgen-memory64-threads web-pkg web-pkg-serial web-pkg-threaded web web-benchmark web-debian-arm64 graphics-roadmap-test graphics-runner-test graphics-source-role-test graphics-normative-root-test graphics-vulkan-definition-test graphics-source-admission-test graphics-full-suite-root-test graphics-gl-flat-ledger-test graphics-gles-full-closure-ledger-test graphics-gl-gles-ledger-receipt-test graphics-vulkan-ledger-taxonomy-test graphics-vulkan-cache-replay-test graphics-vulkan-full-suite-receipt-test graphics-vulkan-local-shard-receipt-test graphics-role-aware-source-contract-test graphics-profile-source-gate-test graphics-opengl-source-authority-test graphics-opengl-normative-pdf-cache-test graphics-opengl-command-object-raw-inventory-test graphics-opengl-state-raw-slice-test graphics-opengl-limit-format-raw-inventory-test graphics-opengl-unadmitted-ledger-test graphics-gles-source-authority-test graphics-gles-unavailable-ledger-test graphics-vulkan-registry-inventory-test graphics-vulkan-source-channel-boundary-test graphics-vulkan-raw-docs-citations-test graphics-vulkan-full-suite-diagnostics-test graphics-f05-source-adapter-test graphics-profile-registration-test test

busybox:
	scripts/update_busybox.sh

iso-debian-arm64:
	DEBIAN_ARM64_ISO_OUT="$(DEBIAN_ARM64_ISO)" \
	DEBIAN_ARM64_ISO_BASE="$(DEBIAN_ARM64_ISO_BASE)" \
	DEBIAN_ARM64_ISO_FILE="$(DEBIAN_ARM64_ISO_FILE)" \
	DRY_RUN="$(DRY_RUN)" \
	scripts/download_debian_arm64_iso.sh

iso-info: iso-debian-arm64
	cargo run -p emulator --example iso_info -- $(DEBIAN_ARM64_ISO)

terminal-image:
	cargo run -p emulator --example terminal --release -- $(ARTIFACTS_DIR)/Image

terminal-debian-arm64: iso-debian-arm64
	cargo run -p emulator --example terminal --release -- $(DEBIAN_ARM64_ISO)

terminal-iso:
	@test -n "$(ISO)" || (echo "usage: make terminal-iso ISO=path/to/arm64.iso" >&2; exit 2)
	cargo run -p emulator --example terminal --release -- "$(ISO)"

wasm-bindgen-memory64-threads: $(WASM_BINDGEN_THREADS)

$(WASM_BINDGEN_THREADS): scripts/build_wasm_bindgen_memory64_threads.sh patches/wasm-bindgen-memory64-threads.patch
	WASM_BINDGEN_INSTALL_ROOT="$(abspath $(WASM_BINDGEN_THREADS_ROOT))" scripts/build_wasm_bindgen_memory64_threads.sh

web-pkg:
	node scripts/stamp_web_asset_version.mjs --check
	$(MAKE) web-pkg-serial
	$(MAKE) web-pkg-threaded

web-pkg-serial: $(WASM_BINDGEN_THREADS)
	cargo $(WEB_TOOLCHAIN) build -p emulator --release --target $(WEB_TARGET) $(WEB_CARGO_FLAGS) --features wasm
	$(WASM_BINDGEN_THREADS) target/$(WEB_TARGET)/release/emulator.wasm --out-dir web/pkg --target web

web-pkg-threaded: $(WASM_BINDGEN_THREADS)
	RUSTFLAGS='$(WEB_THREADS_RUSTFLAGS)' cargo $(WEB_TOOLCHAIN) build -p emulator --release --target $(WEB_TARGET) $(WEB_CARGO_FLAGS) --features wasm
	$(WASM_BINDGEN_THREADS) target/$(WEB_TARGET)/release/emulator.wasm --out-dir web/pkg-threaded --target web

web: web-pkg
	python3 scripts/serve_web.py --port $(WEB_PORT) --directory web

web-benchmark: web-pkg
	@test -f "$(WEB_BENCHMARK_DISK)" || (echo "benchmark disk not found: $(WEB_BENCHMARK_DISK)" >&2; exit 2)
	@printf '%s  %s\n' "$(WEB_BENCHMARK_SHA256)" "$(WEB_BENCHMARK_DISK)" | shasum -a 256 -c -
	mkdir -p web/media
	ln -sf "$(abspath $(WEB_BENCHMARK_DISK))" web/media/benchmark-installed.wbdisk
	@trap 'unlink web/media/benchmark-installed.wbdisk 2>/dev/null || true' EXIT HUP INT TERM; \
		python3 scripts/serve_web.py --port $(WEB_PORT) --directory web

web-debian-arm64: iso-debian-arm64 web-pkg
	mkdir -p web/media
	ln -sf "$(abspath $(DEBIAN_ARM64_ISO))" web/media/debian-arm64-netinst.iso
	python3 scripts/serve_web.py --port $(WEB_PORT) --directory web

graphics-runner-test:
	PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_graphics_runner.py

graphics-roadmap-test:
	PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_check_graphics_roadmap.py
	PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_check_graphics_roadmap_blocked.py
	PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_check_graphics_roadmap_supersession.py

graphics-source-role-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/01-authority-and-transform-boundary/source_role_contract_test.py
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/01-authority-and-transform-boundary/source_role_artifacts_test.py

graphics-normative-root-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/02-normative-build-record/01-normative-root-pins/normative_roots_test.py
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/02-normative-build-record/01-normative-root-pins/normative_notices_test.py

graphics-vulkan-definition-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/02-normative-build-record/02-vulkan-local-definition/webboxvm_source_builder_test.py
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/02-normative-build-record/02-vulkan-local-definition/vulkan_definition_contract_test.py

graphics-source-admission-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/02-normative-build-record/03-admission-receipt/admission_receipt_test.py

graphics-full-suite-root-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/03-full-suite-record/01-canonical-full-suite-roots/full_suite_roots_test.py

graphics-gl-flat-ledger-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/03-full-suite-record/02-gl-gles-full-ledgers/01-gl-flat-ledger/gl_flat_ledger_test.py

graphics-gles-full-closure-ledger-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/03-full-suite-record/02-gl-gles-full-ledgers/02-gles-full-closure-ledger/gles_full_ledger_test.py

graphics-gl-gles-ledger-receipt-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/03-full-suite-record/02-gl-gles-full-ledgers/03-gl-gles-no-claim-receipt/gl_gles_receipt_test.py

graphics-vulkan-ledger-taxonomy-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/03-full-suite-record/03-vulkan-default-full-ledger/01-vulkan-ledger-taxonomy/vulkan_ledger_taxonomy_test.py

graphics-vulkan-cache-replay-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/03-full-suite-record/03-vulkan-default-full-ledger/02-streaming-cache-replay/vulkan_cache_replay_test.py

graphics-vulkan-full-suite-receipt-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/03-full-suite-record/03-vulkan-default-full-ledger/03-fresh-full-suite-receipt/vulkan_full_suite_receipt_test.py

graphics-vulkan-local-shard-receipt-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/03-full-suite-record/04-local-shards-and-no-claim-receipt/webboxvm_byte_preserving_shard_builder_test.py
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/03-full-suite-record/04-local-shards-and-no-claim-receipt/vulkan_local_shard_receipt_test.py

graphics-role-aware-source-contract-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract/role_aware_source_contract_test.py
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract/role_aware_source_cache_test.py

graphics-profile-source-gate-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/01-profile-scope/validate_profile_scope_test.py
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/01-profile-scope/validate_profile_scope_v2_test.py

graphics-opengl-source-authority-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/01-source-authority/opengl_source_authority_test.py

graphics-opengl-normative-pdf-cache-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/02-command-object-state-inventory/01-normative-pdf-cache/opengl_normative_pdf_cache_test.py

graphics-opengl-command-object-raw-inventory-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/02-command-object-state-inventory/02-command-object-raw-inventory/opengl_command_object_raw_inventory_test.py

graphics-opengl-state-raw-slice-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/02-command-object-state-inventory/03-state-and-lifecycle-raw-inventory/01-buffer-binding-lifecycle-raw-slice/opengl_state_raw_inventory_test.py

graphics-opengl-limit-format-raw-inventory-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/03-limit-format-shader-inventory/01-limit-format-raw-inventory/opengl_limit_format_raw_inventory_test.py

graphics-opengl-unadmitted-ledger-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/03-limit-format-shader-inventory/02-unadmitted-shader-extension-ledger/opengl_unadmitted_ledger_test.py

graphics-gles-source-authority-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/01-source-authority/gles_source_authority_test.py

graphics-gles-unavailable-ledger-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/04-unavailable-language-extension-ledger/gles_unavailable_ledger_test.py

graphics-vulkan-registry-inventory-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/01-registry-inventory/v2/registry_inventory_test.py
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/01-registry-inventory/v2/registry_inventory_source_test.py

graphics-vulkan-source-channel-boundary-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/02-provenance-diagnostics/01-source-channel-boundary/vulkan_source_channels_test.py

graphics-vulkan-raw-docs-citations-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/02-provenance-diagnostics/02-raw-docs-provenance/vulkan_raw_docs_citations_test.py

graphics-vulkan-full-suite-diagnostics-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/02-provenance-diagnostics/03-full-suite-diagnostics/vulkan_full_suite_diagnostics_test.py

graphics-f05-source-adapter-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/03-future-f05-adapter-and-receipt/f05_source_adapter_test.py
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/03-future-f05-adapter-and-receipt/f05_source_aggregate_receipt_test.py

graphics-profile-registration-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/02-reproducibility/02-check-runner/02-profile-bound-registration/profile_registration_test.py
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/02-reproducibility/02-check-runner/02-profile-bound-registration/profile_registration_run_test.py

test: graphics-roadmap-test graphics-runner-test graphics-source-role-test graphics-normative-root-test graphics-vulkan-definition-test graphics-source-admission-test graphics-full-suite-root-test graphics-gl-flat-ledger-test graphics-gles-full-closure-ledger-test graphics-gl-gles-ledger-receipt-test graphics-vulkan-ledger-taxonomy-test graphics-vulkan-cache-replay-test graphics-vulkan-full-suite-receipt-test graphics-vulkan-local-shard-receipt-test graphics-role-aware-source-contract-test graphics-profile-source-gate-test graphics-opengl-source-authority-test graphics-opengl-normative-pdf-cache-test graphics-opengl-command-object-raw-inventory-test graphics-opengl-state-raw-slice-test graphics-opengl-limit-format-raw-inventory-test graphics-opengl-unadmitted-ledger-test graphics-gles-source-authority-test graphics-gles-unavailable-ledger-test graphics-vulkan-registry-inventory-test graphics-vulkan-source-channel-boundary-test graphics-vulkan-raw-docs-citations-test graphics-vulkan-full-suite-diagnostics-test graphics-f05-source-adapter-test graphics-profile-registration-test
	cargo test -p emulator
	node scripts/stamp_web_asset_version.mjs --check
	python3 scripts/check_graphics_roadmap.py
	find web/js -name '*.test.mjs' -exec node --test '{}' +
