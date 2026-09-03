CROSS ?= aarch64-elf-
CC := $(CROSS)gcc
TARGET := build/webgpu-demo
SOURCES := demo.c packet.c
OBJECTS := $(SOURCES:%.c=build/%.o)

CPPFLAGS := -I.
CFLAGS := -std=c11 -Os -g0 -march=armv8-a -mstrict-align \
	-ffreestanding -fno-builtin -fno-stack-protector -fno-pic -fno-pie \
	-fno-unwind-tables -fno-asynchronous-unwind-tables \
	-ffunction-sections -fdata-sections -Wall -Wextra -Werror
LDFLAGS := -nostdlib -static -no-pie -Wl,-T,link.ld -Wl,--build-id=none \
	-Wl,--gc-sections -Wl,-z,max-page-size=4096

.PHONY: all clean verify

all: verify

$(TARGET): $(OBJECTS) link.ld
	$(CC) $(CFLAGS) $(LDFLAGS) $(OBJECTS) -o $@

build/%.o: %.c packet.h syscall.h uapi.h | build
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@

build:
	mkdir -p $@

verify: $(TARGET)
	CROSS=$(CROSS) ./verify.sh $(TARGET)

clean:
	rm -f build/demo.o build/packet.o build/webgpu-demo
