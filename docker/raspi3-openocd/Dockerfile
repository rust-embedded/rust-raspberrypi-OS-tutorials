# -*- coding: utf-8 -*-
#
# =============================================================================
#
# MIT License
#
# Copyright (c) 2019 Andre Richter <andre.o.richter@gmail.com>
# Copyright (c) 2019 Nao Taco <naotaco@gmail.com>
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
# =============================================================================
FROM ubuntu:18.04

LABEL maintainer="The Cortex-A team <cortex-a@teams.rust-embedded.org>, Andre Richter <andre.o.richter@gmail.com>"

RUN set -ex;                                                            \
    tempPkgs='                                                          \
        automake                                                        \
        build-essential                                                 \
        ca-certificates                                                 \
        git                                                             \
        libtool                                                         \
        pkg-config                                                      \
    ';                                                                  \
    apt-get update;                                                     \
    apt-get install -q -y --no-install-recommends                       \
        $tempPkgs                                                       \
        libusb-1.0.0-dev                                                \
        ;                                                               \
    git clone --depth 1 https://git.code.sf.net/p/openocd/code openocd; \
    cd openocd;                                                         \
    ./bootstrap;                                                        \
    ./configure --enable-ftdi;                                          \
    make;                                                               \
    make install;                                                       \
    apt-get purge -y --auto-remove $tempPkgs;                           \
    apt-get autoremove -q -y;                                           \
    apt-get clean -q -y;                                                \
    rm -rf /var/lib/apt/lists/*

COPY rpi3.cfg /openocd/

ENTRYPOINT ["openocd"]
CMD ["-f", "/openocd/tcl/interface/ftdi/olimex-arm-usb-tiny-h.cfg", "-f", "/openocd/rpi3.cfg"]
