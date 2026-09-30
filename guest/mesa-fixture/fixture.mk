# Pinned build inputs and image outputs stay outside Git.
MESA_FIXTURE_ARTIFACTS := .artifacts/graphics/i01-mesa-image
MESA_FIXTURE_IMAGE ?= $(MESA_FIXTURE_ARTIFACTS)/image
MESA_FIXTURE_ATTEMPT ?= 01

.PHONY: graphics-mesa-fixture-test graphics-mesa-image graphics-mesa-guest-tools graphics-mesa-guest-startup graphics-mesa-check
graphics-mesa-image:
	PYTHONDONTWRITEBYTECODE=1 python3 guest/mesa-fixture/finish.py --attempt "$(MESA_FIXTURE_ATTEMPT)" --output "$(MESA_FIXTURE_IMAGE)"

graphics-mesa-fixture-test:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s guest/mesa-fixture/build -p 'test_*.py'
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s guest/mesa-fixture/image -p 'test_*.py'
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s guest/mesa-fixture -p 'test_*.py'
	cargo test -p emulator --example guest_command

graphics-mesa-guest-tools:
	cargo build -p emulator --release --example guest_command
	PYTHONDONTWRITEBYTECODE=1 python3 guest/mesa-fixture/run.py --mode tools --image "$(MESA_FIXTURE_IMAGE)" --result $(MESA_FIXTURE_ARTIFACTS)/guest-tools-result.json

graphics-mesa-guest-startup:
	cargo build -p emulator --release --example guest_command
	PYTHONDONTWRITEBYTECODE=1 python3 guest/mesa-fixture/run.py --mode startup --image "$(MESA_FIXTURE_IMAGE)"

# Keep the two real guest checks ordered even when the caller uses make -j.
graphics-mesa-check: graphics-mesa-guest-tools
	$(MAKE) graphics-mesa-guest-startup
