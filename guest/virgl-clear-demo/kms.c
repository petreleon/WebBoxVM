#include "kms.h"
#include "syscall.h"

#define SCANOUT_WIDTH 1024u
#define SCANOUT_HEIGHT 768u
#define MAX_MODES 128u

static int first_connector_and_crtc(long fd, u32 *connector_id, u32 *crtc_id)
{
    struct drm_mode_card_res resources = {0};
    u32 connectors[1] = {0};
    u32 crtcs[1] = {0};
    long result;

    result = sys_ioctl(fd, DRM_IOCTL_MODE_GETRESOURCES, &resources);
    if (result < 0)
        return -1;
    if (resources.count_connectors == 0 || resources.count_crtcs == 0)
        return -2;
    resources.connector_id_ptr = (u64)connectors;
    resources.crtc_id_ptr = (u64)crtcs;
    resources.count_fbs = 0;
    resources.count_connectors = 1;
    resources.count_crtcs = 1;
    resources.count_encoders = 0;
    if (sys_ioctl(fd, DRM_IOCTL_MODE_GETRESOURCES, &resources) < 0)
        return -3;
    if (connectors[0] == 0 || crtcs[0] == 0)
        return -4;
    *connector_id = connectors[0];
    *crtc_id = crtcs[0];
    return 0;
}

static int scanout_mode(long fd, u32 connector_id, struct drm_mode_modeinfo *selected)
{
    struct drm_mode_get_connector connector = {.connector_id = connector_id};
    struct drm_mode_modeinfo modes[MAX_MODES] = {{0}};
    u32 count;
    u32 index;

    if (sys_ioctl(fd, DRM_IOCTL_MODE_GETCONNECTOR, &connector) < 0)
        return -1;
    if (connector.connection != DRM_MODE_CONNECTED || connector.count_modes == 0)
        return -2;
    count = connector.count_modes < MAX_MODES ? connector.count_modes : MAX_MODES;
    connector.modes_ptr = (u64)modes;
    connector.count_modes = count;
    connector.count_props = 0;
    connector.count_encoders = 0;
    if (sys_ioctl(fd, DRM_IOCTL_MODE_GETCONNECTOR, &connector) < 0)
        return -3;
    for (index = 0; index < count; index++) {
        if (modes[index].hdisplay == SCANOUT_WIDTH &&
            modes[index].vdisplay == SCANOUT_HEIGHT) {
            *selected = modes[index];
            return 0;
        }
    }
    return -4;
}

int kms_configure_scanout(long fd, u32 bo_handle)
{
    struct drm_mode_modeinfo mode = {0};
    struct drm_mode_fb_cmd2 framebuffer = {
        .width = SCANOUT_WIDTH,
        .height = SCANOUT_HEIGHT,
        .pixel_format = DRM_FORMAT_XRGB8888,
        .handles = {bo_handle, 0, 0, 0},
        .pitches = {SCANOUT_WIDTH * 4u, 0, 0, 0},
    };
    struct drm_mode_crtc crtc = {0};
    u32 connector_id;
    u32 crtc_id;

    int status = first_connector_and_crtc(fd, &connector_id, &crtc_id);

    if (status != 0)
        return status;
    status = scanout_mode(fd, connector_id, &mode);
    if (status != 0)
        return status - 4;
    if (sys_ioctl(fd, DRM_IOCTL_MODE_ADDFB2, &framebuffer) < 0 || framebuffer.fb_id == 0)
        return -9;
    crtc.set_connectors_ptr = (u64)&connector_id;
    crtc.count_connectors = 1;
    crtc.crtc_id = crtc_id;
    crtc.fb_id = framebuffer.fb_id;
    crtc.mode_valid = 1;
    crtc.mode = mode;
    return sys_ioctl(fd, DRM_IOCTL_MODE_SETCRTC, &crtc) < 0 ? -10 : 0;
}
