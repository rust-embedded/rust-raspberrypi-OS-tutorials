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

OBJCOPY = cargo objcopy --
OBJCOPY_PARAMS = --strip-all -O binary

UTILS_CONTAINER = andrerichter/raspi3-utils
DOCKER_CMD = docker run -it --rm -v $(shell pwd):/work -w /work
DOCKER_TTY = --privileged -v /dev:/dev
QEMU_CMD = qemu-system-aarch64 -M raspi3 -kernel jtag_boot.img
RASPBOOT_CMD = raspbootcom
RASPBOOT_DEV = /dev/ttyUSB0
# RASPBOOT_DEV = /dev/ttyACM0


.PHONY: all qemu raspboot clippy clean objdump nm

all: clean jtag_boot.img

target/$(TARGET)/release/jtag_boot: $(SOURCES)
	cargo xbuild --target=$(TARGET) --release

jtag_boot.img: target/$(TARGET)/release/jtag_boot
	cp $< .
	$(OBJCOPY) $(OBJCOPY_PARAMS) $< jtag_boot.img

qemu: all
	$(DOCKER_CMD) $(UTILS_CONTAINER) $(QEMU_CMD) -serial null -serial stdio

raspboot: all
	$(DOCKER_CMD) $(DOCKER_TTY) $(UTILS_CONTAINER) $(RASPBOOT_CMD) \
	$(RASPBOOT_DEV) jtag_boot.img

clippy:
	cargo xclippy --target=$(TARGET)

clean:
	cargo clean

objdump:
	cargo objdump --target $(TARGET) -- -disassemble -print-imm-hex jtag_boot

nm:
	cargo nm --target $(TARGET) -- jtag_boot | sort
