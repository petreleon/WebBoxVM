#define _GNU_SOURCE
#include <fcntl.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

static const char *RAW = "/vulkan/raw.adoc";
static const char *TEMP = "/work/temporary/out.tmp";
static const char *GENERATED = "/work/generated/out.adoc";

static int close_fd(int descriptor) {
    return (int)syscall(SYS_close, descriptor);
}

static int open_read(const char *path) {
    return (int)syscall(SYS_openat, AT_FDCWD, path, O_RDONLY | O_CLOEXEC | O_NOFOLLOW, 0);
}

static int open_write(const char *path) {
    return (int)syscall(SYS_openat, AT_FDCWD, path, O_WRONLY | O_CREAT | O_EXCL | O_CLOEXEC | O_NOFOLLOW, 0600);
}

int main(void) {
    char bytes[64]; int source, target, output, status; long count; pid_t child;
    source = open_read(RAW); if (source < 0) return 2;
    count = syscall(SYS_read, source, bytes, sizeof(bytes));
    if (count <= 0 || close_fd(source)) return 3;
    target = open_write(TEMP); if (target < 0) return 4;
    if (syscall(SYS_write, target, bytes, (size_t)count) != count || close_fd(target)) return 5;
    if (syscall(SYS_renameat, AT_FDCWD, TEMP, AT_FDCWD, GENERATED)) return 6;
    output = open_read(GENERATED); if (output < 0) return 7;
    count = syscall(SYS_read, output, bytes, sizeof(bytes));
    if (count <= 0 || close_fd(output)) return 8;
    child = fork(); if (child < 0) return 9;
    if (child == 0) { execl("/bin/true", "true", (char *)NULL); _exit(10); }
    if (waitpid(child, &status, 0) != child || !WIFEXITED(status) || WEXITSTATUS(status)) return 11;
    return 0;
}
