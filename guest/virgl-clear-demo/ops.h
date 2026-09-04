#ifndef VIRGL_CLEAR_DEMO_OPS_H
#define VIRGL_CLEAR_DEMO_OPS_H

#include "uapi.h"

struct virgl_resources {
    u32 scanout_bo;
    u32 scanout_resource;
    u32 copy_source_bo;
    u32 copy_source_resource;
    u32 copy_destination_bo;
    u32 copy_destination_resource;
    u32 vertex_source_bo;
    u32 vertex_source_resource;
    u32 vertex_destination_bo;
    u32 vertex_destination_resource;
    u32 triangle_bo;
    u32 triangle_resource;
    u32 index_bo;
    u32 index_resource;
    u32 texture_bo;
    u32 texture_resource;
    u32 texture_pair_bo;
    u32 texture_pair_resource;
    u32 textured_bo;
    u32 textured_resource;
    u32 vertex_color_bo;
    u32 vertex_color_resource;
    u32 uniform_bo;
    u32 uniform_resource;
    u32 depth_bo;
    u32 depth_resource;
    u32 depth_vertex_bo;
    u32 depth_vertex_resource;
};

int virgl_setup(long fd, struct virgl_resources *resources);
int virgl_verify_blob_profiles(long fd);
int virgl_submit_clear(long fd, u32 bo_handle, u32 resource_handle);
int virgl_submit_copy(long fd, u32 source_bo, u32 source_resource,
                      u32 destination_bo, u32 destination_resource);
int virgl_submit_buffer_copy(long fd, u32 source_bo, u32 source_resource,
                             u32 destination_bo, u32 destination_resource);
int virgl_submit_vertex_input(long fd, u32 bo_handle, u32 resource_handle);
int virgl_create_triangle_buffer(long fd, u32 *bo_handle, u32 *resource_handle);
int virgl_create_index_buffer(long fd, u32 *bo_handle, u32 *resource_handle);
int virgl_run_triangle(long fd, const struct virgl_resources *resources);
int virgl_create_uniform_buffer(long fd, u32 *bo_handle, u32 *resource_handle);
int virgl_run_uniform_triangle(long fd, const struct virgl_resources *resources);
int virgl_create_depth_resources(long fd, struct virgl_resources *resources);
int virgl_run_depth_triangle(long fd, const struct virgl_resources *resources);
int virgl_run_depth_batch(long fd, const struct virgl_resources *resources);
int virgl_run_depth_equal(long fd, const struct virgl_resources *resources);
int virgl_run_depth_equal_batch(long fd, const struct virgl_resources *resources);
int virgl_run_solid_batch(long fd, const struct virgl_resources *resources);
int virgl_upload_index_buffer(long fd, u32 bo_handle);
int virgl_create_textured_resources(long fd, struct virgl_resources *resources);
int virgl_run_textured_triangle(long fd, const struct virgl_resources *resources);
int virgl_run_texture_pair(long fd, const struct virgl_resources *resources);
int virgl_create_vertex_color_resource(long fd, struct virgl_resources *resources);
int virgl_run_vertex_color_triangle(long fd, const struct virgl_resources *resources);
int virgl_run_texture_color_triangle(long fd, const struct virgl_resources *resources);
int virgl_upload_textured_vertices(long fd, u32 bo, u32 u, u32 v);
int virgl_submit_textured_triangle(
    long fd, const struct virgl_resources *resources, u32 sampler, u32 object_base);
int virgl_readback_scanout_pixel(long fd, u32 bo_handle, const u8 expected[4]);
int virgl_wait_for_resource(long fd, u32 bo_handle);

#endif
