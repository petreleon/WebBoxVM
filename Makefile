ARTIFACTS_DIR ?= .artifacts
DEBIAN_ARM64_ISO ?= $(ARTIFACTS_DIR)/debian-arm64-netinst.iso
DEBIAN_ARM64_ISO_BASE ?= https://cdimage.debian.org/debian-cd/current/arm64/iso-cd
DEBIAN_ARM64_ISO_FILE ?=
DRY_RUN ?= 0
WEB_PORT ?= 8080
WEB_TARGET ?= wasm32-unknown-unknown

.PHONY: busybox iso-debian-arm64 iso-info terminal-image terminal-debian-arm64 terminal-iso web-pkg web web-debian-arm64 test

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

web-pkg:
	cargo build -p emulator --release --target $(WEB_TARGET) --features wasm
	wasm-bindgen target/$(WEB_TARGET)/release/emulator.wasm --out-dir web/pkg --target web

web: web-pkg
	python3 -m http.server $(WEB_PORT) --directory web

web-debian-arm64: iso-debian-arm64 web-pkg
	mkdir -p web/media
	ln -sf "$(abspath $(DEBIAN_ARM64_ISO))" web/media/debian-arm64-netinst.iso
	python3 -m http.server $(WEB_PORT) --directory web

test:
	cargo test -p emulator
