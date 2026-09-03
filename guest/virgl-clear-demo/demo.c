#include "kms.h"
#include "syscall.h"
#include "virgl.h"

static const char card_node[] = "/dev/dri/card0";
static const char serial_node[] = "/dev/ttyAMA0";
static const char pass[] = "VIRGL_CLEAR_DEMO_PASS card0 capset=1 clear=64,128,191,255\n";
static const char fail_open[] = "VIRGL_CLEAR_DEMO_FAIL open-drm\n";
static const char fail_caps[] = "VIRGL_CLEAR_DEMO_FAIL capset\n";
static const char fail_context[] = "VIRGL_CLEAR_DEMO_FAIL context-init\n";
static const char fail_resource[] = "VIRGL_CLEAR_DEMO_FAIL resource-create\n";
static const char fail_submit[] = "VIRGL_CLEAR_DEMO_FAIL execbuffer\n";
static const char fail_wait[] = "VIRGL_CLEAR_DEMO_FAIL completion-wait\n";

#define SCANOUT_WIDTH 1024u
#define SCANOUT_HEIGHT 768u
#define SCANOUT_BYTES (SCANOUT_WIDTH * SCANOUT_HEIGHT * 4u)

static void emit(const char *message, u64 length)
{
    long fd = sys_open(serial_node, O_WRONLY | O_CLOEXEC);
    u64 written = 0;

    if (fd < 0)
        fd = 1;
    while (written < length) {
        long count = sys_write(fd, message + written, length - written);
        if (count <= 0)
            break;
        written += (u64)count;
    }
    if (fd != 1)
        sys_close(fd);
}

#define EMIT(text) emit((text), sizeof(text) - 1u)

static void emit_kms_failure(int step)
{
    char message[] = "VIRGL_CLEAR_DEMO_FAIL kms-step=00\n";

    message[31] = (char)('0' + step / 10);
    message[32] = (char)('0' + step % 10);
    emit(message, sizeof(message) - 1u);
}

static int virgl_caps(long fd)
{
    u8 caps[308] = {0};
    struct drm_virtgpu_get_caps get = {
        .cap_set_id = VIRTGPU_DRM_CAPSET_VIRGL,
        .cap_set_ver = 1,
        .addr = (u64)caps,
        .size = sizeof(caps),
    };

    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_GET_CAPS, &get) < 0 ||
                   caps[0] != 1 || caps[4] != 2 || caps[68] != 2
               ? -1
               : 0;
}

static int init_context(long fd)
{
    struct drm_virtgpu_context_set_param parameter = {
        .param = VIRTGPU_CONTEXT_PARAM_CAPSET_ID,
        .value = VIRTGPU_DRM_CAPSET_VIRGL,
    };
    struct drm_virtgpu_context_init context = {
        .num_params = 1,
        .ctx_set_params = (u64)&parameter,
    };

    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_CONTEXT_INIT, &context) < 0 ? -1 : 0;
}

static int create_resource(long fd, u32 *bo_handle, u32 *resource_handle)
{
    struct drm_virtgpu_resource_create resource = {
        .target = VIRGL_TARGET_TEXTURE_2D,
        .format = VIRGL_FORMAT_B8G8R8A8_UNORM,
        .bind = VIRGL_BIND_RENDER_TARGET,
        .width = SCANOUT_WIDTH,
        .height = SCANOUT_HEIGHT,
        .depth = 1,
        .array_size = 1,
        .nr_samples = 1,
        .size = SCANOUT_BYTES,
        .stride = SCANOUT_WIDTH * 4u,
    };

    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_RESOURCE_CREATE, &resource) < 0 ||
        resource.bo_handle == 0 || resource.res_handle == 0)
        return -1;
    *bo_handle = resource.bo_handle;
    *resource_handle = resource.res_handle;
    return 0;
}

static int submit_clear(long fd, u32 bo_handle, u32 resource_handle)
{
    u32 words[VIRGL_CLEAR_WORDS] = {0};
    struct drm_virtgpu_execbuffer submit = {
        .size = sizeof(words),
        .command = (u64)words,
        .bo_handles = (u64)&bo_handle,
        .num_bo_handles = 1,
        .fence_fd = -1,
    };

    virgl_clear_stream(words, resource_handle);
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_EXECBUFFER, &submit) < 0 ? -1 : 0;
}

static int wait_for_clear(long fd, u32 bo_handle)
{
    struct drm_virtgpu_3d_wait wait = {.handle = bo_handle};

    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_WAIT, &wait) < 0 ? -1 : 0;
}

__attribute__((noreturn, section(".text.start"))) void _start(void)
{
    u32 bo_handle = 0;
    u32 resource_handle = 0;
    long fd = sys_open(card_node, O_RDWR | O_CLOEXEC);
    int stage = 0;

    if (fd < 0)
        stage = 1;
    else if (virgl_caps(fd) != 0)
        stage = 2;
    else if (init_context(fd) != 0)
        stage = 3;
    else if (create_resource(fd, &bo_handle, &resource_handle) != 0)
        stage = 4;
    else {
        int kms = kms_configure_scanout(fd, bo_handle);

        if (kms != 0)
            stage = 4 - kms;
        else if (submit_clear(fd, bo_handle, resource_handle) != 0)
            stage = 9;
        else if (wait_for_clear(fd, bo_handle) != 0)
            stage = 10;
    }
    if (stage == 0)
        EMIT(pass);
    else if (stage == 1)
        EMIT(fail_open);
    else if (stage == 2)
        EMIT(fail_caps);
    else if (stage == 3)
        EMIT(fail_context);
    else if (stage == 4)
        EMIT(fail_resource);
    else if (stage >= 5 && stage <= 14)
        emit_kms_failure(stage - 4);
    else if (stage == 9)
        EMIT(fail_submit);
    else
        EMIT(fail_wait);
    if (fd >= 0)
        sys_close(fd);
    sys_exit(0);
}
