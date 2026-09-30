GRAPHICS_GLES_CACHE ?= /private/tmp/webboxvm-f0341.cqT6ZX
GRAPHICS_INVENTORY_OUTPUT ?= .artifacts/graphics/vertex-inventory-regenerated

.PHONY: graphics-inventory-automation-test graphics-gles-vertex-inventory-check graphics-gles-vertex-inventory-regenerate

graphics-inventory-automation-test:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest scripts.test_graphics_inventory_batch scripts.test_graphics_inventory_snapshot scripts.test_graphics_inventory_pdf scripts.test_graphics_inventory_bootstrap

graphics-gles-vertex-inventory-check:
	PYTHONDONTWRITEBYTECODE=1 python3 scripts/graphics_inventory_batch.py --cache-root "$(GRAPHICS_GLES_CACHE)"

graphics-gles-vertex-inventory-regenerate:
	PYTHONDONTWRITEBYTECODE=1 python3 scripts/graphics_inventory_batch.py --cache-root "$(GRAPHICS_GLES_CACHE)" --output-dir "$(GRAPHICS_INVENTORY_OUTPUT)"

test: graphics-inventory-automation-test
