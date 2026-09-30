#ifndef WEBBOXVM_MESA_PROBE_H
#define WEBBOXVM_MESA_PROBE_H
#include <ctype.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>
#include <xf86drm.h>

static void probe_text(const char *key, const char *value)
{
    printf("%s=\"", key);
    for (const unsigned char *p = (const unsigned char *)value; *p; ++p) {
        if (*p == '\\' || *p == '"') printf("\\%c", *p);
        else if (*p < 32 || *p >= 127) printf("\\x%02x", *p);
        else putchar(*p);
    }
    puts("\"");
}

static int probe_contains(const char *value, const char *word)
{
    for (; *value; ++value) {
        size_t n = 0;
        while (word[n] && value[n] &&
               tolower((unsigned char)value[n]) == (unsigned char)word[n]) ++n;
        if (!word[n]) return 1;
    }
    return 0;
}

static int probe_software(const char *value)
{
    const char *words[] = {"llvmpipe", "softpipe", "lavapipe", "software", "swrast"};
    for (size_t i = 0; i < sizeof(words) / sizeof(words[0]); ++i)
        if (probe_contains(value, words[i])) return 1;
    return 0;
}

static int probe_mesa_version(const char *value)
{
    const char *version = strstr(value, "Mesa 25.3.6");
    return version && (version[11] == '\0' || version[11] == ' ');
}

static int probe_environment(const char *label)
{
    const char *names[] = {"MESA_GL_VERSION_OVERRIDE", "MESA_GLES_VERSION_OVERRIDE",
        "MESA_GLSL_VERSION_OVERRIDE", "MESA_VK_VERSION_OVERRIDE", "MESA_EXTENSION_OVERRIDE",
        "MESA_LOADER_DRIVER_OVERRIDE", "GALLIUM_DRIVER", "LIBGL_ALWAYS_SOFTWARE",
        "VIRGL_DEBUG", "VN_DEBUG", "VN_PERF"};
    for (size_t i = 0; i < sizeof(names) / sizeof(names[0]); ++i) {
        if (getenv(names[i])) {
            fprintf(stderr, "%s_FAIL stage=environment_override variable=%s\n", label, names[i]);
            return -1;
        }
    }
    return 0;
}

static int probe_drm_open(const char *label, const char *path)
{
    int fd = open(path, O_RDWR | O_CLOEXEC);
    if (fd < 0) {
        fprintf(stderr, "%s_FAIL stage=drm_open errno=%d\n", label, errno);
        return -1;
    }
    struct stat info;
    if (fstat(fd, &info)) {
        fprintf(stderr, "%s_FAIL stage=drm_fstat errno=%d\n", label, errno);
        close(fd);
        return -1;
    }
    if (!S_ISCHR(info.st_mode)) {
        fprintf(stderr, "%s_FAIL stage=drm_character_device\n", label);
        close(fd);
        return -1;
    }
    drmVersionPtr version = drmGetVersion(fd);
    if (!version) {
        fprintf(stderr, "%s_FAIL stage=drmGetVersion errno=%d\n", label, errno);
        close(fd);
        return -1;
    }
    int valid = version->name && version->name_len == 10 &&
                !memcmp(version->name, "virtio_gpu", 10);
    if (!valid) fprintf(stderr, "%s_FAIL stage=drm_driver_identity\n", label);
    else {
        probe_text("DRM_DEVICE", path);
        printf("DRM_DRIVER=virtio_gpu %d.%d.%d\n", version->version_major,
               version->version_minor, version->version_patchlevel);
    }
    drmFreeVersion(version);
    if (!valid) { close(fd); return -1; }
    return fd;
}
#endif
