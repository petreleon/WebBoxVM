#ifndef WEBGPU_DEMO_SYSCALL_H
#define WEBGPU_DEMO_SYSCALL_H

#include "uapi.h"

#define AT_FDCWD (-100L)
#define O_WRONLY 1L
#define O_RDWR 2L
#define O_CLOEXEC 0x80000L

#define NR_IOCTL 29L
#define NR_OPENAT 56L
#define NR_CLOSE 57L
#define NR_WRITE 64L
#define NR_EXIT 93L

static inline long syscall1(long number, long first)
{
    register long x0 __asm__("x0") = first;
    register long x8 __asm__("x8") = number;
    __asm__ volatile("svc 0" : "+r"(x0) : "r"(x8) : "memory", "cc");
    return x0;
}

static inline long syscall3(long number, long first, long second, long third)
{
    register long x0 __asm__("x0") = first;
    register long x1 __asm__("x1") = second;
    register long x2 __asm__("x2") = third;
    register long x8 __asm__("x8") = number;
    __asm__ volatile("svc 0"
                     : "+r"(x0)
                     : "r"(x1), "r"(x2), "r"(x8)
                     : "memory", "cc");
    return x0;
}

static inline long syscall4(long number, long first, long second, long third,
                            long fourth)
{
    register long x0 __asm__("x0") = first;
    register long x1 __asm__("x1") = second;
    register long x2 __asm__("x2") = third;
    register long x3 __asm__("x3") = fourth;
    register long x8 __asm__("x8") = number;
    __asm__ volatile("svc 0"
                     : "+r"(x0)
                     : "r"(x1), "r"(x2), "r"(x3), "r"(x8)
                     : "memory", "cc");
    return x0;
}

static inline long sys_open(const char *path, long flags)
{
    return syscall4(NR_OPENAT, AT_FDCWD, (long)path, flags, 0);
}

static inline long sys_ioctl(long fd, u32 request, void *argument)
{
    return syscall3(NR_IOCTL, fd, request, (long)argument);
}

static inline long sys_write(long fd, const void *data, u64 length)
{
    return syscall3(NR_WRITE, fd, (long)data, (long)length);
}

static inline void sys_close(long fd)
{
    (void)syscall1(NR_CLOSE, fd);
}

__attribute__((noreturn)) static inline void sys_exit(long status)
{
    (void)syscall1(NR_EXIT, status);
    for (;;) {
        __asm__ volatile("wfe");
    }
}

#endif
