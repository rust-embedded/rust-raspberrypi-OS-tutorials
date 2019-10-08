#!/usr/bin/env bash

# SPDX-License-Identifier: MIT
#
# Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

CONTAINER_UTILS="rustembedded/osdev-utils"

DOCKER_CMD="docker run -it --rm"
DOCKER_ARG_CURDIR="-v $(pwd):/work -w /work"
DOCKER_ARG_TTY="--privileged -v /dev:/dev"
DOCKER_EXEC_RASPBOOT="raspbootcom /dev/ttyUSB0"

$DOCKER_CMD \
  $DOCKER_ARG_CURDIR \
  $DOCKER_ARG_TTY \
  $CONTAINER_UTILS \
  $DOCKER_EXEC_RASPBOOT \
  kernel8.img
