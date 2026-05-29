FROM ubuntu:24.04
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential flex bison bc cpio libssl-dev libelf-dev \
    gcc-aarch64-linux-gnu binutils-aarch64-linux-gnu \
    wget xz-utils ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /build
RUN wget -q https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.6.70.tar.xz \
    && tar xf linux-6.6.70.tar.xz && rm linux-6.6.70.tar.xz
WORKDIR /build/linux-6.6.70
RUN make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- defconfig \
    && scripts/config -e CONFIG_RELOCATABLE \
    && scripts/config -e CONFIG_EFI_STUB \
    && scripts/config -d CONFIG_MODULES -d CONFIG_KALLSYMS \
    && scripts/config -d CONFIG_DEBUG_INFO -d CONFIG_DEBUG_INFO_BTF \
    && scripts/config -d CONFIG_SOUND -d CONFIG_DRM -d CONFIG_NET \
    && scripts/config -d CONFIG_USB_SUPPORT -d CONFIG_PCI \
    && make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- olddefconfig
RUN make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- -j$(nproc) Image 2>&1 | tail -5
CMD cp arch/arm64/boot/Image /out/Image && ls -lh /out/Image
