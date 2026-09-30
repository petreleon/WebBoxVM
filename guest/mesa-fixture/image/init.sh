#!/bin/sh
exec </dev/console >/dev/console 2>&1
set -eu
stage=console
trap 'status=$?; printf "I01_INIT_FAIL stage=%s status=%s\n" "$stage" "$status" >/dev/kmsg; exit "$status"' EXIT
printf 'I01_INIT_STAGE=console\n' >/dev/kmsg
export PATH=/usr/sbin:/usr/bin:/sbin:/bin
stage=mkdir
mkdir -p /dev /proc /sys /tmp /run
stage=devtmpfs
mount -t devtmpfs devtmpfs /dev
stage=proc
mount -t proc proc /proc
stage=sysfs
mount -t sysfs sysfs /sys
export LD_LIBRARY_PATH=/opt/mesa-f02/lib:/usr/lib/aarch64-linux-gnu
export VK_DRIVER_FILES=/opt/mesa-f02/share/vulkan/icd.d/virtio_icd.aarch64.json
export VK_ICD_FILENAMES="$VK_DRIVER_FILES"
export XDG_RUNTIME_DIR=/run
stage=virtio_mmio
modprobe virtio_mmio
stage=virtio_gpu
modprobe virtio_gpu
stage=card0
test -c /dev/dri/card0
printf 'I01_INIT_STAGE=ready\n' >/dev/kmsg
printf 'WEBBOXVM_GUEST_READY\n'
exec /bin/sh
