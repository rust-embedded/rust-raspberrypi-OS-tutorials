## SPDX-License-Identifier: MIT
##
## Copyright (c) 2017-2019 Andre Richter <andre.o.richter@gmail.com>
FROM ubuntu:18.04

LABEL maintainer="The resources team <resources@teams.rust-embedded.org>, Andre Richter <andre.o.richter@gmail.com>"

RUN set -ex;                                      \
    apt-get update;                               \
    apt-get install -q -y --no-install-recommends \
        build-essential                           \
        ca-certificates                           \
        git                                       \
        libglib2.0-dev                            \
        libpixman-1-dev                           \
        locales                                   \
        pkg-config                                \
        python                                    \
        screen                                    \
        tmux                                      \
        ;                                         \
    echo "set -g mouse on" > ~/.tmux.conf;        \
    # Cleanup
    apt-get autoremove -q -y;                     \
    apt-get clean -q -y;                          \
    rm -rf /var/lib/apt/lists/*

RUN locale-gen en_US.UTF-8

ENV LANG=en_US.UTF-8  \
    LANGUAGE=en_US:en \
    LC_ALL=en_US.UTF-8

RUN git clone git://git.qemu.org/qemu.git; \
    cd qemu; \
    ./configure --target-list=aarch64-softmmu --enable-modules \
        --enable-tcg-interpreter --enable-debug-tcg \
        --python=/usr/bin/python2.7; \
    make; \
    make install; \
    cd ..; \
    rm -rf qemu

RUN git clone https://github.com/andre-richter/raspbootin.git; \
    cd raspbootin/raspbootcom; \
    make; \
    cp raspbootcom /usr/bin; \
    cd ../..; \
    rm -rf raspbootin
