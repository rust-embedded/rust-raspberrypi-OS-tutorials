## SPDX-License-Identifier: MIT OR Apache-2.0
##
## Copyright (c) 2019-2020 Andre Richter <andre.o.richter@gmail.com>

default: docker_build

docker_build:
	cp ../../Gemfile .
	docker build -t rustembedded/osdev-utils \
	--build-arg VCS_REF=`git rev-parse --short HEAD` .
	rm Gemfile
