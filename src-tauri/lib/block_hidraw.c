// Intercepts open() to block /dev/hidraw* access.
// Wine's winebus.sys hidraw backend grabs gamepads before SDL can,
// and hidraw has no force-feedback support. Blocking hidraw forces
// winebus to use the SDL backend which supports rumble.
#define _GNU_SOURCE
#include <dlfcn.h>
#include <string.h>
#include <errno.h>
#include <stdarg.h>
#include <fcntl.h>

typedef int (*open_fn)(const char *, int, ...);

int open(const char *pathname, int flags, ...) {
    static open_fn real_open = NULL;
    if (!real_open) real_open = (open_fn)dlsym(RTLD_NEXT, "open");

    if (pathname && strncmp(pathname, "/dev/hidraw", 11) == 0) {
        errno = EACCES;
        return -1;
    }

    if (flags & O_CREAT) {
        va_list ap;
        va_start(ap, flags);
        int mode = va_arg(ap, int);
        va_end(ap);
        return real_open(pathname, flags, mode);
    }
    return real_open(pathname, flags);
}

int open64(const char *pathname, int flags, ...) {
    static open_fn real_open64 = NULL;
    if (!real_open64) real_open64 = (open_fn)dlsym(RTLD_NEXT, "open64");

    if (pathname && strncmp(pathname, "/dev/hidraw", 11) == 0) {
        errno = EACCES;
        return -1;
    }

    if (flags & O_CREAT) {
        va_list ap;
        va_start(ap, flags);
        int mode = va_arg(ap, int);
        va_end(ap);
        return real_open64(pathname, flags, mode);
    }
    return real_open64(pathname, flags);
}
