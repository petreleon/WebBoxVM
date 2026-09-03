#ifndef VIRGL_CLEAR_DEMO_TRANSFER_H
#define VIRGL_CLEAR_DEMO_TRANSFER_H

#include "uapi.h"

int virgl_upload_pattern(long fd, u32 bo_handle);
int virgl_readback_clear(long fd, u32 bo_handle);
int virgl_upload_copy_source(long fd, u32 bo_handle);
int virgl_readback_copy_destination(long fd, u32 bo_handle);

#endif
