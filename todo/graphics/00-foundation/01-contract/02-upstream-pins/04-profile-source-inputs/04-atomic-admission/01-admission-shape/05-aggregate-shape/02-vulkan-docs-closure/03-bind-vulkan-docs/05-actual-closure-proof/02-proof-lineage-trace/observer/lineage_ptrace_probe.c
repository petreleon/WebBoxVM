#define _GNU_SOURCE
#include <errno.h>
#include <linux/audit.h>
#include <signal.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <sys/ptrace.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#ifndef PTRACE_GET_SYSCALL_INFO
#define PTRACE_GET_SYSCALL_INFO 0x420e
#endif

struct syscall_info {
    uint8_t op, pad[3];
    uint32_t arch;
    uint64_t instruction_pointer, stack_pointer;
    union {
        struct { uint64_t nr, args[6]; } entry;
        struct { int64_t rval; uint8_t is_error; } exit;
    };
};

static void kill_wait(pid_t pid) {
    int status;
    if (pid > 0) {
        (void)kill(pid, SIGKILL);
        (void)waitpid(pid, &status, __WALL);
    }
}

static int blocked(const char *stage, int error, pid_t child, pid_t forked) {
    kill_wait(forked);
    kill_wait(child);
    printf("{\"contract\":\"webboxvm-graphics-ptrace-capability-v3\",\"status\":\"blocked\",\"stage\":\"%s\",\"errno\":%d}\n", stage, error);
    return 77;
}

static int child_errno(int descriptor) {
    int error = 0;
    ssize_t count = read(descriptor, &error, sizeof(error));
    return count == (ssize_t)sizeof(error) && error > 0 ? error : EPROTO;
}

static int syscall_info(pid_t child, unsigned int operation, long number, long result, const char *stage, pid_t forked) {
    struct syscall_info info;
    memset(&info, 0, sizeof(info));
    if (ptrace(PTRACE_GET_SYSCALL_INFO, child, sizeof(info), &info) < 0)
        return blocked(stage, errno, child, forked);
    if (info.op != operation || info.arch != AUDIT_ARCH_X86_64)
        return blocked(stage, EPROTO, child, forked);
    if (operation == PTRACE_SYSCALL_INFO_ENTRY && (long)info.entry.nr != number)
        return blocked(stage, EPROTO, child, forked);
    if (operation == PTRACE_SYSCALL_INFO_EXIT && (info.exit.is_error || (long)info.exit.rval != result))
        return blocked(stage, EPROTO, child, forked);
    return 0;
}

int main(void) {
    int pipefd[2], status;
    unsigned long event_pid = 0;
    pid_t child, forked = -1, waited;
    if (pipe(pipefd) != 0) return blocked("setup", errno, -1, -1);
    child = fork();
    if (child < 0) return blocked("fork", errno, -1, -1);
    if (child == 0) {
        int error;
        close(pipefd[0]);
        if (ptrace(PTRACE_TRACEME, 0, NULL, NULL) != 0) {
            error = errno;
            (void)write(pipefd[1], &error, sizeof(error));
            _exit(77);
        }
        close(pipefd[1]);
        raise(SIGSTOP);
        (void)syscall(SYS_getpid);
        forked = fork();
        if (forked < 0) _exit(78);
        if (forked == 0) {
            execl("/bin/true", "true", (char *)NULL);
            _exit(79);
        }
        if (waitpid(forked, &status, 0) != forked || !WIFEXITED(status) || WEXITSTATUS(status) != 0) _exit(80);
        _exit(0);
    }
    close(pipefd[1]);
    if (waitpid(child, &status, WUNTRACED) != child) return blocked("initial-wait", errno, child, -1);
    if (WIFEXITED(status)) {
        int error = child_errno(pipefd[0]);
        close(pipefd[0]);
        return blocked("child-traceme", error, -1, -1);
    }
    close(pipefd[0]);
    if (!WIFSTOPPED(status) || WSTOPSIG(status) != SIGSTOP) return blocked("initial-stop", EPROTO, child, -1);
    if (ptrace(PTRACE_SETOPTIONS, child, NULL, (void *)(unsigned long)(PTRACE_O_TRACESYSGOOD | PTRACE_O_TRACEFORK | PTRACE_O_TRACEEXEC)) != 0)
        return blocked("parent-setoptions", errno, child, -1);
    if (ptrace(PTRACE_SYSCALL, child, NULL, NULL) != 0) return blocked("parent-syscall-entry", errno, child, -1);
    if (waitpid(child, &status, __WALL) != child) return blocked("parent-syscall-entry-wait", errno, child, -1);
    if (!WIFSTOPPED(status) || WSTOPSIG(status) != (SIGTRAP | 0x80)) return blocked("parent-syscall-entry-stop", EPROTO, child, -1);
    if (syscall_info(child, PTRACE_SYSCALL_INFO_ENTRY, SYS_getpid, 0, "parent-syscall-entry-info", -1)) return 77;
    if (ptrace(PTRACE_SYSCALL, child, NULL, NULL) != 0) return blocked("parent-syscall-exit", errno, child, -1);
    if (waitpid(child, &status, __WALL) != child) return blocked("parent-syscall-exit-wait", errno, child, -1);
    if (!WIFSTOPPED(status) || WSTOPSIG(status) != (SIGTRAP | 0x80)) return blocked("parent-syscall-exit-stop", EPROTO, child, -1);
    if (syscall_info(child, PTRACE_SYSCALL_INFO_EXIT, 0, child, "parent-syscall-exit-info", -1)) return 77;
    if (ptrace(PTRACE_CONT, child, NULL, NULL) != 0) return blocked("parent-fork", errno, child, -1);
    if (waitpid(child, &status, __WALL) != child) return blocked("parent-fork-wait", errno, child, -1);
    if (!WIFSTOPPED(status) || WSTOPSIG(status) != SIGTRAP || (status >> 16) != PTRACE_EVENT_FORK)
        return blocked("parent-fork-event", EPROTO, child, -1);
    if (ptrace(PTRACE_GETEVENTMSG, child, NULL, &event_pid) != 0 || !event_pid)
        return blocked("parent-fork-pid", errno ? errno : EPROTO, child, -1);
    forked = (pid_t)event_pid;
    if (ptrace(PTRACE_CONT, child, NULL, NULL) != 0) return blocked("parent-fork-parent-cont", errno, child, forked);
    if (waitpid(forked, &status, __WALL) != forked) return blocked("parent-fork-child-wait", errno, child, forked);
    if (!WIFSTOPPED(status) || WSTOPSIG(status) != SIGSTOP) return blocked("parent-fork-child-stop", EPROTO, child, forked);
    if (ptrace(PTRACE_CONT, forked, NULL, NULL) != 0) return blocked("parent-exec", errno, child, forked);
    if (waitpid(forked, &status, __WALL) != forked) return blocked("parent-exec-wait", errno, child, forked);
    if (!WIFSTOPPED(status) || WSTOPSIG(status) != SIGTRAP || (status >> 16) != PTRACE_EVENT_EXEC)
        return blocked("parent-exec-event", EPROTO, child, forked);
    if (ptrace(PTRACE_CONT, forked, NULL, NULL) != 0) return blocked("parent-exec-cont", errno, child, forked);
    for (;;) {
        waited = waitpid(-1, &status, __WALL);
        if (waited < 0) return blocked("parent-exit-wait", errno, child, forked);
        if (WIFSTOPPED(status) && waited == child && WSTOPSIG(status) == SIGCHLD) {
            if (ptrace(PTRACE_CONT, child, NULL, NULL) != 0) return blocked("parent-exit-cont", errno, child, forked);
            continue;
        }
        if (!WIFEXITED(status) || WEXITSTATUS(status) != 0 || (waited != child && waited != forked))
            return blocked("parent-exit", EPROTO, child, forked);
        if (waited == forked) forked = -1;
        else child = -1;
        if (child < 0 && forked < 0) break;
    }
    puts("{\"contract\":\"webboxvm-graphics-ptrace-capability-v3\",\"status\":\"blocked\",\"stage\":\"parent-fork-exec-complete\",\"errno\":0}");
    return 77;
}
