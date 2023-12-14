## SPDX-License-Identifier: MIT OR Apache-2.0
##
## Copyright (c) 2019-2023 Andre Richter <andre.o.richter@gmail.com>

# Reference followed: https://www.docker.com/blog/getting-started-with-docker-for-arm-on-linux

TAG := 2023.09

default: build_local

build_local:
	cp ../../Gemfile .
	docker build                                           \
	    --tag u007d/osdev-utils-rpi4:$(TAG)              \
	    --build-arg VCS_REF=`git rev-parse --short HEAD` .
	rm Gemfile

buildx_push:
	cp ../../Gemfile .
	docker buildx build                                    \
	    --push                                             \
	    --platform linux/arm64/v8,linux/amd64              \
	    --tag u007d/osdev-utils-rpi4:$(TAG)              \
	    --build-arg VCS_REF=`git rev-parse --short HEAD` .
	rm Gemfile
