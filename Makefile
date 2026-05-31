ARTIFACTS_DIR ?= .artifacts
DEBIAN_ARM64_ISO ?= $(ARTIFACTS_DIR)/debian-arm64-netinst.iso
DEBIAN_ARM64_ISO_BASE ?= https://cdimage.debian.org/debian-cd/current/arm64/iso-cd
DEBIAN_ARM64_ISO_FILE ?=
DRY_RUN ?= 0

.PHONY: busybox iso-debian-arm64 iso-info terminal-image terminal-debian-arm64 terminal-iso test

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

test:
	cargo test -p emulator
