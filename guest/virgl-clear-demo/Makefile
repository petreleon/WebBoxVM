CROSS ?= aarch64-elf-
CC := $(CROSS)gcc
TARGET := build/virgl-clear-demo
SOURCES := batch.c blob.c demo.c depth.c depth_batch.c depth_equal.c depth_equal_batch.c depth_mixed_batch.c depth_vertex_color.c depth_write_mask_batch.c draw.c index.c kms.c memory.c ops.c setup.c texture.c texture_color.c texture_draw.c texture_pair.c transfer.c uniform.c vertex.c vertex_color.c
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

build/%.o: %.c blob.h kms.h ops.h syscall.h transfer.h uapi.h virgl.h | build
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@

build:
	mkdir -p $@

verify: $(TARGET)
	CROSS=$(CROSS) ./verify.sh $(TARGET)

clean:
	rm -f build/batch.o build/blob.o build/demo.o build/depth.o build/depth_batch.o build/depth_equal.o build/depth_equal_batch.o build/depth_mixed_batch.o build/depth_vertex_color.o build/depth_write_mask_batch.o build/draw.o build/index.o build/kms.o build/memory.o build/ops.o build/setup.o build/texture.o build/texture_color.o build/texture_draw.o build/texture_pair.o build/transfer.o build/uniform.o build/vertex.o build/vertex_color.o build/virgl-clear-demo
