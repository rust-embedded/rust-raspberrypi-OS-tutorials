#
# MIT License
#
# Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
#

TARGET = aarch64-unknown-none-softfloat

SOURCES = $(wildcard **/*.rs) $(wildcard **/*.S) link.ld


XRUSTC_CMD   = cargo xrustc --target=$(TARGET) --release
CARGO_OUTPUT = target/$(TARGET)/release/kernel8

OBJCOPY        = cargo objcopy --
OBJCOPY_PARAMS = --strip-all -O binary

CONTAINER_UTILS   = andrerichter/raspi3-utils
CONTAINER_OPENOCD = andrerichter/raspi3-openocd
CONTAINER_GDB     = andrerichter/raspi3-gdb

DOCKER_CMD        = docker run -it --rm
DOCKER_ARG_CURDIR = -v $(shell pwd):/work -w /work
DOCKER_ARG_TTY    = --privileged -v /dev:/dev
DOCKER_ARG_JTAG   = -v $(shell pwd)/../X1_JTAG_boot:/jtag
DOCKER_ARG_NET    = --network host
DOCKER_ARG_EMU    = -v $(shell pwd)/../emulation:/emulation

DOCKER_EXEC_QEMU     = bash /emulation/qemu_multi_uart.sh
DOCKER_EXEC_RASPBOOT = raspbootcom /dev/ttyUSB0

.PHONY: all qemu raspboot clippy clean objdump nm jtagboot openocd gdb gdb-opt0

all: clean kernel8.img

$(CARGO_OUTPUT): $(SOURCES)
	$(XRUSTC_CMD)

kernel8.img: $(CARGO_OUTPUT)
	cp $< .
	$(OBJCOPY) $(OBJCOPY_PARAMS) $< kernel8.img

qemu: all
	$(DOCKER_CMD) $(DOCKER_ARG_CURDIR) $(DOCKER_ARG_EMU) \
        $(CONTAINER_UTILS) $(DOCKER_EXEC_QEMU)

raspboot: all
	$(DOCKER_CMD) $(DOCKER_ARG_CURDIR) $(DOCKER_ARG_TTY) \
	$(CONTAINER_UTILS) $(DOCKER_EXEC_RASPBOOT) kernel8.img

clippy:
	cargo xclippy --target=$(TARGET)

clean:
	cargo clean

objdump:
	cargo objdump --target $(TARGET) -- -disassemble -print-imm-hex kernel8

nm:
	cargo nm --target $(TARGET) -- kernel8 | sort

jtagboot:
	$(DOCKER_CMD) $(DOCKER_ARG_TTY) $(DOCKER_ARG_JTAG) $(CONTAINER_UTILS) \
	$(DOCKER_EXEC_RASPBOOT) /jtag/jtag_boot.img

openocd:
	$(DOCKER_CMD) $(DOCKER_ARG_TTY) $(DOCKER_ARG_NET) $(CONTAINER_OPENOCD)

define gen_gdb
	$(XRUSTC_CMD) -- $1
	cp $(CARGO_OUTPUT) kernel8_for_jtag
	$(DOCKER_CMD) $(DOCKER_ARG_CURDIR) $(DOCKER_ARG_NET) $(CONTAINER_GDB) \
	gdb-multiarch -q kernel8_for_jtag
endef

gdb: clean $(SOURCES)
	$(call gen_gdb,-C debuginfo=2)

gdb-opt0: clean $(SOURCES)
	$(call gen_gdb,-C debuginfo=2 -C opt-level=0)
