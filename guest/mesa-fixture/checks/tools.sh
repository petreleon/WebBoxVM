#!/bin/sh
set -eu
uname -r
modprobe --version
/usr/bin/busybox | /usr/bin/busybox head -n 1
test -c /dev/dri/card0
test -f /opt/mesa-f02/lib/libgallium-25.3.6.so
test -f "$VK_DRIVER_FILES"
cat "$VK_DRIVER_FILES"
printf '\nI01_GUEST_TOOLS_PASS\n'
