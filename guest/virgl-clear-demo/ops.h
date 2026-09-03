#ifndef VIRGL_CLEAR_DEMO_OPS_H
#define VIRGL_CLEAR_DEMO_OPS_H

#include "uapi.h"

int virgl_setup(long fd, u32 *bo_handle, u32 *resource_handle);
int virgl_submit_clear(long fd, u32 bo_handle, u32 resource_handle);
int virgl_wait_for_clear(long fd, u32 bo_handle);

#endif
