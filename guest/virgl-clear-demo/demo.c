#include "kms.h"
#include "ops.h"
#include "syscall.h"
#include "transfer.h"

static const char card_node[] = "/dev/dri/card0";
static const char serial_node[] = "/dev/ttyAMA0";
static const char pass[] = "VIRGL_TRANSFER_READBACK_DEMO_PASS card0 capset=1 upload=10,20,30,255 clear=64,128,191,255 readback=64,128,191,255\n";
static const char fail_open[] = "VIRGL_CLEAR_DEMO_FAIL open-drm\n";
static const char fail_caps[] = "VIRGL_CLEAR_DEMO_FAIL capset\n";
static const char fail_context[] = "VIRGL_CLEAR_DEMO_FAIL context-init\n";
static const char fail_resource[] = "VIRGL_CLEAR_DEMO_FAIL resource-create\n";
static const char fail_transfer[] = "VIRGL_CLEAR_DEMO_FAIL transfer-upload\n";
static const char fail_submit[] = "VIRGL_CLEAR_DEMO_FAIL execbuffer\n";
static const char fail_wait[] = "VIRGL_CLEAR_DEMO_FAIL completion-wait\n";
static const char fail_readback[] = "VIRGL_CLEAR_DEMO_FAIL transfer-readback\n";

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

__attribute__((noreturn, section(".text.start"))) void _start(void)
{
    u32 bo_handle = 0;
    u32 resource_handle = 0;
    long fd = sys_open(card_node, O_RDWR | O_CLOEXEC);
    int stage = 0;

    if (fd < 0)
        stage = 1;
    else if ((stage = virgl_setup(fd, &bo_handle, &resource_handle)) == 0) {
        int kms = kms_configure_scanout(fd, bo_handle);

        if (kms != 0)
            stage = 5 - kms;
        else if (virgl_upload_pattern(fd, bo_handle) != 0)
            stage = 5;
        else {
            kms = kms_configure_scanout(fd, bo_handle);
            if (kms != 0)
                stage = 5 - kms;
            else if (virgl_submit_clear(fd, bo_handle, resource_handle) != 0)
                stage = 16;
            else if (virgl_wait_for_clear(fd, bo_handle) != 0)
                stage = 17;
            else if (virgl_readback_clear(fd, bo_handle) != 0)
                stage = 18;
        }
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
    else if (stage == 5)
        EMIT(fail_transfer);
    else if (stage >= 6 && stage <= 15)
        emit_kms_failure(stage - 5);
    else if (stage == 16)
        EMIT(fail_submit);
    else if (stage == 17)
        EMIT(fail_wait);
    else
        EMIT(fail_readback);
    if (fd >= 0)
        sys_close(fd);
    sys_exit(0);
}
