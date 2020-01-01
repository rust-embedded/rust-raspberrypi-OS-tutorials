## SPDX-License-Identifier: MIT OR Apache-2.0
##
## Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

# Default to the RPi3
ifndef BSP
	BSP = rpi3
endif

# BSP-specific arguments
ifeq ($(BSP),rpi3)
	TARGET = aarch64-unknown-none-softfloat
	OUTPUT = kernel8.img
	QEMU_BINARY = qemu-system-aarch64
	QEMU_MACHINE_TYPE = raspi3
	QEMU_RELEASE_ARGS = -serial stdio -display none
	QEMU_TEST_ARGS = $(QEMU_RELEASE_ARGS) -semihosting
	OPENOCD_ARG = -f /openocd/tcl/interface/ftdi/olimex-arm-usb-tiny-h.cfg -f /openocd/rpi3.cfg
	JTAG_BOOT_IMAGE = jtag_boot_rpi3.img
	LINKER_FILE = src/bsp/rpi/link.ld
	RUSTC_MISC_ARGS = -C target-cpu=cortex-a53
else ifeq ($(BSP),rpi4)
	TARGET = aarch64-unknown-none-softfloat
	OUTPUT = kernel8.img
#	QEMU_BINARY = qemu-system-aarch64
#	QEMU_MACHINE_TYPE =
#	QEMU_RELEASE_ARGS = -serial stdio -display none
#	QEMU_TEST_ARGS = $(QEMU_RELEASE_ARGS) -semihosting
	OPENOCD_ARG = -f /openocd/tcl/interface/ftdi/olimex-arm-usb-tiny-h.cfg -f /openocd/rpi4.cfg
	JTAG_BOOT_IMAGE = jtag_boot_rpi4.img
	LINKER_FILE = src/bsp/rpi/link.ld
	RUSTC_MISC_ARGS = -C target-cpu=cortex-a72
endif

# Testing-specific arguments
ifdef TEST
	ifeq ($(TEST),unit)
		TEST_ARG = --lib
	else
		TEST_ARG = --test $(TEST)
	endif
endif

QEMU_MISSING_STRING = "This board is not yet supported for QEMU."

RUSTFLAGS = -C link-arg=-T$(LINKER_FILE) $(RUSTC_MISC_ARGS)
RUSTFLAGS_PEDANTIC = $(RUSTFLAGS) -D warnings -D missing_docs

SOURCES = $(wildcard **/*.rs) $(wildcard **/*.S) $(wildcard **/*.ld)

X_CMD_ARGS = --target=$(TARGET) \
		--features bsp_$(BSP) \
		--release

XRUSTC_CMD = cargo xrustc $(X_CMD_ARGS)
XTEST_CMD = cargo xtest $(X_CMD_ARGS)

CARGO_OUTPUT = target/$(TARGET)/release/kernel

OBJCOPY_CMD = cargo objcopy \
		-- \
		--strip-all \
		-O binary

CONTAINER_UTILS   = rustembedded/osdev-utils

DOCKER_CMD_TEST   = docker run -i --rm
DOCKER_CMD_USER   = $(DOCKER_CMD_TEST) -t
DOCKER_ARG_CURDIR = -v $(shell pwd):/work -w /work
DOCKER_ARG_TTY    = --privileged -v /dev:/dev
DOCKER_ARG_JTAG   = -v $(shell pwd)/../X1_JTAG_boot:/jtag
DOCKER_ARG_NET    = --network host

DOCKER_EXEC_QEMU         = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
DOCKER_EXEC_RASPBOOT     = raspbootcom
DOCKER_EXEC_RASPBOOT_DEV = /dev/ttyUSB0
# DOCKER_EXEC_RASPBOOT_DEV = /dev/ttyACM0

.PHONY: all doc qemu chainboot jtagboot openocd gdb gdb-opt0 clippy clean readelf objdump nm test

all: clean $(OUTPUT)

$(CARGO_OUTPUT): $(SOURCES)
	RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(XRUSTC_CMD)

$(OUTPUT): $(CARGO_OUTPUT)
	cp $< .
	$(OBJCOPY_CMD) $< $(OUTPUT)

doc:
	cargo xdoc --target=$(TARGET) --features bsp_$(BSP) --document-private-items
	xdg-open target/$(TARGET)/doc/libkernel/index.html

ifeq ($(QEMU_MACHINE_TYPE),)
qemu:
	@echo $(QEMU_MISSING_STRING)

test:
	@echo $(QEMU_MISSING_STRING)
else
qemu: all
	$(DOCKER_CMD_USER) $(DOCKER_ARG_CURDIR) $(CONTAINER_UTILS) \
	$(DOCKER_EXEC_QEMU) $(QEMU_RELEASE_ARGS) \
	-kernel $(OUTPUT)

define KERNEL_TEST_RUNNER
	#!/usr/bin/env bash

	$(OBJCOPY_CMD) $$1 $$1.img
	TEST_BINARY=$$(echo $$1.img | sed -e 's/.*target/target/g')
	$(DOCKER_CMD_TEST) $(DOCKER_ARG_CURDIR) $(CONTAINER_UTILS) \
	ruby tests/runner.rb $(DOCKER_EXEC_QEMU) $(QEMU_TEST_ARGS) -kernel $$TEST_BINARY
endef

export KERNEL_TEST_RUNNER
test: $(SOURCES)
	@mkdir -p target
	@echo "$$KERNEL_TEST_RUNNER" > target/kernel_test_runner.sh
	@chmod +x target/kernel_test_runner.sh
	RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(XTEST_CMD) $(TEST_ARG)
endif

chainboot: all
	$(DOCKER_CMD_USER) $(DOCKER_ARG_CURDIR) $(DOCKER_ARG_TTY) \
	$(CONTAINER_UTILS) $(DOCKER_EXEC_RASPBOOT) $(DOCKER_EXEC_RASPBOOT_DEV) \
	$(OUTPUT)

jtagboot:
	$(DOCKER_CMD_USER) $(DOCKER_ARG_TTY) $(DOCKER_ARG_JTAG) $(CONTAINER_UTILS) \
	$(DOCKER_EXEC_RASPBOOT) $(DOCKER_EXEC_RASPBOOT_DEV) \
	/jtag/$(JTAG_BOOT_IMAGE)

openocd:
	$(DOCKER_CMD_USER) $(DOCKER_ARG_TTY) $(DOCKER_ARG_NET) $(CONTAINER_UTILS) \
	openocd $(OPENOCD_ARG)

define gen_gdb
	RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(XRUSTC_CMD) $1
	cp $(CARGO_OUTPUT) kernel_for_jtag
	$(DOCKER_CMD_USER) $(DOCKER_ARG_CURDIR) $(DOCKER_ARG_NET) $(CONTAINER_UTILS) \
	gdb-multiarch -q kernel_for_jtag
endef

gdb: clean $(SOURCES)
	$(call gen_gdb,-C debuginfo=2)

gdb-opt0: clean $(SOURCES)
	$(call gen_gdb,-C debuginfo=2 -C opt-level=0)

clippy:
	cargo xclippy --target=$(TARGET) --features bsp_$(BSP)

clean:
	rm -rf target

readelf:
	readelf -a kernel

objdump:
	cargo objdump --target $(TARGET) -- -disassemble -no-show-raw-insn -print-imm-hex kernel

nm:
	cargo nm --target $(TARGET) -- -print-size kernel | sort
