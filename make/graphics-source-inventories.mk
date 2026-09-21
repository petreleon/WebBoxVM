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
	graphics-gles-buffer-command-inventory-test \
	graphics-gles-program-pipeline-literal-inventory-test \
	graphics-gles-uniform-query-inventory-test \
	graphics-gles-uniform-scalar-vector-template-test \
	graphics-gles-uniform-matrix-template-test \
	graphics-gles-program-uniform-scalar-vector-template-test \
	graphics-gles-program-uniform-matrix-template-test \
	graphics-gles-texture-sampler-object-parameter-test \
	graphics-gles-texture-image-copy-subimage-test \
	graphics-gles-texture-extended-command-test \
	graphics-gles-texture-parameter-query-template-test \
	graphics-gles-framebuffer-object-parameter-query-test \
	graphics-gles-renderbuffer-object-storage-query-test \
	graphics-gles-framebuffer-attachment-status-test \
	graphics-gles-current-vertex-attribute-template-test \
	graphics-gles-vertex-array-binding-command-test \
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

graphics-gles-buffer-command-inventory-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/02-buffer-commands/gles_buffer_command_raw_inventory_test.py

graphics-gles-program-pipeline-literal-inventory-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/03-program-pipeline-commands/01-shader-program-pipeline-binaries/gles_program_pipeline_literal_raw_inventory_test.py

graphics-gles-uniform-query-inventory-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/03-program-pipeline-commands/02-uniform-reflection-and-object-queries/gles_uniform_query_raw_inventory_test.py

graphics-gles-uniform-scalar-vector-template-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/03-program-pipeline-commands/03-uniform-scalar-vector-templates/gles_uniform_scalar_vector_template_inventory_test.py

graphics-gles-uniform-matrix-template-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/03-program-pipeline-commands/04-uniform-matrix-templates/gles_uniform_matrix_template_inventory_test.py

graphics-gles-program-uniform-scalar-vector-template-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/03-program-pipeline-commands/05-program-uniform-scalar-vector-templates/gles_program_uniform_scalar_vector_template_inventory_test.py

graphics-gles-program-uniform-matrix-template-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/03-program-pipeline-commands/06-program-uniform-matrix-templates/gles_program_uniform_matrix_template_inventory_test.py

graphics-gles-texture-sampler-object-parameter-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/04-texture-sampler-commands/01-texture-sampler-objects-parameters/gles_texture_sampler_object_parameter_inventory_test.py

graphics-gles-texture-image-copy-subimage-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/04-texture-sampler-commands/02-texture-image-copy-subimage-commands/gles_texture_image_copy_subimage_inventory_test.py

graphics-gles-texture-extended-command-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/04-texture-sampler-commands/03-compressed-storage-buffer-mipmap-image-commands/gles_texture_extended_command_inventory_test.py

graphics-gles-texture-parameter-query-template-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/04-texture-sampler-commands/04-texture-parameter-query-templates/gles_texture_parameter_query_template_inventory_test.py

graphics-gles-framebuffer-object-parameter-query-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/05-framebuffer-renderbuffer-commands/01-framebuffer-object-parameter-query-commands/gles_framebuffer_object_command_inventory_test.py

graphics-gles-renderbuffer-object-storage-query-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/05-framebuffer-renderbuffer-commands/02-renderbuffer-object-storage-query-commands/gles_renderbuffer_object_command_inventory_test.py

graphics-gles-framebuffer-attachment-status-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/05-framebuffer-renderbuffer-commands/03-framebuffer-attachment-status-commands/gles_framebuffer_attachment_status_command_inventory_test.py

graphics-gles-current-vertex-attribute-template-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/06-vertex-transform-feedback-commands/01-current-generic-attribute-templates/gles_current_vertex_attribute_template_inventory_test.py

graphics-gles-vertex-array-binding-command-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/06-vertex-transform-feedback-commands/02-vertex-array-attribute-binding-and-primitive-restart-commands/gles_vertex_array_binding_command_inventory_test.py

graphics-gles-limit-format-raw-inventory-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/03-limit-format-raw-inventory/gles_limit_format_raw_inventory_test.py

graphics-vulkan-raw-docs-citations-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/02-provenance-diagnostics/02-raw-docs-provenance/vulkan_raw_docs_citations_test.py

graphics-vulkan-no-claim-provenance-receipt-test:
	PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/02-provenance-diagnostics/04-aggregate-no-claim-receipt/vulkan_no_claim_provenance_receipt_test.py

test: $(GRAPHICS_SOURCE_INVENTORY_TESTS)
