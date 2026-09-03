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
};

int virgl_setup(long fd, struct virgl_resources *resources);
int virgl_submit_clear(long fd, u32 bo_handle, u32 resource_handle);
int virgl_submit_copy(long fd, u32 source_bo, u32 source_resource,
                      u32 destination_bo, u32 destination_resource);
int virgl_submit_buffer_copy(long fd, u32 source_bo, u32 source_resource,
                             u32 destination_bo, u32 destination_resource);
int virgl_submit_vertex_input(long fd, u32 bo_handle, u32 resource_handle);
int virgl_wait_for_resource(long fd, u32 bo_handle);

#endif
