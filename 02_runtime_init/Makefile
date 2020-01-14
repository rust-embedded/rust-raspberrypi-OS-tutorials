## SPDX-License-Identifier: MIT OR Apache-2.0
##
## Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

# Default to the RPi3
ifndef BSP
	BSP = rpi3
endif

# BSP-specific arguments
ifeq ($(BSP),rpi3)
	TARGET            = aarch64-unknown-none-softfloat
	OUTPUT            = kernel8.img
	QEMU_BINARY       = qemu-system-aarch64
	QEMU_MACHINE_TYPE = raspi3
	QEMU_RELEASE_ARGS = -serial stdio -display none
	LINKER_FILE       = src/bsp/rpi/link.ld
	RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53
else ifeq ($(BSP),rpi4)
	TARGET            = aarch64-unknown-none-softfloat
	OUTPUT            = kernel8.img
	# QEMU_BINARY       = qemu-system-aarch64
	# QEMU_MACHINE_TYPE =
	# QEMU_RELEASE_ARGS = -serial stdio -display none
	LINKER_FILE       = src/bsp/rpi/link.ld
	RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72
endif

RUSTFLAGS          = -C link-arg=-T$(LINKER_FILE) $(RUSTC_MISC_ARGS)
RUSTFLAGS_PEDANTIC = $(RUSTFLAGS) -D warnings -D missing_docs

SOURCES = $(wildcard **/*.rs) $(wildcard **/*.S) $(wildcard **/*.ld)

XRUSTC_CMD = cargo xrustc     \
	--target=$(TARGET)    \
	--features bsp_$(BSP) \
	--release

CARGO_OUTPUT = target/$(TARGET)/release/kernel

OBJCOPY_CMD = cargo objcopy \
	--                  \
	--strip-all         \
	-O binary

DOCKER_IMAGE         = rustembedded/osdev-utils
DOCKER_CMD           = docker run -it --rm
DOCKER_ARG_DIR_TUT   = -v $(shell pwd):/work -w /work
DOCKER_EXEC_QEMU     = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)

.PHONY: all doc qemu clippy clean readelf objdump nm

all: clean $(OUTPUT)

$(CARGO_OUTPUT): $(SOURCES)
	RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(XRUSTC_CMD)

$(OUTPUT): $(CARGO_OUTPUT)
	cp $< .
	$(OBJCOPY_CMD) $< $(OUTPUT)

doc:
	cargo xdoc --target=$(TARGET) --features bsp_$(BSP) --document-private-items
	xdg-open target/$(TARGET)/doc/kernel/index.html

ifeq ($(QEMU_MACHINE_TYPE),)
qemu:
	@echo "This board is not yet supported for QEMU."
else
qemu: all
	@$(DOCKER_CMD) $(DOCKER_ARG_DIR_TUT) $(DOCKER_IMAGE) \
		$(DOCKER_EXEC_QEMU) $(QEMU_RELEASE_ARGS)     \
		-kernel $(OUTPUT)
endif

clippy:
	RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" cargo xclippy --target=$(TARGET) --features bsp_$(BSP)

clean:
	rm -rf target

readelf:
	readelf -a kernel

objdump:
	cargo objdump --target $(TARGET) -- -disassemble -no-show-raw-insn -print-imm-hex kernel

nm:
	cargo nm --target $(TARGET) -- -print-size kernel | sort
