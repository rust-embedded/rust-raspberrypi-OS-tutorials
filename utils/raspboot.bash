#!/usr/bin/env bash

CONTAINER_UTILS="andrerichter/raspi3-utils"

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
