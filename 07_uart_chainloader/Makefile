## SPDX-License-Identifier: MIT OR Apache-2.0
##
## Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

# Default to the RPi3
BSP ?= rpi3

# Default to a serial device name that is common in Linux.
DEV_SERIAL ?= /dev/ttyUSB0

# Query the host system's kernel name
UNAME_S = $(shell uname -s)

# BSP-specific arguments
ifeq ($(BSP),rpi3)
    TARGET            = aarch64-unknown-none-softfloat
    KERNEL_BIN        = kernel8.img
    QEMU_BINARY       = qemu-system-aarch64
    QEMU_MACHINE_TYPE = raspi3
    QEMU_RELEASE_ARGS = -serial stdio -display none
    LINKER_FILE       = src/bsp/raspberrypi/link.ld
    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53 -C relocation-model=pic
    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi3.img
else ifeq ($(BSP),rpi4)
    TARGET            = aarch64-unknown-none-softfloat
    KERNEL_BIN        = kernel8.img
    QEMU_BINARY       = qemu-system-aarch64
    QEMU_MACHINE_TYPE =
    QEMU_RELEASE_ARGS = -serial stdio -display none
    LINKER_FILE       = src/bsp/raspberrypi/link.ld
    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72 -C relocation-model=pic
    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi4.img
endif

# Export for build.rs
export LINKER_FILE

RUSTFLAGS          = -C link-arg=-T$(LINKER_FILE) $(RUSTC_MISC_ARGS)
RUSTFLAGS_PEDANTIC = $(RUSTFLAGS) -D warnings -D missing_docs

COMPILER_ARGS = --target=$(TARGET) \
    --features bsp_$(BSP)          \
    --release

RUSTC_CMD   = cargo rustc $(COMPILER_ARGS)
DOC_CMD     = cargo doc $(COMPILER_ARGS)
CLIPPY_CMD  = cargo clippy $(COMPILER_ARGS)
CHECK_CMD   = cargo check $(COMPILER_ARGS)
OBJCOPY_CMD = rust-objcopy \
    --strip-all            \
    -O binary

KERNEL_ELF = target/$(TARGET)/release/kernel

DOCKER_IMAGE         = rustembedded/osdev-utils
DOCKER_CMD           = docker run -it --rm -v $(shell pwd):/work/tutorial -w /work/tutorial
DOCKER_ARG_DIR_UTILS = -v $(shell pwd)/../utils:/work/utils
DOCKER_ARG_DEV       = --privileged -v /dev:/dev

DOCKER_QEMU = $(DOCKER_CMD) $(DOCKER_IMAGE)

# Dockerize commands that require USB device passthrough only on Linux
ifeq ($(UNAME_S),Linux)
    DOCKER_CMD_DEV = $(DOCKER_CMD) $(DOCKER_ARG_DEV)

    DOCKER_CHAINBOOT = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_UTILS) $(DOCKER_IMAGE)
endif

EXEC_QEMU     = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
EXEC_MINIPUSH = ruby ../utils/minipush.rb

.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu qemuasm chainboot clippy clean readelf objdump nm \
    check

all: $(KERNEL_BIN)

$(KERNEL_ELF):
	RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(RUSTC_CMD)

$(KERNEL_BIN): $(KERNEL_ELF)
	@$(OBJCOPY_CMD) $(KERNEL_ELF) $(KERNEL_BIN)

doc:
	$(DOC_CMD) --document-private-items --open

ifeq ($(QEMU_MACHINE_TYPE),)
qemu qemuasm:
	@echo "This board is not yet supported for QEMU."
else
qemu: $(KERNEL_BIN)
	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)

qemuasm: $(KERNEL_BIN)
	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN) -d in_asm
endif

chainboot:
	@$(DOCKER_CHAINBOOT) $(EXEC_MINIPUSH) $(DEV_SERIAL) $(CHAINBOOT_DEMO_PAYLOAD)

clippy:
	RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(CLIPPY_CMD)

clean:
	rm -rf target $(KERNEL_BIN)

readelf: $(KERNEL_ELF)
	readelf -a $(KERNEL_ELF)

objdump: $(KERNEL_ELF)
	rust-objdump --arch-name aarch64 --disassemble --demangle --no-show-raw-insn \
	    --print-imm-hex $(KERNEL_ELF)

nm: $(KERNEL_ELF)
	rust-nm --demangle --print-size $(KERNEL_ELF) | sort

# For rust-analyzer
check:
	@RUSTFLAGS="$(RUSTFLAGS)" $(CHECK_CMD) --message-format=json
