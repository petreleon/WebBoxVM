ARTIFACTS_DIR ?= .artifacts
DEBIAN_ARM64_ISO ?= $(ARTIFACTS_DIR)/debian-arm64-netinst.iso
DEBIAN_ARM64_ISO_BASE ?= https://cdimage.debian.org/debian-cd/current/arm64/iso-cd
DEBIAN_ARM64_ISO_FILE ?=
DRY_RUN ?= 0
WEB_PORT ?= 8080
WEB_TARGET ?= wasm64-unknown-unknown
WEB_TOOLCHAIN ?= +nightly
WEB_CARGO_FLAGS ?= -Z build-std=std,panic_abort
WEB_THREADS_RUSTFLAGS ?= -C target-feature=+atomics,+bulk-memory -C link-arg=--shared-memory -C link-arg=--max-memory=4294967296 -C link-arg=--import-memory -C link-arg=--export=__wasm_init_tls -C link-arg=--export=__tls_size -C link-arg=--export=__tls_align -C link-arg=--export=__tls_base -C link-arg=--export=__heap_base
WASM_BINDGEN_THREADS_ROOT ?= $(ARTIFACTS_DIR)/tools/wasm-bindgen-memory64-threads
WASM_BINDGEN_THREADS ?= $(WASM_BINDGEN_THREADS_ROOT)/bin/wasm-bindgen

.PHONY: busybox iso-debian-arm64 iso-info terminal-image terminal-debian-arm64 terminal-iso wasm-bindgen-memory64-threads web-pkg web-pkg-serial web-pkg-threaded web web-debian-arm64 test

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

web-debian-arm64: iso-debian-arm64 web-pkg
	mkdir -p web/media
	ln -sf "$(abspath $(DEBIAN_ARM64_ISO))" web/media/debian-arm64-netinst.iso
	python3 scripts/serve_web.py --port $(WEB_PORT) --directory web

test:
	cargo test -p emulator
