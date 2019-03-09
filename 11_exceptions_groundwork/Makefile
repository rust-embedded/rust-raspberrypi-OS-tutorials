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

TARGET = aarch64-unknown-none

SOURCES = $(wildcard **/*.rs) $(wildcard **/*.S) link.ld

OBJCOPY = cargo objcopy --
OBJCOPY_PARAMS = --strip-all -O binary

UTILS_CONTAINER = andrerichter/raspi3-utils
DOCKER_CMD = docker run -it --rm -v $(shell pwd):/work -w /work
DOCKER_MISC = -v $(shell pwd)/../emulation:/emulation
DOCKER_TTY = --privileged -v /dev:/dev
QEMU_CMD = bash /emulation/qemu_multi_uart.sh
RASPBOOT_CMD = raspbootcom /dev/ttyUSB0 kernel8.img

.PHONY: all qemu raspboot clippy clean objdump nm

all: clean kernel8.img

target/$(TARGET)/release/kernel8: $(SOURCES)
	cargo xbuild --target=$(TARGET) --release

kernel8.img: target/$(TARGET)/release/kernel8
	cp $< .
	$(OBJCOPY) $(OBJCOPY_PARAMS) $< kernel8.img

qemu: all
	$(DOCKER_CMD) $(DOCKER_MISC) $(UTILS_CONTAINER) $(QEMU_CMD)

raspboot: all
	$(DOCKER_CMD) $(DOCKER_TTY) $(UTILS_CONTAINER) $(RASPBOOT_CMD)

clippy:
	cargo xclippy --target=$(TARGET)

clean:
	cargo clean

objdump:
	cargo objdump --target $(TARGET) -- -disassemble -print-imm-hex kernel8

nm:
	cargo nm --target $(TARGET) -- kernel8 | sort
