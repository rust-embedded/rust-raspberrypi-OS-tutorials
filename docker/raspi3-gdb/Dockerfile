## SPDX-License-Identifier: MIT
##
## Copyright (c) 2019 Andre Richter <andre.o.richter@gmail.com>
FROM ubuntu:18.04

LABEL maintainer="The resources team <resources@teams.rust-embedded.org>, Andre Richter <andre.o.richter@gmail.com>"

RUN set -ex;                                      \
    tempPkgs='                                    \
        ca-certificates                           \
        wget                                      \
    ';                                            \
    apt-get update;                               \
    apt-get install -q -y --no-install-recommends \
        $tempPkgs                                 \
        gdb-multiarch                             \
        ;                                         \
    wget -P ~ git.io/.gdbinit;                    \
    apt-get purge -y --auto-remove $tempPkgs;     \
    apt-get autoremove -q -y;                     \
    apt-get clean -q -y;                          \
    rm -rf /var/lib/apt/lists/*

COPY auto /root/.gdbinit.d/auto
