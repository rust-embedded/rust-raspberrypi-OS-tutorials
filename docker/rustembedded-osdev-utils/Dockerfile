## SPDX-License-Identifier: MIT
##
## Copyright (c) 2017-2019 Andre Richter <andre.o.richter@gmail.com>
## Copyright (c) 2019 Nao Taco <naotaco@gmail.com>
FROM ubuntu:18.04

LABEL maintainer="The resources team <resources@teams.rust-embedded.org>, Andre Richter <andre.o.richter@gmail.com>"

RUN set -ex;                                      \
    tempPkgs='                                    \
        automake                                  \
        build-essential                           \
        ca-certificates                           \
        git                                       \
        libglib2.0-dev                            \
        libpixman-1-dev                           \
        libtool                                   \
        pkg-config                                \
        wget                                      \
    ';                                            \
    apt-get update;                               \
    apt-get install -q -y --no-install-recommends \
        $tempPkgs                                 \
        # persistent packages
        gdb-multiarch                             \
        libusb-1.0.0-dev                          \
        python                                    \
        locales                                   \
        ;                                         \
    # QEMU
    git clone --depth 1 git://git.qemu.org/qemu.git;           \
    cd qemu;                                                   \
    ./configure --target-list=aarch64-softmmu --enable-modules \
        --enable-tcg-interpreter --enable-debug-tcg            \
        --python=/usr/bin/python2.7;                           \
    make;                                                      \
    make install;                                              \
    cd ..;                                                     \
    rm -rf qemu;                                               \
    # Raspbootcom
    git clone --depth 1 https://github.com/andre-richter/raspbootin.git; \
    cd raspbootin/raspbootcom;                                           \
    make;                                                                \
    cp raspbootcom /usr/bin;                                             \
    cd ../..;                                                            \
    rm -rf raspbootin;                                                   \
    # Openocd
    git clone --depth 1 https://git.code.sf.net/p/openocd/code openocd; \
    cd openocd;                                                         \
    ./bootstrap;                                                        \
    ./configure --enable-ftdi;                                          \
    make;                                                               \
    make install;                                                       \
    # GDB
    wget -P ~ git.io/.gdbinit;                                          \
    # Cleanup
    apt-get purge -y --auto-remove $tempPkgs;                           \
    apt-get autoremove -q -y;                                           \
    apt-get clean -q -y;                                                \
    rm -rf /var/lib/apt/lists/*

# Locales
RUN locale-gen en_US.UTF-8

ENV LANG=en_US.UTF-8  \
    LANGUAGE=en_US:en \
    LC_ALL=en_US.UTF-8

# Openocd
COPY rpi3.cfg /openocd/
COPY rpi4.cfg /openocd/

# GDB
COPY auto /root/.gdbinit.d/auto
