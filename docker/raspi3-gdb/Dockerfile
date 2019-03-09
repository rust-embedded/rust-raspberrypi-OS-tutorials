# -*- coding: utf-8 -*-
#
# =============================================================================
#
# MIT License
#
# Copyright (c) 2019 Andre Richter <andre.o.richter@gmail.com>
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
