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
};

int virgl_setup(long fd, struct virgl_resources *resources);
int virgl_create_guest_blob(long fd);
int virgl_create_host_blob(long fd);
int virgl_submit_clear(long fd, u32 bo_handle, u32 resource_handle);
int virgl_submit_copy(long fd, u32 source_bo, u32 source_resource,
                      u32 destination_bo, u32 destination_resource);
int virgl_submit_buffer_copy(long fd, u32 source_bo, u32 source_resource,
                             u32 destination_bo, u32 destination_resource);
int virgl_submit_vertex_input(long fd, u32 bo_handle, u32 resource_handle);
int virgl_create_triangle_buffer(long fd, u32 *bo_handle, u32 *resource_handle);
int virgl_create_index_buffer(long fd, u32 *bo_handle, u32 *resource_handle);
int virgl_run_triangle(long fd, const struct virgl_resources *resources);
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
