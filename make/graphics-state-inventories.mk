GRAPHICS_GLES_STATE_ROOT := todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/04-state-execution-command-slices
GRAPHICS_STATE_INVENTORY_OUTPUT ?= .artifacts/graphics/state-inventory-regenerated
GRAPHICS_STATE_INVENTORY_TESTS := \
	graphics-gles-context-state-lifecycle-test \
	graphics-gles-draw-raster-command-raw-inventory-test \
	graphics-gles-pixel-transfer-command-raw-inventory-test \
	graphics-gles-debug-special-query-command-raw-inventory-test

.PHONY: $(GRAPHICS_STATE_INVENTORY_TESTS) graphics-gles-state-inventory-check graphics-gles-state-inventory-regenerate

graphics-gles-context-state-lifecycle-test:
	PYTHONDONTWRITEBYTECODE=1 WEBBOXVM_GRAPHICS_CACHE_ROOT="$(GRAPHICS_GLES_CACHE)" python3 $(GRAPHICS_GLES_STATE_ROOT)/01-context-state-lifecycle/gles_context_state_lifecycle_inventory_test.py

graphics-gles-draw-raster-command-raw-inventory-test:
	PYTHONDONTWRITEBYTECODE=1 WEBBOXVM_GRAPHICS_CACHE_ROOT="$(GRAPHICS_GLES_CACHE)" python3 scripts/test_graphics_state_declarations.py DrawTests

graphics-gles-pixel-transfer-command-raw-inventory-test:
	PYTHONDONTWRITEBYTECODE=1 WEBBOXVM_GRAPHICS_CACHE_ROOT="$(GRAPHICS_GLES_CACHE)" python3 scripts/test_graphics_state_declarations.py PixelTests

graphics-gles-debug-special-query-command-raw-inventory-test:
	PYTHONDONTWRITEBYTECODE=1 WEBBOXVM_GRAPHICS_CACHE_ROOT="$(GRAPHICS_GLES_CACHE)" python3 scripts/test_graphics_state_declarations.py DebugTests

graphics-gles-state-inventory-check:
	PYTHONDONTWRITEBYTECODE=1 python3 scripts/graphics_inventory_batch.py --group state --cache-root "$(GRAPHICS_GLES_CACHE)"

graphics-gles-state-inventory-regenerate:
	PYTHONDONTWRITEBYTECODE=1 python3 scripts/graphics_inventory_batch.py --group state --cache-root "$(GRAPHICS_GLES_CACHE)" --output-dir "$(GRAPHICS_STATE_INVENTORY_OUTPUT)"

test: $(GRAPHICS_STATE_INVENTORY_TESTS)
