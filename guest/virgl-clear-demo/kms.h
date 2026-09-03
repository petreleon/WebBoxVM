#ifndef VIRGL_CLEAR_DEMO_KMS_H
#define VIRGL_CLEAR_DEMO_KMS_H

#include "uapi.h"

#define DRM_DISPLAY_MODE_LEN 32u
#define DRM_MODE_CONNECTED 1u
#define DRM_FOURCC_CODE(a, b, c, d) \
    ((u32)(a) | ((u32)(b) << 8) | ((u32)(c) << 16) | ((u32)(d) << 24))
#define DRM_FORMAT_XRGB8888 DRM_FOURCC_CODE('X', 'R', '2', '4')

struct drm_mode_modeinfo {
    u32 clock;
    u16 hdisplay;
    u16 hsync_start;
    u16 hsync_end;
    u16 htotal;
    u16 hskew;
    u16 vdisplay;
    u16 vsync_start;
    u16 vsync_end;
    u16 vtotal;
    u16 vscan;
    u32 vrefresh;
    u32 flags;
    u32 type;
    char name[DRM_DISPLAY_MODE_LEN];
};

struct drm_mode_card_res {
    u64 fb_id_ptr;
    u64 crtc_id_ptr;
    u64 connector_id_ptr;
    u64 encoder_id_ptr;
    u32 count_fbs;
    u32 count_crtcs;
    u32 count_connectors;
    u32 count_encoders;
    u32 min_width;
    u32 max_width;
    u32 min_height;
    u32 max_height;
};

struct drm_mode_get_connector {
    u64 encoders_ptr;
    u64 modes_ptr;
    u64 props_ptr;
    u64 prop_values_ptr;
    u32 count_modes;
    u32 count_props;
    u32 count_encoders;
    u32 encoder_id;
    u32 connector_id;
    u32 connector_type;
    u32 connector_type_id;
    u32 connection;
    u32 mm_width;
    u32 mm_height;
    u32 subpixel;
    u32 pad;
};

struct drm_mode_fb_cmd2 {
    u32 fb_id;
    u32 width;
    u32 height;
    u32 pixel_format;
    u32 flags;
    u32 handles[4];
    u32 pitches[4];
    u32 offsets[4];
    u64 modifier[4];
};

struct drm_mode_crtc {
    u64 set_connectors_ptr;
    u32 count_connectors;
    u32 crtc_id;
    u32 fb_id;
    u32 x;
    u32 y;
    u32 gamma_size;
    u32 mode_valid;
    struct drm_mode_modeinfo mode;
};

#define DRM_IOCTL_MODE_GETRESOURCES DRM_IOWR(0xa0u, struct drm_mode_card_res)
#define DRM_IOCTL_MODE_SETCRTC DRM_IOWR(0xa2u, struct drm_mode_crtc)
#define DRM_IOCTL_MODE_GETCONNECTOR DRM_IOWR(0xa7u, struct drm_mode_get_connector)
#define DRM_IOCTL_MODE_ADDFB2 DRM_IOWR(0xb8u, struct drm_mode_fb_cmd2)

int kms_configure_scanout(long fd, u32 bo_handle);

_Static_assert(sizeof(struct drm_mode_modeinfo) == 68, "bad mode ABI");
_Static_assert(sizeof(struct drm_mode_card_res) == 64, "bad resources ABI");
_Static_assert(sizeof(struct drm_mode_get_connector) == 80, "bad connector ABI");
_Static_assert(sizeof(struct drm_mode_fb_cmd2) == 104, "bad framebuffer ABI");
_Static_assert(sizeof(struct drm_mode_crtc) == 104, "bad CRTC ABI");
_Static_assert(DRM_IOCTL_MODE_GETRESOURCES == 0xc04064a0u, "bad resources ioctl");
_Static_assert(DRM_IOCTL_MODE_SETCRTC == 0xc06864a2u, "bad CRTC ioctl");
_Static_assert(DRM_IOCTL_MODE_GETCONNECTOR == 0xc05064a7u, "bad connector ioctl");
_Static_assert(DRM_IOCTL_MODE_ADDFB2 == 0xc06864b8u, "bad framebuffer ioctl");

#endif
