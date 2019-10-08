## SPDX-License-Identifier: MIT
##
## Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
## Copyright (c) 2019 Nao Taco <naotaco@gmail.com>
FROM ubuntu:18.04

LABEL maintainer="The resources team <resources@teams.rust-embedded.org>, Andre Richter <andre.o.richter@gmail.com>"

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
