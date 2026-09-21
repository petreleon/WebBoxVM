GRAPHICS_SOURCE_INVENTORY_TESTS := \
	graphics-opengl-command-object-raw-inventory-test \
	graphics-opengl-state-raw-slice-test \
	graphics-opengl-limit-format-raw-inventory-test \
	graphics-opengl-unadmitted-ledger-test \
	graphics-opengl-command-domain-classification-test \
	graphics-opengl-declaration-grammar-test \
	graphics-opengl-generic-object-sync-test \
	graphics-opengl-buffer-command-inventory-test \
	graphics-opengl-global-execution-sync-test \
	graphics-opengl-draw-compute-submission-test \
	graphics-opengl-final-table-anchor-catalog-test \
	graphics-opengl-chapter-local-anchor-manifest-test \
	graphics-opengl-aggregate-anchor-classification-test \
	graphics-gles-normative-pdf-cache-test \
	graphics-gles-command-domain-classification-test \
	graphics-gles-declaration-grammar-test \
	graphics-gles-generic-sync-query-test \
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

graphics-opengl-command-domain-classification-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/02-command-object-state-inventory/05-remaining-command-object-vocabulary/01-command-domain-classification/opengl_command_domain_classification_test.py

graphics-opengl-declaration-grammar-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/02-command-object-state-inventory/05-remaining-command-object-vocabulary/02-template-declaration-grammar/opengl_declaration_grammar_test.py

graphics-opengl-generic-object-sync-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/02-command-object-state-inventory/05-remaining-command-object-vocabulary/03-object-resource-command-slices/01-generic-object-sync/opengl_generic_object_sync_raw_inventory_test.py

graphics-opengl-buffer-command-inventory-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/02-command-object-state-inventory/05-remaining-command-object-vocabulary/03-object-resource-command-slices/02-buffer-commands/opengl_buffer_command_inventory_test.py

graphics-opengl-global-execution-sync-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/02-command-object-state-inventory/05-remaining-command-object-vocabulary/04-non-object-command-slices/01-global-execution-sync/opengl_global_execution_sync_test.py

graphics-opengl-draw-compute-submission-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/02-command-object-state-inventory/05-remaining-command-object-vocabulary/04-non-object-command-slices/02-draw-compute-submission/opengl_draw_compute_submission_test.py

graphics-opengl-final-table-anchor-catalog-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/03-limit-format-shader-inventory/04-remaining-core-limit-format-families/01-anchor-classification/01-final-table-anchors/opengl_final_table_anchor_catalog_test.py

graphics-opengl-chapter-local-anchor-manifest-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/03-limit-format-shader-inventory/04-remaining-core-limit-format-families/01-anchor-classification/02-chapter-local-anchor-manifest/opengl_chapter_local_anchor_manifest_test.py

graphics-opengl-aggregate-anchor-classification-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/03-limit-format-shader-inventory/04-remaining-core-limit-format-families/01-anchor-classification/03-aggregate-anchor-classification/opengl_closed_anchor_classification_test.py

graphics-gles-normative-pdf-cache-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/01-normative-pdf-cache/gles_normative_pdf_cache_test.py

graphics-gles-command-domain-classification-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/01-command-domain-classification/gles_command_domain_classification_test.py
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/01-command-domain-classification/gles_command_domain_crosschecks_test.py

graphics-gles-declaration-grammar-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/02-template-declaration-grammar/gles_declaration_grammar_test.py

graphics-gles-generic-sync-query-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/01-generic-sync-query/gles_generic_sync_query_raw_inventory_test.py

graphics-gles-limit-format-raw-inventory-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/03-limit-format-raw-inventory/gles_limit_format_raw_inventory_test.py

graphics-vulkan-raw-docs-citations-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/02-provenance-diagnostics/02-raw-docs-provenance/vulkan_raw_docs_citations_test.py

graphics-vulkan-no-claim-provenance-receipt-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/02-provenance-diagnostics/04-aggregate-no-claim-receipt/vulkan_no_claim_provenance_receipt_test.py

test: $(GRAPHICS_SOURCE_INVENTORY_TESTS)
