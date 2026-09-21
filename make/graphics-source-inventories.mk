GRAPHICS_SOURCE_INVENTORY_TESTS := \
	graphics-opengl-command-object-raw-inventory-test \
	graphics-opengl-state-raw-slice-test \
	graphics-opengl-limit-format-raw-inventory-test \
	graphics-opengl-unadmitted-ledger-test \
	graphics-gles-normative-pdf-cache-test \
	graphics-gles-limit-format-raw-inventory-test \
	graphics-vulkan-raw-docs-citations-test \
	graphics-vulkan-no-claim-provenance-receipt-test

.PHONY: $(GRAPHICS_SOURCE_INVENTORY_TESTS)

graphics-opengl-command-object-raw-inventory-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/02-command-object-state-inventory/02-command-object-raw-inventory/opengl_command_object_raw_inventory_test.py

graphics-opengl-state-raw-slice-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/02-command-object-state-inventory/03-state-and-lifecycle-raw-inventory/01-buffer-binding-lifecycle-raw-slice/opengl_state_raw_inventory_test.py

graphics-opengl-limit-format-raw-inventory-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/03-limit-format-shader-inventory/01-limit-format-raw-inventory/opengl_limit_format_raw_inventory_test.py

graphics-opengl-unadmitted-ledger-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/03-limit-format-shader-inventory/02-unadmitted-shader-extension-ledger/opengl_unadmitted_ledger_test.py

graphics-gles-normative-pdf-cache-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/01-normative-pdf-cache/gles_normative_pdf_cache_test.py

graphics-gles-limit-format-raw-inventory-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/03-limit-format-raw-inventory/gles_limit_format_raw_inventory_test.py

graphics-vulkan-raw-docs-citations-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/02-provenance-diagnostics/02-raw-docs-provenance/vulkan_raw_docs_citations_test.py

graphics-vulkan-no-claim-provenance-receipt-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/02-provenance-diagnostics/04-aggregate-no-claim-receipt/vulkan_no_claim_provenance_receipt_test.py

test: $(GRAPHICS_SOURCE_INVENTORY_TESTS)
