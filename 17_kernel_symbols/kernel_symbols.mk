## SPDX-License-Identifier: MIT OR Apache-2.0
##
## Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

include ../common/format.mk
include ../common/docker.mk

##--------------------------------------------------------------------------------------------------
## Check for input variables that need be exported by the calling Makefile
##--------------------------------------------------------------------------------------------------
ifndef KERNEL_SYMBOLS_TOOL_PATH
$(error KERNEL_SYMBOLS_TOOL_PATH is not set)
endif

ifndef TARGET
$(error TARGET is not set)
endif

ifndef KERNEL_SYMBOLS_INPUT_ELF
$(error KERNEL_SYMBOLS_INPUT_ELF is not set)
endif

ifndef KERNEL_SYMBOLS_OUTPUT_ELF
$(error KERNEL_SYMBOLS_OUTPUT_ELF is not set)
endif



##--------------------------------------------------------------------------------------------------
## Targets and Prerequisites
##--------------------------------------------------------------------------------------------------
KERNEL_SYMBOLS_MANIFEST      = kernel_symbols/Cargo.toml
KERNEL_SYMBOLS_LINKER_SCRIPT = kernel_symbols/kernel_symbols.ld

KERNEL_SYMBOLS_RS           = $(KERNEL_SYMBOLS_INPUT_ELF)_symbols.rs
KERNEL_SYMBOLS_DEMANGLED_RS = $(shell pwd)/$(KERNEL_SYMBOLS_INPUT_ELF)_symbols_demangled.rs

KERNEL_SYMBOLS_ELF      = target/$(TARGET)/release/kernel_symbols
KERNEL_SYMBOLS_STRIPPED = target/$(TARGET)/release/kernel_symbols_stripped

# Export for build.rs of kernel_symbols crate.
export KERNEL_SYMBOLS_DEMANGLED_RS



##--------------------------------------------------------------------------------------------------
## Command building blocks
##--------------------------------------------------------------------------------------------------
GET_SYMBOLS_SECTION_VIRT_ADDR = $(DOCKER_TOOLS) $(EXEC_SYMBOLS_TOOL) \
    --get_symbols_section_virt_addr $(KERNEL_SYMBOLS_OUTPUT_ELF)

RUSTFLAGS = -C link-arg=--script=$(KERNEL_SYMBOLS_LINKER_SCRIPT) \
    -C link-arg=--section-start=.rodata=$$($(GET_SYMBOLS_SECTION_VIRT_ADDR))

RUSTFLAGS_PEDANTIC = $(RUSTFLAGS) \
    -D warnings                   \
    -D missing_docs

COMPILER_ARGS = --target=$(TARGET) \
    --release

RUSTC_CMD   = cargo rustc $(COMPILER_ARGS) --manifest-path $(KERNEL_SYMBOLS_MANIFEST)
OBJCOPY_CMD = rust-objcopy \
    --strip-all            \
    -O binary

EXEC_SYMBOLS_TOOL  = ruby $(KERNEL_SYMBOLS_TOOL_PATH)/main.rb

##------------------------------------------------------------------------------
## Dockerization
##------------------------------------------------------------------------------
DOCKER_CMD = docker run -t --rm -v $(shell pwd):/work/tutorial -w /work/tutorial

# DOCKER_IMAGE defined in include file (see top of this file).
DOCKER_TOOLS = $(DOCKER_CMD) $(DOCKER_IMAGE)



##--------------------------------------------------------------------------------------------------
## Targets
##--------------------------------------------------------------------------------------------------
.PHONY: all

all:
	@cp $(KERNEL_SYMBOLS_INPUT_ELF) $(KERNEL_SYMBOLS_OUTPUT_ELF)

	@$(DOCKER_TOOLS) $(EXEC_SYMBOLS_TOOL) --gen_symbols $(KERNEL_SYMBOLS_OUTPUT_ELF) \
                $(KERNEL_SYMBOLS_RS)

	$(call color_progress_prefix, "Demangling")
	@echo Symbol names
	@cat $(KERNEL_SYMBOLS_RS) | rustfilt > $(KERNEL_SYMBOLS_DEMANGLED_RS)

	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(RUSTC_CMD)

	$(call color_progress_prefix, "Stripping")
	@echo Symbols ELF file
	@$(OBJCOPY_CMD) $(KERNEL_SYMBOLS_ELF) $(KERNEL_SYMBOLS_STRIPPED)

	@$(DOCKER_TOOLS) $(EXEC_SYMBOLS_TOOL) --patch_data $(KERNEL_SYMBOLS_OUTPUT_ELF) \
                $(KERNEL_SYMBOLS_STRIPPED)

	$(call color_progress_prefix, "Finished")
